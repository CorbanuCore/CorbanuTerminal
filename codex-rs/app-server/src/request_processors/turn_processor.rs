use super::thread_input::ensure_direct_input_allowed;
use super::*;
<<<<<<< HEAD
use codex_agent_extension::AgentInvocation;
use codex_agent_extension::AgentRun;
use codex_agent_extension::AgentRunner;
use codex_core::accounting_extensions::SentModelRequest;
use codex_model_provider_info::WireApi;
use codex_model_provider_info::built_in_model_providers;
use codex_protocol::crew::AgentClass;
||||||| 413492cd6c
use codex_agent_extension::AgentInvocation;
use codex_agent_extension::AgentRun;
use codex_agent_extension::AgentRunner;
=======
use codex_app_server_protocol::ImageReference as V2ImageReference;
use codex_app_server_protocol::ThreadGuardianAssessment;
use codex_app_server_protocol::ThreadGuardianReviewParams;
use codex_app_server_protocol::ThreadGuardianReviewResponse;
use codex_app_server_protocol::ThreadItem;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
use codex_protocol::error::CodexErrorDetails;
use codex_protocol::models::ContentItem;
use codex_protocol::models::FunctionCallOutputBody;
use codex_protocol::models::FunctionCallOutputContentItem;
use codex_protocol::models::FunctionCallOutputPayload;
use codex_protocol::models::ImageReference as CoreImageReference;
use codex_protocol::protocol::TurnEnvironmentRequests;
use codex_protocol::protocol::TurnSettingsUpdate;
use codex_protocol::protocol::TurnSettingsUpdateOutcome;
use codex_protocol::turn_input::AnnotatedResponseItem;
use codex_protocol::turn_input::ResponseItemAnnotations;

use crate::image_url::REMOTE_IMAGE_URL_ERROR;
use crate::image_url::is_remote_image_url;

<<<<<<< HEAD
const DIRECT_INPUT_TO_MULTI_AGENT_V2_SUBAGENT_ERROR: &str =
    "direct app-server input is not allowed for multi-agent v2 sub-agents";

/// The route a reported send may be recorded against, canonically.
///
/// A client reports the account it believes it billed; this server only agrees
/// when that account is one it ships and the reported route is that account's
/// own route in its own dialect. A provider this build does not know, or a
/// known provider on a route it does not serve, is not recorded.
///
/// This is a well-formedness check and not an authorisation boundary, and the
/// difference matters: the caller still names the account, and the accepted
/// base URLs are public constants, so a client that can reach this request can
/// name any built-in provider whose own route it quotes correctly - including
/// two providers that share a route, which this cannot tell apart. What it
/// does guarantee is that nothing lands under a route its named provider does
/// not serve, and that the endpoint recorded is the catalogue's string rather
/// than however the caller spelled it. Nothing monetary rides on the name:
/// `record_sent_request` claims no economics for a reported send.
fn recognised_reported_route(
    provider_id: &str,
    base_url: &str,
    path: &str,
) -> Option<(String, String, WireApi)> {
    let provider = built_in_model_providers(/*openai_base_url*/ None).remove(provider_id)?;
    let api = provider.to_api_provider(/*auth_mode*/ None).ok()?;
    let route = api.base_url.trim_end_matches('/').to_string();
    if route != base_url.trim_end_matches('/') {
        return None;
    }
    let dialect_path = match provider.wire_api {
        WireApi::Anthropic => "messages",
        WireApi::Responses => "responses",
        WireApi::Chat => "chat/completions",
    };
    (path.trim_start_matches('/') == dialect_path)
        .then(|| (route, dialect_path.to_string(), provider.wire_api))
}

/// Mirrors the direct-input policy in both request validation and thread capability responses.
pub(super) fn can_accept_direct_input(
    multi_agent_version: Option<MultiAgentVersion>,
    session_source: &SessionSource,
) -> bool {
    multi_agent_version != Some(MultiAgentVersion::V2)
        || !matches!(
            session_source,
            SessionSource::SubAgent(SubAgentSource::ThreadSpawn { .. })
        )
        || human_addressable_crew_member(session_source)
}

fn human_addressable_crew_member(session_source: &SessionSource) -> bool {
    matches!(
        session_source,
        SessionSource::SubAgent(SubAgentSource::ThreadSpawn {
            agent_class: Some(AgentClass::CrewMember {
                human_addressable: true,
                ..
            }),
            ..
        })
    )
}

fn validate_user_input_image_urls(input: &[V2UserInput]) -> Result<(), JSONRPCErrorError> {
||||||| 413492cd6c
const DIRECT_INPUT_TO_MULTI_AGENT_V2_SUBAGENT_ERROR: &str =
    "direct app-server input is not allowed for multi-agent v2 sub-agents";

/// Mirrors the direct-input policy in both request validation and thread capability responses.
pub(super) fn can_accept_direct_input(
    multi_agent_version: Option<MultiAgentVersion>,
    session_source: &SessionSource,
) -> bool {
    multi_agent_version != Some(MultiAgentVersion::V2)
        || !matches!(
            session_source,
            SessionSource::SubAgent(SubAgentSource::ThreadSpawn { .. })
        )
}

fn validate_user_input_image_urls(input: &[V2UserInput]) -> Result<(), JSONRPCErrorError> {
=======
pub(super) fn validate_user_input_image_urls(
    input: &[V2UserInput],
) -> Result<(), JSONRPCErrorError> {
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
    if input.iter().any(|item| {
        matches!(
            item,
            V2UserInput::Image {
                image: V2ImageReference::Inline { url },
                ..
            } if is_remote_image_url(url)
        )
    }) {
        return Err(invalid_request(REMOTE_IMAGE_URL_ERROR));
    }
    Ok(())
}

fn validate_response_item_image_urls(items: &[ResponseItem]) -> Result<(), JSONRPCErrorError> {
    if items.iter().any(|item| match item {
        ResponseItem::Message { content, .. } => content.iter().any(|item| {
            matches!(
                item,
                ContentItem::InputImage { image: CoreImageReference::Inline { image_url }, .. } if is_remote_image_url(image_url)
            )
        }),
        ResponseItem::FunctionCallOutput { output, .. }
        | ResponseItem::CustomToolCallOutput { output, .. } => {
            output.content_items().is_some_and(|content| {
                content.iter().any(|item| {
                    matches!(
                        item,
                        FunctionCallOutputContentItem::InputImage { image: CoreImageReference::Inline { image_url }, .. }
                            if is_remote_image_url(image_url)
                    )
                })
            })
        }
        ResponseItem::Reasoning { .. }
        | ResponseItem::AgentMessage { .. }
        | ResponseItem::LocalShellCall { .. }
        | ResponseItem::FunctionCall { .. }
        | ResponseItem::ToolSearchCall { .. }
        | ResponseItem::CustomToolCall { .. }
        | ResponseItem::ToolSearchOutput { .. }
        | ResponseItem::WebSearchCall { .. }
        | ResponseItem::ImageGenerationCall { .. }
        | ResponseItem::Compaction { .. }
        | ResponseItem::ConfigurationUpdate { .. }
        | ResponseItem::CompactionTrigger { .. }
        | ResponseItem::ContextCompaction { .. }
        | ResponseItem::AdditionalTools { .. }
        | ResponseItem::Other => false,
    }) {
        return Err(invalid_request(REMOTE_IMAGE_URL_ERROR));
    }
    Ok(())
}

#[derive(Clone)]
pub(crate) struct TurnRequestProcessor {
    auth_manager: Arc<AuthManager>,
    thread_manager: Arc<ThreadManager>,
    outgoing: Arc<OutgoingMessageSender>,
    analytics_events_client: AnalyticsEventsClient,
    config: Arc<Config>,
    config_manager: ConfigManager,
    pending_thread_unloads: PendingThreadUnloads,
    thread_state_manager: ThreadStateManager,
    thread_watch_manager: ThreadWatchManager,
    skills_watcher: Arc<SkillsWatcher>,
    turn_cost_worker: Option<crate::turn_cost_worker::TurnCostWorkerHandle>,
}

fn map_additional_context(
    additional_context: Option<HashMap<String, AdditionalContextEntry>>,
) -> BTreeMap<String, CoreAdditionalContextEntry> {
    additional_context
        .unwrap_or_default()
        .into_iter()
        .map(|(key, entry)| {
            let entry = CoreAdditionalContextEntry {
                value: entry.value,
                kind: match entry.kind {
                    AdditionalContextKind::Untrusted => CoreAdditionalContextKind::Untrusted,
                    AdditionalContextKind::Application => CoreAdditionalContextKind::Application,
                },
                name: entry.name,
            };
            (key, entry)
        })
        .collect()
}

<<<<<<< HEAD
fn turn_start_submission_error(err: &CodexErr) -> JSONRPCErrorError {
    let mut error = internal_error(format!("failed to start turn: {err}"));
    if let CodexErrorDetails::AgentLimitReached { max_threads } = err.details() {
        error.data = Some(serde_json::json!({
            "turn_start_error": "execution_capacity",
            "max_threads": max_threads,
        }));
    }
    error
}

fn direct_input_to_v2_thread_spawn_is_blocked(
    multi_agent_version: Option<MultiAgentVersion>,
    session_source: &SessionSource,
    app_server_client_name: Option<&str>,
) -> bool {
    multi_agent_version == Some(MultiAgentVersion::V2)
        && matches!(
            session_source,
            SessionSource::SubAgent(SubAgentSource::ThreadSpawn { .. })
        )
        && !human_addressable_crew_member(session_source)
        && app_server_client_name != Some("codex-tui")
||||||| 413492cd6c
=======
#[derive(Default)]
struct ThreadEnvironmentOverride {
    environment_requests: Option<TurnEnvironmentRequests>,
    // Only default-environment updates replace the task's separately persisted root selection.
    runtime_workspace_roots: Option<Vec<AbsolutePathBuf>>,
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
}

struct ThreadSettingsBuildParams {
    method: &'static str,
    disabled_plugin_ids: Option<Vec<String>>,
    environment_override: ThreadEnvironmentOverride,
    approval_policy: Option<codex_app_server_protocol::AskForApproval>,
    approvals_reviewer: Option<codex_app_server_protocol::ApprovalsReviewer>,
    sandbox_policy: Option<codex_app_server_protocol::SandboxPolicy>,
    permissions: Option<String>,
    model: Option<String>,
    model_provider: Option<String>,
    service_tier: Option<Option<String>>,
    effort: Option<ReasoningEffort>,
    summary: Option<ReasoningSummary>,
    collaboration_mode: Option<CollaborationMode>,
    personality: Option<Personality>,
}

impl TurnRequestProcessor {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        auth_manager: Arc<AuthManager>,
        thread_manager: Arc<ThreadManager>,
        outgoing: Arc<OutgoingMessageSender>,
        analytics_events_client: AnalyticsEventsClient,
        config: Arc<Config>,
        config_manager: ConfigManager,
        pending_thread_unloads: PendingThreadUnloads,
        thread_state_manager: ThreadStateManager,
        thread_watch_manager: ThreadWatchManager,
        skills_watcher: Arc<SkillsWatcher>,
        turn_cost_worker: Option<crate::turn_cost_worker::TurnCostWorkerHandle>,
    ) -> Self {
        Self {
            auth_manager,
            thread_manager,
            outgoing,
            analytics_events_client,
            config,
            config_manager,
            pending_thread_unloads,
            thread_state_manager,
            thread_watch_manager,
            skills_watcher,
            turn_cost_worker,
        }
    }

    pub(crate) async fn turn_start(
        &self,
        request_id: ConnectionRequestId,
        params: TurnStartParams,
        app_server_client_name: Option<String>,
        app_server_client_version: Option<String>,
    ) -> Result<Option<ClientResponsePayload>, JSONRPCErrorError> {
        validate_user_input_image_urls(&params.input)?;
        self.turn_start_inner(
            request_id,
            params,
            app_server_client_name,
            app_server_client_version,
        )
        .await
        .map(|response| Some(response.into()))
    }

    pub(crate) async fn thread_guardian_review(
        &self,
        request_id: &ConnectionRequestId,
        params: ThreadGuardianReviewParams,
        cancellation: &CancellationToken,
    ) -> Result<Option<ClientResponsePayload>, JSONRPCErrorError> {
        let (thread_id, thread) = self.load_thread(&params.thread_id).await?;
        self.ensure_direct_input_allowed(request_id, thread.as_ref())
            .await?;
        self.config_manager
            .check_thread_model_provider_for_input(thread.config().await.as_ref())
            .await
            .map_err(|error| config_load_error(&error))?;
        match self
            .ensure_conversation_listener(
                thread_id,
                request_id.connection_id,
                /*raw_events_enabled*/ false,
            )
            .await?
        {
            EnsureConversationListenerResult::Attached => {}
            EnsureConversationListenerResult::ConnectionClosed => return Ok(None),
        }
        let result =
            thread
                .guardian_review(cancellation)
                .await
                .map_err(|err| match err.details() {
                    CodexErrorDetails::InvalidRequest(message) => invalid_request(message.clone()),
                    _ => internal_error(format!("Guardian review failed: {err}")),
                })?;
        Ok(Some(
            ThreadGuardianReviewResponse {
                call_id: result.call_id,
                assessment: result
                    .assessment
                    .map(|assessment| ThreadGuardianAssessment {
                        outcome: assessment.outcome,
                        risk_level: assessment.risk_level.into(),
                        user_authorization: assessment.user_authorization.into(),
                        rationale: assessment.rationale,
                    }),
            }
            .into(),
        ))
    }

    pub(crate) async fn thread_inject_items(
        &self,
        request_id: &ConnectionRequestId,
        params: ThreadInjectItemsParams,
    ) -> Result<Option<ClientResponsePayload>, JSONRPCErrorError> {
        self.thread_inject_items_response_inner(request_id, params)
            .await
            .map(|response| Some(response.into()))
    }

    pub(crate) async fn thread_record_sent_model_request(
        &self,
        params: ThreadRecordSentModelRequestParams,
    ) -> Result<Option<ClientResponsePayload>, JSONRPCErrorError> {
        self.thread_record_sent_model_request_inner(params)
            .await
            .map(|response| Some(response.into()))
    }

    pub(crate) async fn thread_settings_update(
        &self,
        request_id: &ConnectionRequestId,
        params: ThreadSettingsUpdateParams,
    ) -> Result<Option<ClientResponsePayload>, JSONRPCErrorError> {
        self.thread_settings_update_inner(request_id, params)
            .await
            .map(|response| response.map(Into::into))
    }

    pub(crate) async fn turn_settings_update(
        &self,
        request_id: &ConnectionRequestId,
        params: TurnSettingsUpdateParams,
    ) -> Result<Option<ClientResponsePayload>, JSONRPCErrorError> {
        let (_, thread) = self.load_thread(&params.thread_id).await?;
        self.ensure_direct_input_allowed(request_id, thread.as_ref())
            .await?;
        let (reply, outcome) = oneshot::channel();
        self.submit_core_op(
            request_id,
            &thread,
            Op::TurnSettings {
                turn_id: params.turn_id,
                update: TurnSettingsUpdate {
                    approvals_reviewer: params
                        .approvals_reviewer
                        .map(codex_app_server_protocol::ApprovalsReviewer::to_core),
                    environments: None,
                    model: params.model,
                    // Match thread/settings/update: public null does not clear effort.
                    effort: params.effort.map(Some),
                    summary: params.summary,
                    service_tier: params.service_tier,
                },
                reply,
            },
        )
        .await
        .map_err(|err| internal_error(format!("failed to submit turn settings: {err}")))?;
        let outcome = outcome
            .await
            .map_err(|_| internal_error("turn settings operation ended before replying"))?;
        let status = match outcome {
            TurnSettingsUpdateOutcome::Applied => TurnSettingsUpdateStatus::Applied,
            TurnSettingsUpdateOutcome::TargetUnavailable => {
                TurnSettingsUpdateStatus::TargetUnavailable
            }
            TurnSettingsUpdateOutcome::Rejected { reason } => return Err(invalid_request(reason)),
        };
        Ok(Some(TurnSettingsUpdateResponse { status }.into()))
    }

    pub(crate) async fn turn_steer(
        &self,
        request_id: &ConnectionRequestId,
        params: TurnSteerParams,
        app_server_client_name: Option<&str>,
    ) -> Result<Option<ClientResponsePayload>, JSONRPCErrorError> {
        validate_user_input_image_urls(&params.input)?;
        self.turn_steer_inner(request_id, params, app_server_client_name)
            .await
            .map(|response| Some(response.into()))
    }

    pub(crate) async fn turn_interrupt(
        &self,
        request_id: &ConnectionRequestId,
        params: TurnInterruptParams,
    ) -> Result<Option<ClientResponsePayload>, JSONRPCErrorError> {
        let result = self.turn_interrupt_inner(request_id, params).await;
        if let Err(error) = &result {
            self.track_error_response(request_id, error, /*error_type*/ None);
        }
        result.map(|response| response.map(Into::into))
    }

    pub(crate) async fn thread_realtime_start(
        &self,
        request_id: &ConnectionRequestId,
        params: ThreadRealtimeStartParams,
        sideband_base_url: Option<String>,
    ) -> Result<Option<ClientResponsePayload>, JSONRPCErrorError> {
        self.thread_realtime_start_inner(request_id, params, sideband_base_url)
            .await
            .map(|response| response.map(Into::into))
    }

    pub(crate) async fn thread_realtime_append_audio(
        &self,
        request_id: &ConnectionRequestId,
        params: ThreadRealtimeAppendAudioParams,
    ) -> Result<Option<ClientResponsePayload>, JSONRPCErrorError> {
        self.thread_realtime_append_audio_inner(request_id, params)
            .await
            .map(|response| response.map(Into::into))
    }

    pub(crate) async fn thread_realtime_append_text(
        &self,
        request_id: &ConnectionRequestId,
        params: ThreadRealtimeAppendTextParams,
    ) -> Result<Option<ClientResponsePayload>, JSONRPCErrorError> {
        self.thread_realtime_append_text_inner(request_id, params)
            .await
            .map(|response| response.map(Into::into))
    }

    pub(crate) async fn thread_realtime_append_speech(
        &self,
        request_id: &ConnectionRequestId,
        params: ThreadRealtimeAppendSpeechParams,
    ) -> Result<Option<ClientResponsePayload>, JSONRPCErrorError> {
        self.thread_realtime_append_speech_inner(request_id, params)
            .await
            .map(|response| response.map(Into::into))
    }

    pub(crate) async fn thread_realtime_stop(
        &self,
        request_id: &ConnectionRequestId,
        params: ThreadRealtimeStopParams,
    ) -> Result<Option<ClientResponsePayload>, JSONRPCErrorError> {
        self.thread_realtime_stop_inner(request_id, params)
            .await
            .map(|response| response.map(Into::into))
    }

    pub(crate) async fn thread_realtime_list_voices(
        &self,
    ) -> Result<Option<ClientResponsePayload>, JSONRPCErrorError> {
        Ok(Some(
            ThreadRealtimeListVoicesResponse {
                voices: RealtimeVoicesList::builtin(),
            }
            .into(),
        ))
    }

    pub(crate) async fn review_start(
        &self,
        request_id: &ConnectionRequestId,
        params: ReviewStartParams,
    ) -> Result<Option<ClientResponsePayload>, JSONRPCErrorError> {
        let (_, thread) = self.load_thread(&params.thread_id).await?;
        self.ensure_direct_input_allowed(request_id, thread.as_ref())
            .await?;
        self.config_manager
            .check_thread_model_provider_for_input(thread.config().await.as_ref())
            .await
            .map_err(|error| config_load_error(&error))?;
        let (review_request, display_text) = Self::review_request_from_target(params.target)?;
        let turn_id = self
            .submit_core_op(request_id, thread.as_ref(), Op::Review { review_request })
            .await
            .map_err(|err| internal_error(format!("failed to start review: {err}")))?;
        Ok(Some(
            ReviewStartResponse {
                turn: Self::build_review_turn(turn_id, display_text),
                review_thread_id: params.thread_id,
            }
            .into(),
        ))
    }

    fn track_error_response(
        &self,
        request_id: &ConnectionRequestId,
        error: &JSONRPCErrorError,
        error_type: Option<AnalyticsJsonRpcError>,
    ) {
        self.analytics_events_client.track_error_response(
            request_id.connection_id.0,
            request_id.request_id.clone(),
            error.clone(),
            error_type,
        );
    }

    async fn load_thread(
        &self,
        thread_id: &str,
    ) -> Result<(ThreadId, Arc<CodexThread>), JSONRPCErrorError> {
        // Resolve the core conversation handle from a v2 thread id string.
        let thread_id = ThreadId::from_string(thread_id)
            .map_err(|err| invalid_request(format!("invalid thread id: {err}")))?;

        let thread = self
            .thread_manager
            .get_thread(thread_id)
            .await
            .map_err(|_| invalid_request(format!("thread not found: {thread_id}")))?;

        Ok((thread_id, thread))
    }

    async fn ensure_direct_input_allowed(
        &self,
        request_id: &ConnectionRequestId,
        thread: &CodexThread,
        app_server_client_name: Option<&str>,
    ) -> Result<(), JSONRPCErrorError> {
<<<<<<< HEAD
        let config_snapshot = thread.config_snapshot().await;
        if direct_input_to_v2_thread_spawn_is_blocked(
            thread.multi_agent_version(),
            &config_snapshot.session_source,
            app_server_client_name,
        ) {
            let error = invalid_request(DIRECT_INPUT_TO_MULTI_AGENT_V2_SUBAGENT_ERROR);
            self.track_error_response(request_id, &error, /*error_type*/ None);
            return Err(error);
        }

        Ok(())
||||||| 413492cd6c
        let config_snapshot = thread.config_snapshot().await;
        if !can_accept_direct_input(
            thread.multi_agent_version(),
            &config_snapshot.session_source,
        ) {
            let error = invalid_request(DIRECT_INPUT_TO_MULTI_AGENT_V2_SUBAGENT_ERROR);
            self.track_error_response(request_id, &error, /*error_type*/ None);
            return Err(error);
        }

        Ok(())
=======
        ensure_direct_input_allowed(thread)
            .await
            .inspect_err(|error| {
                self.track_error_response(request_id, error, /*error_type*/ None);
            })
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
    }

    fn normalize_collaboration_mode(
        &self,
        mut collaboration_mode: CollaborationMode,
    ) -> CollaborationMode {
        if collaboration_mode.settings.developer_instructions.is_none()
            && let Some(instructions) = builtin_collaboration_mode_presets()
                .into_iter()
                .find(|preset| preset.mode == Some(collaboration_mode.mode))
                .and_then(|preset| preset.developer_instructions.flatten())
                .filter(|instructions| !instructions.is_empty())
        {
            collaboration_mode.settings.developer_instructions = Some(instructions);
        }

        collaboration_mode
    }

    fn review_request_from_target(
        target: ApiReviewTarget,
    ) -> Result<(ReviewRequest, String), JSONRPCErrorError> {
        let core_target = match target {
            ApiReviewTarget::UncommittedChanges => CoreReviewTarget::UncommittedChanges,
            ApiReviewTarget::BaseBranch { branch } => {
                let branch = branch.trim().to_string();
                if branch.is_empty() {
                    return Err(invalid_request("branch must not be empty"));
                }
                CoreReviewTarget::BaseBranch { branch }
            }
            ApiReviewTarget::Commit { sha, title } => {
                let sha = sha.trim().to_string();
                if sha.is_empty() {
                    return Err(invalid_request("sha must not be empty"));
                }
                let title = title
                    .map(|t| t.trim().to_string())
                    .filter(|t| !t.is_empty());
                CoreReviewTarget::Commit { sha, title }
            }
            ApiReviewTarget::Custom { instructions } => {
                let instructions = instructions.trim().to_string();
                if instructions.is_empty() {
                    return Err(invalid_request("instructions must not be empty"));
                }
                CoreReviewTarget::Custom { instructions }
            }
        };
        let hint = codex_core::review_prompts::user_facing_hint(&core_target);
        let review_request = ReviewRequest {
            target: core_target,
            user_facing_hint: Some(hint.clone()),
        };

        Ok((review_request, hint))
    }

    async fn request_trace_context(
        &self,
        request_id: &ConnectionRequestId,
    ) -> Option<codex_protocol::protocol::W3cTraceContext> {
        self.outgoing.request_trace_context(request_id).await
    }

    async fn submit_core_op(
        &self,
        request_id: &ConnectionRequestId,
        thread: &CodexThread,
        op: Op,
    ) -> CodexResult<String> {
        thread
            .submit_with_trace(op, self.request_trace_context(request_id).await)
            .await
    }

    pub(super) fn input_too_large_error(actual_chars: usize) -> JSONRPCErrorError {
        let mut error = invalid_params(format!(
            "Input exceeds the maximum length of {MAX_USER_INPUT_TEXT_CHARS} characters."
        ));
        error.data = Some(serde_json::json!({
            "input_error_code": INPUT_TOO_LARGE_ERROR_CODE,
            "max_chars": MAX_USER_INPUT_TEXT_CHARS,
            "actual_chars": actual_chars,
        }));
        error
    }

    pub(super) fn validate_v2_input_limit(items: &[V2UserInput]) -> Result<(), JSONRPCErrorError> {
        let actual_chars: usize = items.iter().map(V2UserInput::text_char_count).sum();
        if actual_chars > MAX_USER_INPUT_TEXT_CHARS {
            return Err(Self::input_too_large_error(actual_chars));
        }
        Ok(())
    }

    async fn turn_start_inner(
        &self,
        request_id: ConnectionRequestId,
        params: TurnStartParams,
        app_server_client_name: Option<String>,
        app_server_client_version: Option<String>,
    ) -> Result<TurnStartResponse, JSONRPCErrorError> {
        let (thread_id, thread) =
            self.load_thread(&params.thread_id)
                .await
                .inspect_err(|error| {
                    self.track_error_response(&request_id, error, /*error_type*/ None);
                })?;
<<<<<<< HEAD
        self.ensure_direct_input_allowed(
            &request_id,
            thread.as_ref(),
            app_server_client_name.as_deref(),
        )
        .await?;
        if let Err(error) = Self::validate_v2_input_limit(&params.input) {
||||||| 413492cd6c
        self.ensure_direct_input_allowed(&request_id, thread.as_ref())
            .await?;
        if let Err(error) = Self::validate_v2_input_limit(&params.input) {
=======
        self.ensure_direct_input_allowed(&request_id, thread.as_ref())
            .await?;
        self.config_manager
            .check_thread_model_provider_for_input(thread.config().await.as_ref())
            .await
            .map_err(|error| config_load_error(&error))?;
        if let Some(tool_output) = &params.tool_output {
            if !params.input.is_empty() {
                return Err(invalid_request(
                    "`toolOutput` cannot be combined with nonempty `input`",
                ));
            }
            if tool_output.name.is_empty() {
                return Err(invalid_request("`toolOutput.name` must not be empty"));
            }
        }
        let actual_chars = params
            .input
            .iter()
            .map(V2UserInput::text_char_count)
            .sum::<usize>()
            + params
                .tool_output
                .as_ref()
                .map_or(0, |output| match &output.output {
                    FunctionCallOutputBody::Text(text) => text.chars().count(),
                    FunctionCallOutputBody::ContentItems(items) => items
                        .iter()
                        .map(|item| match item {
                            FunctionCallOutputContentItem::InputText { text } => {
                                text.chars().count()
                            }
                            _ => 0,
                        })
                        .sum(),
                });
        if actual_chars > MAX_USER_INPUT_TEXT_CHARS {
            let error = Self::input_too_large_error(actual_chars);
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
            self.track_error_response(
                &request_id,
                &error,
                Some(AnalyticsJsonRpcError::Input(InputError::TooLarge)),
            );
            return Err(error);
        }
        // Validate before changing the thread's live MCP policy.
        let additional_context = map_additional_context(params.additional_context);
        additional_context
            .values()
            .try_for_each(CoreAdditionalContextEntry::validate)
            .map_err(invalid_request)
            .inspect_err(|error| {
                self.track_error_response(&request_id, error, /*error_type*/ None);
            })?;
        Self::set_app_server_client_info(
            thread.as_ref(),
            app_server_client_name,
            app_server_client_version,
        )
        .await
        .inspect_err(|error| {
            self.track_error_response(&request_id, error, /*error_type*/ None);
        })?;
        let runtime_workspace_roots = params
            .runtime_workspace_roots
            .map(resolve_runtime_workspace_roots);
        let environment_requests =
            resolve_turn_environment_requests(self.thread_manager.as_ref(), params.environments)?;

        let turn_has_input = !params.input.is_empty();
        let input = if let Some(tool_output) = params.tool_output {
            let item = ResponseItem::FunctionCallOutput {
                id: None,
                call_id: None,
                name: Some(tool_output.name),
                namespace: tool_output.namespace,
                output: FunctionCallOutputPayload {
                    body: tool_output.output,
                    success: None,
                },
                internal_chat_message_metadata_passthrough: None,
            };
            validate_response_item_image_urls(std::slice::from_ref(&item))?;
            if tool_output.retain {
                TurnInput::AnnotatedResponseItem(AnnotatedResponseItem {
                    item,
                    annotations: ResponseItemAnnotations { retain: true },
                })
            } else {
                TurnInput::ResponseItem(item)
            }
        } else {
            TurnInput::UserInput {
                content: params
                    .input
                    .into_iter()
                    .map(V2UserInput::into_core)
                    .collect(),
                client_id: params.client_user_message_id,
            }
        };
        let cwd = resolve_request_cwd(params.cwd)?;
        let environment_override = self
            .build_environment_override(
                thread.as_ref(),
                cwd,
                runtime_workspace_roots,
                environment_requests,
            )
            .await;
        let thread_settings = self
            .build_thread_settings_overrides(
                thread.as_ref(),
                ThreadSettingsBuildParams {
                    method: "turn/start",
                    disabled_plugin_ids: params.disabled_plugin_ids,
                    environment_override,
                    approval_policy: params.approval_policy,
                    approvals_reviewer: params.approvals_reviewer,
                    sandbox_policy: params.sandbox_policy,
                    permissions: params.permissions,
                    model: params.model,
                    model_provider: None,
                    service_tier: params.service_tier,
                    effort: params.effort,
                    summary: params.summary,
                    collaboration_mode: params.collaboration_mode,
                    personality: params.personality,
                },
            )
            .await?;

        let submission = thread
            .start_or_steer_turn(
                TurnInputRequest::new(input)
                    .with_thread_settings(thread_settings)
                    .on_start(TurnStartOptions {
                        turn_trigger: params.turn_trigger,
                        parent_turn_id: params.parent_turn_id,
                        initiating_agent_path: None,
                        root_turn_id: params.root_turn_id,
                        final_output_json_schema: params.output_schema,
                        service_tier: params.service_tier_for_turn,
                        cyber_access_program: params.cyber_access_program.map(Into::into),
                    })
                    .with_additional_context(additional_context)
                    .with_responses_metadata(params.responsesapi_client_metadata)
                    .with_responses_metadata_patch(params.responsesapi_client_metadata_patch)
                    .with_trace(self.request_trace_context(&request_id).await),
            )
            .await
            .map_err(|err| {
<<<<<<< HEAD
                let error = turn_start_submission_error(&err);
||||||| 413492cd6c
                let error = internal_error(format!("failed to start turn: {err}"));
=======
                let error = match err.details() {
                    CodexErrorDetails::InvalidRequest(message) => invalid_request(message.clone()),
                    _ => internal_error(format!("failed to submit turn input: {err}")),
                };
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
                self.track_error_response(&request_id, &error, /*error_type*/ None);
                error
            })?;
        let (turn_id, root_turn_id, started) = match submission {
            TurnInputSubmission::Started {
                turn_id,
                root_turn_id,
            } => (turn_id, root_turn_id, true),
            TurnInputSubmission::Steered {
                turn_id,
                root_turn_id,
            } => (turn_id, root_turn_id, false),
            TurnInputSubmission::NotSubmitted { reason } => {
                let error = if reason == NotSubmittedReason::ServerDraining {
                    crate::error_code::server_draining_error()
                } else {
                    internal_error(format!("failed to submit turn input: {reason:?}"))
                };
                self.track_error_response(&request_id, &error, /*error_type*/ None);
                return Err(error);
            }
        };

        if turn_has_input && started {
            let config_snapshot = thread.config_snapshot().await;
<<<<<<< HEAD
            let parent_permission_profile =
                parent_permission_profile_override.unwrap_or(config_snapshot.permission_profile);
            let thread_config = thread.config().await;
            // #658: the usage check runs on the thread's own account.
            let auth_manager =
                match codex_model_provider::named_sign_in_account(&thread_config.model_provider) {
                    Some(account) => self.auth_manager.for_named_account(account),
                    None => self.auth_manager.default_account_manager(),
                };
            codex_memories_write::start_memories_startup_task(
                Arc::clone(&self.thread_manager),
                auth_manager,
                thread_id,
                Arc::clone(&thread),
                thread_config,
                parent_permission_profile,
                &config_snapshot.session_source,
            );
||||||| 413492cd6c
            let parent_permission_profile =
                parent_permission_profile_override.unwrap_or(config_snapshot.permission_profile);
            codex_memories_write::start_memories_startup_task(
                Arc::clone(&self.thread_manager),
                Arc::clone(&self.auth_manager),
                thread_id,
                Arc::clone(&thread),
                thread.config().await,
                parent_permission_profile,
                &config_snapshot.session_source,
            );
=======
            if config_snapshot.is_primary_environment_configured() {
                codex_memories_write::start_memories_startup_task(
                    Arc::clone(&self.thread_manager),
                    Arc::clone(&self.auth_manager),
                    thread_id,
                    Arc::clone(&thread),
                    thread.config().await,
                    config_snapshot.permission_profile,
                    &config_snapshot.session_source,
                );
            }
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        }

        self.outgoing
            .record_request_turn_id(&request_id, &turn_id)
            .await;
        let turn = Turn {
            id: turn_id,
            root_turn_id: Some(root_turn_id),
            items: vec![],
            items_view: TurnItemsView::NotLoaded,
            error: None,
            status: TurnStatus::InProgress,
            started_at: None,
            completed_at: None,
            duration_ms: None,
        };

        Ok(TurnStartResponse { turn })
    }

    async fn build_environment_override(
        &self,
        thread: &CodexThread,
        cwd: Option<AbsolutePathBuf>,
        workspace_roots: Option<Vec<AbsolutePathBuf>>,
        environment_requests: Option<Vec<TurnEnvironmentRequest>>,
    ) -> ThreadEnvironmentOverride {
        if cwd.is_none() && workspace_roots.is_none() && environment_requests.is_none() {
            return ThreadEnvironmentOverride::default();
        }

        // Explicit environment requests own their workspace roots and pass through unchanged. Top-level
        // `runtimeWorkspaceRoots` is only a compatibility input for default environments.
        if let Some(environment_requests) = environment_requests {
            let legacy_fallback_cwd = match cwd {
                Some(cwd) => cwd,
                None => match environment_requests
                    .iter()
                    .find(|request| request.environment_id == LOCAL_ENVIRONMENT_ID)
                    .and_then(|request| request.cwd.to_abs_path().ok())
                {
                    Some(cwd) => cwd,
                    None => thread.config_snapshot().await.cwd().clone(),
                },
            };
            return ThreadEnvironmentOverride {
                environment_requests: Some(TurnEnvironmentRequests::new(
                    legacy_fallback_cwd,
                    environment_requests,
                )),
                ..Default::default()
            };
        }

        // Default-environment updates retain the task's fallback roots, not its active roots.
        let snapshot = thread.thread_settings_snapshot().await;
        let current_cwd = snapshot.cwd;
        let legacy_fallback_cwd = cwd.unwrap_or_else(|| current_cwd.clone());
        let workspace_roots = match workspace_roots {
            Some(workspace_roots) => workspace_roots,
            None => path_utils::replace_path_and_deduplicate(
                snapshot.runtime_workspace_roots.unwrap_or_default(),
                current_cwd.as_path(),
                legacy_fallback_cwd.clone(),
            ),
        };
        let environment_requests = self
            .thread_manager
            .default_environment_requests(&legacy_fallback_cwd, &workspace_roots);
        ThreadEnvironmentOverride {
            environment_requests: Some(TurnEnvironmentRequests::new(
                legacy_fallback_cwd,
                environment_requests,
            )),
            runtime_workspace_roots: Some(workspace_roots),
        }
    }

    async fn build_thread_settings_overrides(
        &self,
        thread: &CodexThread,
        params: ThreadSettingsBuildParams,
    ) -> Result<codex_protocol::protocol::ThreadSettingsOverrides, JSONRPCErrorError> {
        let ThreadSettingsBuildParams {
            method,
            disabled_plugin_ids,
            environment_override:
                ThreadEnvironmentOverride {
                    environment_requests,
                    runtime_workspace_roots,
                },
            approval_policy,
            approvals_reviewer,
            sandbox_policy,
            permissions,
            model,
            model_provider,
            service_tier,
            effort,
            summary,
            collaboration_mode,
            personality,
        } = params;

        if sandbox_policy.is_some() && permissions.is_some() {
            return Err(invalid_request(
                "`permissions` cannot be combined with `sandboxPolicy`",
            ));
        }

        let collaboration_mode =
            collaboration_mode.map(|mode| self.normalize_collaboration_mode(mode));
<<<<<<< HEAD
        let has_environment_override = environments.is_some();
        // Legacy requests only acknowledge queueing. Dependent partial updates
        // should opt into confirmation or combine their fields in one request.
||||||| 413492cd6c
        let has_environment_override = environments.is_some();
        // `thread/settings/update` only acknowledges that the update was queued.
        // Clients that send dependent partial updates should wait for
        // `thread/settings/updated` or combine the fields in one request.
=======
        let has_environment_override = environment_requests.is_some();
        // `thread/settings/update` only acknowledges that the update was queued.
        // Clients that send dependent partial updates should wait for
        // `thread/settings/updated` or combine the fields in one request.
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        let snapshot = if permissions.is_some() {
            Some(thread.config_snapshot().await)
        } else {
            None
        };

        let has_any_overrides = has_environment_override
            || disabled_plugin_ids.is_some()
            || approval_policy.is_some()
            || approvals_reviewer.is_some()
            || sandbox_policy.is_some()
            || permissions.is_some()
            || model.is_some()
            || model_provider.is_some()
            || service_tier.is_some()
            || effort.is_some()
            || summary.is_some()
            || collaboration_mode.is_some()
            || personality.is_some();

        let approval_policy =
            approval_policy.map(codex_app_server_protocol::AskForApproval::to_core);
        let approvals_reviewer =
            approvals_reviewer.map(codex_app_server_protocol::ApprovalsReviewer::to_core);
        let sandbox_policy = sandbox_policy.map(|policy| policy.to_core());
        let (permission_profile, active_permission_profile, profile_workspace_roots) =
            if let Some(permissions) = permissions {
                let Some(snapshot) = snapshot.as_ref() else {
                    return Err(internal_error(format!(
                        "{method} permission selection missing thread snapshot"
                    )));
                };
<<<<<<< HEAD
                let overrides = ConfigOverrides {
                    // Reload permissions for this thread's runtime, not an unrelated
                    // saved default. CLI/model switches can legitimately differ from
                    // config.toml; validating a mixed pair blocks an otherwise valid turn.
                    model: Some(model.clone().unwrap_or_else(|| snapshot.model.clone())),
                    model_provider: Some(
                        model_provider
                            .clone()
                            .unwrap_or_else(|| snapshot.model_provider_id.clone()),
                    ),
                    cwd: environments
                        .as_ref()
                        .map(|environments| environments.legacy_fallback_cwd.to_path_buf()),
                    default_permissions: Some(permissions),
                    codex_self_exe: self.arg0_paths.codex_self_exe.clone(),
                    codex_linux_sandbox_exe: self.arg0_paths.codex_linux_sandbox_exe.clone(),
                    main_execve_wrapper_exe: self.arg0_paths.main_execve_wrapper_exe.clone(),
                    ..Default::default()
                };
||||||| 413492cd6c
                let overrides = ConfigOverrides {
                    cwd: environments
                        .as_ref()
                        .map(|environments| environments.legacy_fallback_cwd.to_path_buf()),
                    default_permissions: Some(permissions),
                    codex_linux_sandbox_exe: self.arg0_paths.codex_linux_sandbox_exe.clone(),
                    main_execve_wrapper_exe: self.arg0_paths.main_execve_wrapper_exe.clone(),
                    ..Default::default()
                };
=======
                let thread_config = thread.config().await;
                let cwd = environment_requests.as_ref().map_or_else(
                    || snapshot.cwd().clone(),
                    |environment_requests| environment_requests.legacy_fallback_cwd.clone(),
                );
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
                let config = self
                    .config_manager
                    .load_permission_config_for_thread(&thread_config, cwd, permissions)
                    .await
                    .map_err(|error| config_load_error(&error))?;
                // Startup config is allowed to fall back when requirements
                // disallow a configured profile. An explicit settings update
                // is different: reject it before accepting the request.
                if let Some(warning) = config.startup_warnings.iter().find(|warning| {
                    warning.contains("Configured value for `permission_profile` is disallowed")
                }) {
                    return Err(invalid_request(format!(
                        "invalid thread settings override: {warning}"
                    )));
                }
                (
                    Some(config.permissions.permission_profile().clone()),
                    config.permissions.active_permission_profile(),
                    Some(config.permissions.profile_workspace_roots().to_vec()),
                )
            } else {
                (None, None, None)
            };
        let effort = effort.map(Some);

        if has_any_overrides {
            thread
                .preview_thread_settings_overrides(CodexThreadSettingsOverrides {
                    turn_extension_init: None,
                    disabled_plugin_ids: disabled_plugin_ids.clone(),
                    environments: environment_requests.clone(),
                    runtime_workspace_roots: runtime_workspace_roots.clone(),
                    approval_policy,
                    approvals_reviewer,
                    sandbox_policy: sandbox_policy.clone(),
                    permission_profile: permission_profile.clone(),
                    active_permission_profile: active_permission_profile.clone(),
                    profile_workspace_roots: profile_workspace_roots.clone(),
                    windows_sandbox_level: None,
                    model: model.clone(),
                    model_provider: model_provider.clone(),
                    effort: effort.clone(),
                    summary,
                    service_tier: service_tier.clone(),
                    collaboration_mode: collaboration_mode.clone(),
                    personality,
                })
                .await
                .map_err(|err| {
                    invalid_request(format!("invalid thread settings override: {err}"))
                })?;
        }

        Ok(codex_protocol::protocol::ThreadSettingsOverrides {
            disabled_plugin_ids,
            environments: environment_requests,
            runtime_workspace_roots,
            profile_workspace_roots,
            approval_policy,
            approvals_reviewer,
            sandbox_policy,
            permission_profile,
            active_permission_profile,
            windows_sandbox_level: None,
            model,
            model_provider,
            effort,
            summary,
            service_tier,
            collaboration_mode,
            personality,
        })
    }

    async fn thread_settings_update_inner(
        &self,
        request_id: &ConnectionRequestId,
        params: ThreadSettingsUpdateParams,
<<<<<<< HEAD
    ) -> Result<Option<ThreadSettingsUpdateResponse>, JSONRPCErrorError> {
        let (thread_id, thread) = self.load_thread(&params.thread_id).await?;
||||||| 413492cd6c
    ) -> Result<ThreadSettingsUpdateResponse, JSONRPCErrorError> {
        let (_, thread) = self.load_thread(&params.thread_id).await?;
=======
    ) -> Result<ThreadSettingsUpdateResponse, JSONRPCErrorError> {
        let (_, thread) = self.load_thread(&params.thread_id).await?;
        self.ensure_direct_input_allowed(request_id, thread.as_ref())
            .await?;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        let cwd = resolve_request_cwd(params.cwd)?;
        let environment_override = self
            .build_environment_override(
                thread.as_ref(),
                cwd,
                /*workspace_roots*/ None,
                /*environment_requests*/ None,
            )
            .await;
        let thread_settings = self
            .build_thread_settings_overrides(
                thread.as_ref(),
                ThreadSettingsBuildParams {
                    method: "thread/settings/update",
                    disabled_plugin_ids: params.disabled_plugin_ids,
                    environment_override,
                    approval_policy: params.approval_policy,
                    approvals_reviewer: params.approvals_reviewer,
                    sandbox_policy: params.sandbox_policy,
                    permissions: params.permissions,
                    model: params.model,
                    model_provider: params.model_provider,
                    service_tier: params.service_tier,
                    effort: params.effort,
                    summary: params.summary,
                    collaboration_mode: params.collaboration_mode,
                    personality: params.personality,
                },
            )
            .await?;

        if params.confirm {
            let trace = self.request_trace_context(request_id).await;
            let operation_id = uuid::Uuid::new_v4().to_string();
            let (state, completion) = self
                .thread_state_manager
                .register_settings_confirmation(
                    thread_id,
                    &thread,
                    operation_id.clone(),
                    request_id.clone(),
                )
                .await
                .ok_or_else(|| {
                    invalid_request(
                        "settings confirmation requires a live thread listener and connection",
                    )
                })?;
            let submission = codex_protocol::protocol::Submission {
                id: operation_id.clone(),
                op: Op::ThreadSettings { thread_settings },
                client_user_message_id: None,
                parent_turn_id: None,
                trace,
            };
            let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
            tokio::spawn(crate::settings_confirmation::complete_settings_submission(
                state.clone(),
                operation_id.clone(),
                request_id.clone(),
                completion,
                self.outgoing.clone(),
                deadline,
            ));
            // Keep enqueue order under the native per-thread request serializer.
            // The deferred owner already exists if this handler is cancelled.
            if !matches!(
                tokio::time::timeout_at(deadline, thread.submit_with_id(submission)).await,
                Ok(Ok(()))
            ) {
                state.lock().await.pending_settings.remove(&operation_id);
            }
            return Ok(None);
        }
        if thread_settings != codex_protocol::protocol::ThreadSettingsOverrides::default() {
            self.submit_core_op(
                request_id,
                thread.as_ref(),
                Op::ThreadSettings {
                    thread_settings,
                    reply: None,
                },
            )
            .await
            .map_err(|err| internal_error(format!("failed to update thread settings: {err}")))?;
        }

        Ok(Some(ThreadSettingsUpdateResponse::Accepted {}))
    }

    async fn thread_inject_items_response_inner(
        &self,
        request_id: &ConnectionRequestId,
        params: ThreadInjectItemsParams,
    ) -> Result<ThreadInjectItemsResponse, JSONRPCErrorError> {
        let (_, thread) = self.load_thread(&params.thread_id).await?;
        self.ensure_direct_input_allowed(request_id, thread.as_ref())
            .await?;

        let items = params
            .items
            .into_iter()
            .enumerate()
            .map(|(index, value)| {
                serde_json::from_value::<ResponseItem>(value)
                    .map_err(|err| format!("items[{index}] is not a valid response item: {err}"))
            })
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(invalid_request)?;
        validate_response_item_image_urls(&items)?;

        thread
            .inject_response_items(items)
            .await
            .map_err(|err| match err.details() {
                CodexErrorDetails::InvalidRequest(message) => invalid_request(message.clone()),
                _ => internal_error(format!("failed to inject response items: {err}")),
            })?;
        Ok(ThreadInjectItemsResponse {})
    }

    /// Record a model request a client sent itself.
    ///
    /// The Claude panes bridge posts to a provider from inside the TUI, which
    /// does not run the model client, so those requests reached no ledger at
    /// all. The client reports the send; the server decides what it means.
    ///
    /// The client names the account it billed and the route it took; this
    /// server records it only when the two agree with a provider it ships, and
    /// records the catalogue's strings rather than the caller's. See
    /// `recognised_reported_route` for what that does and does not guarantee.
    /// The numbers still come from the provider's own response body, parsed by
    /// the dialect's own parser.
    async fn thread_record_sent_model_request_inner(
        &self,
        params: ThreadRecordSentModelRequestParams,
    ) -> Result<ThreadRecordSentModelRequestResponse, JSONRPCErrorError> {
        let (_, thread) = self.load_thread(&params.thread_id).await?;
        let Some((endpoint, path, wire_api)) =
            recognised_reported_route(&params.provider_id, &params.base_url, &params.path)
        else {
            tracing::warn!(
                provider_id = %params.provider_id,
                base_url = %params.base_url,
                path = %params.path,
                "accounting: reported send is not a known provider's own route"
            );
            return Ok(ThreadRecordSentModelRequestResponse { recorded: false });
        };
        let recorded = thread
            .record_sent_model_request(SentModelRequest {
                provider_id: params.provider_id,
                endpoint,
                path,
                wire_api,
                model: params.model,
                label: "pane".to_string(),
                usage: params.usage,
            })
            .await;
        Ok(ThreadRecordSentModelRequestResponse { recorded })
    }

    async fn set_app_server_client_info(
        thread: &CodexThread,
        app_server_client_name: Option<String>,
        app_server_client_version: Option<String>,
    ) -> Result<(), JSONRPCErrorError> {
        let mcp_elicitations_auto_deny = xcode_26_4_mcp_elicitations_auto_deny(
            app_server_client_name.as_deref(),
            app_server_client_version.as_deref(),
        );
        thread
            .set_app_server_client_info(
                app_server_client_name,
                app_server_client_version,
                mcp_elicitations_auto_deny,
            )
            .await
            .map_err(|err| internal_error(format!("failed to set app server client info: {err}")))
    }

    async fn turn_steer_inner(
        &self,
        request_id: &ConnectionRequestId,
        params: TurnSteerParams,
        app_server_client_name: Option<&str>,
    ) -> Result<TurnSteerResponse, JSONRPCErrorError> {
        let (_, thread) = self
            .load_thread(&params.thread_id)
            .await
            .inspect_err(|error| {
                self.track_error_response(request_id, error, /*error_type*/ None);
            })?;
        self.ensure_direct_input_allowed(request_id, thread.as_ref(), app_server_client_name)
            .await?;
        self.config_manager
            .check_thread_model_provider_for_input(thread.config().await.as_ref())
            .await
            .map_err(|error| config_load_error(&error))?;

        if params.expected_turn_id.is_empty() {
            return Err(invalid_request("expectedTurnId must not be empty"));
        }
        self.outgoing
            .record_request_turn_id(request_id, &params.expected_turn_id)
            .await;
        if let Err(error) = Self::validate_v2_input_limit(&params.input) {
            self.track_error_response(
                request_id,
                &error,
                Some(AnalyticsJsonRpcError::Input(InputError::TooLarge)),
            );
            return Err(error);
        }

        let returned_input = params.input.clone();
        let mapped_items: Vec<CoreInputItem> = params
            .input
            .into_iter()
            .map(V2UserInput::into_core)
            .collect();
        let additional_context = map_additional_context(params.additional_context);

        let submission = thread
            .steer_turn(
                TurnInputRequest::new(TurnInput::UserInput {
                    content: mapped_items,
                    client_id: params.client_user_message_id,
                })
                .with_additional_context(additional_context)
                .with_responses_metadata(params.responsesapi_client_metadata)
                .with_responses_metadata_patch(params.responsesapi_client_metadata_patch),
                params.expected_turn_id,
            )
            .await
            .map_err(|err| {
                let error = match err.details() {
                    CodexErrorDetails::InvalidRequest(message) => invalid_request(message.clone()),
                    _ => internal_error(format!("failed to steer turn: {err}")),
                };
                self.track_error_response(request_id, &error, /*error_type*/ None);
                error
            })?;
        let turn_id = match submission {
            SteerSubmission::Steered { turn_id } => turn_id,
            SteerSubmission::NotSubmitted { reason } => {
                let (message, data, error_type) = match reason {
                    NotSubmittedReason::ServerDraining => {
                        return Err(crate::error_code::server_draining_error());
                    }
                    NotSubmittedReason::NoActiveTurn | NotSubmittedReason::NotIdle => (
                        "no active turn to steer".to_string(),
                        None,
                        Some(AnalyticsJsonRpcError::TurnSteer(
                            TurnSteerRequestError::NoActiveTurn,
                        )),
                    ),
                    NotSubmittedReason::ExpectedTurnMismatch { expected, actual } => (
                        format!("expected active turn id `{expected}` but found `{actual}`"),
                        None,
                        Some(AnalyticsJsonRpcError::TurnSteer(
                            TurnSteerRequestError::ExpectedTurnMismatch,
                        )),
                    ),
                    NotSubmittedReason::ActiveTurnNotSteerable { turn_kind } => {
                        let (message, turn_steer_error) = match turn_kind {
                            codex_protocol::protocol::NonSteerableTurnKind::Review => (
                                "cannot steer a review turn".to_string(),
                                TurnSteerRequestError::NonSteerableReview,
                            ),
                            codex_protocol::protocol::NonSteerableTurnKind::Compact => (
                                "cannot steer a compact turn".to_string(),
                                TurnSteerRequestError::NonSteerableCompact,
                            ),
                        };
                        let error = TurnError {
                            misalignment: None,
                            message: message.clone(),
                            codex_error_info: Some(CodexErrorInfo::ActiveTurnNotSteerable {
                                turn_kind: turn_kind.into(),
                            }),
                            additional_details: None,
                        };
                        let data = match serde_json::to_value(error) {
                            Ok(data) => Some(data),
                            Err(error) => {
                                tracing::error!(
                                    ?error,
                                    "failed to serialize active-turn-not-steerable turn error"
                                );
                                None
                            }
                        };
                        (
                            message,
                            data,
                            Some(AnalyticsJsonRpcError::TurnSteer(turn_steer_error)),
                        )
                    }
<<<<<<< HEAD
                    SteerInputError::AuthorizationChanged(_) => {
                        tracing::info!(
                            boundary = "authorizationChanged",
                            input_disposition = "returned",
                            "turn/steer admission refused"
                        );
                        (
                            "Permissions changed since this turn started. Wait for it to finish or stop it, then submit your message again.".to_string(),
                            Some(serde_json::json!({
                                "code": "authorizationChanged",
                                "inputDisposition": "returned",
                                "input": returned_input,
                            })),
                            None,
                        )
                    }
                    SteerInputError::EmptyInput => (
||||||| 413492cd6c
                    SteerInputError::EmptyInput => (
=======
                    NotSubmittedReason::EmptyInput => (
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
                        "input must not be empty".to_string(),
                        None,
                        Some(AnalyticsJsonRpcError::Input(InputError::Empty)),
                    ),
                    NotSubmittedReason::ActiveTurnOutputSchemaMismatch => (
                        "active turn uses a different output schema".to_string(),
                        None,
                        None,
                    ),
                    NotSubmittedReason::PendingTriggerTurn
                    | NotSubmittedReason::PlanMode
                    | NotSubmittedReason::Superseded => (
                        "no active turn to steer".to_string(),
                        None,
                        Some(AnalyticsJsonRpcError::TurnSteer(
                            TurnSteerRequestError::NoActiveTurn,
                        )),
                    ),
                };
                let mut error = invalid_request(message);
                error.data = data;
                self.track_error_response(request_id, &error, error_type);
                return Err(error);
            }
        };
        Ok(TurnSteerResponse { turn_id })
    }

    async fn prepare_realtime_conversation_thread(
        &self,
        request_id: &ConnectionRequestId,
        thread_id: &str,
    ) -> Result<Option<(ThreadId, Arc<CodexThread>)>, JSONRPCErrorError> {
        let (thread_id, thread) = self.load_thread(thread_id).await?;
        self.ensure_direct_input_allowed(request_id, thread.as_ref())
            .await?;

        match self
            .ensure_conversation_listener(
                thread_id,
                request_id.connection_id,
                /*raw_events_enabled*/ false,
            )
            .await
        {
            Ok(EnsureConversationListenerResult::Attached) => {}
            Ok(EnsureConversationListenerResult::ConnectionClosed) => {
                return Ok(None);
            }
            Err(error) => return Err(error),
        }

        Ok(Some((thread_id, thread)))
    }

    async fn thread_realtime_start_inner(
        &self,
        request_id: &ConnectionRequestId,
        params: ThreadRealtimeStartParams,
        sideband_base_url: Option<String>,
    ) -> Result<Option<ThreadRealtimeStartResponse>, JSONRPCErrorError> {
        let attaches_existing_call = matches!(
            &params.transport,
            Some(ThreadRealtimeStartTransport::ExistingCall { .. })
        );
        if attaches_existing_call {
            let unsupported_option = if params.include_startup_context == Some(true) {
                Some("includeStartupContext")
            } else if params.prompt.is_some() {
                Some("prompt")
            } else if params
                .initial_items
                .as_ref()
                .is_some_and(|items| !items.is_empty())
            {
                Some("initialItems")
            } else if params.model.is_some() {
                Some("model")
            } else if params.voice.is_some() {
                Some("voice")
            } else if params.delegation_ack_filler.is_some() {
                Some("delegationAckFiller")
            } else {
                None
            };
            if let Some(option) = unsupported_option {
                return Err(invalid_request(format!(
                    "existingCall transport does not support {option}"
                )));
            }
        }
        let Some((_, thread)) = self
            .prepare_realtime_conversation_thread(request_id, &params.thread_id)
            .await?
        else {
            return Ok(None);
        };
        self.submit_core_op(
            request_id,
            thread.as_ref(),
            Op::RealtimeConversationStart(ConversationStartParams {
                client_managed_handoffs: params.client_managed_handoffs.unwrap_or(false),
                delegation_ack_filler: params.delegation_ack_filler,
                flush_transcript_tail_on_session_end: params
                    .flush_transcript_tail_on_session_end
                    .unwrap_or(false),
                codex_responses_as_items: params.codex_responses_as_items.unwrap_or(false),
                codex_response_item_prefix: params.codex_response_item_prefix,
                codex_response_handoff_mode: params.codex_response_handoff_mode.unwrap_or_default(),
                backend_reasoning_status: params.backend_reasoning_status,
                codex_response_handoff_channel_prefixes: params
                    .codex_response_handoff_channel_prefixes,
                model: params.model,
                output_modality: params.output_modality,
                include_startup_context: params
                    .include_startup_context
                    .unwrap_or(!attaches_existing_call),
                initial_items: params
                    .initial_items
                    .unwrap_or_default()
                    .into_iter()
                    .map(|item| ConversationTextParams {
                        text: item.text,
                        role: item.role,
                    })
                    .collect(),
                realtime_start_instructions: params.realtime_start_instructions,
                realtime_end_instructions: params.realtime_end_instructions,
                prompt: params.prompt,
                realtime_session_id: params.realtime_session_id,
                voice_session_id: params.voice_session_id,
                transport: params.transport.map(|transport| match transport {
                    ThreadRealtimeStartTransport::Websocket => {
                        ConversationStartTransport::Websocket
                    }
                    ThreadRealtimeStartTransport::Webrtc { sdp } => {
                        ConversationStartTransport::Webrtc { sdp }
                    }
                    ThreadRealtimeStartTransport::ExistingCall { call_id, .. } => {
                        ConversationStartTransport::ExistingCall {
                            call_id,
                            sideband_base_url,
                        }
                    }
                }),
                version: params.version,
                voice: params.voice,
            }),
        )
        .await
        .map_err(|err| internal_error(format!("failed to start realtime conversation: {err}")))?;
        Ok(Some(ThreadRealtimeStartResponse::default()))
    }

    async fn thread_realtime_append_audio_inner(
        &self,
        request_id: &ConnectionRequestId,
        params: ThreadRealtimeAppendAudioParams,
    ) -> Result<Option<ThreadRealtimeAppendAudioResponse>, JSONRPCErrorError> {
        let Some((_, thread)) = self
            .prepare_realtime_conversation_thread(request_id, &params.thread_id)
            .await?
        else {
            return Ok(None);
        };
        self.submit_core_op(
            request_id,
            thread.as_ref(),
            Op::RealtimeConversationAudio(ConversationAudioParams {
                frame: params.audio.into(),
            }),
        )
        .await
        .map_err(|err| {
            internal_error(format!(
                "failed to append realtime conversation audio: {err}"
            ))
        })?;
        Ok(Some(ThreadRealtimeAppendAudioResponse::default()))
    }

    async fn thread_realtime_append_text_inner(
        &self,
        request_id: &ConnectionRequestId,
        params: ThreadRealtimeAppendTextParams,
    ) -> Result<Option<ThreadRealtimeAppendTextResponse>, JSONRPCErrorError> {
        let Some((_, thread)) = self
            .prepare_realtime_conversation_thread(request_id, &params.thread_id)
            .await?
        else {
            return Ok(None);
        };
        self.submit_core_op(
            request_id,
            thread.as_ref(),
            Op::RealtimeConversationText(ConversationTextParams {
                text: params.text,
                role: params.role,
            }),
        )
        .await
        .map_err(|err| {
            internal_error(format!(
                "failed to append realtime conversation text: {err}"
            ))
        })?;
        Ok(Some(ThreadRealtimeAppendTextResponse::default()))
    }

    async fn thread_realtime_append_speech_inner(
        &self,
        request_id: &ConnectionRequestId,
        params: ThreadRealtimeAppendSpeechParams,
    ) -> Result<Option<ThreadRealtimeAppendSpeechResponse>, JSONRPCErrorError> {
        let Some((_, thread)) = self
            .prepare_realtime_conversation_thread(request_id, &params.thread_id)
            .await?
        else {
            return Ok(None);
        };
        self.submit_core_op(
            request_id,
            thread.as_ref(),
            Op::RealtimeConversationSpeech(ConversationSpeechParams { text: params.text }),
        )
        .await
        .map_err(|err| {
            internal_error(format!(
                "failed to append realtime conversation speech: {err}"
            ))
        })?;
        Ok(Some(ThreadRealtimeAppendSpeechResponse::default()))
    }

    async fn thread_realtime_stop_inner(
        &self,
        request_id: &ConnectionRequestId,
        params: ThreadRealtimeStopParams,
    ) -> Result<Option<ThreadRealtimeStopResponse>, JSONRPCErrorError> {
        let Some((_, thread)) = self
            .prepare_realtime_conversation_thread(request_id, &params.thread_id)
            .await?
        else {
            return Ok(None);
        };
        self.submit_core_op(request_id, thread.as_ref(), Op::RealtimeConversationClose)
            .await
            .map_err(|err| {
                internal_error(format!("failed to stop realtime conversation: {err}"))
            })?;
        Ok(Some(ThreadRealtimeStopResponse::default()))
    }

    fn build_review_turn(turn_id: String, display_text: String) -> Turn {
        let item = ThreadItem::untimed(ThreadItemPayload::UserMessage {
            id: turn_id.clone(),
            client_id: None,
            content: vec![V2UserInput::Text {
                text: display_text,
                text_elements: Vec::new(),
            }],
        });

        Turn {
            id: turn_id,
            root_turn_id: None,
            items: vec![item],
            items_view: TurnItemsView::NotLoaded,
            error: None,
            status: TurnStatus::InProgress,
            started_at: None,
            completed_at: None,
            duration_ms: None,
        }
    }

    async fn turn_interrupt_inner(
        &self,
        request_id: &ConnectionRequestId,
        params: TurnInterruptParams,
    ) -> Result<Option<TurnInterruptResponse>, JSONRPCErrorError> {
        let TurnInterruptParams { thread_id, turn_id } = params;
        let is_startup_interrupt = turn_id.is_empty();

        let (thread_uuid, thread) = self.load_thread(&thread_id).await?;

        // Record turn interrupts so we can reply when TurnAborted arrives. Startup
        // interrupts do not have a turn and are acknowledged after submission.
        if !is_startup_interrupt {
            let thread_state = self.thread_state_manager.thread_state(thread_uuid).await;
            let is_running = matches!(thread.agent_status().await, AgentStatus::Running);
            {
                let mut thread_state = thread_state.lock().await;
                if let Some(active_turn_id) = thread_state.active_turn_id() {
                    if active_turn_id != turn_id {
                        return Err(invalid_request(format!(
                            "expected active turn id {turn_id} but found {active_turn_id}"
                        )));
                    }
                } else if thread_state.last_terminal_turn_id.as_deref() == Some(turn_id.as_str())
                    || !is_running
                {
                    return Err(invalid_request("no active turn to interrupt"));
                }
                thread_state.pending_interrupts.push(request_id.clone());
            }

            self.outgoing
                .record_request_turn_id(request_id, &turn_id)
                .await;
        }

        // Submit the interrupt. Turn interrupts respond upon TurnAborted; startup
        // interrupts respond here because startup cancellation has no turn event.
        match self
            .submit_core_op(request_id, thread.as_ref(), Op::Interrupt)
            .await
        {
            Ok(_) if is_startup_interrupt => Ok(Some(TurnInterruptResponse {})),
            Ok(_) => Ok(None),
            Err(err) => {
                if !is_startup_interrupt {
                    let thread_state = self.thread_state_manager.thread_state(thread_uuid).await;
                    let mut thread_state = thread_state.lock().await;
                    thread_state
                        .pending_interrupts
                        .retain(|pending_request_id| pending_request_id != request_id);
                }
                let interrupt_target = if is_startup_interrupt {
                    "startup"
                } else {
                    "turn"
                };
                Err(internal_error(format!(
                    "failed to interrupt {interrupt_target}: {err}"
                )))
            }
        }
    }

    fn listener_task_context(&self) -> ListenerTaskContext {
        ListenerTaskContext {
            thread_manager: Arc::clone(&self.thread_manager),
            thread_state_manager: self.thread_state_manager.clone(),
            outgoing: Arc::clone(&self.outgoing),
            pending_thread_unloads: Arc::clone(&self.pending_thread_unloads),
            thread_watch_manager: self.thread_watch_manager.clone(),
            codex_home: self.config.codex_home.to_path_buf(),
            skills_watcher: Arc::clone(&self.skills_watcher),
            turn_cost_worker: self.turn_cost_worker.clone(),
        }
    }

    async fn ensure_conversation_listener(
        &self,
        conversation_id: ThreadId,
        connection_id: ConnectionId,
        raw_events_enabled: bool,
    ) -> Result<EnsureConversationListenerResult, JSONRPCErrorError> {
        super::thread_lifecycle::ensure_conversation_listener(
            self.listener_task_context(),
            conversation_id,
            connection_id,
            raw_events_enabled,
        )
        .await
    }
}

fn xcode_26_4_mcp_elicitations_auto_deny(
    client_name: Option<&str>,
    client_version: Option<&str>,
) -> bool {
    // Xcode 26.4 shipped before app-server MCP elicitation requests were
    // client-visible. Keep elicitations auto-denied for that client line.
    // TODO: Remove this compatibility hack once Xcode 26.4 ages out.
    client_name == Some("Xcode")
        && client_version.is_some_and(|version| version.starts_with("26.4"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn execution_capacity_turn_start_error_is_structured() {
        let error =
            turn_start_submission_error(&CodexErr::new(CodexErrorDetails::AgentLimitReached {
                max_threads: 4,
            }));

        assert_eq!(
            error.data,
            Some(serde_json::json!({
                "turn_start_error": "execution_capacity",
                "max_threads": 4,
            }))
        );
    }

    #[test]
    fn unrelated_turn_start_error_is_not_marked_as_capacity() {
        let error = turn_start_submission_error(&CodexErr::InternalAgentDied);

        assert_eq!(error.data, None);
    }

    #[test]
    fn codex_tui_can_continue_its_persistent_v2_spawn_pane() {
        let source = SessionSource::SubAgent(SubAgentSource::ThreadSpawn {
            parent_thread_id: ThreadId::new(),
            depth: 1,
            agent_path: None,
            agent_nickname: Some("Burzum".to_string()),
            agent_role: Some("troll".to_string()),
            agent_class: None,
        });

        assert!(!direct_input_to_v2_thread_spawn_is_blocked(
            Some(MultiAgentVersion::V2),
            &source,
            Some("codex-tui"),
        ));
        assert!(direct_input_to_v2_thread_spawn_is_blocked(
            Some(MultiAgentVersion::V2),
            &source,
            Some("another-client"),
        ));
    }

    #[test]
    fn human_addressable_crew_members_accept_direct_input_from_every_client() {
        let source = SessionSource::SubAgent(SubAgentSource::ThreadSpawn {
            parent_thread_id: ThreadId::new(),
            depth: 1,
            agent_path: None,
            agent_nickname: Some("Angmar".to_string()),
            agent_role: Some("nazgul".to_string()),
            agent_class: Some(AgentClass::CrewMember {
                crew_id: "pfterminal-crew".to_string(),
                logical_member_id: "nazgul".to_string(),
                human_addressable: true,
            }),
        });

        assert!(can_accept_direct_input(
            Some(MultiAgentVersion::V2),
            &source,
        ));
        assert!(!direct_input_to_v2_thread_spawn_is_blocked(
            Some(MultiAgentVersion::V2),
            &source,
            Some("another-client"),
        ));
    }

    #[test]
    fn non_human_addressable_crew_members_remain_parent_controlled() {
        let source = SessionSource::SubAgent(SubAgentSource::ThreadSpawn {
            parent_thread_id: ThreadId::new(),
            depth: 1,
            agent_path: None,
            agent_nickname: Some("background-worker".to_string()),
            agent_role: Some("worker".to_string()),
            agent_class: Some(AgentClass::CrewMember {
                crew_id: "background-crew".to_string(),
                logical_member_id: "worker".to_string(),
                human_addressable: false,
            }),
        });

        assert!(!can_accept_direct_input(
            Some(MultiAgentVersion::V2),
            &source,
        ));
        assert!(direct_input_to_v2_thread_spawn_is_blocked(
            Some(MultiAgentVersion::V2),
            &source,
            Some("another-client"),
        ));
    }

    /// Which account a reported send bills is decided here, from the route,
    /// and not by the client that reports it.
    #[test]
    fn only_a_known_provider_s_own_route_is_recorded() {
        // The bridge's upstreams, canonicalised by the catalogue rather than
        // by however the caller spelled them.
        assert_eq!(
            recognised_reported_route(
                "ambient",
                "https://api.ambient.xyz/v1/",
                "/chat/completions"
            ),
            Some((
                "https://api.ambient.xyz/v1".to_string(),
                "chat/completions".to_string(),
                WireApi::Chat
            ))
        );
        assert_eq!(
            recognised_reported_route(
                "vercel-anthropic",
                "https://ai-gateway.vercel.sh/v1",
                "messages"
            ),
            Some((
                "https://ai-gateway.vercel.sh/v1".to_string(),
                "messages".to_string(),
                WireApi::Anthropic
            ))
        );
        // A provider this build does not ship.
        assert_eq!(
            recognised_reported_route("evil", "https://evil.example.com/v1", "messages"),
            None
        );
        // A known provider, on a route it does not serve: the gateway's spend
        // cannot be moved onto Anthropic's route, nor Anthropic's onto the
        // gateway's.
        assert_eq!(
            recognised_reported_route("anthropic", "https://ai-gateway.vercel.sh/v1", "messages"),
            None
        );
        // A known provider, on its own route but in another dialect.
        assert_eq!(
            recognised_reported_route("ambient", "https://api.ambient.xyz/v1", "responses"),
            None
        );
    }
}
