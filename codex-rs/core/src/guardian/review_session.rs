//! Host adapter for synchronous Guardian sessions and the existing context builder.
//! The extension owns review policy and pooling; this module binds runtime operations
//! to the captured parent action, environments, authorization and context snapshots.

#[path = "review_session_setup.rs"]
mod setup;
pub use setup::PreparedGuardianContext;
pub(super) use setup::prepare_review;
pub use setup::prepare_review_prewarm;
pub(crate) use setup::run_guardian_review_session;

#[path = "review_session_prompt.rs"]
mod prompt_builder;

#[path = "review_session_context.rs"]
mod context_policy;
use codex_guardian_reviewer::ReviewContextPolicy;
use codex_guardian_reviewer::ReviewDeadline;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::Ordering;
#[cfg(test)]
use std::time::Duration;

use codex_analytics::GuardianReviewAnalyticsResult;
use codex_analytics::GuardianReviewSessionAnalyticsParams;
use codex_analytics::GuardianReviewSessionKind;
use codex_extension_api::Instructions;
#[cfg(test)]
use codex_guardian_reviewer::ConversationState;
use codex_guardian_reviewer::GuardianReviewForkSnapshot;
pub use codex_guardian_reviewer::GuardianReviewState;
use codex_guardian_reviewer::GuardianTranscriptSource;
use codex_guardian_reviewer::PendingNodeReplEvidenceAdmission;
use codex_guardian_reviewer::ReviewModel;
use codex_guardian_reviewer::ReviewSessionResult;
use codex_guardian_reviewer::SessionDisposition;
use codex_guardian_reviewer::token_usage_delta;
use codex_history::InitialHistory;
#[cfg(test)]
use codex_history::RolloutItem;
use codex_protocol::config_types::AutoCompactTokenLimitScope;
use codex_protocol::config_types::Personality;
use codex_protocol::config_types::ReasoningSummary as ReasoningSummaryConfig;
use codex_protocol::models::ContentItem;
use codex_protocol::models::ResponseItem;
use codex_protocol::openai_models::GuardianScope;
use codex_protocol::openai_models::ReasoningEffort as ReasoningEffortConfig;
use codex_protocol::protocol::CodexErrorInfo;
use codex_protocol::protocol::Event;
#[cfg(test)]
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::Op;
use codex_protocol::protocol::SubAgentSource;
use futures::future::BoxFuture;
use serde_json::Value;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use tracing::warn;

use crate::agents_md_manager::SessionInstructions;
use crate::config::Config;
use crate::config::Constrained;
use crate::config::ManagedFeatures;
use crate::config::Permissions;
use crate::context::ContextualUserFragment;
use crate::context::GuardianContextMode;
use crate::context::GuardianFollowupReviewReminder;
use crate::context::GuardianNodeReplPolicy;
use crate::context_manager::ContextManager;
use crate::environment_selection::TurnEnvironmentSnapshot;
use crate::session::SessionIo;
use crate::session::session::Session;
use crate::session::turn_context::TurnContext;
use codex_config::types::McpServerConfig;
use codex_features::Feature;
use codex_model_provider_info::ModelProviderInfo;
use codex_protocol::turn_input::TurnInputMode;
use codex_protocol::turn_input::TurnInputRequest;
use codex_protocol::turn_input::TurnInputSubmission;
use codex_thread_store::PersistContext;
use codex_utils_path_uri::PathUri;

use super::ApprovalRequestReasons;
use super::GUARDIAN_REVIEWER_NAME;
use super::GuardianApprovalRequest;
use super::GuardianReviewContext;
use super::feedback::record_failed_review;
use super::prompt::GUARDIAN_TRANSCRIPT_START;
use super::prompt::GuardianPromptMode;
#[cfg(test)]
use super::prompt::GuardianTranscriptCursor;
use super::prompt::GuardianTranscriptHistory;
use super::review::guardian_review_session_config;
pub(crate) use super::reviewer_config::build_guardian_review_session_config;
use codex_guardian_reviewer::run_before_review_deadline;
use codex_guardian_reviewer::wait_for_guardian_review;

pub(crate) use codex_guardian_reviewer::GuardianReviewSessionOutcome;

pub(crate) struct GuardianReviewSessionParams {
    pub(crate) parent_session: Arc<Session>,
    pub(crate) parent_context: GuardianReviewContext,
    // Checkpoint selection and thread-owned prompt evidence must use the same history.
    pub(crate) parent_history: ContextManager,
    pub(crate) spawn_config: Config,
    pub(crate) node_repl_policy: GuardianNodeReplPolicy,
    pub(crate) request: GuardianApprovalRequest,
    pub(crate) category: GuardianScope,
    pub(crate) reasons: ApprovalRequestReasons,
    pub(super) authorization: Option<super::review::ReviewAuthorization>,
    pub(crate) schema: Value,
    pub(crate) review_model: ReviewModel,
    pub(crate) reasoning_summary: ReasoningSummaryConfig,
    pub(crate) personality: Option<Personality>,
    pub(crate) external_cancel: Option<CancellationToken>,
    pub(crate) deadline: ReviewDeadline,
}

pub(crate) type GuardianReviewSessionManager =
    codex_guardian_reviewer::ReviewerPool<GuardianReviewSession>;

/// Opaque host session handle. Its state belongs to the existing context builder.
pub struct GuardianReviewSession {
    session: Arc<Session>,
    io: SessionIo,
    cancel_token: CancellationToken,
    reuse_key: GuardianReviewSessionReuseKey,
    state: Mutex<GuardianReviewState>,
}

fn had_prior_review_context(prompt_mode: &GuardianPromptMode) -> bool {
    matches!(prompt_mode, GuardianPromptMode::Delta { .. })
}

/// Opaque compatibility key derived by the existing context builder.
#[derive(Debug, Clone, PartialEq)]
pub struct GuardianReviewSessionReuseKey {
    // Only include settings that affect spawned-session behavior and parent
    // history rewrites that invalidate existing reviewer context.
    parent_history_version: u64,
    parent_reset_version: u64,
    root_review_version: Option<(crate::codex_thread::GuardianAuthorizationVersion, u64)>,
    node_repl_auto_review_required: bool,
    node_repl_policy: String,
    model: Option<String>,
    model_provider_id: String,
    model_provider: ModelProviderInfo,
    model_context_window: Option<i64>,
    model_auto_compact_token_limit: Option<i64>,
    model_auto_compact_token_limit_scope: AutoCompactTokenLimitScope,
    model_reasoning_effort: Option<ReasoningEffortConfig>,
    model_reasoning_summary: Option<ReasoningSummaryConfig>,
    omit_personality: bool,
    permissions: Permissions,
    developer_instructions: Option<String>,
    base_instructions: Option<String>,
    user_instructions: Option<Instructions>,
    thread_instructions: Option<Instructions>,
    project_instructions: Option<Arc<crate::agents_md::LoadedAgentsMd>>,
    compact_prompt: Option<String>,
    cwd: PathUri,
    mcp_servers: Constrained<HashMap<String, McpServerConfig>>,
    codex_linux_sandbox_exe: Option<PathBuf>,
    features: ManagedFeatures,
    environment_ids: Vec<String>,
}

impl GuardianReviewSessionReuseKey {
    fn from_spawn_config(
        spawn_config: &Config,
        instructions: SessionInstructions,
        parent_history_version: u64,
        context_mode: GuardianContextMode,
    ) -> Self {
        Self {
            root_review_version: None,
            parent_reset_version: 0,
            parent_history_version: match ReviewContextPolicy::for_context(
                context_mode,
                &spawn_config.features,
            ) {
                ReviewContextPolicy::Legacy | ReviewContextPolicy::Independent => 0,
                ReviewContextPolicy::LegacyWithCheckpointReuse
                | ReviewContextPolicy::ThreadOwned => parent_history_version,
            },
            node_repl_auto_review_required: false,
            node_repl_policy: String::new(),
            model: spawn_config.model.clone(),
            model_provider_id: spawn_config.model_provider_id.clone(),
            model_provider: spawn_config.model_provider.clone(),
            model_context_window: spawn_config.model_context_window,
            model_auto_compact_token_limit: spawn_config.model_auto_compact_token_limit,
            model_auto_compact_token_limit_scope: spawn_config.model_auto_compact_token_limit_scope,
            model_reasoning_effort: spawn_config.model_reasoning_effort.clone(),
            model_reasoning_summary: spawn_config.model_reasoning_summary,
            omit_personality: spawn_config.personality == Some(Personality::None),
            permissions: spawn_config.permissions.clone(),
            developer_instructions: spawn_config.developer_instructions.clone(),
            base_instructions: spawn_config.base_instructions.clone(),
            user_instructions: instructions.user,
            thread_instructions: instructions.thread,
            project_instructions: instructions.project_snapshot,
            compact_prompt: spawn_config.compact_prompt.clone(),
            cwd: PathUri::from_abs_path(&spawn_config.cwd),
            mcp_servers: spawn_config.mcp_servers.clone(),
            codex_linux_sandbox_exe: spawn_config.codex_linux_sandbox_exe.clone(),
            features: spawn_config.features.clone(),
            environment_ids: Vec::new(),
        }
    }

    fn with_environments(mut self, environments: &TurnEnvironmentSnapshot) -> Self {
        self.environment_ids = environments
            .captured_environments()
            .into_keys()
            .collect::<Vec<_>>();
        self.environment_ids.sort_unstable();
        self
    }

    fn with_node_repl_policy_eligibility(mut self, required: bool) -> Self {
        self.node_repl_auto_review_required = required;
        self
    }

    fn with_node_repl_policy(mut self, policy: &GuardianNodeReplPolicy) -> Self {
        self.node_repl_policy = policy.body().into_text().unwrap_or_default();
        self
    }
<<<<<<< HEAD

    #[expect(
        clippy::await_holding_invalid_type,
        reason = "review session selection and trunk spawning must stay serialized"
    )]
    pub(super) async fn run_review(
        &self,
        params: GuardianReviewSessionParams,
    ) -> (GuardianReviewSessionOutcome, GuardianReviewAnalyticsResult) {
        let deadline = params.deadline;
        let next_reuse_key = GuardianReviewSessionReuseKey::from_spawn_config(
            &params.spawn_config,
            params.parent_session.user_instructions().await,
        );
        let mut stale_trunk_to_shutdown = None;
        let mut spawned_trunk = false;
        let trunk_candidate = match run_before_review_deadline(
            deadline,
            params.external_cancel.as_ref(),
            self.state.lock(),
        )
        .await
        {
            Ok(mut state) => {
                if let Some(trunk) = state.trunk.as_ref()
                    && trunk.reuse_key != next_reuse_key
                    && trunk.review_lock.try_acquire().is_ok()
                {
                    stale_trunk_to_shutdown = state.trunk.take();
                }

                if state.trunk.is_none() {
                    let spawn_cancel_token = self.cancellation_token.child_token();
                    let review_session = match run_before_review_deadline_with_cancel(
                        deadline,
                        params.external_cancel.as_ref(),
                        &spawn_cancel_token,
                        Box::pin(spawn_guardian_review_session(
                            &params.parent_session,
                            &params.parent_turn,
                            params.spawn_config.clone(),
                            next_reuse_key.clone(),
                            spawn_cancel_token.clone(),
                            /*fork_snapshot*/ None,
                        )),
                    )
                    .await
                    {
                        Ok(Ok(review_session)) => Arc::new(review_session),
                        Ok(Err(err)) => {
                            return (
                                GuardianReviewSessionOutcome::PromptBuildFailed(err),
                                GuardianReviewAnalyticsResult::without_session(),
                            );
                        }
                        Err(outcome) => {
                            return (outcome, GuardianReviewAnalyticsResult::without_session());
                        }
                    };
                    state.trunk = Some(Arc::clone(&review_session));
                    spawned_trunk = true;
                }

                state.trunk.as_ref().cloned()
            }
            Err(outcome) => {
                return (outcome, GuardianReviewAnalyticsResult::without_session());
            }
        };

        if let Some(review_session) = stale_trunk_to_shutdown {
            review_session.shutdown_in_background();
        }

        let Some(trunk) = trunk_candidate else {
            return (
                GuardianReviewSessionOutcome::Completed(Err(anyhow!(
                    "guardian review session was not available after spawn"
                ))),
                GuardianReviewAnalyticsResult::without_session(),
            );
        };

        if trunk.reuse_key != next_reuse_key {
            return Box::pin(self.run_ephemeral_review(
                params,
                next_reuse_key,
                deadline,
                /*fork_snapshot*/ None,
            ))
            .await;
        }

        let trunk_guard = match trunk.review_lock.try_acquire() {
            Ok(trunk_guard) => trunk_guard,
            Err(_) => {
                return Box::pin(self.run_ephemeral_review(
                    params,
                    next_reuse_key,
                    deadline,
                    trunk.fork_snapshot().await,
                ))
                .await;
            }
        };

        let guardian_session_kind = if spawned_trunk {
            GuardianReviewSessionKind::TrunkNew
        } else {
            GuardianReviewSessionKind::TrunkReused
        };
        let (outcome, keep_review_session, analytics_result) = Box::pin(run_review_on_session(
            trunk.as_ref(),
            &params,
            guardian_session_kind,
            deadline,
        ))
        .await;
        if keep_review_session && matches!(outcome, GuardianReviewSessionOutcome::Completed(_)) {
            trunk.refresh_last_committed_fork_snapshot().await;
        }
        drop(trunk_guard);

        if keep_review_session {
            (outcome, analytics_result)
        } else {
            if let Some(review_session) = self.remove_trunk_if_current(&trunk).await {
                review_session.shutdown_in_background();
            }
            (outcome, analytics_result)
        }
    }

    #[cfg(test)]
    pub(crate) async fn cache_for_test(&self, session: Arc<Session>, io: SessionIo) {
        let reuse_key = GuardianReviewSessionReuseKey::from_spawn_config(
            session.get_config().await.as_ref(),
            session.user_instructions().await,
        );
        self.state.lock().await.trunk = Some(Arc::new(GuardianReviewSession {
            reuse_key,
            session,
            io,
            cancel_token: CancellationToken::new(),
            review_lock: Semaphore::new(/*permits*/ 1),
            state: Mutex::new(GuardianReviewState {
                prior_review_count: 0,
                last_reviewed_transcript_cursor: None,
                last_committed_fork_snapshot: None,
            }),
        }));
    }

    #[cfg(test)]
    pub(crate) async fn register_ephemeral_for_test(&self, session: Arc<Session>, io: SessionIo) {
        let reuse_key = GuardianReviewSessionReuseKey::from_spawn_config(
            session.get_config().await.as_ref(),
            session.user_instructions().await,
        );
        self.state
            .lock()
            .await
            .ephemeral_reviews
            .push(Arc::new(GuardianReviewSession {
                reuse_key,
                session,
                io,
                cancel_token: CancellationToken::new(),
                review_lock: Semaphore::new(/*permits*/ 1),
                state: Mutex::new(GuardianReviewState {
                    prior_review_count: 0,
                    last_reviewed_transcript_cursor: None,
                    last_committed_fork_snapshot: None,
                }),
            }));
    }

    #[cfg(test)]
    pub(crate) async fn committed_fork_rollout_items_for_test(&self) -> Option<Vec<RolloutItem>> {
        let trunk = self.state.lock().await.trunk.clone()?;
        let state = trunk.state.lock().await;
        let snapshot = state.last_committed_fork_snapshot.as_ref()?;
        match &snapshot.initial_history {
            InitialHistory::Forked(items) => Some(items.clone()),
            InitialHistory::New | InitialHistory::Cleared | InitialHistory::Resumed(_) => None,
        }
    }

    #[cfg(test)]
    pub(crate) async fn send_trunk_event_raw_for_test(&self, event: Event) {
        let trunk = self
            .state
            .lock()
            .await
            .trunk
            .clone()
            .expect("guardian trunk should exist");
        trunk.session.send_event_raw(event).await;
    }

    async fn remove_trunk_if_current(
        &self,
        trunk: &Arc<GuardianReviewSession>,
    ) -> Option<Arc<GuardianReviewSession>> {
        let mut state = self.state.lock().await;
        if state
            .trunk
            .as_ref()
            .is_some_and(|current| Arc::ptr_eq(current, trunk))
        {
            state.trunk.take()
        } else {
            None
        }
    }

    async fn register_active_ephemeral(&self, review_session: Arc<GuardianReviewSession>) {
        self.state
            .lock()
            .await
            .ephemeral_reviews
            .push(review_session);
    }

    async fn take_active_ephemeral(
        &self,
        review_session: &Arc<GuardianReviewSession>,
    ) -> Option<Arc<GuardianReviewSession>> {
        let mut state = self.state.lock().await;
        let ephemeral_review_index = state
            .ephemeral_reviews
            .iter()
            .position(|active_review| Arc::ptr_eq(active_review, review_session))?;
        Some(state.ephemeral_reviews.swap_remove(ephemeral_review_index))
    }

    async fn run_ephemeral_review(
        &self,
        params: GuardianReviewSessionParams,
        reuse_key: GuardianReviewSessionReuseKey,
        deadline: tokio::time::Instant,
        fork_snapshot: Option<GuardianReviewForkSnapshot>,
    ) -> (GuardianReviewSessionOutcome, GuardianReviewAnalyticsResult) {
        let spawn_cancel_token = self.cancellation_token.child_token();
        let mut fork_config = params.spawn_config.clone();
        fork_config.ephemeral = true;
        let review_session = match run_before_review_deadline_with_cancel(
            deadline,
            params.external_cancel.as_ref(),
            &spawn_cancel_token,
            Box::pin(spawn_guardian_review_session(
                &params.parent_session,
                &params.parent_turn,
                fork_config,
                reuse_key,
                spawn_cancel_token.clone(),
                fork_snapshot,
            )),
        )
        .await
        {
            Ok(Ok(review_session)) => Arc::new(review_session),
            Ok(Err(err)) => {
                return (
                    GuardianReviewSessionOutcome::PromptBuildFailed(err),
                    GuardianReviewAnalyticsResult::without_session(),
                );
            }
            Err(outcome) => {
                return (outcome, GuardianReviewAnalyticsResult::without_session());
            }
        };
        self.register_active_ephemeral(Arc::clone(&review_session))
            .await;
        let mut cleanup =
            EphemeralReviewCleanup::new(Arc::clone(&self.state), Arc::clone(&review_session));

        let (outcome, _, analytics_result) = Box::pin(run_review_on_session(
            review_session.as_ref(),
            &params,
            GuardianReviewSessionKind::EphemeralForked,
            deadline,
        ))
        .await;
        if let Some(review_session) = self.take_active_ephemeral(&review_session).await {
            cleanup.disarm();
            review_session.shutdown_in_background();
        }
        (outcome, analytics_result)
    }
}

async fn spawn_guardian_review_session(
    parent_session: &Arc<Session>,
    parent_turn: &Arc<TurnContext>,
    spawn_config: Config,
    reuse_key: GuardianReviewSessionReuseKey,
    cancel_token: CancellationToken,
    fork_snapshot: Option<GuardianReviewForkSnapshot>,
) -> anyhow::Result<GuardianReviewSession> {
    let (initial_history, prior_review_count, initial_transcript_cursor) = match fork_snapshot {
        Some(fork_snapshot) => (
            Some(fork_snapshot.initial_history),
            fork_snapshot.prior_review_count,
            fork_snapshot.last_reviewed_transcript_cursor,
        ),
        None => (None, 0, None),
    };
    let (session, io) = Box::pin(run_codex_thread_interactive(
        spawn_config,
        parent_session.services.auth_manager(),
        parent_session.services.models_manager.clone(),
        Arc::clone(parent_session),
        Arc::clone(parent_turn),
        cancel_token.clone(),
        SubAgentSource::Other(GUARDIAN_REVIEWER_NAME.to_string()),
        initial_history,
        GitEnrichmentPolicy::Skip,
        codex_sandboxing::WindowsSandboxProxySettingsMode::Preserve,
    ))
    .await?;

    Ok(GuardianReviewSession {
        session,
        io,
        cancel_token,
        reuse_key,
        review_lock: Semaphore::new(/*permits*/ 1),
        state: Mutex::new(GuardianReviewState {
            prior_review_count,
            last_reviewed_transcript_cursor: initial_transcript_cursor,
            last_committed_fork_snapshot: None,
        }),
    })
||||||| 413492cd6c

    #[expect(
        clippy::await_holding_invalid_type,
        reason = "review session selection and trunk spawning must stay serialized"
    )]
    pub(super) async fn run_review(
        &self,
        params: GuardianReviewSessionParams,
    ) -> (GuardianReviewSessionOutcome, GuardianReviewAnalyticsResult) {
        let deadline = params.deadline;
        let next_reuse_key = GuardianReviewSessionReuseKey::from_spawn_config(
            &params.spawn_config,
            params.parent_session.user_instructions().await,
        );
        let mut stale_trunk_to_shutdown = None;
        let mut spawned_trunk = false;
        let trunk_candidate = match run_before_review_deadline(
            deadline,
            params.external_cancel.as_ref(),
            self.state.lock(),
        )
        .await
        {
            Ok(mut state) => {
                if let Some(trunk) = state.trunk.as_ref()
                    && trunk.reuse_key != next_reuse_key
                    && trunk.review_lock.try_acquire().is_ok()
                {
                    stale_trunk_to_shutdown = state.trunk.take();
                }

                if state.trunk.is_none() {
                    let spawn_cancel_token = self.cancellation_token.child_token();
                    let review_session = match run_before_review_deadline_with_cancel(
                        deadline,
                        params.external_cancel.as_ref(),
                        &spawn_cancel_token,
                        Box::pin(spawn_guardian_review_session(
                            &params.parent_session,
                            &params.parent_turn,
                            params.spawn_config.clone(),
                            next_reuse_key.clone(),
                            spawn_cancel_token.clone(),
                            /*fork_snapshot*/ None,
                        )),
                    )
                    .await
                    {
                        Ok(Ok(review_session)) => Arc::new(review_session),
                        Ok(Err(err)) => {
                            return (
                                GuardianReviewSessionOutcome::PromptBuildFailed(err),
                                GuardianReviewAnalyticsResult::without_session(),
                            );
                        }
                        Err(outcome) => {
                            return (outcome, GuardianReviewAnalyticsResult::without_session());
                        }
                    };
                    state.trunk = Some(Arc::clone(&review_session));
                    spawned_trunk = true;
                }

                state.trunk.as_ref().cloned()
            }
            Err(outcome) => {
                return (outcome, GuardianReviewAnalyticsResult::without_session());
            }
        };

        if let Some(review_session) = stale_trunk_to_shutdown {
            review_session.shutdown_in_background();
        }

        let Some(trunk) = trunk_candidate else {
            return (
                GuardianReviewSessionOutcome::Completed(Err(anyhow!(
                    "guardian review session was not available after spawn"
                ))),
                GuardianReviewAnalyticsResult::without_session(),
            );
        };

        if trunk.reuse_key != next_reuse_key {
            return Box::pin(self.run_ephemeral_review(
                params,
                next_reuse_key,
                deadline,
                /*fork_snapshot*/ None,
            ))
            .await;
        }

        let trunk_guard = match trunk.review_lock.try_acquire() {
            Ok(trunk_guard) => trunk_guard,
            Err(_) => {
                return Box::pin(self.run_ephemeral_review(
                    params,
                    next_reuse_key,
                    deadline,
                    trunk.fork_snapshot().await,
                ))
                .await;
            }
        };

        let guardian_session_kind = if spawned_trunk {
            GuardianReviewSessionKind::TrunkNew
        } else {
            GuardianReviewSessionKind::TrunkReused
        };
        let (outcome, keep_review_session, analytics_result) = Box::pin(run_review_on_session(
            trunk.as_ref(),
            &params,
            guardian_session_kind,
            deadline,
        ))
        .await;
        if keep_review_session && matches!(outcome, GuardianReviewSessionOutcome::Completed(_)) {
            trunk.refresh_last_committed_fork_snapshot().await;
        }
        drop(trunk_guard);

        if keep_review_session {
            (outcome, analytics_result)
        } else {
            if let Some(review_session) = self.remove_trunk_if_current(&trunk).await {
                review_session.shutdown_in_background();
            }
            (outcome, analytics_result)
        }
    }

    #[cfg(test)]
    pub(crate) async fn cache_for_test(&self, session: Arc<Session>, io: SessionIo) {
        let reuse_key = GuardianReviewSessionReuseKey::from_spawn_config(
            session.get_config().await.as_ref(),
            session.user_instructions().await,
        );
        self.state.lock().await.trunk = Some(Arc::new(GuardianReviewSession {
            reuse_key,
            session,
            io,
            cancel_token: CancellationToken::new(),
            review_lock: Semaphore::new(/*permits*/ 1),
            state: Mutex::new(GuardianReviewState {
                prior_review_count: 0,
                last_reviewed_transcript_cursor: None,
                last_committed_fork_snapshot: None,
            }),
        }));
    }

    #[cfg(test)]
    pub(crate) async fn register_ephemeral_for_test(&self, session: Arc<Session>, io: SessionIo) {
        let reuse_key = GuardianReviewSessionReuseKey::from_spawn_config(
            session.get_config().await.as_ref(),
            session.user_instructions().await,
        );
        self.state
            .lock()
            .await
            .ephemeral_reviews
            .push(Arc::new(GuardianReviewSession {
                reuse_key,
                session,
                io,
                cancel_token: CancellationToken::new(),
                review_lock: Semaphore::new(/*permits*/ 1),
                state: Mutex::new(GuardianReviewState {
                    prior_review_count: 0,
                    last_reviewed_transcript_cursor: None,
                    last_committed_fork_snapshot: None,
                }),
            }));
    }

    #[cfg(test)]
    pub(crate) async fn committed_fork_rollout_items_for_test(&self) -> Option<Vec<RolloutItem>> {
        let trunk = self.state.lock().await.trunk.clone()?;
        let state = trunk.state.lock().await;
        let snapshot = state.last_committed_fork_snapshot.as_ref()?;
        match &snapshot.initial_history {
            InitialHistory::Forked(items) => Some(items.clone()),
            InitialHistory::New | InitialHistory::Cleared | InitialHistory::Resumed(_) => None,
        }
    }

    #[cfg(test)]
    pub(crate) async fn send_trunk_event_raw_for_test(&self, event: Event) {
        let trunk = self
            .state
            .lock()
            .await
            .trunk
            .clone()
            .expect("guardian trunk should exist");
        trunk.session.send_event_raw(event).await;
    }

    async fn remove_trunk_if_current(
        &self,
        trunk: &Arc<GuardianReviewSession>,
    ) -> Option<Arc<GuardianReviewSession>> {
        let mut state = self.state.lock().await;
        if state
            .trunk
            .as_ref()
            .is_some_and(|current| Arc::ptr_eq(current, trunk))
        {
            state.trunk.take()
        } else {
            None
        }
    }

    async fn register_active_ephemeral(&self, review_session: Arc<GuardianReviewSession>) {
        self.state
            .lock()
            .await
            .ephemeral_reviews
            .push(review_session);
    }

    async fn take_active_ephemeral(
        &self,
        review_session: &Arc<GuardianReviewSession>,
    ) -> Option<Arc<GuardianReviewSession>> {
        let mut state = self.state.lock().await;
        let ephemeral_review_index = state
            .ephemeral_reviews
            .iter()
            .position(|active_review| Arc::ptr_eq(active_review, review_session))?;
        Some(state.ephemeral_reviews.swap_remove(ephemeral_review_index))
    }

    async fn run_ephemeral_review(
        &self,
        params: GuardianReviewSessionParams,
        reuse_key: GuardianReviewSessionReuseKey,
        deadline: tokio::time::Instant,
        fork_snapshot: Option<GuardianReviewForkSnapshot>,
    ) -> (GuardianReviewSessionOutcome, GuardianReviewAnalyticsResult) {
        let spawn_cancel_token = self.cancellation_token.child_token();
        let mut fork_config = params.spawn_config.clone();
        fork_config.ephemeral = true;
        let review_session = match run_before_review_deadline_with_cancel(
            deadline,
            params.external_cancel.as_ref(),
            &spawn_cancel_token,
            Box::pin(spawn_guardian_review_session(
                &params.parent_session,
                &params.parent_turn,
                fork_config,
                reuse_key,
                spawn_cancel_token.clone(),
                fork_snapshot,
            )),
        )
        .await
        {
            Ok(Ok(review_session)) => Arc::new(review_session),
            Ok(Err(err)) => {
                return (
                    GuardianReviewSessionOutcome::PromptBuildFailed(err),
                    GuardianReviewAnalyticsResult::without_session(),
                );
            }
            Err(outcome) => {
                return (outcome, GuardianReviewAnalyticsResult::without_session());
            }
        };
        self.register_active_ephemeral(Arc::clone(&review_session))
            .await;
        let mut cleanup =
            EphemeralReviewCleanup::new(Arc::clone(&self.state), Arc::clone(&review_session));

        let (outcome, _, analytics_result) = Box::pin(run_review_on_session(
            review_session.as_ref(),
            &params,
            GuardianReviewSessionKind::EphemeralForked,
            deadline,
        ))
        .await;
        if let Some(review_session) = self.take_active_ephemeral(&review_session).await {
            cleanup.disarm();
            review_session.shutdown_in_background();
        }
        (outcome, analytics_result)
    }
}

async fn spawn_guardian_review_session(
    parent_session: &Arc<Session>,
    parent_turn: &Arc<TurnContext>,
    spawn_config: Config,
    reuse_key: GuardianReviewSessionReuseKey,
    cancel_token: CancellationToken,
    fork_snapshot: Option<GuardianReviewForkSnapshot>,
) -> anyhow::Result<GuardianReviewSession> {
    let (initial_history, prior_review_count, initial_transcript_cursor) = match fork_snapshot {
        Some(fork_snapshot) => (
            Some(fork_snapshot.initial_history),
            fork_snapshot.prior_review_count,
            fork_snapshot.last_reviewed_transcript_cursor,
        ),
        None => (None, 0, None),
    };
    let (session, io) = Box::pin(run_codex_thread_interactive(
        spawn_config,
        parent_session.services.auth_manager.clone(),
        parent_session.services.models_manager.clone(),
        Arc::clone(parent_session),
        Arc::clone(parent_turn),
        cancel_token.clone(),
        SubAgentSource::Other(GUARDIAN_REVIEWER_NAME.to_string()),
        initial_history,
        GitEnrichmentPolicy::Skip,
        codex_sandboxing::WindowsSandboxProxySettingsMode::Preserve,
    ))
    .await?;

    Ok(GuardianReviewSession {
        session,
        io,
        cancel_token,
        reuse_key,
        review_lock: Semaphore::new(/*permits*/ 1),
        state: Mutex::new(GuardianReviewState {
            prior_review_count,
            last_reviewed_transcript_cursor: initial_transcript_cursor,
            last_committed_fork_snapshot: None,
        }),
    })
=======
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
}

async fn run_review_on_session(
    review_session: &GuardianReviewSession,
    params: &GuardianReviewSessionParams,
    guardian_session_kind: GuardianReviewSessionKind,
<<<<<<< HEAD
    deadline: tokio::time::Instant,
) -> (
    GuardianReviewSessionOutcome,
    bool,
    GuardianReviewAnalyticsResult,
) {
    let (send_followup_reminder, prompt_mode) = {
        let state = review_session.state.lock().await;

        let send_followup_reminder = state.prior_review_count == 1;
        let prompt_mode = if state.prior_review_count == 0 {
            GuardianPromptMode::Full
        } else if let Some(cursor) = state.last_reviewed_transcript_cursor {
            GuardianPromptMode::Delta { cursor }
        } else {
            GuardianPromptMode::Full
        };

        (send_followup_reminder, prompt_mode)
    };
    let model_info = crate::model_info_for_provider::model_info_for_config(
        &params.parent_session.services.models_manager,
        params.model.as_str(),
        &params.spawn_config,
    )
    .await;
    let guardian_reasoning_effort = params
||||||| 413492cd6c
    deadline: tokio::time::Instant,
) -> (
    GuardianReviewSessionOutcome,
    bool,
    GuardianReviewAnalyticsResult,
) {
    let (send_followup_reminder, prompt_mode) = {
        let state = review_session.state.lock().await;

        let send_followup_reminder = state.prior_review_count == 1;
        let prompt_mode = if state.prior_review_count == 0 {
            GuardianPromptMode::Full
        } else if let Some(cursor) = state.last_reviewed_transcript_cursor {
            GuardianPromptMode::Delta { cursor }
        } else {
            GuardianPromptMode::Full
        };

        (send_followup_reminder, prompt_mode)
    };
    let model_info = params
        .parent_session
        .services
        .models_manager
        .get_model_info(
            params.model.as_str(),
            &params.spawn_config.to_models_manager_config(),
        )
        .await;
    let guardian_reasoning_effort = params
=======
    deadline: ReviewDeadline,
) -> ReviewSessionResult {
    let review_model = &params.review_model;
    let model_info = params
        .parent_session
        .services
        .models_manager
        .get_model_info(
            review_model.model.as_str(),
            &params.spawn_config.to_models_manager_config(),
        )
        .await;
    let guardian_reasoning_effort = review_model
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        .reasoning_effort
        .clone()
        .or_else(|| model_info.default_reasoning_level.clone());
    let (prior_review_count, had_prior_context) = {
        let state = review_session.state.lock().await;
        (
            state.conversation.completed_review_count(),
            state.conversation.cursor().is_some(),
        )
    };
    let mut analytics_result =
        GuardianReviewAnalyticsResult::from_session(GuardianReviewSessionAnalyticsParams {
            guardian_thread_id: review_session.session.thread_id().to_string(),
            guardian_session_kind,
            guardian_model: review_model.model.clone(),
            guardian_reasoning_effort: guardian_reasoning_effort.map(|effort| effort.to_string()),
            guardian_default_review_model_id: review_model.default_review_model_id.clone(),
            guardian_catalog_contains_auto_review: review_model.catalog_contains_auto_review,
            guardian_review_model_overridden: review_model.model_overridden,
            guardian_review_model_override: review_model.model_override.clone(),
            guardian_model_provider_id: params.spawn_config.model_provider_id.clone(),
            had_prior_review_context: had_prior_context,
        });
    if prior_review_count > 0 {
        ensure_guardian_followup_reminder(review_session).await;
    }

    match run_before_review_deadline(
        deadline,
        params.external_cancel.as_ref(),
        Box::pin(ensure_guardian_node_repl_policy(review_session, params)),
    )
    .await
    {
        Ok(Ok(())) => {}
        Ok(Err(error)) => {
            return ReviewSessionResult {
                outcome: GuardianReviewSessionOutcome::SessionFailed {
                    error,
                    error_info: None,
                    retry_at: None,
                },
                disposition: SessionDisposition::Discard,
                analytics: analytics_result,
            };
        }
        Err(outcome) => {
            return ReviewSessionResult {
                outcome,
                disposition: SessionDisposition::Discard,
                analytics: analytics_result,
            };
        }
    }

    let needs_compaction = params.spawn_config.features.enabled(Feature::TokenBudget)
        && crate::session::context_window::context_window_token_status_for_model(
            review_session.session.as_ref(),
            &params.spawn_config,
            params.parent_context.turn(),
            &model_info,
        )
        .await
        .token_limit_reached;
    let should_compact = if needs_compaction {
        match super::input_budget::should_compact(&review_session.session) {
            Ok(should_compact) => should_compact,
            Err(error) => {
                return ReviewSessionResult {
                    outcome: GuardianReviewSessionOutcome::PromptBuildFailed(error.into()),
                    disposition: SessionDisposition::Discard,
                    analytics: analytics_result,
                };
            }
        }
    } else {
        false
    };
    if should_compact {
        let compact_submission = run_before_review_deadline(
            deadline,
            params.external_cancel.as_ref(),
            review_session.io.submit(Op::Compact),
        )
        .await;
        let compact_turn_id = match compact_submission {
            Ok(Ok(turn_id)) => turn_id,
            Ok(Err(error)) => {
                return ReviewSessionResult {
                    outcome: GuardianReviewSessionOutcome::SessionFailed {
                        error: error.into(),
                        error_info: None,
                        retry_at: None,
                    },
                    disposition: SessionDisposition::Discard,
                    analytics: analytics_result,
                };
            }
            Err(outcome) => {
                return ReviewSessionResult {
                    outcome,
                    disposition: SessionDisposition::Discard,
                    analytics: analytics_result,
                };
            }
        };
        let result = wait_for_guardian_review(
            review_session,
            &compact_turn_id,
            deadline,
            params.external_cancel.as_ref(),
            &mut analytics_result,
        )
        .await;
        if !matches!(
            result.outcome,
            GuardianReviewSessionOutcome::Completed(Ok(_))
        ) {
            return ReviewSessionResult {
                outcome: result.outcome,
                disposition: result.disposition,
                analytics: analytics_result,
            };
        }

        if prior_review_count > 0 {
            ensure_guardian_followup_reminder(review_session).await;
        }
    }

    let reviewer_history = review_session.session.clone_history().await;
    let reviewer_history_version = reviewer_history.history_version();
    let reviewer_has_full_transcript = reviewer_history.raw_items().any(|item| {
        matches!(item, ResponseItem::Message { role, content, .. }
        if role == "user" && content.iter().any(|content| {
            matches!(content, ContentItem::InputText { text }
                if text == GUARDIAN_TRANSCRIPT_START)
        }))
    });
    let (prompt_mode, last_admitted_node_repl_response_sequence, previous_transcript_source) = {
        let mut state = review_session.state.lock().await;
        state.pending_node_repl_evidence_admission = None;
        if state.transcript_history_version != reviewer_history_version
            && let Some(recovery) = review_session
                .session
                .services
                .thread_extension_data
                .get::<super::input_budget::CheckpointRecovery>()
        {
            recovery.requested.store(/*val*/ true, Ordering::Release);
            return ReviewSessionResult {
                outcome: GuardianReviewSessionOutcome::PromptBuildFailed(anyhow::anyhow!(
                    "Guardian history changed; restart from the parent checkpoint"
                )),
                disposition: SessionDisposition::Discard,
                analytics: analytics_result,
            };
        }
        if !reviewer_has_full_transcript {
            state.conversation.reset_transcript();
            state.transcript_source = None;
            state.last_admitted_node_repl_response_sequence = 0;
        }

        let prompt_mode = state
            .conversation
            .cursor()
            .map_or(GuardianPromptMode::Full, |cursor| {
                GuardianPromptMode::Delta { cursor }
            });
        (
            prompt_mode,
            state.last_admitted_node_repl_response_sequence,
            state.transcript_source,
        )
    };
    let prompt_items = run_before_review_deadline(
        deadline,
        params.external_cancel.as_ref(),
        Box::pin(async {
            params
                .parent_session
                .services
                .network_approval
                .sync_session_approved_hosts_to(
                    &review_session.session.services.network_approval,
                    params.parent_session.services.authority_marker(),
                    review_session.session.services.authority_marker(),
                )
                .await;

            let parent_history = params.parent_history.conversation_history_snapshot();
            let policy = ReviewContextPolicy::for_context(
                GuardianContextMode::from_history(parent_history.as_ref()),
                &params.spawn_config.features,
            );
            let loaded_parent_checkpoint = policy
                .parent_compaction(params.parent_history.annotated_items())?
                .as_ref()
                .zip(codex_history::CompactionCheckpoint::latest(
                    reviewer_history.annotated_items(),
                ))
                .is_some_and(|(parent, loaded)| parent == loaded.item);
            // Keep the action-time snapshot: a later parent compaction must not
            // replace evidence with a checkpoint this reviewer never received.
            let (history, transcript_source) = if loaded_parent_checkpoint {
                (
                    GuardianTranscriptHistory::LoadedParentCheckpoint(parent_history.as_ref()),
                    GuardianTranscriptSource::LoadedParentCheckpoint,
                )
            } else {
                (
                    GuardianTranscriptHistory::Retained(parent_history.as_ref()),
                    GuardianTranscriptSource::Retained,
                )
            };
            let prompt_mode = if previous_transcript_source == Some(transcript_source) {
                prompt_mode
            } else {
                GuardianPromptMode::Full
            };
            analytics_result.had_prior_review_context =
                Some(had_prior_review_context(&prompt_mode));
            let prompt_items = prompt_builder::build_guardian_prompt_items(
                params,
                review_session,
                &model_info,
                &reviewer_history,
                history,
                prompt_mode,
                last_admitted_node_repl_response_sequence,
            )
            .await?;

            let items = match prompt_items.context.clone().into_user_inputs() {
                Ok(items) => items,
                Err(codex_guardian_context::SectionError::UnsupportedDelivery {
                    section: "conversation_transcript" | "planned_action",
                }) => {
                    // Final admission replaces this turn-start marker with the complete,
                    // budgeted context, including native encrypted messages and tool calls.
                    vec![codex_protocol::user_input::UserInput::Text {
                        text: super::prompt::GUARDIAN_TRANSCRIPT_START.to_owned(),
                        text_elements: Vec::new(),
                    }]
                }
                Err(error) => return Err(error.into()),
            };
            Ok::<_, anyhow::Error>((prompt_items, items, transcript_source))
        }),
    )
    .await;
    // Release the shared snapshot before recording input so it does not force
    // a copy of the accumulated reviewer history during inference.
    drop(reviewer_history);
    let prompt_items = match prompt_items {
        Ok(prompt_items) => prompt_items,
        Err(outcome) => {
            return ReviewSessionResult {
                outcome,
                disposition: SessionDisposition::Discard,
                analytics: analytics_result,
            };
        }
    };
    let (prompt_items, items, transcript_source) = match prompt_items {
        Ok(prompt_items) => prompt_items,
        Err(err) => {
            return ReviewSessionResult {
                outcome: GuardianReviewSessionOutcome::PromptBuildFailed(err),
                disposition: SessionDisposition::Discard,
                analytics: analytics_result,
            };
        }
    };
    let transcript_cursor = prompt_items.transcript_cursor;
    let node_repl_evidence_admission = (prompt_items.node_repl_evidence_sequence
        > last_admitted_node_repl_response_sequence)
        .then_some(prompt_items.node_repl_evidence_sequence);
    let token_usage_at_review_start = review_session
        .session
        .total_token_usage()
        .await
        .unwrap_or_default();
    let parent_turn_environments = params
        .parent_context
        .environments()
        .turn_environments()
        .map(|environment| {
            let mut selection = environment.selection();
            selection.config = codex_protocol::protocol::EnvironmentConfigState::Ready(
                environment.config().clone(),
            );
            selection
        })
        .collect();
    // TODO(anp): Migrate guardian review thread settings to a PathUri fallback cwd so foreign
    // parent environments do not fall back to the host-native config cwd.
    let parent_turn_legacy_fallback_cwd = params
        .parent_context
        .environments()
        .primary()
        .and_then(|environment| environment.cwd().to_abs_path().ok())
        .unwrap_or_else(|| params.parent_context.turn().config.cwd.clone());

<<<<<<< HEAD
    // The review prompt carries the parent's transcript: it has the parent's
    // standing, never human standing, in the reviewer's history (PF-30-S02).
    review_session
        .session
        .services
        .model_client()
        .mark_agent_input(
            &prompt_items.items,
            params.parent_session.agent_handoff_origin().await,
        );
    let submission = review_session.io.submit_with_trace(
        Op::UserInput {
            items: prompt_items.items,
            final_output_json_schema: Some(params.schema.clone()),
            responsesapi_client_metadata: None,
            additional_context: Default::default(),
            thread_settings: codex_protocol::protocol::ThreadSettingsOverrides {
                environments: Some(codex_protocol::protocol::TurnEnvironmentSelections::new(
                    parent_turn_legacy_fallback_cwd,
                    parent_turn_environments,
                )),
                approval_policy: Some(AskForApproval::Never),
                sandbox_policy: None,
                permission_profile: Some(guardian_permission_profile),
                summary: Some(params.reasoning_summary),
                personality: params.personality,
                collaboration_mode: Some(codex_protocol::config_types::CollaborationMode {
                    mode: codex_protocol::config_types::ModeKind::Default,
                    settings: codex_protocol::config_types::Settings {
                        model: params.model.clone(),
                        reasoning_effort: params.reasoning_effort.clone(),
                        developer_instructions: None,
                    },
                }),
                ..Default::default()
            },
        },
        /*trace*/ None,
        Some(params.parent_turn.sub_id.clone()),
    );
    let submit_result = run_before_review_deadline(
||||||| 413492cd6c
    let submission = review_session.io.submit_with_trace(
        Op::UserInput {
            items: prompt_items.items,
            final_output_json_schema: Some(params.schema.clone()),
            responsesapi_client_metadata: None,
            additional_context: Default::default(),
            thread_settings: codex_protocol::protocol::ThreadSettingsOverrides {
                environments: Some(codex_protocol::protocol::TurnEnvironmentSelections::new(
                    parent_turn_legacy_fallback_cwd,
                    parent_turn_environments,
                )),
                approval_policy: Some(AskForApproval::Never),
                sandbox_policy: None,
                permission_profile: Some(guardian_permission_profile),
                summary: Some(params.reasoning_summary),
                personality: params.personality,
                collaboration_mode: Some(codex_protocol::config_types::CollaborationMode {
                    mode: codex_protocol::config_types::ModeKind::Default,
                    settings: codex_protocol::config_types::Settings {
                        model: params.model.clone(),
                        reasoning_effort: params.reasoning_effort.clone(),
                        developer_instructions: None,
                    },
                }),
                ..Default::default()
            },
        },
        /*trace*/ None,
        Some(params.parent_turn.sub_id.clone()),
    );
    let submit_result = run_before_review_deadline(
=======
    let parent_turn = params.parent_context.turn();
    review_session
        .session
        .services
        .thread_extension_data
        .insert(super::input_budget::PendingReviewContext {
            context: prompt_items.context,
        });
    review_session
        .session
        .services
        .thread_extension_data
        .insert(super::PendingReviewRouting(
            params.parent_context.parent_response.clone(),
        ));
    let request = codex_guardian_reviewer::ReviewerTurn {
        items,
        environments: codex_protocol::protocol::TurnEnvironmentSelections::new(
            parent_turn_legacy_fallback_cwd,
            parent_turn_environments,
        ),
        permission_profile: params.spawn_config.permissions.permission_profile().clone(),
        reasoning_summary: params.reasoning_summary,
        personality: params.personality,
        model: review_model.model.clone(),
        reasoning_effort: review_model.reasoning_effort.clone(),
        cyber_access_program: parent_turn.cyber_access_program,
        parent_response_id: params
            .parent_context
            .parent_response
            .as_ref()
            .map(|parent| parent.id.clone()),
        schema: params.schema.clone(),
        parent_turn_id: parent_turn.sub_id.clone(),
        root_turn_id: parent_turn.turn_metadata_state.root_turn_id(),
    }
    .into_request();
    let child_turn_id = match codex_guardian_reviewer::start_review_turn(
        review_session,
        request,
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        deadline,
        params.external_cancel.as_ref(),
    )
    .await
    {
        Ok(turn_id) => turn_id,
        Err(outcome) => {
            review_session
                .session
                .services
                .thread_extension_data
                .remove::<super::PendingReviewRouting>();
            review_session
                .session
                .services
                .thread_extension_data
                .remove::<super::input_budget::PendingReviewContext>();
            return ReviewSessionResult {
                outcome,
                disposition: SessionDisposition::Discard,
                analytics: analytics_result,
            };
        }
    };
    if let Some(response_sequence) = node_repl_evidence_admission {
        let mut state = review_session.state.lock().await;
        state.pending_node_repl_evidence_admission = Some(PendingNodeReplEvidenceAdmission {
            turn_id: child_turn_id.clone(),
            response_sequence,
        });
    }

    let turn_result = wait_for_guardian_review(
        review_session,
        child_turn_id.as_str(),
        deadline,
        params.external_cancel.as_ref(),
        &mut analytics_result,
    )
    .await;
    review_session
        .session
        .services
        .thread_extension_data
        .remove::<super::PendingReviewRouting>();
    // Finalizing input consumes this context; failures before admission leave it pending.
    let transcript_admitted = review_session
        .session
        .services
        .thread_extension_data
        .remove::<super::input_budget::PendingReviewContext>()
        .is_none();
    let completed_review = matches!(
        turn_result.outcome,
        GuardianReviewSessionOutcome::Completed(_)
    );
    if completed_review
        && turn_result.turn_completed
        && let Some(total_token_usage) = review_session.session.total_token_usage().await
    {
        analytics_result.token_usage = Some(token_usage_delta(
            &token_usage_at_review_start,
            &total_token_usage,
        ));
    }
    if completed_review
        || (transcript_admitted
            && turn_result.turn_completed
            && turn_result.disposition == SessionDisposition::Reusable)
    {
        let mut state = review_session.state.lock().await;
        if completed_review {
            state.conversation.complete_review(transcript_cursor);
        } else {
            // A failed inference still retains its input. Keep admission progress
            // in sync without counting the failure as a completed review.
            state.conversation.admit_transcript(transcript_cursor);
        }
        state.transcript_history_version = reviewer_history_version;
        state.transcript_source = Some(transcript_source);
    }
    let budget_exhausted = review_session
        .session
        .services
        .thread_extension_data
        .remove::<super::request_budget::ExhaustedReviewBudget>();
    let result = match turn_result.outcome {
        GuardianReviewSessionOutcome::SessionFailed {
            error_info: Some(CodexErrorInfo::ContextWindowExceeded),
            ..
        } if matches!(
            budget_exhausted.as_deref(),
            Some(super::request_budget::ExhaustedReviewBudget::Detected)
        ) =>
        {
            GuardianReviewSessionOutcome::InputBudgetExceeded
        }
        result => result,
    };
    ReviewSessionResult {
        outcome: result,
        disposition: if budget_exhausted.is_some() {
            SessionDisposition::Discard
        } else {
            turn_result.disposition
        },
        analytics: analytics_result,
    }
}

async fn ensure_guardian_followup_reminder(review_session: &GuardianReviewSession) {
    let followup_reminder = GuardianFollowupReviewReminder
        .body()
        .into_text()
        .unwrap_or_default();
    let already_injected = review_session
        .session
        .clone_history()
        .await
        .raw_items()
        .any(|item| {
            matches!(item, ResponseItem::Message { role, content, .. }
            if role == "developer"
                && content.iter().any(|content| {
                    matches!(content, ContentItem::InputText { text }
                        if text == &followup_reminder)
                }))
        });
    if already_injected {
        return;
    }

    let reminder: ResponseItem = ContextualUserFragment::into(GuardianFollowupReviewReminder);
    review_session
        .session
        .inject_no_new_turn(vec![reminder], /*current_turn_context*/ None)
        .await;
}

async fn ensure_guardian_node_repl_policy(
    review_session: &GuardianReviewSession,
    params: &GuardianReviewSessionParams,
) -> anyhow::Result<()> {
    if !params
        .parent_context
        .turn()
        .model_info()
        .computer_use_review_required()
        || params.category != GuardianScope::ComputerUse
    {
        return Ok(());
    }

    let policy = &params.node_repl_policy;
    let policy_body = policy.body().into_text().unwrap_or_default();
    if policy_body.is_empty() {
        return Ok(());
    }
    let already_injected = review_session
        .session
        .clone_history()
        .await
        .raw_items()
        .any(|item| {
            matches!(item, ResponseItem::Message { role, content, .. }
            if role == "developer"
                && content.iter().any(|content| {
                    matches!(content, ContentItem::InputText { text } if text == &policy_body)
                }))
        });
    if already_injected {
        return Ok(());
    }

    let turn_context = review_session.session.new_default_turn().await;
    if review_session
        .session
        .reference_context_item()
        .await
        .is_none()
    {
        let initialize_context: BoxFuture<'_, anyhow::Result<()>> = Box::pin(async {
            let step_context = review_session
                .session
                .capture_step_context(Arc::clone(&turn_context), &review_session.cancel_token)
                .await?;
            review_session
                .session
                .record_context_updates_and_set_reference_context_item(step_context.as_ref())
                .await?;
            Ok(())
        });
        initialize_context.await?;
    }

    let item: ResponseItem = ContextualUserFragment::into(policy.clone());
    review_session
        .session
        .inject_client_response_items(vec![item], turn_context.as_ref())
        .await;

    Ok(())
}

impl codex_guardian_reviewer::ReviewerRuntime for GuardianReviewSession {
    async fn submit_turn(&self, request: TurnInputRequest) -> anyhow::Result<TurnInputSubmission> {
        Ok(self
            .io
            .submit_turn_input(request, TurnInputMode::StartIfIdle)
            .await?)
    }

    async fn next_event(&self) -> anyhow::Result<Event> {
        Ok(self.io.next_event().await?)
    }

    async fn admit_context(&self, event: &Event) {
        codex_guardian_reviewer::admit_node_repl_evidence(&self.state, event).await;
    }

    async fn interrupt(&self) -> anyhow::Result<()> {
        self.io.submit(Op::Interrupt).await?;
        Ok(())
    }

<<<<<<< HEAD
    #[tokio::test]
    async fn spawned_guardian_session_preserves_windows_sandbox_proxy_settings() {
        let params = test_review_params().await;
        let manager = GuardianReviewSessionManager::default();
        manager
            .initialize(params.parent_session, params.parent_turn)
            .await
            .expect("initialize Guardian session");
        let mode = manager
            .state
            .lock()
            .await
            .trunk
            .as_ref()
            .expect("Guardian session")
            .session
            .windows_sandbox_proxy_settings_mode;

        assert_eq!(
            mode,
            codex_sandboxing::WindowsSandboxProxySettingsMode::Preserve
        );
        manager.shutdown().await;
    }

    #[tokio::test]
    async fn guardian_review_session_config_change_invalidates_cached_session() {
        let parent_config = crate::config::test_config().await;
        let cached_spawn_config = build_guardian_review_session_config(
            &parent_config,
            /*live_network_config*/ None,
            "active-model",
            /*reasoning_effort*/ None,
            /*model_messages*/ None,
        )
        .expect("cached guardian config");
        let cached_reuse_key = GuardianReviewSessionReuseKey::from_spawn_config(
            &cached_spawn_config,
            /*user_instructions*/ None,
        );

        let mut changed_parent_config = parent_config;
        changed_parent_config.model_provider.base_url =
            Some("https://guardian.example.invalid/v1".to_string());
        let next_spawn_config = build_guardian_review_session_config(
            &changed_parent_config,
            /*live_network_config*/ None,
            "active-model",
            /*reasoning_effort*/ None,
            /*model_messages*/ None,
        )
        .expect("next guardian config");
        let next_reuse_key = GuardianReviewSessionReuseKey::from_spawn_config(
            &next_spawn_config,
            /*user_instructions*/ None,
        );

        assert_eq!(
            cached_reuse_key.cwd,
            PathUri::from_abs_path(&cached_spawn_config.cwd)
        );
        assert_ne!(cached_reuse_key, next_reuse_key);
        assert_eq!(
            cached_reuse_key,
            GuardianReviewSessionReuseKey::from_spawn_config(
                &cached_spawn_config,
                /*user_instructions*/ None,
            )
        );
    }

    #[tokio::test]
    async fn guardian_prompt_cache_key_is_scoped_to_parent_thread() {
        let session_source =
            SessionSource::SubAgent(SubAgentSource::Other(GUARDIAN_REVIEWER_NAME.to_string()));
        let parent_thread_id = ThreadId::new();
        let key =
            prompt_cache_key_override_for_review_session(&session_source, Some(parent_thread_id))
                .expect("guardian prompt cache key");

        assert_eq!(key, format!("guardian:{parent_thread_id}"));
        assert!(
            key.len() <= 64,
            "guardian prompt cache key should fit the Responses API limit"
        );
        assert_eq!(
            key,
            prompt_cache_key_override_for_review_session(&session_source, Some(parent_thread_id))
                .expect("same guardian prompt cache key")
        );
        assert_ne!(
            key,
            prompt_cache_key_override_for_review_session(&session_source, Some(ThreadId::new()))
                .expect("different parent guardian prompt cache key")
        );
        assert_eq!(
            None,
            prompt_cache_key_override_for_review_session(
                &SessionSource::Cli,
                Some(parent_thread_id)
            )
        );
        assert_eq!(
            None,
            prompt_cache_key_override_for_review_session(
                &session_source,
                /*parent_thread_id*/ None
            )
        );
    }

    #[tokio::test]
    async fn guardian_review_session_compact_scope_change_invalidates_cached_session() {
        let parent_config = crate::config::test_config().await;
        let cached_spawn_config = build_guardian_review_session_config(
            &parent_config,
            /*live_network_config*/ None,
            "active-model",
            /*reasoning_effort*/ None,
            /*model_messages*/ None,
        )
        .expect("cached guardian config");
        let cached_reuse_key = GuardianReviewSessionReuseKey::from_spawn_config(
            &cached_spawn_config,
            /*user_instructions*/ None,
        );

        let mut changed_parent_config = parent_config;
        changed_parent_config.model_auto_compact_token_limit_scope =
            AutoCompactTokenLimitScope::BodyAfterPrefix;
        let next_spawn_config = build_guardian_review_session_config(
            &changed_parent_config,
            /*live_network_config*/ None,
            "active-model",
            /*reasoning_effort*/ None,
            /*model_messages*/ None,
        )
        .expect("next guardian config");
        let next_reuse_key = GuardianReviewSessionReuseKey::from_spawn_config(
            &next_spawn_config,
            /*user_instructions*/ None,
        );

        assert_ne!(cached_reuse_key, next_reuse_key);
    }

    #[tokio::test]
    async fn guardian_review_session_config_disables_hooks() {
        let mut parent_config = crate::config::test_config().await;
        parent_config
            .features
            .enable(Feature::CodexHooks)
            .expect("enable hooks on parent config");

        let guardian_config = build_guardian_review_session_config(
            &parent_config,
            /*live_network_config*/ None,
            "active-model",
            /*reasoning_effort*/ None,
            /*model_messages*/ None,
        )
        .expect("guardian config");

        assert!(!guardian_config.features.enabled(Feature::CodexHooks));
    }

    #[tokio::test]
    async fn guardian_review_session_config_disables_skill_instructions() {
        let mut parent_config = crate::config::test_config().await;
        parent_config.include_skill_instructions = true;

        let guardian_config = build_guardian_review_session_config(
            &parent_config,
            /*live_network_config*/ None,
            "active-model",
            /*reasoning_effort*/ None,
            /*model_messages*/ None,
        )
        .expect("guardian config");

        assert!(!guardian_config.include_skill_instructions);
    }

    #[tokio::test]
    async fn guardian_review_session_config_prefers_managed_policy_and_uses_catalog_template() {
        let mut parent_config = crate::config::test_config().await;
        let managed_policy = "Use the managed Guardian policy.";
        let catalog_template = "Catalog Guardian template:\n{{ tenant_policy_config }}";
        parent_config.guardian_policy_config = Some(managed_policy.to_string());
        let model_messages = ModelMessages {
            instructions_template: None,
            instructions_variables: None,
            approvals: None,
            auto_review: Some(AutoReviewMessages {
                policy: Some("Use the catalog Guardian policy.".to_string()),
                policy_template: Some(catalog_template.to_string()),
            }),
            permissions: None,
            token_budget: None,
        };

        let guardian_config = build_guardian_review_session_config(
            &parent_config,
            /*live_network_config*/ None,
            "active-model",
            /*reasoning_effort*/ None,
            Some(&model_messages),
        )
        .expect("guardian config");

        assert_eq!(
            guardian_config.base_instructions,
            Some(guardian_policy_prompt_with_config_and_template(
                managed_policy,
                catalog_template,
            ))
        );
    }

    #[tokio::test]
    async fn guardian_review_session_config_preserves_explicit_empty_catalog_policy() {
        let parent_config = crate::config::test_config().await;
        let model_messages = ModelMessages {
            instructions_template: None,
            instructions_variables: None,
            approvals: None,
            auto_review: Some(AutoReviewMessages {
                policy: Some(String::new()),
                policy_template: None,
            }),
            permissions: None,
            token_budget: None,
        };

        let guardian_config = build_guardian_review_session_config(
            &parent_config,
            /*live_network_config*/ None,
            "active-model",
            /*reasoning_effort*/ None,
            Some(&model_messages),
        )
        .expect("guardian config");

        assert_eq!(
            guardian_config.base_instructions,
            Some(guardian_policy_prompt_with_config_and_template(
                "",
                BUNDLED_GUARDIAN_POLICY_TEMPLATE,
            ))
        );
        assert_ne!(
            guardian_config.base_instructions,
            Some(guardian_policy_prompt_with_config_and_template(
                BUNDLED_GUARDIAN_POLICY,
                BUNDLED_GUARDIAN_POLICY_TEMPLATE,
            ))
        );
    }

    #[tokio::test]
    async fn guardian_review_session_config_preserves_explicit_empty_catalog_template() {
        let parent_config = crate::config::test_config().await;
        let catalog_policy = "Use the catalog Guardian policy.";
        let model_messages = ModelMessages {
            instructions_template: None,
            instructions_variables: None,
            approvals: None,
            auto_review: Some(AutoReviewMessages {
                policy: Some(catalog_policy.to_string()),
                policy_template: Some(String::new()),
            }),
            permissions: None,
            token_budget: None,
        };

        let guardian_config = build_guardian_review_session_config(
            &parent_config,
            /*live_network_config*/ None,
            "active-model",
            /*reasoning_effort*/ None,
            Some(&model_messages),
        )
        .expect("guardian config");

        assert_eq!(
            guardian_config.base_instructions,
            Some(guardian_policy_prompt_with_config_and_template(
                catalog_policy,
                "",
            ))
        );
        assert_ne!(
            guardian_config.base_instructions,
            Some(guardian_policy_prompt_with_config_and_template(
                catalog_policy,
                BUNDLED_GUARDIAN_POLICY_TEMPLATE,
            ))
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn run_before_review_deadline_times_out_before_future_completes() {
        let outcome = run_before_review_deadline(
            tokio::time::Instant::now() + Duration::from_millis(10),
            /*external_cancel*/ None,
            // A future that cannot finish first. A 50 ms sleep raced the 10 ms
            // deadline: after a long Windows scheduler stall both were ready
            // and `select!` could pick the future.
            std::future::pending::<()>(),
        )
        .await;

        assert!(matches!(
            outcome,
            Err(GuardianReviewSessionOutcome::TimedOut)
        ));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn run_before_review_deadline_aborts_when_cancelled() {
        let cancel_token = CancellationToken::new();
        let canceller = cancel_token.clone();
        drop(tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(10)).await;
            canceller.cancel();
        }));

        let outcome = run_before_review_deadline(
            tokio::time::Instant::now() + Duration::from_secs(1),
            Some(&cancel_token),
            std::future::pending::<()>(),
        )
        .await;

        assert!(matches!(
            outcome,
            Err(GuardianReviewSessionOutcome::Aborted)
        ));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn run_before_review_deadline_with_cancel_cancels_token_on_timeout() {
        let cancel_token = CancellationToken::new();

        let outcome = run_before_review_deadline_with_cancel(
            tokio::time::Instant::now() + Duration::from_millis(10),
            /*external_cancel*/ None,
            &cancel_token,
            // A future that cannot finish first. A 50 ms sleep raced the 10 ms
            // deadline: after a long Windows scheduler stall both were ready
            // and `select!` could pick the future.
            std::future::pending::<()>(),
        )
        .await;

        assert!(matches!(
            outcome,
            Err(GuardianReviewSessionOutcome::TimedOut)
        ));
        assert!(cancel_token.is_cancelled());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn run_before_review_deadline_with_cancel_cancels_token_on_abort() {
        let external_cancel = CancellationToken::new();
        let external_canceller = external_cancel.clone();
        let cancel_token = CancellationToken::new();
        drop(tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(10)).await;
            external_canceller.cancel();
        }));

        let outcome = run_before_review_deadline_with_cancel(
            tokio::time::Instant::now() + Duration::from_secs(1),
            Some(&external_cancel),
            &cancel_token,
            std::future::pending::<()>(),
        )
        .await;

        assert!(matches!(
            outcome,
            Err(GuardianReviewSessionOutcome::Aborted)
        ));
        assert!(cancel_token.is_cancelled());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn run_before_review_deadline_with_cancel_preserves_token_on_success() {
        let cancel_token = CancellationToken::new();

        let outcome = run_before_review_deadline_with_cancel(
            tokio::time::Instant::now() + Duration::from_secs(1),
            /*external_cancel*/ None,
            &cancel_token,
            async { 42usize },
        )
        .await;

        assert_eq!(outcome.unwrap(), 42);
        assert!(!cancel_token.is_cancelled());
    }

    #[test]
    fn had_prior_review_context_tracks_prompt_mode() {
        assert!(!had_prior_review_context(&GuardianPromptMode::Full));
        assert!(had_prior_review_context(&GuardianPromptMode::Delta {
            cursor: GuardianTranscriptCursor {
                parent_history_version: 7,
                transcript_entry_count: 42,
            }
        }));
    }

    #[test]
    fn token_usage_delta_never_reports_negative_usage() {
        let start = TokenUsage {
            input_tokens: 10,
            cached_input_tokens: 8,
            cache_write_input_tokens: 8,
            output_tokens: 6,
            reasoning_output_tokens: 4,
            total_tokens: 28,
        };
        let end = TokenUsage {
            input_tokens: 15,
            cached_input_tokens: 7,
            cache_write_input_tokens: 7,
            output_tokens: 10,
            reasoning_output_tokens: 2,
            total_tokens: 34,
        };

        assert_eq!(
            token_usage_delta(&start, &end),
            TokenUsage {
                input_tokens: 5,
                cached_input_tokens: 0,
                cache_write_input_tokens: 0,
                output_tokens: 4,
                reasoning_output_tokens: 0,
                total_tokens: 6,
            }
        );
    }

    #[tokio::test]
    async fn run_review_on_reused_session_waits_for_submitted_turn() {
        let (review_session, tx_event, rx_sub) = test_review_session().await;
        {
            let mut state = review_session.state.lock().await;
            state.prior_review_count = 1;
            state.last_reviewed_transcript_cursor = Some(GuardianTranscriptCursor {
                parent_history_version: 0,
                transcript_entry_count: 0,
            });
        }
        let params = test_review_params().await;

        let review = tokio::spawn(async move {
            run_review_on_session(
                &review_session,
                &params,
                GuardianReviewSessionKind::TrunkReused,
                tokio::time::Instant::now() + Duration::from_secs(1),
            )
            .await
        });
        let submission = rx_sub.recv().await.expect("guardian submission");
        tx_event
            .send(turn_complete_event("prior-turn", Some("stale"), Some(9)))
            .await
            .expect("queue prior turn completion");
        tx_event
            .send(turn_complete_event(
                submission.id.as_str(),
                Some("fresh"),
                Some(42),
            ))
            .await
            .expect("queue submitted turn completion");

        let (outcome, keep_review_session, analytics_result) =
            review.await.expect("review task should complete");
        let GuardianReviewSessionOutcome::Completed(Ok(last_agent_message)) = outcome else {
            panic!("expected submitted turn completion");
        };
        assert_eq!(last_agent_message.as_deref(), Some("fresh"));
        assert_eq!(analytics_result.time_to_first_token_ms, Some(42));
        assert!(keep_review_session);
    }

    #[tokio::test]
    async fn run_review_removes_trunk_when_event_stream_is_broken() {
        let (mut review_session, tx_event, _rx_sub) = test_review_session().await;
        let params = test_review_params().await;
        review_session.reuse_key = GuardianReviewSessionReuseKey::from_spawn_config(
            &params.spawn_config,
            params.parent_session.user_instructions().await,
        );
        let manager = GuardianReviewSessionManager {
            state: Arc::new(Mutex::new(GuardianReviewSessionState {
                trunk: Some(Arc::new(review_session)),
                ephemeral_reviews: Vec::new(),
            })),
            ..Default::default()
        };
        drop(tx_event);

        let (outcome, _) = manager.run_review(params).await;

        assert!(matches!(
            outcome,
            GuardianReviewSessionOutcome::Completed(Err(_))
        ));
        assert!(manager.state.lock().await.trunk.is_none());
    }

    #[tokio::test]
    async fn wait_for_guardian_review_ignores_prior_turn_completion() {
        let (review_session, tx_event, _rx_sub) = test_review_session().await;
        tx_event
            .send(turn_complete_event("prior-turn", Some("stale"), Some(9)))
            .await
            .expect("queue prior turn completion");
        tx_event
            .send(turn_complete_event("current-turn", Some("fresh"), Some(42)))
            .await
            .expect("queue current turn completion");

        let mut analytics_result = GuardianReviewAnalyticsResult::without_session();
        let (outcome, keep_review_session, capture_token_usage) = wait_for_guardian_review(
            &review_session,
            "current-turn",
            tokio::time::Instant::now() + Duration::from_secs(1),
            /*external_cancel*/ None,
            &mut analytics_result,
        )
        .await;

        let GuardianReviewSessionOutcome::Completed(Ok(last_agent_message)) = outcome else {
            panic!("expected current turn completion");
        };
        assert_eq!(last_agent_message.as_deref(), Some("fresh"));
        assert_eq!(analytics_result.time_to_first_token_ms, Some(42));
        assert!(keep_review_session);
        assert!(capture_token_usage);
    }

    #[tokio::test]
    async fn wait_for_guardian_review_ignores_prior_turn_errors() {
        let (review_session, tx_event, _rx_sub) = test_review_session().await;
        tx_event
            .send(Event {
                id: "prior-turn".to_string(),
                msg: EventMsg::Error(ErrorEvent {
                    message: "stale guardian error".to_string(),
                    codex_error_info: None,
                }),
            })
            .await
            .expect("queue prior turn error");
        tx_event
            .send(turn_complete_event(
                "current-turn",
                /*last_agent_message*/ None,
                Some(42),
            ))
            .await
            .expect("queue current turn completion");

        let mut analytics_result = GuardianReviewAnalyticsResult::without_session();
        let (outcome, keep_review_session, capture_token_usage) = wait_for_guardian_review(
            &review_session,
            "current-turn",
            tokio::time::Instant::now() + Duration::from_secs(1),
            /*external_cancel*/ None,
            &mut analytics_result,
        )
        .await;

        let GuardianReviewSessionOutcome::Completed(Ok(last_agent_message)) = outcome else {
            panic!("expected current turn completion");
        };
        assert_eq!(last_agent_message, None);
        assert_eq!(analytics_result.time_to_first_token_ms, Some(42));
        assert!(keep_review_session);
        assert!(capture_token_usage);
    }

    #[tokio::test]
    async fn wait_for_guardian_review_preserves_structured_session_error() {
        let (review_session, tx_event, _rx_sub) = test_review_session().await;
        tx_event
            .send(Event {
                id: "current-turn".to_string(),
                msg: EventMsg::Error(ErrorEvent {
                    message: "temporary failure".to_string(),
                    codex_error_info: Some(CodexErrorInfo::ServerOverloaded),
                }),
            })
            .await
            .expect("queue guardian error");
        tx_event
            .send(turn_complete_event(
                "current-turn",
                /*last_agent_message*/ None,
                Some(42),
            ))
            .await
            .expect("queue current turn completion");

        let mut analytics_result = GuardianReviewAnalyticsResult::without_session();
        let (outcome, keep_review_session, capture_token_usage) = wait_for_guardian_review(
            &review_session,
            "current-turn",
            tokio::time::Instant::now() + Duration::from_secs(1),
            /*external_cancel*/ None,
            &mut analytics_result,
        )
        .await;

        let GuardianReviewSessionOutcome::SessionFailed { error, error_info } = outcome else {
            panic!("expected structured session failure");
        };
        assert_eq!(error.to_string(), "temporary failure");
        assert_eq!(error_info, Some(CodexErrorInfo::ServerOverloaded));
        assert!(keep_review_session);
        assert!(capture_token_usage);
    }

    #[tokio::test]
    async fn wait_for_guardian_review_ignores_prior_turn_aborts() {
        let (review_session, tx_event, _rx_sub) = test_review_session().await;
        tx_event
            .send(turn_aborted_event("prior-turn"))
            .await
            .expect("queue prior turn abort");
        tx_event
            .send(turn_complete_event("current-turn", Some("fresh"), Some(42)))
            .await
            .expect("queue current turn completion");

        let mut analytics_result = GuardianReviewAnalyticsResult::without_session();
        let (outcome, keep_review_session, capture_token_usage) = wait_for_guardian_review(
            &review_session,
            "current-turn",
            tokio::time::Instant::now() + Duration::from_secs(1),
            /*external_cancel*/ None,
            &mut analytics_result,
        )
        .await;

        let GuardianReviewSessionOutcome::Completed(Ok(last_agent_message)) = outcome else {
            panic!("expected current turn completion");
        };
        assert_eq!(last_agent_message.as_deref(), Some("fresh"));
        assert_eq!(analytics_result.time_to_first_token_ms, Some(42));
        assert!(keep_review_session);
        assert!(capture_token_usage);
    }

    #[tokio::test]
    async fn wait_for_guardian_review_timeout_drains_expected_turn_after_stale_terminal_event() {
        let (review_session, tx_event, rx_sub) = test_review_session().await;
        tx_event
            .send(turn_complete_event("prior-turn", Some("stale"), Some(9)))
            .await
            .expect("queue prior turn completion");
        let tx_interrupt_event = tx_event.clone();
        let interrupt_response = tokio::spawn(async move {
            let submission = rx_sub.recv().await.expect("interrupt submission");
            assert!(matches!(submission.op, Op::Interrupt));
            tx_interrupt_event
                .send(turn_aborted_event("current-turn"))
                .await
                .expect("queue current turn abort");
        });

        let mut analytics_result = GuardianReviewAnalyticsResult::without_session();
        let (outcome, keep_review_session, capture_token_usage) = wait_for_guardian_review(
            &review_session,
            "current-turn",
            tokio::time::Instant::now() + Duration::from_millis(10),
            /*external_cancel*/ None,
            &mut analytics_result,
        )
        .await;

        interrupt_response
            .await
            .expect("interrupt response task should complete");
        assert!(matches!(outcome, GuardianReviewSessionOutcome::TimedOut));
        assert!(keep_review_session);
        assert!(!capture_token_usage);
    }

    #[tokio::test]
    async fn wait_for_guardian_review_cancel_drains_expected_turn_after_stale_terminal_event() {
        let (review_session, tx_event, rx_sub) = test_review_session().await;
        tx_event
            .send(turn_complete_event("prior-turn", Some("stale"), Some(9)))
            .await
            .expect("queue prior turn completion");
        let tx_interrupt_event = tx_event.clone();
        let interrupt_response = tokio::spawn(async move {
            let submission = rx_sub.recv().await.expect("interrupt submission");
            assert!(matches!(submission.op, Op::Interrupt));
            tx_interrupt_event
                .send(turn_aborted_event("current-turn"))
                .await
                .expect("queue current turn abort");
        });
        let external_cancel = CancellationToken::new();
        external_cancel.cancel();

        let mut analytics_result = GuardianReviewAnalyticsResult::without_session();
        let (outcome, keep_review_session, capture_token_usage) = wait_for_guardian_review(
            &review_session,
            "current-turn",
            tokio::time::Instant::now() + Duration::from_secs(1),
            Some(&external_cancel),
            &mut analytics_result,
        )
        .await;

        interrupt_response
            .await
            .expect("interrupt response task should complete");
        assert!(matches!(outcome, GuardianReviewSessionOutcome::Aborted));
        assert!(keep_review_session);
        assert!(!capture_token_usage);
    }

    #[tokio::test]
    async fn interrupt_and_drain_turn_ignores_prior_turn_completion() {
        let (review_session, tx_event, _rx_sub) = test_review_session().await;
        tx_event
            .send(turn_complete_event("prior-turn", Some("stale"), Some(9)))
            .await
            .expect("queue prior turn completion");
        tx_event
            .send(turn_aborted_event("current-turn"))
            .await
            .expect("queue current turn abort");

        interrupt_and_drain_turn(&review_session.io, "current-turn")
            .await
            .expect("drain current turn");

        assert!(review_session.io.rx_event.try_recv().is_err());
||||||| 413492cd6c
    #[tokio::test]
    async fn spawned_guardian_session_preserves_windows_sandbox_proxy_settings() {
        let params = test_review_params().await;
        let manager = GuardianReviewSessionManager::default();
        manager
            .initialize(params.parent_session, params.parent_turn)
            .await
            .expect("initialize Guardian session");
        let mode = manager
            .state
            .lock()
            .await
            .trunk
            .as_ref()
            .expect("Guardian session")
            .session
            .windows_sandbox_proxy_settings_mode;

        assert_eq!(
            mode,
            codex_sandboxing::WindowsSandboxProxySettingsMode::Preserve
        );
        manager.shutdown().await;
    }

    #[tokio::test]
    async fn guardian_review_session_config_change_invalidates_cached_session() {
        let parent_config = crate::config::test_config().await;
        let cached_spawn_config = build_guardian_review_session_config(
            &parent_config,
            /*live_network_config*/ None,
            "active-model",
            /*reasoning_effort*/ None,
            /*model_messages*/ None,
        )
        .expect("cached guardian config");
        let cached_reuse_key = GuardianReviewSessionReuseKey::from_spawn_config(
            &cached_spawn_config,
            /*user_instructions*/ None,
        );

        let mut changed_parent_config = parent_config;
        changed_parent_config.model_provider.base_url =
            Some("https://guardian.example.invalid/v1".to_string());
        let next_spawn_config = build_guardian_review_session_config(
            &changed_parent_config,
            /*live_network_config*/ None,
            "active-model",
            /*reasoning_effort*/ None,
            /*model_messages*/ None,
        )
        .expect("next guardian config");
        let next_reuse_key = GuardianReviewSessionReuseKey::from_spawn_config(
            &next_spawn_config,
            /*user_instructions*/ None,
        );

        assert_eq!(
            cached_reuse_key.cwd,
            PathUri::from_abs_path(&cached_spawn_config.cwd)
        );
        assert_ne!(cached_reuse_key, next_reuse_key);
        assert_eq!(
            cached_reuse_key,
            GuardianReviewSessionReuseKey::from_spawn_config(
                &cached_spawn_config,
                /*user_instructions*/ None,
            )
        );
    }

    #[tokio::test]
    async fn guardian_prompt_cache_key_is_scoped_to_parent_thread() {
        let session_source =
            SessionSource::SubAgent(SubAgentSource::Other(GUARDIAN_REVIEWER_NAME.to_string()));
        let parent_thread_id = ThreadId::new();
        let key =
            prompt_cache_key_override_for_review_session(&session_source, Some(parent_thread_id))
                .expect("guardian prompt cache key");

        assert_eq!(key, format!("guardian:{parent_thread_id}"));
        assert!(
            key.len() <= 64,
            "guardian prompt cache key should fit the Responses API limit"
        );
        assert_eq!(
            key,
            prompt_cache_key_override_for_review_session(&session_source, Some(parent_thread_id))
                .expect("same guardian prompt cache key")
        );
        assert_ne!(
            key,
            prompt_cache_key_override_for_review_session(&session_source, Some(ThreadId::new()))
                .expect("different parent guardian prompt cache key")
        );
        assert_eq!(
            None,
            prompt_cache_key_override_for_review_session(
                &SessionSource::Cli,
                Some(parent_thread_id)
            )
        );
        assert_eq!(
            None,
            prompt_cache_key_override_for_review_session(
                &session_source,
                /*parent_thread_id*/ None
            )
        );
    }

    #[tokio::test]
    async fn guardian_review_session_compact_scope_change_invalidates_cached_session() {
        let parent_config = crate::config::test_config().await;
        let cached_spawn_config = build_guardian_review_session_config(
            &parent_config,
            /*live_network_config*/ None,
            "active-model",
            /*reasoning_effort*/ None,
            /*model_messages*/ None,
        )
        .expect("cached guardian config");
        let cached_reuse_key = GuardianReviewSessionReuseKey::from_spawn_config(
            &cached_spawn_config,
            /*user_instructions*/ None,
        );

        let mut changed_parent_config = parent_config;
        changed_parent_config.model_auto_compact_token_limit_scope =
            AutoCompactTokenLimitScope::BodyAfterPrefix;
        let next_spawn_config = build_guardian_review_session_config(
            &changed_parent_config,
            /*live_network_config*/ None,
            "active-model",
            /*reasoning_effort*/ None,
            /*model_messages*/ None,
        )
        .expect("next guardian config");
        let next_reuse_key = GuardianReviewSessionReuseKey::from_spawn_config(
            &next_spawn_config,
            /*user_instructions*/ None,
        );

        assert_ne!(cached_reuse_key, next_reuse_key);
    }

    #[tokio::test]
    async fn guardian_review_session_config_disables_hooks() {
        let mut parent_config = crate::config::test_config().await;
        parent_config
            .features
            .enable(Feature::CodexHooks)
            .expect("enable hooks on parent config");

        let guardian_config = build_guardian_review_session_config(
            &parent_config,
            /*live_network_config*/ None,
            "active-model",
            /*reasoning_effort*/ None,
            /*model_messages*/ None,
        )
        .expect("guardian config");

        assert!(!guardian_config.features.enabled(Feature::CodexHooks));
    }

    #[tokio::test]
    async fn guardian_review_session_config_disables_skill_instructions() {
        let mut parent_config = crate::config::test_config().await;
        parent_config.include_skill_instructions = true;

        let guardian_config = build_guardian_review_session_config(
            &parent_config,
            /*live_network_config*/ None,
            "active-model",
            /*reasoning_effort*/ None,
            /*model_messages*/ None,
        )
        .expect("guardian config");

        assert!(!guardian_config.include_skill_instructions);
    }

    #[tokio::test]
    async fn guardian_review_session_config_prefers_managed_policy_and_uses_catalog_template() {
        let mut parent_config = crate::config::test_config().await;
        let managed_policy = "Use the managed Guardian policy.";
        let catalog_template = "Catalog Guardian template:\n{{ tenant_policy_config }}";
        parent_config.guardian_policy_config = Some(managed_policy.to_string());
        let model_messages = ModelMessages {
            instructions_template: None,
            instructions_variables: None,
            approvals: None,
            auto_review: Some(AutoReviewMessages {
                policy: Some("Use the catalog Guardian policy.".to_string()),
                policy_template: Some(catalog_template.to_string()),
            }),
            permissions: None,
            token_budget: None,
        };

        let guardian_config = build_guardian_review_session_config(
            &parent_config,
            /*live_network_config*/ None,
            "active-model",
            /*reasoning_effort*/ None,
            Some(&model_messages),
        )
        .expect("guardian config");

        assert_eq!(
            guardian_config.base_instructions,
            Some(guardian_policy_prompt_with_config_and_template(
                managed_policy,
                catalog_template,
            ))
        );
    }

    #[tokio::test]
    async fn guardian_review_session_config_preserves_explicit_empty_catalog_policy() {
        let parent_config = crate::config::test_config().await;
        let model_messages = ModelMessages {
            instructions_template: None,
            instructions_variables: None,
            approvals: None,
            auto_review: Some(AutoReviewMessages {
                policy: Some(String::new()),
                policy_template: None,
            }),
            permissions: None,
            token_budget: None,
        };

        let guardian_config = build_guardian_review_session_config(
            &parent_config,
            /*live_network_config*/ None,
            "active-model",
            /*reasoning_effort*/ None,
            Some(&model_messages),
        )
        .expect("guardian config");

        assert_eq!(
            guardian_config.base_instructions,
            Some(guardian_policy_prompt_with_config_and_template(
                "",
                BUNDLED_GUARDIAN_POLICY_TEMPLATE,
            ))
        );
        assert_ne!(
            guardian_config.base_instructions,
            Some(guardian_policy_prompt_with_config_and_template(
                BUNDLED_GUARDIAN_POLICY,
                BUNDLED_GUARDIAN_POLICY_TEMPLATE,
            ))
        );
    }

    #[tokio::test]
    async fn guardian_review_session_config_preserves_explicit_empty_catalog_template() {
        let parent_config = crate::config::test_config().await;
        let catalog_policy = "Use the catalog Guardian policy.";
        let model_messages = ModelMessages {
            instructions_template: None,
            instructions_variables: None,
            approvals: None,
            auto_review: Some(AutoReviewMessages {
                policy: Some(catalog_policy.to_string()),
                policy_template: Some(String::new()),
            }),
            permissions: None,
            token_budget: None,
        };

        let guardian_config = build_guardian_review_session_config(
            &parent_config,
            /*live_network_config*/ None,
            "active-model",
            /*reasoning_effort*/ None,
            Some(&model_messages),
        )
        .expect("guardian config");

        assert_eq!(
            guardian_config.base_instructions,
            Some(guardian_policy_prompt_with_config_and_template(
                catalog_policy,
                "",
            ))
        );
        assert_ne!(
            guardian_config.base_instructions,
            Some(guardian_policy_prompt_with_config_and_template(
                catalog_policy,
                BUNDLED_GUARDIAN_POLICY_TEMPLATE,
            ))
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn run_before_review_deadline_times_out_before_future_completes() {
        let outcome = run_before_review_deadline(
            tokio::time::Instant::now() + Duration::from_millis(10),
            /*external_cancel*/ None,
            async {
                tokio::time::sleep(Duration::from_millis(50)).await;
            },
        )
        .await;

        assert!(matches!(
            outcome,
            Err(GuardianReviewSessionOutcome::TimedOut)
        ));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn run_before_review_deadline_aborts_when_cancelled() {
        let cancel_token = CancellationToken::new();
        let canceller = cancel_token.clone();
        drop(tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(10)).await;
            canceller.cancel();
        }));

        let outcome = run_before_review_deadline(
            tokio::time::Instant::now() + Duration::from_secs(1),
            Some(&cancel_token),
            std::future::pending::<()>(),
        )
        .await;

        assert!(matches!(
            outcome,
            Err(GuardianReviewSessionOutcome::Aborted)
        ));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn run_before_review_deadline_with_cancel_cancels_token_on_timeout() {
        let cancel_token = CancellationToken::new();

        let outcome = run_before_review_deadline_with_cancel(
            tokio::time::Instant::now() + Duration::from_millis(10),
            /*external_cancel*/ None,
            &cancel_token,
            async {
                tokio::time::sleep(Duration::from_millis(50)).await;
            },
        )
        .await;

        assert!(matches!(
            outcome,
            Err(GuardianReviewSessionOutcome::TimedOut)
        ));
        assert!(cancel_token.is_cancelled());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn run_before_review_deadline_with_cancel_cancels_token_on_abort() {
        let external_cancel = CancellationToken::new();
        let external_canceller = external_cancel.clone();
        let cancel_token = CancellationToken::new();
        drop(tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(10)).await;
            external_canceller.cancel();
        }));

        let outcome = run_before_review_deadline_with_cancel(
            tokio::time::Instant::now() + Duration::from_secs(1),
            Some(&external_cancel),
            &cancel_token,
            std::future::pending::<()>(),
        )
        .await;

        assert!(matches!(
            outcome,
            Err(GuardianReviewSessionOutcome::Aborted)
        ));
        assert!(cancel_token.is_cancelled());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn run_before_review_deadline_with_cancel_preserves_token_on_success() {
        let cancel_token = CancellationToken::new();

        let outcome = run_before_review_deadline_with_cancel(
            tokio::time::Instant::now() + Duration::from_secs(1),
            /*external_cancel*/ None,
            &cancel_token,
            async { 42usize },
        )
        .await;

        assert_eq!(outcome.unwrap(), 42);
        assert!(!cancel_token.is_cancelled());
    }

    #[test]
    fn had_prior_review_context_tracks_prompt_mode() {
        assert!(!had_prior_review_context(&GuardianPromptMode::Full));
        assert!(had_prior_review_context(&GuardianPromptMode::Delta {
            cursor: GuardianTranscriptCursor {
                parent_history_version: 7,
                transcript_entry_count: 42,
            }
        }));
    }

    #[test]
    fn token_usage_delta_never_reports_negative_usage() {
        let start = TokenUsage {
            input_tokens: 10,
            cached_input_tokens: 8,
            cache_write_input_tokens: 8,
            output_tokens: 6,
            reasoning_output_tokens: 4,
            total_tokens: 28,
        };
        let end = TokenUsage {
            input_tokens: 15,
            cached_input_tokens: 7,
            cache_write_input_tokens: 7,
            output_tokens: 10,
            reasoning_output_tokens: 2,
            total_tokens: 34,
        };

        assert_eq!(
            token_usage_delta(&start, &end),
            TokenUsage {
                input_tokens: 5,
                cached_input_tokens: 0,
                cache_write_input_tokens: 0,
                output_tokens: 4,
                reasoning_output_tokens: 0,
                total_tokens: 6,
            }
        );
    }

    #[tokio::test]
    async fn run_review_on_reused_session_waits_for_submitted_turn() {
        let (review_session, tx_event, rx_sub) = test_review_session().await;
        {
            let mut state = review_session.state.lock().await;
            state.prior_review_count = 1;
            state.last_reviewed_transcript_cursor = Some(GuardianTranscriptCursor {
                parent_history_version: 0,
                transcript_entry_count: 0,
            });
        }
        let params = test_review_params().await;

        let review = tokio::spawn(async move {
            run_review_on_session(
                &review_session,
                &params,
                GuardianReviewSessionKind::TrunkReused,
                tokio::time::Instant::now() + Duration::from_secs(1),
            )
            .await
        });
        let submission = rx_sub.recv().await.expect("guardian submission");
        tx_event
            .send(turn_complete_event("prior-turn", Some("stale"), Some(9)))
            .await
            .expect("queue prior turn completion");
        tx_event
            .send(turn_complete_event(
                submission.id.as_str(),
                Some("fresh"),
                Some(42),
            ))
            .await
            .expect("queue submitted turn completion");

        let (outcome, keep_review_session, analytics_result) =
            review.await.expect("review task should complete");
        let GuardianReviewSessionOutcome::Completed(Ok(last_agent_message)) = outcome else {
            panic!("expected submitted turn completion");
        };
        assert_eq!(last_agent_message.as_deref(), Some("fresh"));
        assert_eq!(analytics_result.time_to_first_token_ms, Some(42));
        assert!(keep_review_session);
    }

    #[tokio::test]
    async fn run_review_removes_trunk_when_event_stream_is_broken() {
        let (mut review_session, tx_event, _rx_sub) = test_review_session().await;
        let params = test_review_params().await;
        review_session.reuse_key = GuardianReviewSessionReuseKey::from_spawn_config(
            &params.spawn_config,
            params.parent_session.user_instructions().await,
        );
        let manager = GuardianReviewSessionManager {
            state: Arc::new(Mutex::new(GuardianReviewSessionState {
                trunk: Some(Arc::new(review_session)),
                ephemeral_reviews: Vec::new(),
            })),
            ..Default::default()
        };
        drop(tx_event);

        let (outcome, _) = manager.run_review(params).await;

        assert!(matches!(
            outcome,
            GuardianReviewSessionOutcome::Completed(Err(_))
        ));
        assert!(manager.state.lock().await.trunk.is_none());
    }

    #[tokio::test]
    async fn wait_for_guardian_review_ignores_prior_turn_completion() {
        let (review_session, tx_event, _rx_sub) = test_review_session().await;
        tx_event
            .send(turn_complete_event("prior-turn", Some("stale"), Some(9)))
            .await
            .expect("queue prior turn completion");
        tx_event
            .send(turn_complete_event("current-turn", Some("fresh"), Some(42)))
            .await
            .expect("queue current turn completion");

        let mut analytics_result = GuardianReviewAnalyticsResult::without_session();
        let (outcome, keep_review_session, capture_token_usage) = wait_for_guardian_review(
            &review_session,
            "current-turn",
            tokio::time::Instant::now() + Duration::from_secs(1),
            /*external_cancel*/ None,
            &mut analytics_result,
        )
        .await;

        let GuardianReviewSessionOutcome::Completed(Ok(last_agent_message)) = outcome else {
            panic!("expected current turn completion");
        };
        assert_eq!(last_agent_message.as_deref(), Some("fresh"));
        assert_eq!(analytics_result.time_to_first_token_ms, Some(42));
        assert!(keep_review_session);
        assert!(capture_token_usage);
    }

    #[tokio::test]
    async fn wait_for_guardian_review_ignores_prior_turn_errors() {
        let (review_session, tx_event, _rx_sub) = test_review_session().await;
        tx_event
            .send(Event {
                id: "prior-turn".to_string(),
                msg: EventMsg::Error(ErrorEvent {
                    message: "stale guardian error".to_string(),
                    codex_error_info: None,
                }),
            })
            .await
            .expect("queue prior turn error");
        tx_event
            .send(turn_complete_event(
                "current-turn",
                /*last_agent_message*/ None,
                Some(42),
            ))
            .await
            .expect("queue current turn completion");

        let mut analytics_result = GuardianReviewAnalyticsResult::without_session();
        let (outcome, keep_review_session, capture_token_usage) = wait_for_guardian_review(
            &review_session,
            "current-turn",
            tokio::time::Instant::now() + Duration::from_secs(1),
            /*external_cancel*/ None,
            &mut analytics_result,
        )
        .await;

        let GuardianReviewSessionOutcome::Completed(Ok(last_agent_message)) = outcome else {
            panic!("expected current turn completion");
        };
        assert_eq!(last_agent_message, None);
        assert_eq!(analytics_result.time_to_first_token_ms, Some(42));
        assert!(keep_review_session);
        assert!(capture_token_usage);
    }

    #[tokio::test]
    async fn wait_for_guardian_review_preserves_structured_session_error() {
        let (review_session, tx_event, _rx_sub) = test_review_session().await;
        tx_event
            .send(Event {
                id: "current-turn".to_string(),
                msg: EventMsg::Error(ErrorEvent {
                    message: "temporary failure".to_string(),
                    codex_error_info: Some(CodexErrorInfo::ServerOverloaded),
                }),
            })
            .await
            .expect("queue guardian error");
        tx_event
            .send(turn_complete_event(
                "current-turn",
                /*last_agent_message*/ None,
                Some(42),
            ))
            .await
            .expect("queue current turn completion");

        let mut analytics_result = GuardianReviewAnalyticsResult::without_session();
        let (outcome, keep_review_session, capture_token_usage) = wait_for_guardian_review(
            &review_session,
            "current-turn",
            tokio::time::Instant::now() + Duration::from_secs(1),
            /*external_cancel*/ None,
            &mut analytics_result,
        )
        .await;

        let GuardianReviewSessionOutcome::SessionFailed { error, error_info } = outcome else {
            panic!("expected structured session failure");
        };
        assert_eq!(error.to_string(), "temporary failure");
        assert_eq!(error_info, Some(CodexErrorInfo::ServerOverloaded));
        assert!(keep_review_session);
        assert!(capture_token_usage);
    }

    #[tokio::test]
    async fn wait_for_guardian_review_ignores_prior_turn_aborts() {
        let (review_session, tx_event, _rx_sub) = test_review_session().await;
        tx_event
            .send(turn_aborted_event("prior-turn"))
            .await
            .expect("queue prior turn abort");
        tx_event
            .send(turn_complete_event("current-turn", Some("fresh"), Some(42)))
            .await
            .expect("queue current turn completion");

        let mut analytics_result = GuardianReviewAnalyticsResult::without_session();
        let (outcome, keep_review_session, capture_token_usage) = wait_for_guardian_review(
            &review_session,
            "current-turn",
            tokio::time::Instant::now() + Duration::from_secs(1),
            /*external_cancel*/ None,
            &mut analytics_result,
        )
        .await;

        let GuardianReviewSessionOutcome::Completed(Ok(last_agent_message)) = outcome else {
            panic!("expected current turn completion");
        };
        assert_eq!(last_agent_message.as_deref(), Some("fresh"));
        assert_eq!(analytics_result.time_to_first_token_ms, Some(42));
        assert!(keep_review_session);
        assert!(capture_token_usage);
    }

    #[tokio::test]
    async fn wait_for_guardian_review_timeout_drains_expected_turn_after_stale_terminal_event() {
        let (review_session, tx_event, rx_sub) = test_review_session().await;
        tx_event
            .send(turn_complete_event("prior-turn", Some("stale"), Some(9)))
            .await
            .expect("queue prior turn completion");
        let tx_interrupt_event = tx_event.clone();
        let interrupt_response = tokio::spawn(async move {
            let submission = rx_sub.recv().await.expect("interrupt submission");
            assert!(matches!(submission.op, Op::Interrupt));
            tx_interrupt_event
                .send(turn_aborted_event("current-turn"))
                .await
                .expect("queue current turn abort");
        });

        let mut analytics_result = GuardianReviewAnalyticsResult::without_session();
        let (outcome, keep_review_session, capture_token_usage) = wait_for_guardian_review(
            &review_session,
            "current-turn",
            tokio::time::Instant::now() + Duration::from_millis(10),
            /*external_cancel*/ None,
            &mut analytics_result,
        )
        .await;

        interrupt_response
            .await
            .expect("interrupt response task should complete");
        assert!(matches!(outcome, GuardianReviewSessionOutcome::TimedOut));
        assert!(keep_review_session);
        assert!(!capture_token_usage);
    }

    #[tokio::test]
    async fn wait_for_guardian_review_cancel_drains_expected_turn_after_stale_terminal_event() {
        let (review_session, tx_event, rx_sub) = test_review_session().await;
        tx_event
            .send(turn_complete_event("prior-turn", Some("stale"), Some(9)))
            .await
            .expect("queue prior turn completion");
        let tx_interrupt_event = tx_event.clone();
        let interrupt_response = tokio::spawn(async move {
            let submission = rx_sub.recv().await.expect("interrupt submission");
            assert!(matches!(submission.op, Op::Interrupt));
            tx_interrupt_event
                .send(turn_aborted_event("current-turn"))
                .await
                .expect("queue current turn abort");
        });
        let external_cancel = CancellationToken::new();
        external_cancel.cancel();

        let mut analytics_result = GuardianReviewAnalyticsResult::without_session();
        let (outcome, keep_review_session, capture_token_usage) = wait_for_guardian_review(
            &review_session,
            "current-turn",
            tokio::time::Instant::now() + Duration::from_secs(1),
            Some(&external_cancel),
            &mut analytics_result,
        )
        .await;

        interrupt_response
            .await
            .expect("interrupt response task should complete");
        assert!(matches!(outcome, GuardianReviewSessionOutcome::Aborted));
        assert!(keep_review_session);
        assert!(!capture_token_usage);
    }

    #[tokio::test]
    async fn interrupt_and_drain_turn_ignores_prior_turn_completion() {
        let (review_session, tx_event, _rx_sub) = test_review_session().await;
        tx_event
            .send(turn_complete_event("prior-turn", Some("stale"), Some(9)))
            .await
            .expect("queue prior turn completion");
        tx_event
            .send(turn_aborted_event("current-turn"))
            .await
            .expect("queue current turn abort");

        interrupt_and_drain_turn(&review_session.io, "current-turn")
            .await
            .expect("drain current turn");

        assert!(review_session.io.rx_event.try_recv().is_err());
=======
    fn retry_at(&self, turn_id: &str) -> Option<tokio::time::Instant> {
        self.session
            .services
            .thread_extension_data
            .get::<crate::responses_retry::ExhaustedResponseRetry>()
            .filter(|advice| advice.turn_id == turn_id)
            .and_then(|advice| advice.retry_at)
    }
}

#[cfg(test)]
#[path = "review_session_tests.rs"]
mod tests;

impl codex_guardian_reviewer::ReviewerSession for GuardianReviewSession {
    type Setup = PreparedGuardianContext;
    type Context = GuardianReviewSessionReuseKey;
    type Snapshot = GuardianReviewForkSnapshot;

    fn context(&self) -> &Self::Context {
        &self.reuse_key
    }
    async fn snapshot(&self) -> Option<GuardianReviewForkSnapshot> {
        self.state.lock().await.conversation.snapshot().cloned()
    }
}

impl GuardianReviewSession {
    pub(crate) async fn rollout_path(&self) -> Option<PathBuf> {
        self.session
            .ensure_rollout_materialized(PersistContext::Standard)
            .await;
        match self.session.current_rollout_path().await {
            Ok(path) => path,
            Err(error) => {
                warn!("failed to resolve guardian trunk rollout path: {error}");
                None
            }
        }
    }
}

#[cfg(test)]
impl GuardianReviewSession {
    pub(crate) async fn committed_fork_rollout_items_for_test(&self) -> Option<Vec<RolloutItem>> {
        let state = self.state.lock().await;
        let snapshot = state.conversation.snapshot()?;
        match &snapshot.history().initial_history {
            InitialHistory::Forked(items) => Some(items.clone()),
            InitialHistory::New | InitialHistory::Cleared | InitialHistory::Resumed(_) => None,
        }
    }

    pub(crate) async fn send_trunk_event_raw_for_test(&self, event: Event) {
        self.session.send_event_raw(event).await;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
    }
}
