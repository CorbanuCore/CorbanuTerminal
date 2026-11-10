pub(crate) mod startup;

use crate::PromptCacheKey;
use crate::WithTurnExtensionData;
use std::borrow::Cow;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Debug;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::time::Instant;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use crate::agent::AgentStatus;
use crate::agent::agent_status_from_event;
use crate::agent::api::AgentConfigUpdate;
use crate::agent::api::AgentTurnOutcome;
use crate::agent::control::AgentControlInit;
use crate::agent::status::is_final;
use crate::agents_md_manager::SessionInstructions;
use crate::attestation::AttestationProvider;
use crate::compact;
use crate::compact::CompactedHistoryMetadata;
use crate::config::ManagedFeatures;
use crate::context::ContextualUserFragment;
use crate::context::DeveloperInstructions;
use crate::context::GuardianPolicy;
use crate::context::ManagedDeveloperInstructions;
use crate::context::ModelSwitchInstructions;
use crate::context::MultiAgentRoleInstructions;
use crate::context::NetworkRuleSaved;
use crate::context::RecommendedPluginsInstructions;
use crate::context::world_state::Placement;
use crate::context::world_state::WorldState;
use crate::context::world_state::WorldStateSnapshot;
use crate::context::world_state::WorldStateUpdate;
use crate::context::world_state::WorldStateUpdateContent;
use crate::context::world_state::split_prefix_updates;
use crate::current_time::TimeProvider;
use crate::environment_selection::TurnEnvironmentSnapshot;
use crate::exec_policy::BANNED_PREFIX_SUGGESTIONS;
use crate::exec_policy::ExecPolicyManager;
use crate::exec_policy::default_policy_path;
use crate::image_preparation::ImagePreparationMode;
use crate::image_preparation::ImageResizeNoticeMode;
use crate::image_preparation::prepare_response_items as prepare_image_response_items;
use crate::image_preparation::unified_image_budget_enabled;
use crate::parse_turn_item;
use crate::realtime_conversation::RealtimeConversationManager;
use crate::realtime_history::RealtimeEventOrder;
use crate::session::step_context::StepContext;
use crate::session::step_settings::ResolvedStepSettings;
use crate::session::step_settings::StepSettings;
use crate::session::turn_context::TurnEnvironment;
use crate::shell_snapshot::SnapshotCredentialBrokerState;
use crate::stream_events_utils::mark_thread_memory_mode_polluted_if_external_context;
use crate::turn_metadata::TurnMetadataState;
use crate::turn_timing::now_unix_timestamp_ms;
use async_channel::Receiver;
use async_channel::Sender;
use chrono::DateTime;
use chrono::Local;
use chrono::Utc;
use codex_analytics::AnalyticsEventsClient;
use codex_analytics::ImagePreparationFact;
use codex_analytics::ImagePreparationMetadata;
use codex_analytics::SubAgentThreadStartedInput;
use codex_analytics::ThreadInitializationMode;
use codex_analytics::TurnCodexErrorFact;
use codex_async_utils::OrCancelExt;
use codex_attachment_store::AttachmentStore;
use codex_attachment_store::InlineAttachmentStore;
use codex_connectors::connector_runtime_context_key;
<<<<<<< HEAD
use codex_core_skills::injection::HOST_SKILLS_CONTEXT_SOURCE_ID;
use codex_core_skills::injection::HostSkillsCatalogInWorldState;
||||||| base
use codex_core_skills::injection::HostSkillsCatalogInWorldState;
=======
use codex_context_fragments::RenderedFragment;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
use codex_exec_server::Environment;
use codex_exec_server::EnvironmentManager;
use codex_execpolicy::prefix_rule_migration;
use codex_extension_api::ConversationHistorySnapshot;
use codex_extension_api::ExtensionDataInit;
use codex_extension_api::PromptSlot;
use codex_extension_api::TurnContextContributionInput;
use codex_features::FEATURES;
use codex_features::Feature;
use codex_features::Features;
use codex_features::unstable_features_warning_event;
use codex_history::HistoryInitialization;
use codex_history::RolloutItem;
use codex_hooks::Hooks;
use codex_hooks::HooksConfig;
use codex_login::AuthManager;
use codex_login::CodexAuth;
use codex_login::auth_env_telemetry::collect_auth_env_telemetry;
use codex_mcp::McpResourceClient;
use codex_mcp::McpRuntime;
use codex_mcp::McpRuntimeContext;
use codex_mcp::McpRuntimeInput;
use codex_models_manager::manager::RefreshStrategy;
use codex_models_manager::manager::SharedModelsManager;
use codex_network_proxy::NetworkProxy;
use codex_network_proxy::NetworkProxyAuditMetadata;
use codex_network_proxy::normalize_host;
use codex_otel::auth_storage::AuthStorageOriginator;
use codex_otel::current_span_trace_id;
use codex_otel::current_span_w3c_trace_context;
use codex_otel::set_parent_from_w3c_trace_context;
use codex_prompts::ResolvedModelMessages;
use codex_prompts::render_model_instructions;
use codex_protocol::ResponseUsageMetadata;
use codex_protocol::SessionId;
use codex_protocol::ThreadId;
use codex_protocol::approvals::ElicitationRequest;
use codex_protocol::approvals::ElicitationRequestEvent;
use codex_protocol::approvals::ExecPolicyAmendment;
use codex_protocol::approvals::NetworkPolicyAmendment;
use codex_protocol::approvals::NetworkPolicyRuleAction;
use codex_protocol::capabilities::SelectedCapabilityRoot;
use codex_protocol::config_types::ApprovalsReviewer;
use codex_protocol::config_types::AutoCompactTokenLimitScope;
use codex_protocol::config_types::SERVICE_TIER_DEFAULT_REQUEST_VALUE;
use codex_protocol::config_types::ServiceTier;
use codex_protocol::config_types::WebSearchMode;
use codex_protocol::dynamic_tools::DynamicToolResponse;
use codex_protocol::dynamic_tools::DynamicToolSpec;
use codex_protocol::items::EnteredReviewModeItem;
use codex_protocol::items::ModelInvocationContext;
use codex_protocol::items::TurnItem;
use codex_protocol::items::UserMessageItem;
use codex_protocol::models::ActivePermissionProfile;
use codex_protocol::models::AdditionalPermissionProfile;
use codex_protocol::models::BaseInstructions;
use codex_protocol::models::BaseInstructionsProvenance;
use codex_protocol::models::ContentItem;
use codex_protocol::models::ContentItemKind;
use codex_protocol::models::ContentItemMetadata;
use codex_protocol::models::ContentItemNamespace;
use codex_protocol::models::ImageReference;
use codex_protocol::models::InternalChatMessageMetadataPassthrough;
use codex_protocol::models::PermissionProfile;
use codex_protocol::models::SandboxEnforcement;
<<<<<<< HEAD
use codex_protocol::models::format_allow_prefixes;
use codex_protocol::openai_models::ModelBilling;
use codex_protocol::openai_models::ModelCapabilityTier;
||||||| base
use codex_protocol::models::format_allow_prefixes;
=======
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
use codex_protocol::openai_models::ModelInfo;
use codex_protocol::openai_models::ModelOrchestrationMetadata;
use codex_protocol::openai_models::ModelPreset;
use codex_protocol::permissions::FileSystemSandboxPolicy;
use codex_protocol::permissions::FileSystemSandboxPolicyContext;
use codex_protocol::permissions::NetworkSandboxPolicy;
use codex_protocol::protocol::FileChange;
use codex_protocol::protocol::HasLegacyEvent;
use codex_protocol::protocol::HistoryPosition;
use codex_protocol::protocol::ItemCompletedEvent;
use codex_protocol::protocol::ItemStartedEvent;
use codex_protocol::protocol::MULTI_AGENT_MODE_OPEN_TAG;
use codex_protocol::protocol::MultiAgentVersion;
use codex_protocol::protocol::PrivateSessionEvent;
use codex_protocol::protocol::PrivateSessionEventPayload;
use codex_protocol::protocol::RawResponseItemEvent;
use codex_protocol::protocol::SessionSource;
use codex_protocol::protocol::SubAgentSource;
use codex_protocol::protocol::ThreadHistoryMode;
use codex_protocol::protocol::ThreadSettingsAppliedEvent;
use codex_protocol::protocol::ThreadSettingsOverrides;
use codex_protocol::protocol::ThreadSettingsSnapshot;
use codex_protocol::protocol::ThreadSource;
use codex_protocol::protocol::TurnAbortReason;
use codex_protocol::protocol::TurnContextItem;
use codex_protocol::protocol::TurnEnvironmentRequest;
use codex_protocol::protocol::TurnEnvironmentSelection;
use codex_protocol::protocol::TurnStartedEvent;
use codex_protocol::protocol::W3cTraceContext;
use codex_protocol::protocol::WorldStateItem;
use codex_protocol::request_permissions::PermissionGrantScope;
use codex_protocol::request_permissions::RequestPermissionProfile;
use codex_protocol::request_permissions::RequestPermissionsEvent;
use codex_protocol::request_permissions::RequestPermissionsResponse;
use codex_protocol::request_user_input::RequestUserInputArgs;
use codex_protocol::request_user_input::RequestUserInputResponse;
use codex_rmcp_client::ElicitationResponse;
use codex_rollout::should_persist_response_item;
use codex_rollout::state_db;
use codex_rollout_trace::ThreadStartedTraceMetadata;
use codex_rollout_trace::ThreadTraceContext;
use codex_sandboxing::SandboxType;
use codex_sandboxing::policy_transforms::intersect_permission_profiles_with_context;
use codex_shell_command::parse_command::parse_command;
use codex_terminal_detection::user_agent;
use codex_thread_store::CreateThreadParams;
use codex_thread_store::LiveThread;
use codex_thread_store::LiveThreadInitGuard;
use codex_thread_store::LocalThreadStore;
use codex_thread_store::PersistContext;
use codex_thread_store::ReadThreadParams;
use codex_thread_store::ResumeThreadParams;
use codex_thread_store::ThreadPersistenceMetadata;
use codex_thread_store::ThreadStore;
use codex_thread_store::ThreadStoreResumePreparation;
use codex_utils_audio::prepare_response_items as prepare_audio_response_items;
use codex_utils_git_discovery::GitRootDiscovery;
use codex_utils_output_truncation::with_serialization_allowance;
use codex_utils_path_uri::PathUri;
use futures::future::BoxFuture;
use futures::future::Shared;
use futures::prelude::*;
use rmcp::model::RequestId;
use serde_json::Value;
use tokio::sync::Mutex;
use tokio::sync::RwLock;
use tokio::sync::oneshot;
use tokio::sync::watch;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use toml::Value as TomlValue;
use tracing::Instrument;
use tracing::debug;
use tracing::error;
use tracing::info;
use tracing::info_span;
use tracing::instrument;
use tracing::warn;
use uuid::Uuid;

use crate::client::ModelClient;
use crate::codex_thread::CodexThreadSettingsOverrides;
use crate::codex_thread::ThreadConfigSnapshot;
#[cfg(test)]
use crate::compact::collect_user_messages;
use crate::config::Config;
use crate::config::ConstraintResult;
use crate::config::PermissionProfileSnapshot;
use crate::config::PermissionProfileState;
use crate::config::StartedNetworkProxy;
use crate::config::resolve_web_search_mode_for_turn;
use crate::context_manager::ContextManager;
<<<<<<< HEAD
use crate::context_manager::updates::ContextSection;
||||||| base
=======
use crate::context_manager::HistoryReplacement;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
use crate::thread_rollout_truncation::initial_history_has_prior_user_turns;
use codex_config::CONFIG_TOML_FILE;
use codex_config::ConfigLayerSource;
use codex_config::types::McpServerConfig;
use codex_model_provider::create_model_provider;
use codex_model_provider_info::ModelProviderInfo;
use codex_protocol::error::CodexErr;
use codex_protocol::error::CodexErrorDetails;
use codex_protocol::error::ResponseIncompleteError;
use codex_protocol::error::Result as CodexResult;
#[cfg(test)]
use codex_protocol::exec_output::StreamOutput;

pub(crate) mod async_compaction;
mod code_mode_warning;
<<<<<<< HEAD
mod config_lock;
mod confirmed_aggressive;
||||||| base
mod config_lock;
=======
mod config_refresh;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
pub(crate) mod context_window;
mod daemon_recovery;
mod environment;
mod extension_interruption;
pub(crate) mod extension_metrics;
mod guardian_checkpoint;
mod handlers;
pub(crate) mod inject;
mod reasoning_effort;
mod submission;
pub(crate) use reasoning_effort::RequestEffortUsage;
pub(crate) use submission::Submission;
mod input_queue;
mod mcp;
mod mcp_prewarm;
mod mcp_refresh;
mod mcp_runtime;
pub(crate) mod memory_stage_one;
pub(crate) mod multi_agents;
<<<<<<< HEAD
mod output_text_stream;
||||||| base
=======
mod plugin_selection;
mod realtime_history;
mod retained_context;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
mod review;
mod rollout_budget;
mod rollout_reconstruction;
#[allow(clippy::module_inception)]
pub(crate) mod session;
pub(crate) mod startup_prewarm;
mod step_activation;
pub(crate) mod step_context;
pub(crate) mod step_settings;
mod thread_settings;
pub(crate) mod time_reminder;
mod token_budget;
pub(crate) mod turn;
mod turn_completion;
pub(crate) mod turn_context;
mod turn_input;
mod turn_suspension;
mod world_state;
use self::code_mode_warning::unsupported_code_mode_warning;
pub(crate) use self::environment::ThreadEnvironmentDefaults;
#[cfg(test)]
use self::handlers::submission_dispatch_span;
use self::handlers::submission_loop;
pub(crate) use self::input_queue::InputQueueActivity;
pub(crate) use self::input_queue::TurnInput;
pub(crate) use self::input_queue::TurnInputQueue;
pub(crate) use self::input_queue::UserInputMetadata;
use self::review::spawn_review_thread;
use self::session::AppServerClientMetadata;
use self::session::Session;
use self::session::SessionConfiguration;
use self::session::SessionSettingsCommit;
pub(crate) use self::session::SessionSettingsUpdate;
#[cfg(test)]
use self::turn::AssistantMessageStreamParsers;
use self::turn::RealtimeEventText;
use self::turn::agent_message_text;
#[cfg(test)]
use self::turn::collect_explicit_app_ids_from_skill_items;
use self::turn::realtime_text_for_event;
use self::turn_context::TurnAuthority;
use self::turn_context::TurnContext;
#[cfg(test)]
mod rollout_reconstruction_tests;

<<<<<<< HEAD
#[derive(Debug, PartialEq)]
pub enum SteerInputError {
    NoActiveTurn(Vec<UserInput>),
    ExpectedTurnMismatch { expected: String, actual: String },
    ActiveTurnNotSteerable { turn_kind: NonSteerableTurnKind },
    AuthorizationChanged(Vec<UserInput>),
    EmptyInput,
}

impl SteerInputError {
    fn to_error_event(&self) -> ErrorEvent {
        match self {
            Self::NoActiveTurn(_) => ErrorEvent {
                message: "no active turn to steer".to_string(),
                codex_error_info: Some(CodexErrorInfo::BadRequest),
            },
            Self::ExpectedTurnMismatch { expected, actual } => ErrorEvent {
                message: format!("expected active turn id `{expected}` but found `{actual}`"),
                codex_error_info: Some(CodexErrorInfo::BadRequest),
            },
            Self::ActiveTurnNotSteerable { turn_kind } => {
                let turn_kind_label = match turn_kind {
                    NonSteerableTurnKind::Review => "review",
                    NonSteerableTurnKind::Compact => "compact",
                };
                ErrorEvent {
                    message: format!("cannot steer a {turn_kind_label} turn"),
                    codex_error_info: Some(CodexErrorInfo::ActiveTurnNotSteerable {
                        turn_kind: *turn_kind,
                    }),
                }
            }
            Self::AuthorizationChanged(_) => ErrorEvent {
                message: "Permissions changed since this turn started. Wait for it to finish or stop it, then submit your message again.".to_string(),
                codex_error_info: Some(CodexErrorInfo::BadRequest),
            },
            Self::EmptyInput => ErrorEvent {
                message: "input must not be empty".to_string(),
                codex_error_info: Some(CodexErrorInfo::BadRequest),
            },
        }
    }
}

/// PF-84: how long a switch to an account without a fresh cached model list
/// waits for that list (API-key sign-ins need no fetch and never wait).
const FIRST_MODEL_LIST_WAIT: std::time::Duration = std::time::Duration::from_secs(5);

/// Sends [`EventMsg::ExecApprovalRequestWithdrawn`] when a command approval
/// is dropped before it was answered (#722, D279).
struct WithdrawApprovalOnDrop {
    tx_event: Sender<Event>,
    event: Option<ExecApprovalRequestWithdrawnEvent>,
}

impl WithdrawApprovalOnDrop {
    fn disarm(mut self) {
        self.event = None;
    }
}

impl Drop for WithdrawApprovalOnDrop {
    fn drop(&mut self) {
        if let Some(event) = self.event.take() {
            // Unbounded channel: never blocks. A closed channel has no client.
            let _ = self.tx_event.try_send(Event {
                id: event.turn_id.clone(),
                msg: EventMsg::ExecApprovalRequestWithdrawn(event),
            });
        }
    }
}

/// Notes from the previous real user turn.
///
/// Conceptually this is the same role that `previous_model` used to fill, but
/// it can carry other prior-turn settings that matter when constructing
/// sensible state-change diffs or full-context reinjection, such as model
/// switches, compaction compatibility, or detecting a prior
/// `realtime_active -> false` transition.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PreviousTurnSettings {
    pub(crate) model: String,
    pub(crate) model_provider: Option<String>,
    pub(crate) comp_hash: Option<String>,
    pub(crate) realtime_active: Option<bool>,
}

#[cfg(test)]
use crate::SkillMetadata;
use crate::SkillsService;
||||||| base
#[derive(Debug, PartialEq)]
pub enum SteerInputError {
    NoActiveTurn(Vec<UserInput>),
    ExpectedTurnMismatch { expected: String, actual: String },
    ActiveTurnNotSteerable { turn_kind: NonSteerableTurnKind },
    EmptyInput,
}

impl SteerInputError {
    fn to_error_event(&self) -> ErrorEvent {
        match self {
            Self::NoActiveTurn(_) => ErrorEvent {
                message: "no active turn to steer".to_string(),
                codex_error_info: Some(CodexErrorInfo::BadRequest),
            },
            Self::ExpectedTurnMismatch { expected, actual } => ErrorEvent {
                message: format!("expected active turn id `{expected}` but found `{actual}`"),
                codex_error_info: Some(CodexErrorInfo::BadRequest),
            },
            Self::ActiveTurnNotSteerable { turn_kind } => {
                let turn_kind_label = match turn_kind {
                    NonSteerableTurnKind::Review => "review",
                    NonSteerableTurnKind::Compact => "compact",
                };
                ErrorEvent {
                    message: format!("cannot steer a {turn_kind_label} turn"),
                    codex_error_info: Some(CodexErrorInfo::ActiveTurnNotSteerable {
                        turn_kind: *turn_kind,
                    }),
                }
            }
            Self::EmptyInput => ErrorEvent {
                message: "input must not be empty".to_string(),
                codex_error_info: Some(CodexErrorInfo::BadRequest),
            },
        }
    }
}

/// Notes from the previous real user turn.
///
/// Conceptually this is the same role that `previous_model` used to fill, but
/// it can carry other prior-turn settings that matter when constructing
/// sensible state-change diffs or full-context reinjection, such as model
/// switches, compaction compatibility, or detecting a prior
/// `realtime_active -> false` transition.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PreviousTurnSettings {
    pub(crate) model: String,
    pub(crate) comp_hash: Option<String>,
    pub(crate) realtime_active: Option<bool>,
}

#[cfg(test)]
use crate::SkillMetadata;
use crate::SkillsService;
=======
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
use crate::exec_policy::ExecPolicyUpdateError;
use crate::guardian::GuardianReviewSessionManager;
use crate::mcp::McpEnvironmentScope;
use crate::mcp::McpManager;
use crate::mcp::McpThreadIdentity;
use crate::network_policy_decision::execpolicy_network_rule_amendment;
use crate::rollout::map_session_init_error;
use crate::session::startup_prewarm::SessionStartupPrewarmHandle;
use crate::shell;
use crate::state::AcceptedUserInputResponse;
use crate::state::AutoCompactWindowIds;
use crate::state::AutoCompactWindowSnapshot;
use crate::state::PendingApproval;
use crate::state::PendingRequestPermissions;
use crate::state::ReasoningEffortPin;
use crate::state::SessionServices;
use crate::state::SessionState;
use crate::state::TaskKind;
#[cfg(test)]
use crate::stream_events_utils::HandleOutputCtx;
#[cfg(test)]
use crate::stream_events_utils::handle_output_item_done;
use crate::tasks::ReviewTask;
use crate::tools::ApprovalContext;
use crate::tools::handlers::PreparedPermissionRequest;
use crate::tools::network_approval::NetworkApprovalService;
use crate::tools::network_approval::build_blocked_request_observer;
use crate::tools::network_approval::build_network_policy_decider;
#[cfg(test)]
use crate::tools::parallel::ToolCallRuntime;
use crate::tools::sandboxing::ApprovalStore;
use crate::turn_timing::TurnTimingState;
use crate::turn_timing::record_turn_ttfm_metric;
use crate::unified_exec::UnifiedExecProcessManager;
use crate::windows_sandbox::WindowsSandboxLevelExt;
use crate::windows_sandbox::local_binding_policy_for_sandbox;
use crate::windows_sandbox::managed_proxy_routing_for_windows_sandbox;
use codex_core_plugins::PluginCommandAttribution;
use codex_core_plugins::PluginsManager;
use codex_core_plugins::RecommendedPluginCandidatesInput;
use codex_git_utils::get_git_repo_root;
use codex_history::CodexHarnessMetadata;
use codex_history::CompactedItem;
use codex_history::CompactionResumeMetadata;
use codex_history::DynamicToolsSnapshot;
use codex_history::InitialHistory;
pub(crate) use codex_history::PreviousTurnSettings;
use codex_history::ResponseItemEnvelope;
use codex_mcp::McpConfig;
use codex_mcp::effective_mcp_servers;
use codex_otel::SessionTelemetry;
use codex_otel::THREAD_STARTED_METRIC;
use codex_otel::TelemetryAuthMode;
use codex_protocol::ResponseItemId;
use codex_protocol::approvals::ExecApprovalKind;
use codex_protocol::config_types::CollaborationMode;
use codex_protocol::config_types::ModeKind;
use codex_protocol::config_types::Personality;
use codex_protocol::config_types::ReasoningSummary as ReasoningSummaryConfig;
use codex_protocol::config_types::Settings;
use codex_protocol::config_types::WindowsSandboxLevel;
use codex_protocol::mcp::ClientMcpExtensions;
use codex_protocol::models::LocalImagePreparation;
use codex_protocol::models::ResponseInputItem;
use codex_protocol::models::ResponseItem;
use codex_protocol::openai_models::ReasoningEffort as ReasoningEffortConfig;
use codex_protocol::protocol::AccountTokenUsage;
use codex_protocol::protocol::ApplyPatchApprovalRequestEvent;
use codex_protocol::protocol::AskForApproval;
use codex_protocol::protocol::CodexErrorInfo;
use codex_protocol::protocol::DeprecationNoticeEvent;
use codex_protocol::protocol::Event;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::ExecApprovalRequestEvent;
use codex_protocol::protocol::ExecApprovalRequestWithdrawnEvent;
use codex_protocol::protocol::InitialHistory;
use codex_protocol::protocol::ModelRerouteEvent;
use codex_protocol::protocol::ModelRerouteReason;
use codex_protocol::protocol::ModelResponseCompletedEvent;
use codex_protocol::protocol::ModelVerification;
use codex_protocol::protocol::ModelVerificationEvent;
use codex_protocol::protocol::NetworkApprovalContext;
use codex_protocol::protocol::Op;
use codex_protocol::protocol::RateLimitSnapshot;
use codex_protocol::protocol::RawResponseCompletedEvent;
use codex_protocol::protocol::RequestUserInputEvent;
use codex_protocol::protocol::ReviewDecision;
use codex_protocol::protocol::SandboxPolicy;
use codex_protocol::protocol::SessionConfiguredEvent;
use codex_protocol::protocol::SessionNetworkProxyRuntime;
use codex_protocol::protocol::StreamErrorEvent;
use codex_protocol::protocol::ThreadMemoryMode;
use codex_protocol::protocol::TokenCountEvent;
use codex_protocol::protocol::TokenUsage;
use codex_protocol::protocol::TokenUsageInfo;
use codex_protocol::protocol::TokenUsageRecord;
use codex_protocol::protocol::TurnModerationMetadataEvent;
use codex_protocol::protocol::WarningEvent;
use codex_protocol::turn_input::TurnInputMode;
use codex_protocol::turn_input::TurnInputRequest;
use codex_protocol::turn_input::TurnInputSubmission;
use codex_protocol::turn_input::TurnStartOptions;
use codex_protocol::user_input::UserInput;
use codex_skills_extension::HostSkillsService;
use codex_tools::ToolName;
use codex_utils_absolute_path::AbsolutePathBuf;
#[cfg(test)]
use codex_utils_stream_parser::ProposedPlanSegment;

/// Whether reported usage updates the installed context baseline or only spending.
#[derive(Clone, Copy)]
pub(crate) enum TokenUsageContext {
    Sampling {
        includes_reasoning: bool,
        /// Input history length before the response, including requests with no output items.
        input_item_count: usize,
    },
    SpendingOnly,
}

/// Which active turn can service a network approval.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum NetworkApprovalTurn {
    Originating,
    AnyActive,
}

/// Queue and lifecycle endpoints for a running [`Session`].
///
/// Runtime state lives on `Session`; keeping these endpoints separate lets all
/// submission senders be dropped to terminate the session loop. The shared
/// completion future observes that shutdown.
#[derive(Clone)]
pub(crate) struct SessionIo {
    pub(crate) tx_sub: Sender<Submission>,
    pub(crate) rx_event: Receiver<Event>,
    // Last known status of the agent.
    pub(crate) agent_status: watch::Receiver<AgentStatus>,
    // Shared future for the background submission loop completion so multiple
    // callers can wait for shutdown.
    pub(crate) session_loop_termination: SessionLoopTermination,
}

pub(crate) type SessionLoopTermination = Shared<BoxFuture<'static, ()>>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GitEnrichmentPolicy {
    Fresh,
    Skip,
}

/// Controls which fork history belongs in the newly created thread's own rollout.
pub(crate) enum ForkPersistence {
    Copied,
    /// The spawn caller owns the durability barrier before acknowledging the child.
    CopiedDeferred,
    Referenced {
        history_base: Option<HistoryPosition>,
        inherited_item_count: usize,
    },
}

/// Normal appends keep this generation; compaction and rollback start a new one.
#[derive(Clone, Copy)]
pub(crate) struct CompactionPrefix {
    pub(crate) generation: u64,
    pub(crate) len: usize,
    pub(crate) auth_owner_generation: Option<u64>,
}

pub(crate) enum CompactedHistoryInstallation<'a> {
    Complete(CompactedHistoryMetadata),
    Prefix {
        prefix: CompactionPrefix,
        prefix_context: Vec<ResponseItem>,
        initial_context: Vec<ResponseItemEnvelope>,
        model_info: &'a ModelInfo,
        compaction_response_id: String,
        reviewer_compaction_hash: Option<String>,
        turn_context: &'a TurnContext,
        started_item: Option<&'a TurnItem>,
    },
}

impl From<CompactedHistoryMetadata> for CompactedHistoryInstallation<'_> {
    fn from(metadata: CompactedHistoryMetadata) -> Self {
        Self::Complete(metadata)
    }
}

pub(crate) struct SessionSpawnArgs {
    pub(crate) attached_thread: bool,
    pub(crate) startup: Option<Arc<startup::SessionStartup>>,
    pub(crate) config: Config,
    pub(crate) prompt_cache_key: PromptCacheKey,
    pub(crate) allow_provider_model_fallback: bool,
    pub(crate) instructions: SessionInstructions,
    pub(crate) installation_id: String,
    pub(crate) auth_manager: Arc<AuthManager>,
    pub(crate) models_manager: SharedModelsManager,
    pub(crate) git_root_discovery: Arc<GitRootDiscovery>,
    pub(crate) environment_manager: Arc<EnvironmentManager>,
    pub(crate) skills_service: Arc<HostSkillsService>,
    pub(crate) plugins_manager: Arc<PluginsManager>,
    pub(crate) mcp_manager: Arc<McpManager>,
    pub(crate) code_mode_session_provider: Arc<dyn codex_code_mode::CodeModeSessionProvider>,
    pub(crate) extensions: Arc<codex_extension_api::ExtensionRegistry<crate::config::Config>>,
    pub(crate) conversation_history: InitialHistory,
    pub(crate) resume_preparation: Option<ThreadStoreResumePreparation>,
    pub(crate) history_initialization: HistoryInitialization,
    pub(crate) disabled_plugin_ids: Option<Vec<String>>,
    pub(crate) requested_history_mode: Option<ThreadHistoryMode>,
    pub(crate) fork_persistence: ForkPersistence,
    pub(crate) session_source: SessionSource,
    pub(crate) forked_from_thread_id: Option<ThreadId>,
    pub(crate) parent_thread_id: Option<ThreadId>,
    pub(crate) thread_source: Option<ThreadSource>,
    pub(crate) originator: String,
    pub(crate) agent_control: AgentControlInit,
    pub(crate) dynamic_tools: Vec<DynamicToolSpec>,
    pub(crate) metrics_service_name: Option<String>,
    pub(crate) inherited_exec_policy: Option<Arc<ExecPolicyManager>>,
    pub(crate) inherited_environments: Option<TurnEnvironmentSnapshot>,
    /// Parent rollout trace used only to derive fresh spawned child traces.
    ///
    /// Root sessions and non-thread-spawn subagents pass a disabled context;
    /// `Session::new` creates the root trace itself when rollout tracing is enabled.
    pub(crate) parent_rollout_thread_trace: ThreadTraceContext,
    pub(crate) user_shell_override: Option<shell::Shell>,
    pub(crate) parent_trace: Option<W3cTraceContext>,
    pub(crate) environment_requests: Vec<TurnEnvironmentRequest>,
    pub(crate) thread_extension_init: ExtensionDataInit,
    pub(crate) turn_extension_init: ExtensionDataInit,
    pub(crate) client_mcp_extensions: ClientMcpExtensions,
    pub(crate) reserved_thread_id: Option<ThreadId>,
    pub(crate) analytics_events_client: Option<AnalyticsEventsClient>,
    pub(crate) image_store: Arc<dyn AttachmentStore>,
    pub(crate) thread_store: Arc<dyn ThreadStore>,
    pub(crate) attestation_provider: Option<Arc<dyn AttestationProvider>>,
    pub(crate) external_time_provider: Option<Arc<dyn TimeProvider>>,
    pub(crate) inherited_multi_agent_version: Option<MultiAgentVersion>,
    pub(crate) git_enrichment_policy: GitEnrichmentPolicy,
    pub(crate) windows_sandbox_proxy_settings_mode:
        codex_sandboxing::WindowsSandboxProxySettingsMode,
}

pub(crate) fn resolve_multi_agent_version(
    conversation_history: &InitialHistory,
    inherited_multi_agent_version: Option<MultiAgentVersion>,
) -> Option<MultiAgentVersion> {
    if inherited_multi_agent_version == Some(MultiAgentVersion::Disabled) {
        return Some(MultiAgentVersion::Disabled);
    }

    conversation_history
        .get_multi_agent_version()
        .or(inherited_multi_agent_version)
        .or(match conversation_history {
            InitialHistory::New | InitialHistory::Cleared => None,
            // Threads created before runtime metadata existed keep the legacy V1 tool surface.
            InitialHistory::Resumed(_) | InitialHistory::Forked(_) => Some(MultiAgentVersion::V1),
        })
}

pub(crate) const INITIAL_SUBMIT_ID: &str = "";
pub(crate) const SUBMISSION_CHANNEL_CAPACITY: usize = 512;
const CYBER_VERIFY_URL: &str = "https://chatgpt.com/cyber";
const CYBER_SAFETY_URL: &str = "https://developers.openai.com/codex/concepts/cyber-safety";

impl Session {
    /// Spawn and initialize a new session.
    /// Hide the concrete startup future from callers while keeping initialization lazy.
    #[inline(never)]
    pub(crate) fn spawn(
        args: SessionSpawnArgs,
    ) -> BoxFuture<'static, CodexResult<(Arc<Self>, SessionIo)>> {
        let tree_shutdown = args.agent_control.runtime().shutdown.clone();
        Box::pin(async move {
            let parent_trace = match args.parent_trace {
                Some(trace) => {
                    if codex_otel::context_from_w3c_trace_context(&trace).is_some() {
                        Some(trace)
                    } else {
                        warn!("ignoring invalid thread spawn trace carrier");
                        None
                    }
                }
                None => None,
            };
            let thread_spawn_span = info_span!("thread_spawn", otel.name = "thread_spawn");
            if let Some(trace) = parent_trace.as_ref() {
                let _ = set_parent_from_w3c_trace_context(&thread_spawn_span, trace);
            }
            let spawn = Self::spawn_internal(SessionSpawnArgs {
                parent_trace,
                ..args
            })
            .instrument(thread_spawn_span);
            tokio::select! {
                biased;
                _ = tree_shutdown.cancelled() => Err(CodexErr::TurnAborted),
                result = spawn => result,
            }
        })
    }

    async fn spawn_internal(args: SessionSpawnArgs) -> CodexResult<(Arc<Self>, SessionIo)> {
        args.config
            .config_layer_stack
            .ensure_config_server_applied()?;
        args.config.validate_windows_mxc_requirement()?;
        let SessionSpawnArgs {
            attached_thread,
            startup,
            config,
            prompt_cache_key,
            allow_provider_model_fallback,
            instructions,
            installation_id,
            auth_manager,
            models_manager,
            git_root_discovery,
            environment_manager,
            skills_service,
            plugins_manager,
            mcp_manager,
            code_mode_session_provider,
            extensions,
            conversation_history,
            resume_preparation,
            history_initialization,
            disabled_plugin_ids,
            requested_history_mode,
            fork_persistence,
            session_source,
            forked_from_thread_id,
            parent_thread_id,
            thread_source,
            originator,
            agent_control,
            dynamic_tools,
            metrics_service_name,
            user_shell_override,
            inherited_exec_policy,
            inherited_environments,
            parent_rollout_thread_trace,
            parent_trace: _,
            environment_requests,
            mut thread_extension_init,
            turn_extension_init,
            client_mcp_extensions,
            reserved_thread_id,
            analytics_events_client,
            image_store,
            thread_store,
            attestation_provider,
            external_time_provider,
            inherited_multi_agent_version,
            git_enrichment_policy,
            windows_sandbox_proxy_settings_mode,
        } = args;
        let (tx_sub, rx_sub) = async_channel::bounded(SUBMISSION_CHANNEL_CAPACITY);
        let (tx_event, rx_event) = async_channel::unbounded();

        let isolation = thread_extension_init
            .get::<codex_extension_api::SessionIsolation>()
            .map(|policy| *policy)
            .unwrap_or_default();
        // Manager-owned threads and inline delegates bind roots at the same startup boundary.
        let selected_capability_roots =
            match thread_extension_init.get::<Vec<SelectedCapabilityRoot>>() {
                Some(roots) => roots.as_ref().clone(),
                None => {
                    // Resume restores this thread's saved roots, including roots for environments
                    // it will select again later. New children inherit roots regardless of copied history.
                    let roots = if !matches!(&conversation_history, InitialHistory::Resumed(_))
                        && isolation == codex_extension_api::SessionIsolation::Inherit
                        && let Some(environments) = &inherited_environments
                    {
                        environments.selected_capability_roots()
                    } else {
                        conversation_history.get_selected_capability_roots()
                    };
                    if !roots.is_empty() {
                        thread_extension_init.insert(roots.clone());
                    }
                    roots
                }
            };
        let environment_selections = environment_requests
            .into_iter()
            .map(|request| TurnEnvironmentSelection::new(request, &selected_capability_roots))
            .collect::<Vec<_>>();
        // Enforce snapshot-only instructions for both managed and inline isolated sessions.
        let instructions = if isolation == codex_extension_api::SessionIsolation::Isolated {
            SessionInstructions {
                user: instructions.user,
                thread: instructions.thread,
                project_snapshot: instructions.project_snapshot,
                ..Default::default()
            }
        } else {
            instructions
        };
        let exec_policy = if isolation == codex_extension_api::SessionIsolation::Isolated {
            let source_requirements = config
                .config_layer_stack
                .requirements()
                .exec_policy
                .as_deref()
                .cloned()
                .map(Arc::new);
            let managed_policy = source_requirements
                .as_deref()
                .map_or_else(codex_execpolicy::Policy::empty, |policy| {
                    policy.as_ref().clone()
                });
            Arc::new(ExecPolicyManager::from_snapshot(
                Arc::new(managed_policy),
                source_requirements,
            ))
        } else if let Some(exec_policy) = &inherited_exec_policy {
            Arc::clone(exec_policy)
        } else {
            if !config
                .config_layer_stack
                .ignore_user_and_project_exec_policy_rules()
            {
                let codex_home = config.codex_home.clone();
                let policy_path = default_policy_path(codex_home.as_path());
                if let Err(err) = prefix_rule_migration(
                    codex_home.as_path(),
                    policy_path.as_path(),
                    BANNED_PREFIX_SUGGESTIONS,
                )
                .await
                {
                    tracing::warn!(error = %err, "failed to run prefix rule migration");
                }
            }
            Arc::new(
                ExecPolicyManager::load(&config.config_layer_stack, config.strict_rules)
                    .await
                    .map_err(|err| match err {
                        err @ crate::exec_policy::ExecPolicyError::ParsePolicy { .. }
                            if config.strict_rules =>
                        {
                            CodexErr::Fatal(format!(
                                "failed to load rules: {}. A rules file that does not parse stops new threads while strict_rules is on (security level Aggressive); fix or remove it, then start a new thread.",
                                crate::exec_policy::format_exec_policy_error_with_source(&err)
                            ))
                        }
                        err => CodexErr::Fatal(format!("failed to load rules: {err}")),
                    })?,
            )
        };

        let mut config = Arc::new(config);
        // PF-84: a user's session lists models with its own account's sign-in.
        if !session_source.is_non_root_agent() {
            let account = codex_model_provider::named_sign_in_account(&config.model_provider);
            models_manager.follow_sign_in_account(account.as_ref());
        }
        let refresh_strategy = if session_source.is_non_root_agent() {
            codex_models_manager::manager::RefreshStrategy::Offline
        } else {
            codex_models_manager::manager::RefreshStrategy::OnlineIfUncached
        };
        if config.model.is_none()
            || !matches!(
                refresh_strategy,
                codex_models_manager::manager::RefreshStrategy::Offline
            )
        {
            let _ = models_manager
                .list_models(refresh_strategy, config.http_client_factory())
                .await;
        }
        let model = models_manager
            .get_default_model(
                &config.model,
                allow_provider_model_fallback,
                refresh_strategy,
                config.http_client_factory(),
            )
            .await;
        if model.trim().is_empty() {
            return Err(CodexErr::InvalidRequest(
                "No models are available. Set `model` explicitly or check your model catalog configuration.".to_string(),
            ));
        }
        let trusted_guardian_reviewer = crate::guardian::is_basic_session_source(&session_source)
            && !matches!(conversation_history, InitialHistory::Resumed(_));
        if config
            .config_layer_stack
            .requirements()
            .auto_review_required_for_model(&model)
            && !trusted_guardian_reviewer
        {
            let config = Arc::make_mut(&mut config);
            if matches!(
                config.legacy_sandbox_policy(),
                SandboxPolicy::DangerFullAccess
            ) {
                let permission_profile = PermissionProfile::workspace_write();
                config
                    .permissions
                    .set_permission_profile(permission_profile.clone())
                    .map_err(|err| CodexErr::InvalidRequest(err.to_string()))?;
                if let Some(network) = config.permissions.network.as_ref() {
                    config.permissions.network = Some(
                        network
                            .recompute_for_permission_profile(&permission_profile)
                            .map_err(|err| CodexErr::InvalidRequest(err.to_string()))?,
                    );
                }
            }
            config
                .config_layer_stack
                .requirements()
                .approvals_reviewer
                .can_set(&ApprovalsReviewer::AutoReview)
                .map_err(|err| CodexErr::InvalidRequest(err.to_string()))?;
            config.approvals_reviewer = ApprovalsReviewer::AutoReview;
        }
        if allow_provider_model_fallback
            && let Some(requested_model) = config.model.as_ref()
            && model != *requested_model
        {
            info!(
                model_provider = %config.model_provider_id,
                requested_model,
                fallback_model = %model,
                "replaced unavailable requested model with provider default"
            );
        }

        // Resolve base instructions for the session. Priority order:
        // 1. config.base_instructions override
        // 2. conversation history => session_meta.base_instructions
        // 3. rendered instructions_template for current model
        let model_info = models_manager
            .get_model_info(
                model.as_str(),
                &config.to_models_manager_config_for_model(Some(model.as_str())),
            )
            .await;
        let auth = auth_manager.auth_cached();
        // Forked subagents keep their parent's activation with the copied history.
        // Fresh children restore configured preferences before applying startup defaults.
        let inherits_token_budget = matches!(&conversation_history, InitialHistory::Forked(_))
            && config.token_budget_startup_config.is_some();
        if !inherits_token_budget {
            Arc::make_mut(&mut config)
                .prepare_token_budget_for_startup()
                .map_err(|err| CodexErr::InvalidRequest(err.to_string()))?;
            // Resolve activation for this runtime, including when resuming saved history.
            token_budget::apply_experimental_context(
                Arc::make_mut(&mut config),
                auth.as_ref(),
                &model_info,
            )?;
            token_budget::apply_model_defaults(Arc::make_mut(&mut config), &model_info);
        }
        let configured_config = Arc::clone(&config);
        let multi_agent_version = config.multi_agent_version_override().or_else(|| {
            resolve_multi_agent_version(&conversation_history, inherited_multi_agent_version)
        });
        let history_mode = conversation_history.get_history_mode(
            requested_history_mode.unwrap_or_else(|| thread_store.default_history_mode()),
        );
        let base_instructions = config
            .base_instructions
            .clone()
            .or_else(|| conversation_history.get_base_instructions().map(|s| s.text))
            .unwrap_or_else(|| render_model_instructions(&model_info));

        // A fork records its initial catalog in the new thread's creation metadata.
        let dynamic_tools = if dynamic_tools.is_empty()
            && matches!(conversation_history, InitialHistory::Forked(_))
        {
            conversation_history.get_dynamic_tools().unwrap_or_default()
        } else {
            dynamic_tools
        };
        let disabled_plugin_ids = disabled_plugin_ids.unwrap_or_else(|| {
            let settings_owner = match &conversation_history {
                InitialHistory::Resumed(resumed) => Some(resumed.conversation_id),
                InitialHistory::Forked(_) => forked_from_thread_id,
                InitialHistory::New | InitialHistory::Cleared => None,
            };
            settings_owner
                .and_then(|thread_id| {
                    codex_history::latest_disabled_plugin_ids(
                        conversation_history.get_rollout_items(),
                        thread_id,
                    )
                })
                .map(<[String]>::to_vec)
                .unwrap_or_default()
        });
        // TODO (aibrahim): Consolidate config.model and config.model_reasoning_effort into config.collaboration_mode
        // to avoid extracting these fields separately and constructing CollaborationMode here.
        let persisted_collaboration_mode = match &conversation_history {
            InitialHistory::Resumed(resumed) => resumed
                .history
                .iter()
                .rev()
                .find_map(|item| match item {
                    RolloutItem::EventMsg(EventMsg::ThreadSettingsApplied(event))
                        if event.thread_id == Some(resumed.conversation_id) =>
                    {
                        Some(&event.thread_settings.collaboration_mode)
                    }
                    _ => None,
                })
                .or_else(|| {
                    resumed
                        .history
                        .iter()
                        .rev()
                        .find_map(|item| match item {
                            RolloutItem::TurnContext(context) => {
                                Some(context.collaboration_mode.as_ref())
                            }
                            _ => None,
                        })
                        .flatten()
                }),
            InitialHistory::New | InitialHistory::Cleared | InitialHistory::Forked(_) => None,
        };
        let collaboration_mode = CollaborationMode {
            mode: ModeKind::Default,
            settings: Settings {
                model: model.clone(),
                reasoning_effort: config.model_reasoning_effort.clone(),
                developer_instructions: None,
            },
        };
        let collaboration_mode = persisted_collaboration_mode.map_or(collaboration_mode, |saved| {
            saved.with_updates(
                Some(model.clone()),
                Some(config.model_reasoning_effort.clone()),
                /*developer_instructions*/ None,
            )
        });
        let initial_service_tier_warning = unsupported_service_tier_warning(
            config.service_tier.as_deref(),
            &config.features,
            &model_info,
        );
        let storage_originator = AuthStorageOriginator::from_client_name(&originator);
        let session_configuration = SessionConfiguration {
<<<<<<< HEAD
            provider: config.model_provider.clone(),
            collaboration_mode,
            model_reasoning_summary: config.model_reasoning_summary,
            service_tier,
            runtime_model_context_window: None,
            developer_instructions: config.developer_instructions.clone(),
            personality: config.personality,
            base_instructions,
            compact_prompt: config.compact_prompt.clone(),
            approval_policy: config.permissions.approval_policy.clone(),
            approvals_reviewer: config.approvals_reviewer,
            aggressive_approvals: false,
            permission_profile_state: session_permission_profile_state_from_config(&config)?,
            windows_sandbox_level: WindowsSandboxLevel::from_config(&config),
            environments: TurnEnvironmentSelections::new(
                config.cwd.clone(),
                environment_selections,
||||||| base
            provider: config.model_provider.clone(),
            collaboration_mode,
            model_reasoning_summary: config.model_reasoning_summary,
            service_tier,
            developer_instructions: config.developer_instructions.clone(),
            personality: config.personality,
            base_instructions,
            compact_prompt: config.compact_prompt.clone(),
            approval_policy: config.permissions.approval_policy.clone(),
            approvals_reviewer: config.approvals_reviewer,
            permission_profile_state: session_permission_profile_state_from_config(&config)?,
            windows_sandbox_level: WindowsSandboxLevel::from_config(&config),
            environments: TurnEnvironmentSelections::new(
                config.cwd.clone(),
                environment_selections,
=======
            turn_extension_init,
            provider: create_model_provider(
                config.model_provider.clone(),
                Some(Arc::clone(&auth_manager)),
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
            ),
            step_settings: Arc::new(StepSettings {
                collaboration_mode,
                reasoning_summary: config.model_reasoning_summary,
                service_tier: config.service_tier.clone(),
                personality: config.personality,
                approval_policy: config.permissions.approval_policy.clone(),
                approvals_reviewer: config.approvals_reviewer,
            }),
            environments: environment_selections.clone(),
            model_info_overrides: config.to_models_manager_config().into(),
            developer_instructions: config.developer_instructions.clone(),
            base_instructions,
            permission_profile_state: session_permission_profile_state_from_config(&config)?,
            allow_login_shell: config.permissions.allow_login_shell,
            shell_environment_policy: config.permissions.shell_environment_policy.clone(),
            windows_sandbox_level: WindowsSandboxLevel::from_config(&config),
            windows_sandbox_type: config.effective_local_windows_sandbox_type(),
            use_legacy_landlock: config.features.use_legacy_landlock(),
            legacy_fallback_cwd: config.cwd.clone(),
            runtime_workspace_roots: config.workspace_roots.clone(),
            codex_home: config.codex_home.clone(),
            thread_name: None,
            disabled_plugin_ids,
            original_config_do_not_use: Arc::clone(&config),
            metrics_service_name,
            app_server_client_name: None,
            app_server_client_version: None,
            trusted_guardian_reviewer,
            session_source,
            history_mode,
            forked_from_thread_id,
            parent_thread_id,
            thread_source,
            originator,
            dynamic_tools: Arc::from(dynamic_tools),
            user_shell_override,
        };
        session_configuration
            .validate(&environment_selections)
            .map_err(|err| CodexErr::InvalidRequest(err.to_string()))?;
        crate::windows_sandbox::log_windows_sandbox_startup(&config);

        // Generate a unique ID for the lifetime of this session.
        let session_source_clone = session_configuration.session_source.clone();
        let (agent_status_tx, agent_status_rx) = watch::channel(AgentStatus::PendingInit);

        let session = Box::pin(Session::new(
            startup.clone(),
            session_configuration,
            attached_thread,
            prompt_cache_key,
            &environment_selections,
            config.clone(),
            instructions,
            installation_id,
            auth_manager.clone(),
            models_manager.clone(),
            git_root_discovery,
            model_info,
            exec_policy,
            tx_sub.clone(),
            tx_event.clone(),
            agent_status_tx.clone(),
            conversation_history,
            history_initialization,
            fork_persistence,
            session_source_clone,
            skills_service,
            plugins_manager,
            mcp_manager.clone(),
            code_mode_session_provider,
            extensions,
            thread_extension_init,
            client_mcp_extensions,
            agent_control,
            reserved_thread_id,
            environment_manager,
            inherited_environments,
            analytics_events_client,
            image_store,
            thread_store,
            resume_preparation,
            parent_rollout_thread_trace,
            attestation_provider,
            external_time_provider,
            multi_agent_version,
            git_enrichment_policy,
            windows_sandbox_proxy_settings_mode,
        ))
        .await
        .map_err(|e| {
            error!("Failed to create session: {e:#}");
            match e.downcast::<CodexErr>() {
                Ok(error) => error,
                Err(error) => map_session_init_error(&error, &config.codex_home),
            }
        })?;
        if let Some(message) = initial_service_tier_warning {
            session
                .send_event_raw(Event {
                    id: INITIAL_SUBMIT_ID.to_owned(),
                    msg: EventMsg::Warning(WarningEvent { message }),
                })
                .await;
        }
        let thread_id = session.thread_id;

        // This task will run until Op::Shutdown is received.
        let tree_teardown = startup
            .as_ref()
            .and_then(|startup| startup.session_teardown(thread_id));
        let session_for_loop = Arc::clone(&session);
        let session_loop_handle = tokio::spawn(Box::pin(storage_originator.scope(async move {
            submission_loop(session_for_loop, configured_config, rx_sub)
                .instrument(info_span!("session_loop", thread_id = %thread_id))
                .await;
            if let Some(tree_teardown) = tree_teardown {
                tree_teardown.complete();
            }
        })));
        let io = SessionIo {
            tx_sub,
            rx_event,
            agent_status: agent_status_rx,
            session_loop_termination: session_loop_termination_from_handle(session_loop_handle),
        };

        if let Some(startup) = startup {
            let _ = startup.io.set(io.clone());
            startup.persistence.lock().await.commit();
        }
        Ok((session, io))
    }
}

impl SessionIo {
    /// Submit the `op` wrapped in a `Submission` with a unique ID.
    pub(crate) async fn submit(
        &self,
        op: impl Into<WithTurnExtensionData<Op>>,
    ) -> CodexResult<String> {
        self.submit_with_trace(
            op, /*trace*/ None, /*parent_turn_id*/ None, /*root_turn_id*/ None,
            /*residency_guard*/ None, /*message_delivery*/ None,
        )
        .await
    }

    pub(crate) async fn submit_with_trace(
        &self,
        op: impl Into<WithTurnExtensionData<Op>>,
        trace: Option<W3cTraceContext>,
        parent_turn_id: Option<String>,
        root_turn_id: Option<String>,
        residency_guard: Option<tokio::sync::OwnedRwLockReadGuard<()>>,
        message_delivery: Option<crate::agent_communication::MessageDelivery>,
    ) -> CodexResult<String> {
        let id = new_submission_id();
        let WithTurnExtensionData {
            request: op,
            turn_extension_init,
        } = op.into();
        let sub = Submission {
            id: id.clone(),
            op,
            turn_extension_init,
            trace,
            parent_turn_id,
            root_turn_id,
            residency_guard,
            message_delivery,
        };
        self.submit_with_id(sub).await?;
        Ok(id)
    }

    /// Use sparingly: prefer `submit()` so submission IDs are generated consistently.
    pub(crate) async fn submit_with_id(&self, mut sub: Submission) -> CodexResult<()> {
        if sub.trace.is_none() {
            sub.trace = current_span_w3c_trace_context();
        }
        self.tx_sub
            .send(sub)
            .await
            .map_err(|_| CodexErr::InternalAgentDied)?;
        Ok(())
    }

    /// Submits an ordered turn-input call and waits only for Core's routing decision.
    ///
    /// Once queued, dropping the waiter does not retract the call. If the
    /// session loop exits before replying, the caller gets `InternalAgentDied`.
    pub(crate) async fn submit_turn_input(
        &self,
        request: impl Into<WithTurnExtensionData<TurnInputRequest>>,
        mode: TurnInputMode,
    ) -> CodexResult<TurnInputSubmission> {
        let id = new_submission_id();
        let (reply_tx, reply_rx) = oneshot::channel();
        let WithTurnExtensionData {
            mut request,
            turn_extension_init,
        } = request.into();
        let trace = request.trace.take();
        self.submit_with_id(Submission {
            id,
            op: Op::TurnInput {
                request: Box::new(request),
                mode,
                reply: reply_tx,
            },
            trace,
            turn_extension_init,
            parent_turn_id: None,
            root_turn_id: None,
            residency_guard: None,
            message_delivery: None,
        })
        .await?;
        reply_rx.await.unwrap_or(Err(CodexErr::InternalAgentDied))
    }

    pub(crate) async fn submit_recover_turn(
        &self,
        thread_settings: ThreadSettingsOverrides,
        start_options: TurnStartOptions,
        trace: Option<W3cTraceContext>,
        turn_id: String,
        turn_extension_init: Option<ExtensionDataInit>,
    ) -> CodexResult<TurnInputSubmission> {
        let (reply_tx, reply_rx) = oneshot::channel();
        self.submit_with_id(Submission {
            id: turn_id,
            op: Op::RecoverTurn {
                thread_settings,
                start_options,
                reply: reply_tx,
            },
            trace,
            turn_extension_init,
            parent_turn_id: None,
            root_turn_id: None,
            residency_guard: None,
            message_delivery: None,
        })
        .await?;
        reply_rx.await.unwrap_or(Err(CodexErr::InternalAgentDied))
    }

    pub(crate) async fn shutdown_and_wait(&self) -> CodexResult<()> {
        let session_loop_termination = self.session_loop_termination.clone();
        match self.submit(Op::Shutdown).await {
            Ok(_) => {}
            Err(err) if matches!(err.details(), CodexErrorDetails::InternalAgentDied) => {}
            Err(err) => return Err(err),
        }
        session_loop_termination.await;
        Ok(())
    }

    pub(crate) async fn next_event(&self) -> CodexResult<Event> {
        loop {
            let event = self
                .rx_event
                .recv()
                .await
                .map_err(|_| CodexErr::InternalAgentDied)?;
            // PF-28-S01: one choke point for every producer, including those
            // holding a cloned sender (MCP startup, elicitation).
            if let Some(msg) = crate::security::disclosure_gate::gate_delivered(event.msg) {
                return Ok(Event { id: event.id, msg });
            }
        }
    }

    pub(crate) async fn agent_status(&self) -> AgentStatus {
        self.agent_status.borrow().clone()
    }
}

/// Generate a core submission ID. App-server exposes submission IDs that
/// create turns as a public-facing turn ID.
///
/// Some use cases take advantage of the fact that these are UUID7 which
/// encodes a timestamp, so think carefully before changing this.
pub(crate) fn new_submission_id() -> String {
    Uuid::now_v7().to_string()
}

pub(crate) fn get_service_tier(
    configured_service_tier: Option<String>,
    features: &Features,
    model_info: &ModelInfo,
) -> Option<String> {
    let service_tier = configured_service_tier?;
    if service_tier == ServiceTier::Flex.request_value() {
        return Some(service_tier);
    }
    if features.service_tier_enabled(&service_tier)
        && (service_tier == SERVICE_TIER_DEFAULT_REQUEST_VALUE
            || model_info.supports_service_tier(&service_tier))
    {
        return Some(service_tier);
    }
    None
}

fn unsupported_service_tier_warning(
    configured_service_tier: Option<&str>,
    features: &Features,
    model_info: &ModelInfo,
) -> Option<String> {
    let service_tier = configured_service_tier.filter(|service_tier| {
        features.service_tier_enabled(service_tier)
            && *service_tier != SERVICE_TIER_DEFAULT_REQUEST_VALUE
            && !model_info.supports_service_tier(service_tier)
    })?;
    Some(format!(
        "Configured service tier `{service_tier}` is not advertised as supported for model `{}` and will be omitted from requests.",
        model_info.slug
    ))
}

fn session_permission_profile_state_from_config(
    config: &Config,
) -> CodexResult<PermissionProfileState> {
    Ok(config.permissions.permission_profile_state().clone())
}

#[cfg(test)]
pub(crate) fn completed_session_loop_termination() -> SessionLoopTermination {
    futures::future::ready(()).boxed().shared()
}

/// `source_envelopes`: an MCP-supplied token-budget hint moves out of the host
/// developer message into its own data message. Returns (inline, separate).
pub(crate) fn token_budget_hint_placement(
    separate: bool,
    hint: Option<String>,
) -> (Option<String>, Option<String>) {
    if separate { (None, hint) } else { (hint, None) }
}

pub(crate) fn session_loop_termination_from_handle(
    handle: JoinHandle<()>,
) -> SessionLoopTermination {
    async move {
        let _ = handle.await;
    }
    .boxed()
    .shared()
}

async fn thread_title_from_thread_store(
    live_thread: Option<&LiveThread>,
    thread_store: &Arc<dyn ThreadStore>,
    conversation_id: ThreadId,
) -> Option<String> {
    let thread = match live_thread {
        Some(live_thread) => {
            live_thread
                .read_thread(
                    /*include_archived*/ true, /*include_history*/ false,
                )
                .await
        }
        None => {
            thread_store
                .read_thread(ReadThreadParams {
                    thread_id: conversation_id,
                    include_archived: true,
                    include_history: false,
                })
                .await
        }
    }
    .ok()?;

    let title = thread.name.as_deref()?.trim();
    (!title.is_empty() && thread.preview.trim() != title).then(|| title.to_string())
}

<<<<<<< HEAD
fn push_prompt_fragment(
    fragment: PromptFragment,
    developer_sections: &mut Vec<ContextSection>,
    contextual_user_sections: &mut Vec<String>,
    separate_developer_sections: &mut Vec<ContextSection>,
) {
    let developer_section =
        || ContextSection::new(fragment.text().to_string(), fragment.source_id());
    match fragment.slot() {
        PromptSlot::DeveloperPolicy | PromptSlot::DeveloperCapabilities => {
            developer_sections.push(developer_section());
        }
        PromptSlot::ContextualUser => {
            contextual_user_sections.push(fragment.text().to_string());
        }
        PromptSlot::SeparateDeveloper => {
            separate_developer_sections.push(developer_section());
        }
    }
}

||||||| base
fn push_prompt_fragment(
    fragment: PromptFragment,
    developer_sections: &mut Vec<String>,
    contextual_user_sections: &mut Vec<String>,
    separate_developer_sections: &mut Vec<String>,
) {
    match fragment.slot() {
        PromptSlot::DeveloperPolicy | PromptSlot::DeveloperCapabilities => {
            developer_sections.push(fragment.text().to_string());
        }
        PromptSlot::ContextualUser => {
            contextual_user_sections.push(fragment.text().to_string());
        }
        PromptSlot::SeparateDeveloper => {
            separate_developer_sections.push(fragment.text().to_string());
        }
    }
}

=======
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
impl Session {
    fn trace_session_timing(label: &str, start: Instant) {
        if std::env::var_os("PFTERMINAL_TRACE_STREAM_TIMING").is_some() {
            debug!(
                target: "corbanu_session",
                label,
                elapsed_ms = start.elapsed().as_millis(),
                "pfterminal session timing"
            );
        }
    }

    pub(crate) async fn app_server_client_metadata(&self) -> AppServerClientMetadata {
        let state = self.state.lock().await;
        AppServerClientMetadata {
            client_name: state.session_configuration.app_server_client_name.clone(),
            client_version: state
                .session_configuration
                .app_server_client_version
                .clone(),
        }
    }

    fn managed_network_proxy_active_for_permission_profile(
        permission_profile: &PermissionProfile,
    ) -> bool {
        !matches!(permission_profile, PermissionProfile::Disabled)
    }

    /// Builds the `x-codex-beta-features` header value for this session.
    ///
    /// `ModelClient` is session-scoped and intentionally does not depend on the full `Config`, so
    /// we precompute the comma-separated list of enabled experimental feature keys at session
    /// creation time and thread it into the client. Remote compaction stays advertised unconditionally.
    fn build_model_client_beta_features_header(config: &Config) -> Option<String> {
        Some(
            FEATURES
                .iter()
                .filter(|spec| {
                    spec.id == Feature::RemoteCompactionV2
                        || (spec.stage.experimental_menu_description().is_some()
                            && config.features.enabled(spec.id))
                })
                .map(|spec| spec.key)
                .collect::<Vec<_>>()
                .join(","),
        )
    }

    #[allow(clippy::too_many_arguments)]
    async fn start_managed_network_proxy(
        spec: &crate::config::NetworkProxySpec,
        exec_policy: &codex_execpolicy::Policy,
        permission_profile: &PermissionProfile,
        windows_sandbox_type: SandboxType,
        network_policy_decider: Option<Arc<dyn codex_network_proxy::NetworkPolicyDecider>>,
        blocked_request_observer: Option<Arc<dyn codex_network_proxy::BlockedRequestObserver>>,
        managed_network_requirements_enabled: bool,
        audit_metadata: NetworkProxyAuditMetadata,
    ) -> anyhow::Result<(StartedNetworkProxy, SessionNetworkProxyRuntime)> {
        let spec = spec
            .with_exec_policy_network_rules(exec_policy)
            .map_err(|err| {
                tracing::warn!(
                    "failed to apply execpolicy network rules to managed proxy; continuing with configured network policy: {err}"
                );
                err
            })
            .unwrap_or_else(|_| spec.clone());
        let network_proxy = spec
            .start_proxy(
                permission_profile,
                managed_proxy_routing_for_windows_sandbox(windows_sandbox_type),
                local_binding_policy_for_sandbox(windows_sandbox_type, Some(std::env::consts::OS)),
                network_policy_decider,
                blocked_request_observer,
                managed_network_requirements_enabled,
                audit_metadata,
            )
            .await
            .map_err(|err| anyhow::anyhow!("failed to start managed network proxy: {err}"))?;
        let session_network_proxy = {
            let proxy = network_proxy.proxy();
            SessionNetworkProxyRuntime {
                http_addr: proxy.http_addr().to_string(),
                socks_addr: proxy.socks_addr().to_string(),
            }
        };
        Ok((network_proxy, session_network_proxy))
    }

    async fn refresh_managed_network_proxy_for_current_permission_profile(&self) {
        let Ok(_refresh_guard) = self.managed_network_proxy_refresh_lock.acquire().await else {
            error!("managed network proxy refresh semaphore closed");
            return;
        };
        let session_configuration = {
            let state = self.state.lock().await;
            state.session_configuration.clone()
        };
        let Some(spec) = session_configuration
            .original_config_do_not_use
            .permissions
            .network
            .as_ref()
            .cloned()
        else {
            self.services.network_proxy.store(None);
            self.services
                .turn_environments
                .set_snapshot_credential_broker(SnapshotCredentialBrokerState::Inactive);
            return;
        };

        let spec = match spec
            .recompute_for_permission_profile(&session_configuration.permission_profile())
        {
            Ok(spec) => spec,
            Err(err) => {
                warn!("failed to rebuild managed network proxy policy for sandbox change: {err}");
                return;
            }
        };
        // D52 (#1001): under Aggressive, network allow rules saved outside it
        // are suspended.
        let current_exec_policy = self
            .services
            .exec_policy
            .current_for(self.services.model_client().live_level_is_aggressive());
        let spec = match spec.with_exec_policy_network_rules(current_exec_policy.as_ref()) {
            Ok(spec) => spec,
            Err(err) => {
                warn!(
                    "failed to apply execpolicy network rules while refreshing managed network proxy: {err}"
                );
                spec
            }
        };
        // Disabled specs still carry managed requirements and constraints, but they do not have
        // listeners and must not be exposed as active managed proxy runtimes.
        if !spec.enabled() {
            self.services.network_proxy.store(None);
            self.services
                .turn_environments
                .set_snapshot_credential_broker(SnapshotCredentialBrokerState::Inactive);
            return;
        }
        let managed_proxy_routing =
            managed_proxy_routing_for_windows_sandbox(session_configuration.windows_sandbox_type);
        let started_proxy = self.services.network_proxy.load_full();
        let network_policy_decider = started_proxy
            .as_ref()
            .and_then(|started_proxy| started_proxy.network_policy_decider());
        if let Some(started_proxy) = started_proxy
            && started_proxy.proxy().managed_proxy_routing() == managed_proxy_routing
        {
            if let Err(err) = spec.apply_to_started_proxy(started_proxy.as_ref()).await {
                warn!("failed to refresh managed network proxy for sandbox change: {err}");
            } else {
                self.services
                    .turn_environments
                    .set_snapshot_credential_broker(if spec.credential_broker_enabled() {
                        SnapshotCredentialBrokerState::Ready(started_proxy.proxy())
                    } else {
                        SnapshotCredentialBrokerState::Inactive
                    });
            }
            return;
        }

        match Self::start_managed_network_proxy(
            &spec,
            current_exec_policy.as_ref(),
            &session_configuration.permission_profile(),
            session_configuration.windows_sandbox_type,
            network_policy_decider,
            Some(build_blocked_request_observer(Arc::clone(
                &self.services.network_approval,
            ))),
            self.services.managed_network_requirements_configured,
            self.services.network_proxy_audit_metadata.clone(),
        )
        .await
        {
            Ok((started_proxy, _session_network_proxy)) => {
                if spec.credential_broker_enabled() {
                    self.services
                        .turn_environments
                        .set_snapshot_credential_broker(SnapshotCredentialBrokerState::Ready(
                            started_proxy.proxy(),
                        ));
                }
                self.services
                    .network_proxy
                    .store(Some(Arc::new(started_proxy)));
            }
            Err(err) => {
                self.services
                    .turn_environments
                    .set_snapshot_credential_broker(if spec.credential_broker_enabled() {
                        SnapshotCredentialBrokerState::Unavailable
                    } else {
                        SnapshotCredentialBrokerState::Inactive
                    });
                warn!("failed to start managed network proxy for sandbox change: {err}");
            }
        }
    }

    #[cfg(test)]
    pub(crate) async fn codex_home(&self) -> AbsolutePathBuf {
        let state = self.state.lock().await;
        state.session_configuration.codex_home().clone()
    }

    pub(crate) fn subscribe_elicitation_pause_state(&self) -> watch::Receiver<bool> {
        self.services.elicitations.subscribe()
    }

    pub(crate) fn mark_interrupted(&self) {
        self.agent_status.send_replace(AgentStatus::Interrupted);
    }

    pub(crate) fn is_interrupted(&self) -> bool {
        matches!(*self.agent_status.borrow(), AgentStatus::Interrupted)
    }

    pub(crate) fn get_tx_event(&self) -> Sender<Event> {
        self.tx_event.clone()
    }

    pub(crate) fn state_db(&self) -> Option<state_db::StateDbHandle> {
        self.services.state_db.clone()
    }

    pub(crate) fn live_thread_for_persistence(
        &self,
        operation: &str,
    ) -> anyhow::Result<&LiveThread> {
        self.live_thread()
            .ok_or_else(|| anyhow::anyhow!("Session persistence is disabled; cannot {operation}."))
    }

    pub(crate) fn live_thread(&self) -> Option<&LiveThread> {
        self.services.live_thread.as_ref()
    }

    pub(crate) async fn set_thread_memory_mode(
        self: &Arc<Self>,
        mode: ThreadMemoryMode,
    ) -> anyhow::Result<()> {
        handlers::persist_thread_memory_mode_update(self, mode).await
    }

    /// Flush rollout writes and return the final durability-barrier result.
    #[instrument(name = "session.flush_rollout", level = "trace", skip_all)]
    pub(crate) async fn flush_rollout(&self) -> std::io::Result<()> {
        if let Some(live_thread) = self.live_thread() {
            live_thread.flush().await.map_err(std::io::Error::other)
        } else {
            Ok(())
        }
    }

    pub(crate) async fn try_ensure_rollout_materialized(
        &self,
        context: PersistContext,
    ) -> std::io::Result<()> {
        if let Some(live_thread) = self.live_thread() {
            live_thread
                .persist(context)
                .await
                .map_err(std::io::Error::other)?;
        }
        Ok(())
    }

    pub(crate) async fn ensure_rollout_materialized(&self, context: PersistContext) {
        if let Err(e) = self.try_ensure_rollout_materialized(context).await {
            warn!("failed to materialize thread persistence: {e}");
        }
    }

    pub(crate) fn next_internal_sub_id(&self) -> String {
        let id = self
            .next_internal_sub_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        format!("auto-compact-{id}")
    }

    pub(crate) async fn get_total_token_usage(&self) -> i64 {
        let state = self.state.lock().await;
        state.get_total_token_usage()
    }

    pub(crate) async fn auto_compact_window_snapshot(&self) -> AutoCompactWindowSnapshot {
        let state = self.state.lock().await;
        state.auto_compact_window_snapshot()
    }

    pub(crate) async fn estimated_tokens_since_last_usage(&self) -> i64 {
        let state = self.state.lock().await;
        state.history.estimated_tokens_since_last_usage()
    }

    pub(crate) async fn total_token_usage(&self) -> Option<TokenUsage> {
        let state = self.state.lock().await;
        state.token_info().map(|info| info.total_token_usage)
    }

    /// Returns the complete token usage snapshot currently cached for this session.
    ///
    /// Resume and fork reconstruction seed this state from the last persisted rollout
    /// `TokenCount` event. Callers that need to replay restored usage to a client
    /// should use this accessor instead of `total_token_usage`, because the app-server
    /// notification includes both total and last-turn usage.
    pub(crate) async fn token_usage_info(&self) -> Option<TokenUsageInfo> {
        let state = self.state.lock().await;
        state.token_info()
    }

    /// PF-84: usage of requests made for this conversation outside its own
    /// turn loop (a guardian or `/review` delegate, stage-one memory
    /// extraction), which the accounting ledger also records under it. Sent
    /// at once on the active turn, if any; otherwise with the next count.
    pub(crate) async fn record_delegated_account_usage(&self, usage: &[AccountTokenUsage]) {
        if usage.is_empty() || !self.features().enabled(Feature::NamedAccounts) {
            return;
        }
        {
            let mut state = self.state.lock().await;
            for entry in usage {
                state.record_account_token_usage(&entry.provider_id, &entry.account, &entry.usage);
            }
        }
        if let Some((turn_context, _)) = self.active_turn_context_and_cancellation_token().await {
            self.send_token_count_event(&turn_context).await;
        }
    }

    /// PF-84: this thread's provider-reported usage per named account, for
    /// replay to a client that attaches after resume.
    pub(crate) async fn account_token_usage(&self) -> Option<Vec<AccountTokenUsage>> {
        let state = self.state.lock().await;
        state.account_token_usage()
    }

    pub(crate) async fn get_estimated_token_count(
        &self,
        turn_context: &TurnContext,
    ) -> Option<i64> {
        let history = self.clone_history().await;
        history.estimate_token_count(turn_context)
    }

    pub(crate) async fn get_base_instructions(&self) -> BaseInstructions {
        let state = self.state.lock().await;
        BaseInstructions {
            text: state.session_configuration.base_instructions.clone(),
            provenance: state.base_instructions_provenance.clone(),
        }
    }

    /// Render the request copy without changing instructions persisted or inherited by forks.
    pub(crate) async fn get_prompt_base_instructions(&self) -> BaseInstructions {
        let config = self.get_config().await;
        let instructions = self.get_base_instructions().await;
        if !config.update_plan_enabled
            && config.model_catalog.is_none()
            && matches!(
                instructions.provenance,
                Some(BaseInstructionsProvenance::Model { .. })
            )
        {
            BaseInstructions {
                text: codex_prompts::without_update_plan_instructions(&instructions.text),
                ..instructions
            }
        } else {
            instructions
        }
    }

    // Merges connector IDs into the session-level explicit connector selection.
    #[tracing::instrument(
        level = "trace",
        skip_all,
        fields(connector_count = connector_ids.len())
    )]
    pub(crate) async fn merge_connector_selection(
        &self,
        connector_ids: HashSet<String>,
    ) -> HashSet<String> {
        let mut state = self.state.lock().await;
        state.merge_connector_selection(connector_ids)
    }

    // Returns the connector IDs currently selected for this session.
    pub(crate) async fn get_connector_selection(&self) -> HashSet<String> {
        let state = self.state.lock().await;
        state.get_connector_selection()
    }

    // Clears connector IDs that were accumulated for explicit selection.
    pub(crate) async fn clear_connector_selection(&self) {
        let mut state = self.state.lock().await;
        state.clear_connector_selection();
    }

    async fn record_initial_history(&self, conversation_history: InitialHistory) {
        let initial_history_started_at = Instant::now();
        Self::trace_session_timing("record_initial_history_start", initial_history_started_at);
        let (is_subagent, is_paginated_subagent) = {
            let state = self.state.lock().await;
            let session_configuration = &state.session_configuration;
            (
                session_configuration.session_source.is_non_root_agent(),
                matches!(
                    session_configuration.history_mode,
                    ThreadHistoryMode::Paginated
                ) && matches!(
                    session_configuration.thread_source.as_ref(),
                    Some(ThreadSource::Subagent | ThreadSource::GuardianReview)
                ),
            )
        };
        Self::trace_session_timing(
            "record_initial_history_after_subagent_check",
            initial_history_started_at,
        );
        let has_prior_user_turns = initial_history_has_prior_user_turns(&conversation_history);
        {
            let mut state = self.state.lock().await;
            state.set_next_turn_is_first(!has_prior_user_turns);
        }
<<<<<<< HEAD
        Self::trace_session_timing(
            "record_initial_history_after_first_turn_state",
            initial_history_started_at,
        );
        match conversation_history {
||||||| base
        match conversation_history {
=======
        let turn_context = match conversation_history {
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
            InitialHistory::New | InitialHistory::Cleared => {
                // Defer initial context insertion until the first real turn starts so
                // turn/start overrides can be merged before we write model-visible context.
                self.set_previous_turn_settings(/*previous_turn_settings*/ None)
                    .await;
<<<<<<< HEAD
                Self::trace_session_timing(
                    "record_initial_history_new_done",
                    initial_history_started_at,
                );
||||||| base
=======
                None
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
            }
            InitialHistory::Resumed(resumed_history) => {
                let turn_context = self.new_default_turn().await;
                let rollout_items = resumed_history.history;
<<<<<<< HEAD
                Self::trace_session_timing(
                    "record_initial_history_resumed_after_default_turn",
                    initial_history_started_at,
                );
||||||| base
=======
                if matches!(
                    rollout_items.iter().rev().find_map(|item| match item {
                        RolloutItem::EventMsg(event) => agent_status_from_event(event),
                        _ => None,
                    }),
                    Some(AgentStatus::Interrupted)
                ) {
                    self.agent_status.send_replace(AgentStatus::Interrupted);
                }
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
                let previous_turn_settings = self
                    .apply_rollout_reconstruction(&turn_context, &rollout_items)
                    .await;
                Self::trace_session_timing(
                    "record_initial_history_resumed_after_reconstruction",
                    initial_history_started_at,
                );

                // If resuming, warn when the last recorded model differs from the current one.
                let curr: &str = turn_context.model_info().slug.as_str();
                if let Some(prev) = previous_turn_settings
                    .as_ref()
                    .map(|settings| settings.model.as_str())
                    .filter(|model| *model != curr)
                {
                    warn!("resuming session with different model: previous={prev}, current={curr}");
                    self.send_event(
                        &turn_context,
                        EventMsg::Warning(WarningEvent {
                            message: format!(
                                "This session was recorded with model `{prev}` but is resuming with `{curr}`. \
                         Consider switching back to `{prev}` as it may affect Codex performance."
                            ),
                        }),
                    )
                    .await;
                }

                if let Some(event) =
                    Self::last_model_response_completed_from_rollout(&rollout_items)
                    && event.model == turn_context.model_info.slug.as_str()
                    && event.model_provider_id == turn_context.config.model_provider_id.as_str()
                {
                    self.services
                        .model_client()
                        .seed_server_response_id(event.response_id);
                }

                // Seed usage info from the recorded rollout so UIs can show token counts
                // immediately on resume/fork.
                if let Some(info) = Self::last_token_info_from_rollout(&rollout_items) {
                    let mut state = self.state.lock().await;
                    state.set_token_info(Some(info));
                }
<<<<<<< HEAD
                // PF-84: a resumed thread keeps its usage per account, only while
                // named accounts are on. A fork does not: its parent's requests
                // stay the parent's.
                if turn_context.config.features.enabled(Feature::NamedAccounts)
                    && let Some(usage) = Self::last_account_token_usage_from_rollout(&rollout_items)
                {
                    let mut state = self.state.lock().await;
                    state.set_account_token_usage(usage);
                }
                Self::trace_session_timing(
                    "record_initial_history_resumed_after_token_seed",
                    initial_history_started_at,
                );
||||||| base
=======
                self.state.lock().await.latest_token_usage_record =
                    Self::last_token_usage_record_from_rollout(&rollout_items);

                let thread_id = self.thread_id();
                let current_settings = self.thread_settings_snapshot().await;
                let persisted_settings = rollout_items.iter().rev().find_map(|item| match item {
                    RolloutItem::EventMsg(EventMsg::ThreadSettingsApplied(event))
                        if event.thread_id == Some(thread_id) =>
                    {
                        Some(&event.thread_settings)
                    }
                    _ => None,
                });
                // Persist changed effective settings even when no turn follows the resume.
                if persisted_settings != Some(&current_settings) {
                    self.persist_rollout_items(&[RolloutItem::EventMsg(
                        EventMsg::ThreadSettingsApplied(ThreadSettingsAppliedEvent {
                            thread_id: Some(thread_id),
                            thread_settings: current_settings,
                        }),
                    )])
                    .await;
                }
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a

                // Defer seeding the session's initial context until the first turn starts so
                // turn/start overrides can be merged before we write model-visible context.
                if !is_subagent {
                    let _ = self.flush_rollout().await;
                }
<<<<<<< HEAD
                Self::trace_session_timing(
                    "record_initial_history_resumed_done",
                    initial_history_started_at,
                );
||||||| base
=======
                None
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
            }
            InitialHistory::Forked(mut rollout_items) => {
                let turn_context = self.new_default_turn().await;
                Self::assign_missing_rollout_response_item_ids(&mut rollout_items);
                self.apply_rollout_reconstruction(&turn_context, &rollout_items)
                    .await;
                Self::trace_session_timing(
                    "record_initial_history_forked_after_reconstruction",
                    initial_history_started_at,
                );

                // Seed usage info from the recorded rollout so UIs can show token counts
                // immediately on resume/fork.
                if let Some(info) = Self::last_token_info_from_rollout(&rollout_items) {
                    let mut state = self.state.lock().await;
                    state.set_token_info(Some(info));
                }
                self.state.lock().await.latest_token_usage_record =
                    Self::last_token_usage_record_from_rollout(&rollout_items);

                let thread_settings_applied =
                    RolloutItem::EventMsg(thread_settings::applied_event(self).await);
                match &self.fork_persistence {
                    ForkPersistence::Referenced {
                        inherited_item_count,
                        ..
                    } => {
                        // Ancestor records remain behind history_base; only effective child
                        // settings and boundaries synthesized by snapshot processing are local.
                        rollout_items.drain(..*inherited_item_count);
                        rollout_items.insert(0, thread_settings_applied);
                    }
                    ForkPersistence::Copied | ForkPersistence::CopiedDeferred
                        if is_paginated_subagent =>
                    {
                        // Paginated subagents already persist inherited context when their live
                        // thread is created.
                        rollout_items.clear();
                        rollout_items.push(thread_settings_applied);
                    }
                    ForkPersistence::Copied | ForkPersistence::CopiedDeferred => {
                        // Keep the copied prefix and effective child settings in one append so a
                        // cold resume cannot observe inherited settings as the latest value.
                        rollout_items.push(thread_settings_applied);
                    }
                }
                // PF-84: the copied prefix holds the parent's usage per account; mark
                // where this fork's own (none yet) begins, so a resume before its first
                // reply does not count the parent's again.
                if turn_context.config.features.enabled(Feature::NamedAccounts) {
                    rollout_items.push(RolloutItem::EventMsg(EventMsg::TokenCount(
                        TokenCountEvent {
                            info: None,
                            rate_limits: None,
                            account_usage: Some(Vec::new()),
                        },
                    )));
                }
                self.persist_rollout_items(&rollout_items).await;
                if self.preserve_parent_tool_declarations() {
                    let notice = DeveloperInstructions::new(
                        "<mcp_tools>\nMCP tools are unavailable for this request.\n</mcp_tools>",
                    )
                    .with_metadata(ContentItemMetadata::tool(/*namespace*/ None));
                    self.record_conversation_items(
                        &turn_context,
                        turn_context.model_info(),
                        &[ContextualUserFragment::into(notice)],
                    )
                    .await;
                }

                // Agent spawn owns its final durability barrier so it can overlap the edge write.
                let persist_context = match self.fork_persistence {
                    ForkPersistence::CopiedDeferred => PersistContext::SubagentSpawn,
                    ForkPersistence::Copied | ForkPersistence::Referenced { .. } => {
                        PersistContext::Standard
                    }
                };
                self.ensure_rollout_materialized(persist_context).await;

                // Flush after seeding history and any persisted rollout copy.
                if !is_subagent {
                    let _ = self.flush_rollout().await;
                }
<<<<<<< HEAD
                Self::trace_session_timing(
                    "record_initial_history_forked_done",
                    initial_history_started_at,
                );
||||||| base
=======
                Some(turn_context)
            }
        };
        if let Some(turn_context) = turn_context
            && turn_context.config.memories.disable_on_external_context
        {
            let history = self.clone_history().await;
            if let Some(item) = history
                .raw_items()
                .find(|item| matches!(item, ResponseItem::FunctionCallOutput { call_id: None, .. }))
            {
                mark_thread_memory_mode_polluted_if_external_context(self, &turn_context, item)
                    .await;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
            }
        }
    }

    #[instrument(
        level = "trace",
        skip_all,
        fields(
            thread_id = %self.thread_id(),
            rollout_item_count = rollout_items.len()
        )
    )]
    async fn apply_rollout_reconstruction(
        &self,
        turn_context: &Arc<TurnContext>,
        rollout_items: &[RolloutItem],
    ) -> Option<PreviousTurnSettings> {
        let rollout_reconstruction::RolloutReconstruction {
            mut history,
            retained_context,
            guardian_history,
            last_started_turn_id,
            previous_turn_settings,
            reference_context_item,
            turn_attribution,
            world_state_baseline,
            window_number,
            first_window_id,
            previous_window_id,
            window_id,
        } = self
            .reconstruct_history_from_rollout(turn_context, rollout_items)
            .await;
        // Keep the recorded rollout unchanged. Prepare its reconstructed history before
        // installing it, so legacy media is processed once for this resume or fork and
        // will be processed again if the rollout is reconstructed in a future session.
<<<<<<< HEAD
        // This meets media preparation requirements without modifying persisted rollouts.
        prepare_image_response_items(&mut history);
        prepare_audio_response_items(&mut history);
        // Restored content keeps the standing its origin records give it;
        // anything unrecorded stays labelled (`source_envelopes`).
        self.services.model_client().note_restored_history(
            &history,
            rollout_items.iter().filter_map(|item| match item {
                RolloutItem::SourceOrigin(record) => Some(record),
                _ => None,
            }),
        );
||||||| base
        // This meets media preparation requirements without modifying persisted rollouts.
        prepare_image_response_items(&mut history);
        prepare_audio_response_items(&mut history);
=======
        // Replay disables image-resize notices, so media preparation remains one-to-one. Keep
        // the prior batch behavior and carry history-only metadata in a positional sidecar.
        let (mut prepared_history, metadata): (Vec<_>, Vec<_>) = history
            .into_iter()
            .map(|envelope| (envelope.item, envelope.metadata))
            .unzip();
        // Replay must not upload or migrate recorded history. The inline store returns prepared
        // inline bytes, while existing file references bypass preparation and remain unchanged.
        // Bound replay future size now that image preparation can await storage.
        let _ = Box::pin(prepare_image_response_items(
            &self.thread_id.to_string(),
            turn_context.config.ephemeral,
            &mut prepared_history,
            ImagePreparationMode::DetailBased,
            ImageResizeNoticeMode::Disabled,
            &InlineAttachmentStore,
        ))
        .await;
        prepare_audio_response_items(&mut prepared_history);
        assert_eq!(
            prepared_history.len(),
            metadata.len(),
            "replay media preparation must remain one-to-one when resize notices are disabled"
        );
        history = prepared_history
            .into_iter()
            .zip(metadata)
            .map(|(item, metadata)| ResponseItemEnvelope { item, metadata })
            .collect();
        // Ordinary replay needs no reviewer or network refresh. Only an opaque
        // checkpoint can make reconstruction depend on reviewer compatibility.
        let reviewer_compaction_hash = if codex_history::CompactionCheckpoint::latest(&history)
            .is_some_and(codex_history::CompactionCheckpoint::is_usable)
            && turn_context
                .config
                .features
                .enabled(Feature::GuardianReuseParentCompaction)
        {
            let context = crate::guardian::GuardianReviewContext::from(turn_context);
            let (_, reviewer) = crate::guardian::resolve_review_model(self, &context).await;
            reviewer.comp_hash.clone()
        } else {
            None
        };
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        {
            let mut state = self.state.lock().await;
            state.replace_annotated_history(
                history,
                reference_context_item,
                HistoryReplacement::Reset,
            );
            state.history.restore_review_context(
                Some(&retained_context),
                guardian_history.as_ref(),
                reviewer_compaction_hash.as_deref(),
            );
            state.last_started_turn_id = last_started_turn_id;
            state.turn_attribution = turn_attribution;
            if let Some(world_state) = world_state_baseline {
                state.history.set_world_state_baseline(world_state);
            }
            let fallback_ids = state.auto_compact_window_ids();
            let window_id = window_id.unwrap_or(fallback_ids.window_id);
            state.restore_auto_compact_window(
                window_number,
                AutoCompactWindowIds {
                    first_window_id: first_window_id.unwrap_or(window_id),
                    previous_window_id,
                    window_id,
                },
            );
            state.set_previous_turn_settings(previous_turn_settings.clone());
        }
        let prefix_tokens = if matches!(
            turn_context.config.model_auto_compact_token_limit_scope,
            AutoCompactTokenLimitScope::BodyAfterPrefix
        ) {
            let history = self.clone_history().await;
            let base_instructions = self.get_base_instructions().await;
            history.estimate_token_count_with_base_instructions(&base_instructions)
        } else {
            None
        };
        if let Some(prefix_tokens) = prefix_tokens {
            self.set_auto_compact_window_estimated_prefill_for_scope(turn_context, prefix_tokens)
                .await;
        }
        previous_turn_settings
    }

    async fn set_auto_compact_window_estimated_prefill_for_scope(
        &self,
        turn_context: &TurnContext,
        tokens: i64,
    ) {
        if !matches!(
            turn_context.config.model_auto_compact_token_limit_scope,
            AutoCompactTokenLimitScope::BodyAfterPrefix
        ) {
            return;
        }

        let mut state = self.state.lock().await;
        state.set_auto_compact_window_estimated_prefill(tokens);
    }

    fn last_token_info_from_rollout(rollout_items: &[RolloutItem]) -> Option<TokenUsageInfo> {
        rollout_items.iter().rev().find_map(|item| match item {
            RolloutItem::EventMsg(EventMsg::TokenCount(ev)) => ev.info.clone(),
            _ => None,
        })
    }

<<<<<<< HEAD
    fn last_account_token_usage_from_rollout(
        rollout_items: &[RolloutItem],
    ) -> Option<Vec<AccountTokenUsage>> {
        rollout_items.iter().rev().find_map(|item| match item {
            RolloutItem::EventMsg(EventMsg::TokenCount(ev)) => ev.account_usage.clone(),
            _ => None,
        })
    }

    fn last_model_response_completed_from_rollout(
        rollout_items: &[RolloutItem],
    ) -> Option<ModelResponseCompletedEvent> {
        rollout_items.iter().rev().find_map(|item| match item {
            RolloutItem::EventMsg(EventMsg::ModelResponseCompleted(ev))
                if !ev.response_id.is_empty() =>
            {
                Some(ev.clone())
            }
            _ => None,
        })
||||||| base
=======
    fn last_token_usage_record_from_rollout(
        rollout_items: &[RolloutItem],
    ) -> Option<TokenUsageRecord> {
        for item in rollout_items.iter().rev() {
            match item {
                RolloutItem::TokenUsageRecord(record) => return Some(record.clone()),
                RolloutItem::Compacted(compacted) => {
                    return compacted.latest_token_usage_record.clone();
                }
                _ => {}
            }
        }
        None
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
    }

    async fn previous_turn_settings(&self) -> Option<PreviousTurnSettings> {
        let state = self.state.lock().await;
        state.previous_turn_settings()
    }

    #[tracing::instrument(level = "trace", skip_all)]
    pub(crate) async fn set_previous_turn_settings(
        &self,
        previous_turn_settings: Option<PreviousTurnSettings>,
    ) {
        let mut state = self.state.lock().await;
        state.set_previous_turn_settings(previous_turn_settings);
    }

    pub(crate) async fn update_settings(
        &self,
        updates: SessionSettingsUpdate,
    ) -> ConstraintResult<SessionSettingsCommit> {
        let Some(commit) = self.update_settings_if(updates, |_, _| true).await? else {
            unreachable!("unconditional settings updates must commit");
        };
        Ok(commit)
    }

    /// Evaluates the caller's synchronous predicate against the current configuration
    /// and validated candidate under the same state lock used for publication.
    /// This prevents a stale admission decision. Rejection has no settings effects; the
    /// GPU runtime-provider refresh below runs first either way.
    ///
    /// The predicate must be fast and side-effect-free. It must not block, acquire
    /// other locks, or call back into `Session`.
    async fn update_settings_if(
        &self,
        mut updates: SessionSettingsUpdate,
        should_commit: impl FnOnce(&SessionConfiguration, &SessionConfiguration) -> bool + Send,
    ) -> ConstraintResult<Option<SessionSettingsCommit>> {
        // GPU endpoints are process-local controller transports and can change while a
        // session remains open. Refresh the selected runtime provider before every update
        // (turns included) so recovery does not leave the session's ModelClient pinned to a
        // dead local port.
        self.ensure_runtime_model_provider(&mut updates).await;
        let notify_config_contributors = !self.services.extensions.config_contributors().is_empty();
        let windows_sandbox_changed;
        let (
            commit,
            previous_config,
            new_config,
            permission_profile_changed,
            mcp_inputs_changed,
            credential_masking_warning,
            stale_startup_prewarm,
            follows_model_list,
        ) = {
            let mut state = self.state.lock().await;
            let updated = match self.apply_session_settings(&state.session_configuration, &updates)
            {
                Ok(updated) => updated,
                Err(err) => {
                    warn!("rejected session settings update: {err}");
                    return Err(err);
                }
            };

            if !should_commit(&state.session_configuration, &updated) {
                return Ok(None);
            }

            let credential_masking_warning = updated
                .original_config_do_not_use
                .credential_masking_warning(&updated.permission_profile())
                .filter(|warning| {
                    Some(*warning)
                        != state
                            .session_configuration
                            .original_config_do_not_use
                            .credential_masking_warning(
                                &state.session_configuration.permission_profile(),
                            )
                });
            let previous_config = notify_config_contributors
                .then(|| self.build_effective_session_config(&state.session_configuration));
            let previous_permission_profile = state.session_configuration.permission_profile();
            let updated_permission_profile = updated.permission_profile();
            let permission_profile_changed =
                previous_permission_profile != updated_permission_profile;
            let root_service_tier_changed = updated.parent_thread_id.is_none()
                && state.session_configuration.step_settings.service_tier
                    != updated.step_settings.service_tier;
            let mcp_inputs_changed = self.mcp_inputs_differ(&state.session_configuration, &updated);
            let model_provider_changed = state
                .session_configuration
                .original_config_do_not_use
                .model_provider_id
                != updated.original_config_do_not_use.model_provider_id
                || state.session_configuration.provider.info() != updated.provider.info();
            // PF-84: the account follows the provider first.
            let account_switched = model_provider_changed && self.follow_provider_account(&updated);
            if model_provider_changed {
                // Publish while updates are serialized, before the new configuration
                // becomes observable. Frame guards read this client without state locks.
                self.services
                    .replace_model_client(self.build_model_client_for_configuration(&updated));
            }
            let stale_startup_prewarm = if model_provider_changed {
                state.take_session_startup_prewarm()
            } else {
                None
            };
            let follows_model_list =
                model_provider_changed && self.model_list_follows_this_session(&updated);
            if mcp_inputs_changed {
                self.mark_mcp_runtime_dirty();
            }
            // Save new environment defaults for future turns. The running turn keeps its own.
            windows_sandbox_changed =
                state.session_configuration.windows_sandbox_level != updated.windows_sandbox_level;
            self.services
                .turn_environments
                .set_configured_thread_defaults(updated.inferred_environment_config());
            if updated.environments != state.session_configuration.environments {
                self.services
                    .turn_environments
                    .update_configured_selections(&updated.environments);
            }
            state.session_configuration = updated;
            if root_service_tier_changed {
                self.services.agent_control.propagate_config_update(
                    AgentConfigUpdate::ServiceTier(
                        state
                            .session_configuration
                            .step_settings
                            .service_tier
                            .clone(),
                    ),
                );
            }
            let new_config = notify_config_contributors
                .then(|| self.build_effective_session_config(&state.session_configuration));
            let commit = SessionSettingsCommit {
                configuration: state.session_configuration.clone(),
                snapshot: state
                    .session_configuration
                    .thread_settings_snapshot(&state.session_configuration.environments),
            };
            (
                commit,
                previous_config,
                new_config,
                permission_profile_changed,
                mcp_inputs_changed || account_switched,
                credential_masking_warning,
                stale_startup_prewarm,
                follows_model_list,
            )
        };
        if follows_model_list {
            self.follow_model_list_account().await;
        }
        if windows_sandbox_changed {
            crate::windows_sandbox::log_windows_sandbox_change(
                commit.configuration.codex_home.as_path(),
                commit.configuration.windows_sandbox_type,
                commit.configuration.windows_sandbox_level,
            );
        }
        self.emit_config_changed_contributors(previous_config.as_ref(), new_config.as_ref());
        if let Some(startup_prewarm) = stale_startup_prewarm {
            startup_prewarm.abort().await;
        }
        if permission_profile_changed {
            self.refresh_managed_network_proxy_for_current_permission_profile()
                .await;
        }
        if let Some(message) = credential_masking_warning {
            warn!("{message}");
            self.send_event_raw(Event {
                id: INITIAL_SUBMIT_ID.to_owned(),
                msg: EventMsg::Warning(WarningEvent {
                    message: message.to_string(),
                }),
            })
            .await;
        }
        if mcp_inputs_changed {
            self.schedule_mcp_prewarm();
        }
        Ok(Some(commit))
    }

    /// PF-84: whether this session's account drives the process's model
    /// list: a user's session, not a sub-agent's or an internal one's.
    pub(super) fn model_list_follows_this_session(
        &self,
        configuration: &SessionConfiguration,
    ) -> bool {
        !configuration.session_source.is_non_root_agent()
    }

    /// PF-84: on a provider change, makes the new provider's sign-in (a named
    /// account's or the default login) the one connectors, plugin installs and
    /// accounting use, before the model client is built. Called under the
    /// state lock by every path that changes the provider. Returns whether the
    /// account changed; connectors are then marked for reconnection.
    pub(super) fn follow_provider_account(&self, next: &SessionConfiguration) -> bool {
        if !self.services.active_auth.follow_provider(&next.provider) {
            return false;
        }
        let account = codex_model_provider::named_sign_in_account(&next.provider)
            .map_or_else(|| "default".to_string(), |account| account.to_string());
        info!(
            %account,
            "connectors, plugin installs, accounting and the model list follow the switch"
        );
        self.mark_mcp_runtime_dirty();
        true
    }

    /// PF-84: lists models with the sign-in of the account this session's
    /// provider uses (the default login for any other provider). When that
    /// changed, the account's list is fetched again: in the background when
    /// its own cache (see `AccountModelsManager`) is fresh, otherwise waited
    /// for, at most [`FIRST_MODEL_LIST_WAIT`], so the client's next
    /// `model/list` has it.
    pub(super) async fn follow_model_list_account(&self) {
        let (provider, http_client_factory) = {
            let state = self.state.lock().await;
            (
                state.session_configuration.provider.clone(),
                state
                    .session_configuration
                    .original_config_do_not_use
                    .http_client_factory(),
            )
        };
        let account = codex_model_provider::named_sign_in_account(&provider);
        if !self
            .services
            .models_manager
            .follow_sign_in_account(account.as_ref())
        {
            return;
        }
        let models_manager = Arc::clone(&self.services.models_manager);
        let cached = models_manager.load_cached_list().await;
        let fetch = self.services.runtime_handle.spawn(async move {
            models_manager
                .list_models(RefreshStrategy::Online, http_client_factory)
                .await;
        });
        if !cached {
            // On timeout the fetch goes on in the background.
            let _ = tokio::time::timeout(FIRST_MODEL_LIST_WAIT, fetch).await;
        }
    }

    async fn ensure_runtime_model_provider(&self, updates: &mut SessionSettingsUpdate) {
        let model_provider_id = match updates.model_provider.clone() {
            Some(model_provider_id) => model_provider_id,
            None => {
                let current_model_provider_id = {
                    let state = self.state.lock().await;
                    state
                        .session_configuration
                        .original_config_do_not_use
                        .model_provider_id
                        .clone()
                };
                if !current_model_provider_id.starts_with("gpu-") {
                    return;
                }
                updates.model_provider = Some(current_model_provider_id.clone());
                current_model_provider_id
            }
        };
        if !model_provider_id.starts_with("gpu-") {
            updates.runtime_model_context_window = Some(None);
            return;
        }
        {
            let runtime_overlay_source = {
                let state = self.state.lock().await;
                (
                    state
                        .session_configuration
                        .original_config_do_not_use
                        .sqlite
                        .home()
                        .to_path_buf(),
                    state
                        .session_configuration
                        .original_config_do_not_use
                        .codex_home
                        .clone(),
                )
            };
            let (sqlite_home, codex_home) = runtime_overlay_source;
            let records =
                crate::config::load_gpu_runtime_model_provider_records(&sqlite_home).await;
            updates.runtime_model_context_window = Some(
                records
                    .iter()
                    .find(|record| record.provider_id == model_provider_id)
                    .and_then(|record| record.maximum_context_tokens),
            );
            let runtime_providers = records
                .into_iter()
                .filter_map(|record| crate::config::gpu_runtime_model_provider(record, &codex_home))
                .collect::<HashMap<_, _>>();
            if runtime_providers.contains_key(&model_provider_id) {
                let mut state = self.state.lock().await;
                let mut config = (*state.session_configuration.original_config_do_not_use).clone();
                // Runtime endpoints are controller-owned process-local transports. Replacing an
                // existing entry is required after controller or SSH-forward recovery; presence
                // alone does not mean the cached base URL is still current.
                config.model_providers.extend(runtime_providers);
                state.session_configuration.original_config_do_not_use = Arc::new(config);
            }
        }
    }

    fn build_model_client_for_configuration(
        &self,
        configuration: &SessionConfiguration,
    ) -> ModelClient {
        let config = configuration.original_config_do_not_use.as_ref();
        ModelClient::new(
            Some(self.services.auth_manager()),
            if config.features.enabled(Feature::UseAgentIdentity) {
                codex_login::auth::AgentIdentityAuthPolicy::ChatGptAuth
            } else {
                codex_login::auth::AgentIdentityAuthPolicy::JwtOnly
            },
            self.thread_id,
            configuration.provider.clone(),
            configuration.session_source.clone(),
            configuration.originator.clone(),
            config.model_verbosity,
            config.features.enabled(Feature::EnableRequestCompression),
            config.features.enabled(Feature::RuntimeMetrics),
            Self::build_model_client_beta_features_header(config),
            config
                .features
                .enabled(Feature::ConcurrentReasoningSummaries),
            self.services.attestation_provider.clone(),
            config.http_client_factory(),
        )
        .with_ingress_policy(
            config.security_level,
            self.services.agent_control.effective_security_policy(),
        )
        .with_native_ingress_from(&self.services.model_client())
        .with_source_envelopes(config.features.enabled(Feature::SourceEnvelopes))
        .with_source_origin_key(config.codex_home.as_path())
        .with_prompt_cache_key_override(
            crate::guardian::prompt_cache_key_override_for_review_session(
                &configuration.session_source,
                configuration.parent_thread_id,
            ),
        )
    }

    pub(crate) async fn preview_settings(
        &self,
        updates: &SessionSettingsUpdate,
    ) -> ConstraintResult<ThreadConfigSnapshot> {
        let mut updates = updates.clone();
        self.ensure_runtime_model_provider(&mut updates).await;
        let configuration = {
            let state = self.state.lock().await;
            self.apply_session_settings(&state.session_configuration, &updates)?
        };
        Ok(self
            .effective_thread_config_snapshot(&configuration, configuration.environments.clone())
            .await)
    }

    pub(crate) async fn thread_config_snapshot(&self) -> ThreadConfigSnapshot {
        self.resume_config_snapshot().await.0
    }

    pub(crate) async fn resume_config_snapshot(&self) -> (ThreadConfigSnapshot, Option<String>) {
        let configuration = self.state.lock().await.session_configuration.clone();
        let selected_service_tier = configuration.step_settings.service_tier.clone();
        let snapshot = self
            .effective_thread_config_snapshot(&configuration, configuration.environments.clone())
            .await;
        (snapshot, selected_service_tier)
    }

    /// Captures configured child bindings, or the active turn's bindings for legacy inheritance.
    pub(crate) async fn inherited_environment_snapshot(&self) -> TurnEnvironmentSnapshot {
        let snapshot = {
            let _state = self.state.lock().await;
            if self.features.enabled(Feature::SubagentThreadEnvironments) {
                return self.services.turn_environments.configured_snapshot_now();
            }
            self.services.turn_environments.snapshot()
        };
        snapshot.await
    }

    pub(crate) async fn configured_environment_selections(&self) -> Vec<TurnEnvironmentSelection> {
        let state = self.state.lock().await;
        state.session_configuration.environments.clone()
    }

    pub(crate) async fn thread_settings_snapshot(&self) -> ThreadSettingsSnapshot {
        let state = self.state.lock().await;
        state
            .session_configuration
            .thread_settings_snapshot(&state.session_configuration.environments)
    }

    pub(crate) async fn restorable_thread_settings(&self) -> CodexThreadSettingsOverrides {
        let state = self.state.lock().await;
        state
            .session_configuration
            .restorable_thread_settings(state.session_configuration.environments.clone())
    }

    pub(crate) async fn set_app_server_client_info(
        &self,
        app_server_client_name: Option<String>,
        app_server_client_version: Option<String>,
        mcp_elicitations_auto_deny: bool,
    ) -> ConstraintResult<()> {
        self.update_settings(SessionSettingsUpdate {
            app_server_client_name,
            app_server_client_version,
            ..Default::default()
        })
        .await?;
        self.services
            .mcp_runtime
            .set_elicitations_auto_deny(mcp_elicitations_auto_deny);
        Ok(())
    }

    pub(crate) async fn instruction_sources(&self) -> Vec<PathUri> {
        self.services
            .agents_md_manager
            .get_loaded()
            .await
            .as_ref()
            .map_or_else(Vec::new, |instructions| instructions.sources().collect())
    }

    #[cfg(test)]
    pub(crate) async fn set_session_startup_prewarm(
        &self,
        startup_prewarm: SessionStartupPrewarmHandle,
    ) {
        let mut state = self.state.lock().await;
        state.set_session_startup_prewarm(startup_prewarm);
    }

    pub(crate) async fn take_session_startup_prewarm(&self) -> Option<SessionStartupPrewarmHandle> {
        let mut state = self.state.lock().await;
        state.take_session_startup_prewarm()
    }

    pub(crate) async fn fork_config(
        &self,
    ) -> (
        Config,
        ThreadConfigSnapshot,
        WindowsSandboxLevel,
        TurnEnvironmentSnapshot,
        ReasoningEffortPin,
    ) {
        let (configuration, environments, mut reasoning_effort_pin) = {
            let state = self.state.lock().await;
            (
                state.session_configuration.clone(),
                self.services.turn_environments.snapshot_now(),
                state.reasoning_effort_pin.clone(),
            )
        };
        let config = self.build_effective_session_config(&configuration);
        if !config.features.enabled(Feature::ReasoningEffortOverride) {
            let model = configuration
                .step_settings
                .resolve_model_info(
                    self.services.models_manager.as_ref(),
                    &configuration.model_info_overrides,
                )
                .await;
            if let Some(effort) = config
                .model_reasoning_effort
                .clone()
                .or_else(|| model.default_reasoning_level.clone())
            {
                // Without overrides, the parent's selected effort is its request baseline.
                reasoning_effort_pin = ReasoningEffortPin::Active {
                    model: model.slug.clone(),
                    effort: model.resolve_reasoning_effort(effort),
                };
            }
        }
        (
            config,
            self.effective_thread_config_snapshot(&configuration, environments.all_selections())
                .await,
            configuration.windows_sandbox_level,
            environments,
            reasoning_effort_pin,
        )
    }

    pub(crate) async fn get_config(&self) -> std::sync::Arc<Config> {
        let state = self.state.lock().await;
        state
            .session_configuration
            .original_config_do_not_use
            .clone()
    }

    pub(crate) async fn inherited_instructions(&self) -> SessionInstructions {
        self.services
            .agents_md_manager
            .inherited_instructions()
            .await
    }

    pub(crate) async fn emit_instruction_warnings(&self, warnings: Vec<String>) {
        for message in warnings {
            self.send_event_raw(Event {
                id: INITIAL_SUBMIT_ID.to_owned(),
                msg: EventMsg::Warning(WarningEvent { message }),
            })
            .await;
        }
    }

    pub(crate) async fn provider(&self) -> ModelProviderInfo {
        let state = self.state.lock().await;
        state.session_configuration.provider.info().clone()
    }

<<<<<<< HEAD
    /// The session's configured definition of provider `id`, user overrides
    /// such as `billing` included.
    pub(crate) async fn configured_model_provider(
        &self,
        id: &str,
    ) -> Option<codex_model_provider_info::ModelProviderInfo> {
        let state = self.state.lock().await;
        state
            .session_configuration
            .original_config_do_not_use
            .model_providers
            .get(id)
            .cloned()
    }

    /// Where this session's paid requests are recorded (PF-60-S05, PF-60-S04,
    /// PF-60-S06).
    ///
    /// A persisted session records under its own thread. A session with no
    /// thread of its own records under the conversation it works for, in that
    /// conversation's ledger, with a turn label saying what it is:
    /// - a guardian reviewer (the reusable trunk and its ephemeral forks): the
    ///   conversation it reviews for, as `review:`;
    /// - a memory consolidation agent: the conversation that started it, as
    ///   `consolidation:` (bound when the session is built, so its first
    ///   request, a startup prewarm included, already has its owner);
    /// - a sub-agent (a spawned agent, a `/review` task): the conversation it
    ///   works for, as `agent:` or `code-review:`. A `/review` task records
    ///   there even though it is persisted as a thread of its own (#898): that
    ///   thread names no parent the ledger can resolve, so its requests would
    ///   otherwise be counted under no conversation;
    /// - an ephemeral fork: the conversation it was forked from, as `side:`
    ///   for a TUI `/side` conversation and `fork:` for any other.
    ///
    /// When the conversation worked for is not persisted either (a guardian
    /// review or a sub-agent inside `/side`), its own owner is used and the
    /// labels nest (`side:review:`, `side:agent:`), so the work is counted
    /// once under the persisted conversation at the root.
    ///
    /// `Err` says why there is none: a session that is not collected by design
    /// (`exec --ephemeral`, or work for a conversation that is itself not
    /// collected), or a ledger that could not be reached.
    pub(crate) async fn accounting_owner(
        &self,
    ) -> Result<crate::accounting::AccountingOwner, crate::accounting::Uncollected> {
        self.accounting_owner_at(/*depth*/ 0).await
    }

    async fn accounting_owner_at(
        &self,
        depth: usize,
    ) -> Result<crate::accounting::AccountingOwner, crate::accounting::Uncollected> {
        use crate::accounting::Uncollected;
        use codex_protocol::protocol::SubAgentSource;
        let (guardian_parent, code_review_parent, worked_for, forked_from) = {
            let state = self.state.lock().await;
            let configuration = &state.session_configuration;
            let guardian_parent = match &configuration.session_source {
                SessionSource::SubAgent(SubAgentSource::Other(name))
                    if name == crate::guardian::GUARDIAN_REVIEWER_NAME =>
                {
                    Some(configuration.parent_thread_id)
                }
                _ => None,
            };
            let code_review_parent = match &configuration.session_source {
                SessionSource::SubAgent(SubAgentSource::Review) => configuration.parent_thread_id,
                _ => None,
            };
            let worked_for = match &configuration.session_source {
                SessionSource::SubAgent(SubAgentSource::ThreadSpawn {
                    parent_thread_id, ..
                }) => Some((*parent_thread_id, "agent:")),
                SessionSource::SubAgent(
                    SubAgentSource::Compact
                    | SubAgentSource::MemoryConsolidation
                    | SubAgentSource::Other(_),
                ) => configuration
                    .parent_thread_id
                    .map(|parent| (parent, "agent:")),
                _ => None,
            };
            let forked_from = configuration.forked_from_thread_id.map(|parent| {
                let side = matches!(
                    &configuration.thread_source,
                    Some(ThreadSource::Feature(feature))
                        if feature == ThreadSource::SIDE_CONVERSATION
                );
                (parent, if side { "side:" } else { "fork:" })
            });
            (guardian_parent, code_review_parent, worked_for, forked_from)
        };
        if let Some(parent) = guardian_parent {
            let parent = parent.ok_or(Uncollected::Excluded("guardian_without_parent"))?;
            return self
                .accounting_owner_elsewhere(parent, "review:", depth)
                .await;
        }
        if let Some(parent) = code_review_parent {
            return self
                .accounting_owner_elsewhere(parent, "code-review:", depth)
                .await;
        }
        if self.live_thread().is_some() {
            let db = self
                .state_db()
                .ok_or(Uncollected::Failed("no state database"))?;
            return Ok(crate::accounting::AccountingOwner {
                thread: self.thread_id,
                db,
                label_prefix: String::new(),
            });
        }
        if let Some(started_by) = self
            .services
            .thread_extension_data
            .get::<crate::accounting::StartedBy>()
        {
            return self
                .accounting_owner_elsewhere(started_by.0, "consolidation:", depth)
                .await;
        }
        if let Some((parent, label)) = worked_for.or(forked_from) {
            return self.accounting_owner_elsewhere(parent, label, depth).await;
        }
        Err(Uncollected::Excluded("ephemeral_session"))
    }

    /// `parent`'s ledger, for a session with no thread of its own that works
    /// for `parent`. Such a session has no database of its own; it shares the
    /// parent's thread store, and so the parent's ledger. A parent that is not
    /// persisted either is followed, while it is live, to its own owner.
    async fn accounting_owner_elsewhere(
        &self,
        parent: ThreadId,
        label: &'static str,
        depth: usize,
    ) -> Result<crate::accounting::AccountingOwner, crate::accounting::Uncollected> {
        use crate::accounting::Uncollected;
        /// Nested temporary work deeper than this is not followed further.
        const MAX_DEPTH: usize = 4;
        let db = match self.state_db() {
            Some(db) => Some(db),
            None => match self
                .services
                .thread_store
                .as_any()
                .downcast_ref::<codex_thread_store::LocalThreadStore>()
            {
                Some(store) => store.state_db().await,
                None => None,
            },
        }
        .ok_or(Uncollected::Failed("no state database"))?;
        match db.get_thread(parent).await {
            Ok(Some(_)) => Ok(crate::accounting::AccountingOwner {
                thread: parent,
                db,
                label_prefix: label.to_string(),
            }),
            // Work for a conversation that is itself not persisted (a side
            // conversation, a sub-agent of one) records where that
            // conversation records; work for one that is not collected at all
            // (exec --ephemeral) has no thread to record under.
            Ok(None) => {
                let parent_session = if depth < MAX_DEPTH && parent != self.thread_id {
                    self.services.agent_control.live_session(parent).await
                } else {
                    None
                };
                let Some(parent_session) = parent_session else {
                    return Err(Uncollected::Excluded("parent_not_persisted"));
                };
                match Box::pin(parent_session.accounting_owner_at(depth + 1)).await {
                    Ok(owner) => {
                        // A saved conversation whose thread row is not
                        // written yet (work started before its first saved
                        // item, such as `/review` racing the item that opens
                        // it) records under its own thread; save it now, as
                        // its own first request would, so the ledger has the
                        // thread to record under (#898).
                        if owner.thread == parent {
                            parent_session
                                .try_ensure_rollout_materialized()
                                .await
                                .map_err(|_| Uncollected::Failed("parent thread not saved"))?;
                        }
                        Ok(crate::accounting::AccountingOwner {
                            label_prefix: format!("{}{label}", owner.label_prefix),
                            ..owner
                        })
                    }
                    Err(Uncollected::Excluded(_)) => {
                        Err(Uncollected::Excluded("parent_not_persisted"))
                    }
                    Err(error) => Err(error),
                }
            }
            Err(_) => Err(Uncollected::Failed("parent thread unreadable")),
        }
    }

    /// The accounting mode and provider identity in force right now.
    ///
    /// Read live rather than snapshotted: a session's provider can change, and
    /// a client that records against the identity it had at startup would
    /// attribute a request to a provider it no longer uses.
    pub(crate) async fn accounting_binding(&self) -> (crate::config::AccountingMode, String) {
        let state = self.state.lock().await;
        let config = &state.session_configuration.original_config_do_not_use;
        (config.accounting.clone(), config.model_provider_id.clone())
    }

    pub(crate) async fn refresh_runtime_config(&self, next_config: Config) {
        // Refresh only the user layer from the incoming snapshot. Preserve thread-local
        // layers such as request/session overrides that were present when this session
        // was created.
        let notify_config_contributors = !self.services.extensions.config_contributors().is_empty();
        let (previous_config, new_config, config) = {
            let mut state = self.state.lock().await;
            let previous_config = notify_config_contributors
                .then(|| Self::build_effective_session_config(&state.session_configuration));
            let mut config = (*state.session_configuration.original_config_do_not_use).clone();
            config.config_layer_stack = config
                .config_layer_stack
                .with_user_layer_from(&next_config.config_layer_stack);
            config.tool_suggest =
                resolve_tool_suggest_config_from_layer_stack(&config.config_layer_stack);
            config.mcp_servers = next_config.mcp_servers.clone();
            config.mcp_oauth_credentials_store_mode = next_config.mcp_oauth_credentials_store_mode;
            if let Err(err) = config.features.set_enabled(
                Feature::SecretAuthStorage,
                next_config.features.enabled(Feature::SecretAuthStorage),
            ) {
                warn!("failed to refresh MCP auth storage config: {err}");
||||||| base
    pub(crate) async fn refresh_runtime_config(&self, next_config: Config) {
        // Refresh only the user layer from the incoming snapshot. Preserve thread-local
        // layers such as request/session overrides that were present when this session
        // was created.
        let notify_config_contributors = !self.services.extensions.config_contributors().is_empty();
        let (previous_config, new_config, config) = {
            let mut state = self.state.lock().await;
            let previous_config = notify_config_contributors
                .then(|| Self::build_effective_session_config(&state.session_configuration));
            let mut config = (*state.session_configuration.original_config_do_not_use).clone();
            config.config_layer_stack = config
                .config_layer_stack
                .with_user_layer_from(&next_config.config_layer_stack);
            config.tool_suggest =
                resolve_tool_suggest_config_from_layer_stack(&config.config_layer_stack);
            config.mcp_servers = next_config.mcp_servers.clone();
            config.mcp_oauth_credentials_store_mode = next_config.mcp_oauth_credentials_store_mode;
            if let Err(err) = config.features.set_enabled(
                Feature::SecretAuthStorage,
                next_config.features.enabled(Feature::SecretAuthStorage),
            ) {
                warn!("failed to refresh MCP auth storage config: {err}");
=======
    pub(crate) async fn refresh_hooks(&self, mut config: Arc<Config>) {
        loop {
            let disabled_plugin_ids = self.state.lock().await.active_disabled_plugin_ids.clone();
            let environments = self.services.turn_environments.snapshot().await;
            let hooks_config = build_hooks_config(
                config.as_ref(),
                self.services.plugins_manager.as_ref(),
                environments.single_local_environment(),
                &disabled_plugin_ids,
            )
            .await;

            let state = self.state.lock().await;
            if Arc::ptr_eq(
                &state.session_configuration.original_config_do_not_use,
                &config,
            ) && state.active_disabled_plugin_ids == disabled_plugin_ids
            {
                let hooks = self.hooks().reconfigured(hooks_config);
                self.services.hooks.store(Arc::new(hooks));
                return;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
            }
            // Unrelated updates can replace the owner too. Rebuild current inputs
            // so they cannot strand a pending plugin-policy refresh.
            config = Arc::clone(&state.session_configuration.original_config_do_not_use);
        }
    }

    fn emit_config_changed_contributors(
        &self,
        previous_config: Option<&Config>,
        new_config: Option<&Config>,
    ) {
        let (Some(previous_config), Some(new_config)) = (previous_config, new_config) else {
            return;
        };
        if previous_config == new_config {
            return;
        }
        for contributor in self.services.extensions.config_contributors() {
            contributor.on_config_changed(
                &self.services.session_extension_data,
                &self.services.thread_extension_data,
                previous_config,
                new_config,
            );
        }
    }

    pub(crate) async fn reload_user_config_layer(&self) {
        // Refresh layer-backed runtime state for an existing session, including enabled plugin,
        // skill, hook, and MCP feature state. Other feature gates and legacy notify settings
        // remain session-static.
        //
        // Prefer `refresh_runtime_config()` when the host can already provide a materialized
        // config snapshot. This file-based path exists for legacy local reload flows.
        // Retry from the current owner if another publication wins while files are read.
        loop {
            let (expected_config, config_toml_paths) = {
                let state = self.state.lock().await;
                let config = Arc::clone(&state.session_configuration.original_config_do_not_use);
                let user_config_paths = config
                    .config_layer_stack
                    .all_layers_low_to_high()
                    .filter_map(|layer| match &layer.name {
                        ConfigLayerSource::User { file, .. } => Some(file.clone()),
                        _ => None,
                    })
                    .collect::<Vec<_>>();
                let paths = if user_config_paths.is_empty() {
                    vec![
                        state
                            .session_configuration
                            .codex_home
                            .join(CONFIG_TOML_FILE),
                    ]
                } else {
                    user_config_paths
                };
                (config, paths)
            };

            let mut reloaded_user_configs = Vec::with_capacity(config_toml_paths.len());
            for config_toml_path in config_toml_paths {
                let user_config = match std::fs::read_to_string(&config_toml_path) {
                    Ok(contents) => match toml::from_str::<toml::Value>(&contents) {
                        Ok(config) => config,
                        Err(err) => {
                            warn!("failed to parse user config while reloading layer: {err}");
                            return;
                        }
                    },
                    Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                        toml::Value::Table(Default::default())
                    }
                    Err(err) => {
                        warn!("failed to read user config while reloading layer: {err}");
                        return;
                    }
                };
                let Some(config_dir) = config_toml_path.parent() else {
                    warn!("user config path has no parent directory");
                    return;
                };
                let user_config = match codex_config::loader::resolve_relative_paths_in_config_toml(
                    user_config,
                    &config_dir,
                ) {
                    Ok(config) => config,
                    Err(err) => {
                        warn!("failed to resolve paths while reloading user config: {err}");
                        return;
                    }
                };
                reloaded_user_configs.push((config_toml_path, user_config));
            }

            let mut next_config = expected_config.as_ref().clone();
            for (config_toml_path, user_config) in reloaded_user_configs {
                let config_layer_stack = match next_config
                    .config_layer_stack
                    .with_user_config(&config_toml_path, user_config)
                {
                    Ok(config_layer_stack) => config_layer_stack,
                    Err(err) => {
                        warn!("failed to validate user config while reloading layer: {err}");
                        return;
                    }
                };
                next_config.config_layer_stack = config_layer_stack;
            }
            match self
                .refresh_config(
                    expected_config,
                    next_config,
                    crate::config::RuntimeConfigRefresh::UserFiles,
                )
                .await
            {
                crate::ConfigRefreshOutcome::Stale => continue,
                crate::ConfigRefreshOutcome::Published | crate::ConfigRefreshOutcome::Rejected => {
                    return;
                }
            }
        }
    }

    /// Record a terminal CodexErr before the app-server completion notification is reduced.
    pub(crate) fn track_turn_codex_error(&self, turn_context: &TurnContext, error: &CodexErr) {
        self.services
            .analytics_events_client
            .track_turn_codex_error(TurnCodexErrorFact::from_codex_err(
                self.thread_id.to_string(),
                turn_context.sub_id.clone(),
                error,
            ));
    }

    /// Returns the reviewer pool installed by the Guardian extension.
    pub(crate) fn guardian_review_session(&self) -> Option<Arc<GuardianReviewSessionManager>> {
        self.services
            .thread_extension_data
            .get::<GuardianReviewSessionManager>()
    }

    pub(crate) async fn is_private_guardian_reviewer(&self) -> bool {
        self.state
            .lock()
            .await
            .session_configuration
            .trusted_guardian_reviewer
    }

    pub(crate) async fn emit_turn_started(&self, turn_context: &TurnContext, task_kind: TaskKind) {
        let event = TurnStartedEvent {
            turn_attribution: (task_kind == TaskKind::Regular).then(|| turn_context.attribution()),
            turn_id: turn_context.sub_id.clone(),
            root_turn_id: Some(turn_context.root_turn_id()),
            trace_id: turn_context.trace_id.clone(),
            started_at: turn_context.turn_timing_state.started_at_unix_secs().await,
            model_context_window: turn_context.model_context_window(),
            collaboration_mode_kind: turn_context.mode(),
        };
        self.send_event(turn_context, EventMsg::TurnStarted(event))
            .await;
    }

    /// Persist the event to rollout and send it to clients.
    pub(crate) async fn send_event(&self, turn_context: &TurnContext, msg: EventMsg) {
<<<<<<< HEAD
        // PF-28-S01: gate before traces, rollout, realtime mirroring and clients.
        for msg in crate::security::disclosure_gate::gate_event(&turn_context.sub_id, msg) {
            self.send_gated_event(turn_context, msg).await;
        }
    }

    async fn send_gated_event(&self, turn_context: &TurnContext, msg: EventMsg) {
||||||| base
=======
        if let Some((item_id, started_at_ms)) = msg.command_approval_start() {
            // The client can see the command before execution. Keep its first start
            // for the canonical lifecycle events that will be saved in history.
            turn_context
                .turn_timing_state
                .record_item_started(item_id.to_owned(), started_at_ms)
                .await;
        }
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        let legacy_source = msg.clone();
        if let EventMsg::Error(error) = &legacy_source
            && error
                .codex_error_info
                .as_ref()
                .is_some_and(CodexErrorInfo::affects_turn_status)
        {
            turn_context
                .terminal_error
                .lock()
                .await
                .replace(error.clone());
        }
        self.services
            .rollout_thread_trace
            .record_codex_turn_event(&turn_context.sub_id, &legacy_source);
        self.services
            .rollout_thread_trace
            .record_tool_call_event(turn_context.sub_id.clone(), &legacy_source);
        let event = Event {
            id: turn_context.sub_id.clone(),
            msg,
        };
        // Private reviewers have no app-server listener; publicly resumed Guardian threads do.
        if matches!(
            &turn_context.session_source,
            SessionSource::SubAgent(SubAgentSource::Other(name))
                if name == crate::guardian::GUARDIAN_REVIEWER_NAME
        ) && self.services.analytics_events_client.is_enabled()
            && turn_context.parent_thread_id.is_some()
            && self
                .state
                .lock()
                .await
                .session_configuration
                .trusted_guardian_reviewer
        {
            self.services
                .analytics_events_client
                .track_guardian_session_event(self.thread_id, &event);
        }
        self.send_event_raw(event).await;
        self.maybe_notify_parent_of_terminal_turn(turn_context, &legacy_source)
            .await;
        self.maybe_mirror_event_text_to_realtime(&legacy_source)
            .await;
        self.maybe_clear_realtime_handoff_for_event(&legacy_source)
            .await;

        let show_raw_agent_reasoning = self.show_raw_agent_reasoning();
        for legacy in legacy_source.as_legacy_events(show_raw_agent_reasoning) {
            self.services
                .rollout_thread_trace
                .record_tool_call_event(turn_context.sub_id.clone(), &legacy);
            let legacy_event = Event {
                id: turn_context.sub_id.clone(),
                msg: legacy,
            };
            self.send_event_raw(legacy_event).await;
        }
    }

    /// Forwards terminal turn events from spawned MultiAgentV2 children to their direct parent.
    async fn maybe_notify_parent_of_terminal_turn(
        &self,
        turn_context: &TurnContext,
        msg: &EventMsg,
    ) {
        if turn_context.multi_agent_version != MultiAgentVersion::V2 {
            return;
        }

        if !matches!(msg, EventMsg::TurnComplete(_) | EventMsg::TurnAborted(_)) {
            return;
        }

        if !turn_context
            .parent_completion_expected
            .load(std::sync::atomic::Ordering::Acquire)
        {
            return;
        }

        let SessionSource::SubAgent(SubAgentSource::ThreadSpawn {
            agent_path: Some(_),
            ..
        }) = &turn_context.session_source
        else {
            return;
        };

        let mut error_info = None;
        let status = match turn_context.terminal_error.lock().await.take() {
            Some(error) => {
                let status = AgentStatus::Errored(error.message);
                self.agent_status.send_replace(status.clone());
                status
            }
            None => {
                if let EventMsg::TurnAborted(event) = msg
                    && event.reason == TurnAbortReason::Interrupted
                    && let Some(error) = &event.error
                    && error.codex_error_info == Some(CodexErrorInfo::TooManyDenials)
                {
                    // Report the safety stop to the parent without changing the child's
                    // interrupted status or notifying for ordinary interruptions.
                    error_info = error.codex_error_info.clone();
                    AgentStatus::Errored(error.message.clone())
                } else {
                    let Some(status) = agent_status_from_event(msg) else {
                        return;
                    };
                    status
                }
            }
        };
        if !is_final(&status) {
            return;
        }

<<<<<<< HEAD
        self.forward_child_completion_to_parent(
            turn_context,
            *parent_thread_id,
            child_agent_path,
            status,
        )
        .await;
    }

    /// Sends the standard completion envelope from a spawned MultiAgentV2 child to its parent.
    async fn forward_child_completion_to_parent(
        &self,
        turn_context: &TurnContext,
        parent_thread_id: ThreadId,
        child_agent_path: &codex_protocol::AgentPath,
        status: AgentStatus,
    ) {
        let Some(parent_agent_path) = child_agent_path
            .as_str()
            .rsplit_once('/')
            .and_then(|(parent, _)| codex_protocol::AgentPath::try_from(parent).ok())
        else {
            return;
        };

        let Some(message) = format_inter_agent_completion_message(
            parent_agent_path.clone(),
            child_agent_path.clone(),
            &status,
        ) else {
            return;
        };
        // `communication` owns the message. Keep a second copy only when the
        // recorder will actually need it after parent delivery succeeds.
        let trace_message = self
            .services
            .rollout_thread_trace
            .is_enabled()
            .then(|| message.clone());
        let trigger_turn = self
            .services
            .agent_control
            .auto_processes_terminal_results(parent_thread_id)
            .await;
        let mut communication = InterAgentCommunication::new(
            child_agent_path.clone(),
            parent_agent_path,
            Vec::new(),
            message,
            trigger_turn,
        );
        communication.kind = Some(codex_protocol::protocol::AgentMessageKind::TerminalResult);
        communication.message_id = Some(format!(
            "completion:{}:{}",
            self.thread_id, turn_context.sub_id
        ));
        let context =
            AgentCommunicationContext::new(AgentCommunicationKind::Result, self.thread_id);
        if let Err(err) = self
            .services
||||||| base
        self.forward_child_completion_to_parent(
            turn_context,
            *parent_thread_id,
            child_agent_path,
            status,
        )
        .await;
    }

    /// Sends the standard completion envelope from a spawned MultiAgentV2 child to its parent.
    async fn forward_child_completion_to_parent(
        &self,
        turn_context: &TurnContext,
        parent_thread_id: ThreadId,
        child_agent_path: &codex_protocol::AgentPath,
        status: AgentStatus,
    ) {
        let Some(parent_agent_path) = child_agent_path
            .as_str()
            .rsplit_once('/')
            .and_then(|(parent, _)| codex_protocol::AgentPath::try_from(parent).ok())
        else {
            return;
        };

        let Some(message) = format_inter_agent_completion_message(
            parent_agent_path.clone(),
            child_agent_path.clone(),
            &status,
        ) else {
            return;
        };
        // `communication` owns the message. Keep a second copy only when the
        // recorder will actually need it after parent delivery succeeds.
        let trace_message = self
            .services
            .rollout_thread_trace
            .is_enabled()
            .then(|| message.clone());
        let communication = InterAgentCommunication::new(
            child_agent_path.clone(),
            parent_agent_path,
            Vec::new(),
            message,
            /*trigger_turn*/ false,
        );
        let context =
            AgentCommunicationContext::new(AgentCommunicationKind::Result, self.thread_id);
        if let Err(err) = self
            .services
=======
        self.services
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
            .agent_control
            .turn_finished(
                AgentTurnOutcome {
                    thread_id: self.thread_id,
                    turn_id: turn_context.sub_id.clone(),
                    source: turn_context.session_source.clone(),
                    parent_turn_id: turn_context.turn_metadata_state.parent_turn_id(),
                    initiating_agent_path: turn_context
                        .turn_metadata_state
                        .initiating_agent_path()
                        .cloned(),
                    status,
                    error_info,
                },
                &self.services.rollout_thread_trace,
            )
            .await;
    }

    async fn maybe_mirror_event_text_to_realtime(&self, msg: &EventMsg) {
        if self.conversation.running_state().await.is_none() {
            return;
        }
        match msg {
            EventMsg::ItemStarted(event) => {
                if let TurnItem::AgentMessage(item) = &event.item {
                    self.conversation
                        .register_handoff_stream_item(
                            item.id.clone(),
                            item.phase.clone(),
                            agent_message_text(item),
                        )
                        .await;
                }
                return;
            }
            EventMsg::AgentMessageContentDelta(event) => {
                if let Err(err) = self
                    .conversation
                    .stream_handoff_delta(&event.item_id, event.delta.clone())
                    .await
                {
                    debug!("failed to stream event text to realtime conversation: {err}");
                }
                return;
            }
            EventMsg::ItemCompleted(event) => {
                if let TurnItem::AgentMessage(item) = &event.item
                    && self.conversation.finish_handoff_stream_item(&item.id).await
                {
                    return;
                }
            }
            _ => {}
        }
        let result = match realtime_text_for_event(msg) {
            Some(RealtimeEventText::Handoff(text, phase)) => {
                self.conversation.handoff_out(text, phase).await
            }
            Some(RealtimeEventText::QuietReasoning(text)) => {
                self.conversation.send_reasoning_status(&text).await
            }
            None => return,
        };
        if let Err(err) = result {
            debug!("failed to mirror event text to realtime conversation: {err}");
        }
    }

    async fn maybe_clear_realtime_handoff_for_event(&self, msg: &EventMsg) {
        if !matches!(msg, EventMsg::TurnComplete(_)) {
            return;
        }
        if let Err(err) = self.conversation.handoff_complete().await {
            debug!("failed to finalize realtime handoff output: {err}");
        }
        self.conversation.clear_active_handoff().await;
    }

    pub(crate) async fn send_event_raw(&self, event: Event) {
        self.send_event_raw_with_persistence(event, /*persist*/ true)
            .await;
    }

    /// Delivers an event to the client without writing it to the rollout: a
    /// notice about this session's own start that a later reopen must not
    /// replay as if it had happened again.
    pub(crate) async fn send_event_raw_unpersisted(&self, event: Event) {
        self.send_event_raw_with_persistence(event, /*persist*/ false)
            .await;
    }

    /// Delivers an event without creating a local rollout for a thread that has not materialized.
    pub(crate) async fn send_event_raw_without_materializing_rollout(&self, event: Event) {
        let persist = match self.current_rollout_path().await {
            Ok(Some(path)) => codex_rollout::existing_rollout_path(&path).await.is_some(),
            Ok(None) => true,
            Err(err) => {
                warn!("failed to check whether thread persistence is materialized: {err}");
                true
            }
        };
        self.send_event_raw_with_persistence(event, persist).await;
    }

    async fn send_event_raw_with_persistence(&self, event: Event, persist: bool) {
<<<<<<< HEAD
        let mut event = event;
        match crate::security::disclosure_gate::gate_presented(event.msg) {
            Some(msg) => event.msg = msg,
            None => return,
||||||| base
=======
        let flush_guardian_completion = persist
            && matches!(event.msg, EventMsg::TurnComplete(_))
            && self.is_private_guardian_reviewer().await;
        // Keep realtime reduction, canonical append, and delivery in the same order.
        // This lock must not acquire SessionState or ActiveTurn: event producers can
        // already hold those locks. Host presentation policies are synchronous.
        let mut realtime_history = match &self.realtime_history {
            Some(history) => {
                let history = history.lock().await;
                history.should_observe(&event.msg).then_some(history)
            }
            None => None,
        };
        self.services.mcp_runtime.observe_event(&event.msg);
        let (before_event, after_event) = match realtime_history.as_mut() {
            Some(history) => {
                let effects = history.observe(&event.msg);
                match effects.order {
                    RealtimeEventOrder::BeforeEvent => (Some(effects), None),
                    RealtimeEventOrder::AfterEvent => (None, Some(effects)),
                }
            }
            None => (None, None),
        };
        if let Some(effects) = before_event
            && let Err(error) = self.send_realtime_history_effects(&event.id, effects).await
        {
            warn!("failed to persist realtime history: {error}");
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        }
        // Persist the event into rollout storage; the store applies its persistence policy.
        if persist {
            let rollout_items = vec![RolloutItem::EventMsg(event.msg.clone())];
            self.persist_rollout_items(&rollout_items).await;
        }
        self.services
            .rollout_thread_trace
            .record_protocol_event(&event.msg);
        if let Some(effects) = after_event
            && let Err(error) = self.send_realtime_history_effects(&event.id, effects).await
        {
            warn!("failed to persist realtime history: {error}");
        }
        // Save the completed review and its terminal event together before the parent sees
        // the decision. Turn completion skips its separate before/after barriers for this case.
        if flush_guardian_completion && let Err(err) = self.flush_rollout().await {
            warn!("failed to flush completed Guardian review: {err}");
        }
        self.deliver_event_raw(event).await;
    }

    /// Sets the agent status a later event would set, for tests that need a
    /// status (`PendingInit`, say) a real turn would leave at any moment.
    #[cfg(test)]
    pub(crate) fn set_agent_status_for_test(&self, status: AgentStatus) {
        self.agent_status.send_replace(status);
    }

    async fn deliver_event_raw(&self, event: Event) {
        if let Some(parent_tx) = self.parent_observer_event_tx.get() {
            let forwarded = match &event.msg {
                EventMsg::RawResponseItem(item) => {
                    Some(PrivateSessionEventPayload::RecordedItem(item.clone()))
                }
                EventMsg::TurnAborted(aborted) => {
                    Some(PrivateSessionEventPayload::TurnAborted(aborted.clone()))
                }
                _ => None,
            };
            if let Some(forwarded) = forwarded {
                // Queue observer events before the private consumer can finish the review.
                // They bypass the parent's history and retain the reviewer's identity.
                let _ = parent_tx
                    .send(Event {
                        id: event.id.clone(),
                        msg: EventMsg::PrivateSessionEvent(PrivateSessionEvent {
                            thread_id: self.thread_id,
                            event: forwarded,
                        }),
                    })
                    .await;
            }
        }
        // Record the last known agent status.
        if let Some(status) = agent_status_from_event(&event.msg) {
            self.agent_status.send_replace(status);
        }
        if let Err(e) = self.tx_event.send(event).await {
            debug!("dropping event because channel is closed: {e}");
        }
    }

    pub(crate) async fn emit_turn_item_started(&self, turn_context: &TurnContext, item: &TurnItem) {
        let started_at_ms = turn_context
            .turn_timing_state
            .record_item_started(item.id(), now_unix_timestamp_ms())
            .await;
        self.send_event(
            turn_context,
            EventMsg::ItemStarted(ItemStartedEvent {
                thread_id: self.thread_id,
                turn_id: turn_context.sub_id.clone(),
                item: item.clone(),
                started_at_ms,
            }),
        )
        .await;
    }

    pub(crate) async fn emit_turn_item_completed(
        &self,
        turn_context: &TurnContext,
        item: TurnItem,
    ) {
        record_turn_ttfm_metric(turn_context, &item).await;
        for contributor in self.services.extensions.turn_lifecycle_contributors() {
            contributor
                .on_item_completed(
                    &self.services.thread_extension_data,
                    turn_context.extension_data.as_ref(),
                    &item,
                )
                .await;
        }
        let completed_at_ms = now_unix_timestamp_ms();
        let item_id = item.id();
        let started_at_ms = turn_context
            .turn_timing_state
            .take_item_started(&item_id)
            .await
            .unwrap_or_else(|| {
                warn!(
                    thread_id = %self.thread_id,
                    turn_id = %turn_context.sub_id,
                    item_id = %item_id,
                    "item completed without a recorded start timestamp"
                );
                completed_at_ms
            });
        self.send_event(
            turn_context,
            EventMsg::ItemCompleted(ItemCompletedEvent {
                thread_id: self.thread_id,
                turn_id: turn_context.sub_id.clone(),
                item,
                started_at_ms: Some(started_at_ms),
                completed_at_ms,
            }),
        )
        .await;
    }

    /// Adds an execpolicy amendment to both the in-memory and on-disk policies so future
    /// commands can use the newly approved prefix. #626: one approved while
    /// Aggressive is active is recorded as such, so it applies under
    /// Aggressive too (the others are suspended there).
    /// Saves a "don't ask again" answer. #945: `shown_under` is the policy
    /// marker its prompt was shown under; when a security transition was
    /// committed since, the answer was for a stale prompt and nothing is
    /// saved (`Ok(None)`).
    pub(crate) async fn persist_execpolicy_amendment(
        &self,
        amendment: &ExecPolicyAmendment,
        shown_under: Option<(u64, u64)>,
    ) -> Result<Option<crate::exec_policy::AmendmentScope>, ExecPolicyUpdateError> {
        let codex_home = self
            .state
            .lock()
            .await
            .session_configuration
            .codex_home()
            .clone();

        // Aggressive in force now (an unreadable live level counts as
        // Aggressive, as it is enforced): the person approved it under
        // Aggressive. Read before the marker: the marker only moves forward,
        // so an unchanged marker means this was the level at the prompt.
        let approved_under = if self.services.model_client().live_level_is_aggressive() {
            crate::exec_policy::ApprovedUnder::Aggressive
        } else {
            crate::exec_policy::ApprovedUnder::LessStrict
        };
        if self.services.transition_since(shown_under) {
            return Ok(None);
        }
        self.services
            .exec_policy
            .append_amendment_and_update(&codex_home, amendment, approved_under)
            .await
            .map(Some)
    }

    /// Whether `captured` (a running turn) was admitted under the execution
    /// authority `current` grants now: approval policy, permission profile,
    /// approvals reviewer and Windows sandbox level (see [`TurnAuthority`]).
    pub(super) fn authorization_matches(
        current: &SessionConfiguration,
        captured: &TurnContext,
    ) -> bool {
        TurnAuthority::of(current) == captured.authority
    }

    /// #945: the policy marker the open approval prompt `approval_id` was
    /// shown under; `None` when no such prompt is open.
    #[expect(
        clippy::await_holding_invalid_type,
        reason = "active turn reads must stay consistent with the matching turn state"
    )]
    pub(crate) async fn pending_approval_shown_under(
        &self,
        approval_id: &str,
    ) -> Option<Option<(u64, u64)>> {
        let active = self.active_turn.lock().await;
        let active = active.as_ref()?;
        let ts = active.turn_state.lock().await;
        ts.pending_approval_shown_under(approval_id)
    }

    pub(crate) async fn turn_context_for_sub_id(&self, sub_id: &str) -> Option<Arc<TurnContext>> {
        let active = self.active_turn.lock().await;
        active
            .as_ref()
            .and_then(|turn| turn.task.as_ref())
            .filter(|task| task.turn_context.sub_id == sub_id)
            .map(|task| Arc::clone(&task.turn_context))
    }

    async fn active_turn_context_and_cancellation_token(
        &self,
    ) -> Option<(Arc<TurnContext>, CancellationToken)> {
        let active = self.active_turn.lock().await;
        let task = active.as_ref()?.task.as_ref()?;
        Some((
            Arc::clone(&task.turn_context),
            task.cancellation_token.child_token(),
        ))
    }

    pub(crate) async fn record_execpolicy_amendment_message(
        &self,
        sub_id: &str,
        amendment: &ExecPolicyAmendment,
    ) {
        let Some(prefixes) = format_allow_prefixes(vec![amendment.command.clone()]) else {
            warn!("execpolicy amendment for {sub_id} had no command prefix");
            return;
        };
        let message: ResponseItem =
            ContextualUserFragment::into(ApprovedCommandPrefixSaved::new(prefixes));
        let turn_context = self.turn_context_for_sub_id(sub_id).await;
        self.inject_no_new_turn(vec![message], turn_context.as_deref())
            .await;
    }

    /// `shown_under` (#945): the policy marker an allowing answer's prompt
    /// was shown under; when a security transition was committed since,
    /// nothing is changed.
    pub(crate) async fn persist_network_policy_amendment(
        &self,
        amendment: &NetworkPolicyAmendment,
        network_approval_context: &NetworkApprovalContext,
        shown_under: Option<Option<(u64, u64)>>,
        on_policy_applied: impl FnOnce() + Send,
    ) -> anyhow::Result<()> {
        let _refresh_guard = self
            .managed_network_proxy_refresh_lock
            .acquire()
            .await
            .map_err(|_| anyhow::anyhow!("managed network proxy refresh semaphore closed"))?;
        // D52 (#1001): one allowed while Aggressive is in force applies under
        // Aggressive too (the others are suspended there). An unreadable live
        // level counts as Aggressive, as it is enforced. Read before the
        // marker: the marker only moves forward, so an unchanged marker means
        // this was the level at the prompt (as for command rules, #945).
        let approved_under = if self.services.model_client().live_level_is_aggressive() {
            crate::exec_policy::ApprovedUnder::Aggressive
        } else {
            crate::exec_policy::ApprovedUnder::LessStrict
        };
        if let Some(shown_under) = shown_under
            && self.services.transition_since(shown_under)
        {
            anyhow::bail!("the security level changed while network access was being approved");
        }
        let host =
            Self::validated_network_policy_amendment_host(amendment, network_approval_context)?;
        let codex_home = self
            .state
            .lock()
            .await
            .session_configuration
            .codex_home()
            .clone();
        let execpolicy_amendment =
            execpolicy_network_rule_amendment(amendment, network_approval_context, &host);
        let mut on_policy_applied = Some(on_policy_applied);

        if let Some(started_network_proxy) = self.services.network_proxy.load_full() {
            let proxy = started_network_proxy.proxy();
            match amendment.action {
                NetworkPolicyRuleAction::Allow => proxy
                    .add_allowed_domain(&host)
                    .await
                    .map_err(|err| anyhow::anyhow!("failed to update runtime allowlist: {err}"))?,
                NetworkPolicyRuleAction::Deny => proxy
                    .add_denied_domain(&host)
                    .await
                    .map_err(|err| anyhow::anyhow!("failed to update runtime denylist: {err}"))?,
            }
            // Active enforcement changed successfully. Notify the owner before
            // the next fallible await so cancellation cannot contradict it.
            if let Some(on_policy_applied) = on_policy_applied.take() {
                on_policy_applied();
            }
        }

        self.services
            .exec_policy
            .append_network_rule_and_update(
                &codex_home,
                &host,
                execpolicy_amendment.protocol,
                execpolicy_amendment.decision,
                Some(execpolicy_amendment.justification),
                approved_under,
            )
            .await
            .map_err(|err| {
                anyhow::anyhow!("failed to persist network policy amendment to execpolicy: {err}")
            })?;

        // Without a running proxy, persistence is the first effective policy change.
        if let Some(on_policy_applied) = on_policy_applied {
            on_policy_applied();
        }

        Ok(())
    }

    fn validated_network_policy_amendment_host(
        amendment: &NetworkPolicyAmendment,
        network_approval_context: &NetworkApprovalContext,
    ) -> anyhow::Result<String> {
        let approved_host = normalize_host(&network_approval_context.host);
        let amendment_host = normalize_host(&amendment.host);
        if amendment_host != approved_host {
            return Err(anyhow::anyhow!(
                "network policy amendment host '{}' does not match approved host '{}'",
                amendment.host,
                network_approval_context.host
            ));
        }
        Ok(approved_host)
    }

    pub(crate) async fn record_network_policy_amendment_message(
        &self,
        sub_id: &str,
        amendment: &NetworkPolicyAmendment,
    ) {
        let message: ResponseItem = ContextualUserFragment::into(NetworkRuleSaved::new(amendment));
        let turn_context = self.turn_context_for_sub_id(sub_id).await;
        self.inject_no_new_turn(vec![message], turn_context.as_deref())
            .await;
    }

    fn user_wait_timing_owner(
        &self,
        turn_context: &TurnContext,
        call_id: Option<&str>,
    ) -> Option<Arc<codex_direct_tool_timing::ToolTimingGuard>> {
        if !turn_context
            .config
            .features
            .enabled(Feature::CriticalPathTiming)
        {
            return None;
        }
        // Retain the invocation identity before request preparation can suspend.
        call_id.and_then(|call_id| {
            turn_context
                .turn_timing_state
                .direct_tool_timings
                .owner_for_turn(call_id, &turn_context.sub_id)
        })
    }

    fn start_user_wait_timing(
        &self,
        turn_context: &TurnContext,
        owner: Option<&codex_direct_tool_timing::ToolTimingGuard>,
    ) -> Option<codex_direct_tool_timing::ToolBlockingTimingGuard> {
        let timing = owner.and_then(codex_direct_tool_timing::ToolTimingGuard::start_user_wait);
        if timing.is_none()
            && turn_context
                .config
                .features
                .enabled(Feature::CriticalPathTiming)
        {
            turn_context.turn_timing_state.invalidate_critical_path();
        }
        timing
    }

    /// Keep tool results paused and attribute a queued network prompt to its triggering call.
    pub(crate) fn hold_queued_network_approval(
        &self,
        turn: &TurnContext,
        call_id: Option<&str>,
    ) -> (
        crate::elicitation::ElicitationRegistration,
        Option<codex_direct_tool_timing::ToolBlockingTimingGuard>,
    ) {
        let owner = self.user_wait_timing_owner(turn, call_id);
        (
            self.services.elicitations.register(),
            self.start_user_wait_timing(turn, owner.as_deref()),
        )
    }

    /// Emit an exec approval request event and await the user's decision.
    ///
    /// The request is keyed by `call_id` + `approval_id` so matching responses
    /// are delivered to the correct in-flight turn. If the pending approval is
    /// cleared before a response arrives, treat it as an abort so interrupted
    /// turns do not continue on a synthetic denial.
    ///
    /// Note that if `available_decisions` is `None`, then the other fields will
    /// be used to derive the available decisions via
    /// [ExecApprovalRequestEvent::default_available_decisions].
    #[allow(clippy::too_many_arguments)]
    #[expect(
        clippy::await_holding_invalid_type,
        reason = "active turn checks and turn state updates must remain atomic"
    )]
    pub async fn request_command_approval(
        &self,
        turn_context: &TurnContext,
        kind: ExecApprovalKind,
        model_context: ModelInvocationContext,
        call_id: String,
        timing_call_id: Option<&str>,
        approval_id: Option<String>,
        environment_id: Option<String>,
        command: Vec<String>,
        cwd: PathUri,
        reason: Option<String>,
        network_approval: Option<(NetworkApprovalContext, NetworkApprovalTurn)>,
        proposed_execpolicy_amendment: Option<ExecPolicyAmendment>,
        additional_permissions: Option<AdditionalPermissionProfile>,
        available_decisions: Option<Vec<ReviewDecision>>,
        plugin_attribution_override: Option<PluginCommandAttribution>,
    ) -> ReviewDecision {
        let (network_approval_context, turn_binding) = network_approval.unzip();
        let user_wait_owner = self.user_wait_timing_owner(turn_context, timing_call_id);
        let _elicitation = self.services.elicitations.register();
        //  command-level approvals use `call_id`.
        // `approval_id` identifies subcommand callbacks and stdin writes.
        let effective_approval_id = approval_id.clone().unwrap_or_else(|| call_id.clone());
        // Add the tx_approve callback to the map before sending the request.
        let (tx_approve, rx_approve) = oneshot::channel();
        let (prev_entry, _network_prompt) = {
            let mut active = self.active_turn.lock().await;
            // The proxy reviews with the active turn's settings. A sandbox retry
            // keeps its originating step's settings even after a background cell yields.
            if let Some(binding) = turn_binding
                && active
                    .as_ref()
                    .and_then(|turn| turn.task.as_ref())
                    .is_none_or(|task| {
                        binding == NetworkApprovalTurn::Originating
                            && task.turn_context.sub_id != turn_context.sub_id
                    })
            {
                return ReviewDecision::Abort;
            }
            match active.as_mut() {
                Some(at) => {
                    let mut ts = at.turn_state.lock().await;
                    let prev = ts.insert_pending_approval(
                        effective_approval_id.clone(),
                        tx_approve,
                        kind,
                        turn_context.sub_id.clone(),
                        self.services.authority_marker(),
                    );
                    let prompt = network_approval_context.as_ref().map(|_| {
                        ts.track_network_prompt(
                            &self.tx_event,
                            &turn_context.sub_id,
                            &effective_approval_id,
                        )
                    });
                    (prev, prompt)
                }
                None => (None, None),
            }
        };
        if prev_entry.is_some() {
            warn!("Overwriting existing pending approval for call_id: {effective_approval_id}");
        }

        let parsed_cmd = parse_command(&command);
        let proposed_network_policy_amendments = network_approval_context.as_ref().map(|context| {
            vec![
                NetworkPolicyAmendment {
                    host: context.host.clone(),
                    action: NetworkPolicyRuleAction::Allow,
                },
                NetworkPolicyAmendment {
                    host: context.host.clone(),
                    action: NetworkPolicyRuleAction::Deny,
                },
            ]
        });
        let available_decisions = available_decisions.unwrap_or_else(|| {
            ExecApprovalRequestEvent::default_available_decisions(
                network_approval_context.as_ref(),
                proposed_execpolicy_amendment.as_ref(),
                proposed_network_policy_amendments.as_deref(),
                additional_permissions.as_ref(),
            )
        });
        let plugin_attribution = plugin_attribution_override.or_else(|| {
            cwd.to_abs_path()
                .ok()
                .and_then(|cwd| turn_context.plugin_attribution_for_command(&command, &cwd))
        });
        let (plugin_id, script_path) = plugin_attribution
            .as_ref()
            .map(PluginCommandAttribution::serialized_fields)
            .unzip();
        let withdrawn = ExecApprovalRequestWithdrawnEvent {
            call_id: call_id.clone(),
            approval_id: approval_id.clone(),
            turn_id: turn_context.sub_id.clone(),
        };
        let event = EventMsg::ExecApprovalRequest(ExecApprovalRequestEvent {
            model_context: Some(model_context),
            kind,
            call_id,
            plugin_id,
            script_path,
            approval_id,
            turn_id: turn_context.sub_id.clone(),
            environment_id,
            started_at_ms: now_unix_timestamp_ms(),
            command,
            cwd: cwd.into(),
            reason,
            network_approval_context,
            proposed_execpolicy_amendment,
            proposed_network_policy_amendments,
            additional_permissions,
            available_decisions: Some(available_decisions),
            parsed_cmd,
        });
        let _user_wait = self.start_user_wait_timing(turn_context, user_wait_owner.as_deref());
        // #722 (D279): a caller that stops waiting (a remote network request
        // timed out or its process ended) drops this future, and the action is
        // already refused. Tell clients to close the prompt.
        let withdraw_on_drop = WithdrawApprovalOnDrop {
            tx_event: self.tx_event.clone(),
            event: Some(withdrawn),
        };
        self.send_event(turn_context, event).await;
        let decision = rx_approve.await.unwrap_or(ReviewDecision::Abort);
        withdraw_on_drop.disarm();
        decision
    }

    #[expect(
        clippy::await_holding_invalid_type,
        reason = "active turn checks and turn state updates must remain atomic"
    )]
    pub async fn request_patch_approval(
        &self,
        turn_context: &TurnContext,
        call_id: String,
        changes: HashMap<PathBuf, FileChange>,
        reason: Option<String>,
        grant_root: Option<PathBuf>,
    ) -> ReviewDecision {
        let user_wait_owner = self.user_wait_timing_owner(turn_context, Some(&call_id));
        let _elicitation = self.services.elicitations.register();
        // Add the tx_approve callback to the map before sending the request.
        let (tx_approve, rx_approve) = oneshot::channel();
        let approval_id = call_id.clone();
        let prev_entry = {
            let mut active = self.active_turn.lock().await;
            match active.as_mut() {
                Some(at) => {
                    let mut ts = at.turn_state.lock().await;
                    ts.insert_pending_approval(
                        approval_id.clone(),
                        tx_approve,
                        ExecApprovalKind::Command,
                        turn_context.sub_id.clone(),
                        self.services.authority_marker(),
                    )
                }
                None => None,
            }
        };
        if prev_entry.is_some() {
            warn!("Overwriting existing pending approval for call_id: {approval_id}");
        }

        let event = EventMsg::ApplyPatchApprovalRequest(ApplyPatchApprovalRequestEvent {
            call_id,
            turn_id: turn_context.sub_id.clone(),
            started_at_ms: now_unix_timestamp_ms(),
            changes,
            reason,
            grant_root,
        });
        let _user_wait = self.start_user_wait_timing(turn_context, user_wait_owner.as_deref());
        self.send_event(turn_context, event).await;
        rx_approve.await.unwrap_or(ReviewDecision::Abort)
    }

    pub(crate) async fn request_permissions_for_environment(
        self: &Arc<Self>,
        step_context: &Arc<StepContext>,
        call_id: String,
        request: PreparedPermissionRequest,
        cancellation_token: CancellationToken,
    ) -> Option<RequestPermissionsResponse> {
        self.request_permissions_explained(step_context, call_id, request, cancellation_token)
            .await
            .unwrap_or_else(|_refusal| {
                Some(RequestPermissionsResponse {
                    permissions: RequestPermissionProfile::default(),
                    scope: PermissionGrantScope::Turn,
                    strict_auto_review: false,
                })
            })
    }

    /// [`Self::request_permissions_for_environment`], or `Err` with why when
    /// the request was refused before anyone was asked (#584).
    #[expect(
        clippy::await_holding_invalid_type,
        reason = "active turn checks and turn state updates must remain atomic"
    )]
    pub(crate) async fn request_permissions_explained(
        self: &Arc<Self>,
        step_context: &Arc<StepContext>,
        call_id: String,
        mut request: PreparedPermissionRequest,
        cancellation_token: CancellationToken,
    ) -> Result<Option<RequestPermissionsResponse>, String> {
        let nothing_granted = || RequestPermissionsResponse {
            permissions: RequestPermissionProfile::default(),
            scope: PermissionGrantScope::Turn,
            strict_auto_review: false,
        };
        let turn_context = &step_context.turn;
        let user_wait_owner = self.user_wait_timing_owner(turn_context, Some(&call_id));
        let approval_policy = step_context.settings.approval_policy();
        let Some(environment) = step_context
            .environments
            .turn_environments()
            .find(|candidate| candidate.selection.environment_id == request.environment_id)
            .cloned()
        else {
            return Ok(Some(nothing_granted()));
        };
        let cwd = environment.cwd();
        match approval_policy {
            AskForApproval::Never => return Ok(Some(nothing_granted())),
            AskForApproval::Granular(granular_config)
                if !granular_config.allows_request_permissions() =>
            {
                return Ok(Some(nothing_granted()));
            }
            AskForApproval::OnRequest
            | AskForApproval::UnlessTrusted
            | AskForApproval::Granular(_) => {}
        }

        // The protected-path checks (#584, #245) need a path on this host.
        // TODO(anp): Migrate request_permissions to support paths from foreign environments.
        let Ok(native_environment_cwd) = cwd.to_abs_path() else {
            warn!(
                cwd = %cwd,
                "request_permissions requires a cwd native to the Codex host"
            );
            return Ok(Some(nothing_granted()));
        };
        request.permissions = tokio::select! {
            biased;
            _ = cancellation_token.cancelled() => return Ok(None),
            kept = self.grantable_request(
                turn_context.as_ref(),
                std::mem::take(&mut request.permissions),
                &native_environment_cwd,
            ) => kept?,
        };
        let action = request.approval_action(&call_id, &turn_context.sub_id);
        let requested_permissions = request.permissions.clone();
        let sandbox_context = environment.sandbox_context(/*additional_permissions*/ None);
        let context = sandbox_context.policy_context();
        {
            // #945: the policy the request is reviewed under.
            let shown_under = self.services.authority_marker();
            let approval_context = ApprovalContext {
                exec_requirement: None,
                review_context: crate::guardian::GuardianReviewContext::from(step_context),
                cancellation_token: Some(cancellation_token.clone()),
                call_id: call_id.clone(),
                tool_name: ToolName::plain("request_permissions"),
                strict_auto_review: false,
                approval_reason: None,
                retry_reason: None,
                network_approval_context: None,
            };
            let decision = tokio::select! {
                biased;
                _ = cancellation_token.cancelled() => return Ok(None),
                decision = self.request_guardian_approval(
                    action,
                    &approval_context,
                ) => decision,
            };
            if let Some(decision) = decision {
                let (permissions, scope) = match decision {
                    ReviewDecision::Approved
                    | ReviewDecision::ApprovedExecpolicyAmendment { .. }
                    | ReviewDecision::NetworkPolicyAmendment {
                        network_policy_amendment:
                            NetworkPolicyAmendment {
                                action: NetworkPolicyRuleAction::Allow,
                                ..
                            },
                    } => (requested_permissions.clone(), PermissionGrantScope::Turn),
                    ReviewDecision::ApprovedForSession => {
                        (requested_permissions.clone(), PermissionGrantScope::Session)
                    }
                    ReviewDecision::ApprovedMcpPolicyAmendment
                    | ReviewDecision::NetworkPolicyAmendment {
                        network_policy_amendment:
                            NetworkPolicyAmendment {
                                action: NetworkPolicyRuleAction::Deny,
                                ..
                            },
                    }
                    | ReviewDecision::Abort
                    | ReviewDecision::Denied { .. }
                    | ReviewDecision::TimedOut => (
                        RequestPermissionProfile::default(),
                        PermissionGrantScope::Turn,
                    ),
                };
                let response = RequestPermissionsResponse {
                    permissions,
                    scope,
                    strict_auto_review: false,
                };
                let response = self.unless_stale_grant(response, shown_under);
                let response = Self::normalize_request_permissions_response(
                    requested_permissions,
                    response,
                    &context,
                );
                let response = self
                    .without_protected_grant(
                        response,
                        &native_environment_cwd,
                        Some(turn_context.as_ref()),
                    )
                    .await;
                let recorded = self.record_granted_request_permissions_for_turn(
                    &response,
                    &environment.selection.environment_id,
                    turn_context,
                    shown_under,
                );
                return Ok(Some(Self::as_recorded(recorded, response)));
            }
        }

        let _elicitation = self.services.elicitations.register();
        let (tx_response, rx_response) = oneshot::channel();
        let prev_entry = {
            let mut active = self.active_turn.lock().await;
            match active.as_mut() {
                Some(at) => {
                    let mut ts = at.turn_state.lock().await;
                    ts.insert_pending_request_permissions(
                        call_id.clone(),
                        PendingRequestPermissions {
                            tx_response,
                            requested_permissions: requested_permissions.clone(),
                            environment: environment.clone(),
                            turn_context: Arc::clone(turn_context),
                            shown_under: self.services.authority_marker(),
                        },
                    )
                }
                None => None,
            }
        };
        if prev_entry.is_some() {
            warn!("Overwriting existing pending request_permissions for call_id: {call_id}");
        }

        let event = EventMsg::RequestPermissions(RequestPermissionsEvent {
            call_id: call_id.clone(),
            turn_id: turn_context.sub_id.clone(),
            environment_id: Some(environment.selection.environment_id.clone()),
            started_at_ms: now_unix_timestamp_ms(),
            reason: request.reason,
            permissions: requested_permissions,
            cwd: Some(cwd.clone().into()),
        });
        let _user_wait = self.start_user_wait_timing(turn_context, user_wait_owner.as_deref());
        self.send_event(turn_context.as_ref(), event).await;
        Ok(tokio::select! {
            biased;
            _ = cancellation_token.cancelled() => {
                drop(_user_wait);
                let mut active = self.active_turn.lock().await;
                if let Some(at) = active.as_mut() {
                    let mut ts = at.turn_state.lock().await;
                    let _ = ts.remove_pending_request_permissions(&call_id);
                }
                None
            }
            response = rx_response => response.ok(),
        })
    }

    #[expect(
        clippy::await_holding_invalid_type,
        reason = "active turn checks and turn state updates must remain atomic"
    )]
    pub(crate) async fn request_user_input(
        &self,
        turn_context: &TurnContext,
        call_id: String,
        args: RequestUserInputArgs,
    ) -> Option<AcceptedUserInputResponse> {
        let user_wait_owner = self.user_wait_timing_owner(turn_context, Some(&call_id));
        let _elicitation = self.services.elicitations.register();
        let sub_id = turn_context.sub_id.clone();
        let (tx_response, rx_response) = oneshot::channel();
        let event_id = sub_id.clone();
        let prev_entry = {
            let mut active = self.active_turn.lock().await;
            match active.as_mut() {
                Some(at) => {
                    let mut ts = at.turn_state.lock().await;
                    ts.insert_pending_user_input(sub_id, tx_response)
                }
                None => None,
            }
        };
        if prev_entry.is_some() {
            warn!("Overwriting existing pending user input for sub_id: {event_id}");
        }

        let event = EventMsg::RequestUserInput(RequestUserInputEvent {
            call_id,
            turn_id: turn_context.sub_id.clone(),
            questions: args.questions,
            is_blocking: args.is_blocking,
            auto_resolution_ms: args.auto_resolution_ms,
        });
        turn_context
            .turn_metadata_state
            .mark_user_input_requested_during_turn();
        let _user_wait = self.start_user_wait_timing(turn_context, user_wait_owner.as_deref());
        self.send_event(turn_context, event).await;
        rx_response.await.ok()
    }

    #[expect(
        clippy::await_holding_invalid_type,
        reason = "active turn checks and turn state updates must remain atomic"
    )]
    pub async fn notify_user_input_response(
        &self,
        sub_id: &str,
        response: RequestUserInputResponse,
    ) {
        let entry = {
            let mut active = self.active_turn.lock().await;
            match active.as_mut() {
                Some(at) => {
                    let sender = at.turn_state.lock().await.remove_pending_user_input(sub_id);
                    match sender {
                        Some(sender) => Some((sender, self.reserve_user_input_order().await)),
                        None => None,
                    }
                }
                None => None,
            }
        };
        match entry {
            Some((tx_response, acceptance_order)) => {
                tx_response
                    .send(AcceptedUserInputResponse {
                        response,
                        acceptance_order,
                    })
                    .ok();
            }
            None => {
                warn!("No pending user input found for sub_id: {sub_id}");
            }
        }
    }

    #[expect(
        clippy::await_holding_invalid_type,
        reason = "active turn checks and turn state updates must remain atomic"
    )]
    pub async fn notify_request_permissions_response(
        &self,
        call_id: &str,
        response: RequestPermissionsResponse,
    ) {
        let entry = {
            let mut active = self.active_turn.lock().await;
            match active.as_mut() {
                Some(at) => {
                    let mut ts = at.turn_state.lock().await;
                    ts.remove_pending_request_permissions(call_id)
                }
                None => None,
            }
        };
        match entry {
            Some(entry) => {
                let sandbox_context = entry
                    .environment
                    .sandbox_context(/*additional_permissions*/ None);
                let response = Self::normalize_request_permissions_response(
                    entry.requested_permissions,
                    response,
                    &sandbox_context.policy_context(),
                );
                // TODO(anp): Migrate request_permissions to support paths from foreign environments.
                let response = match entry.environment.cwd().to_abs_path() {
                    Ok(native_environment_cwd) => {
                        self.without_protected_grant(
                            response,
                            &native_environment_cwd,
                            Some(entry.turn_context.as_ref()),
                        )
                        .await
                    }
                    Err(err) => {
                        warn!(
                            cwd = %entry.environment.cwd(),
                            %err,
                            "request_permissions requires a cwd native to the Codex host"
                        );
                        RequestPermissionsResponse {
                            permissions: RequestPermissionProfile::default(),
                            scope: PermissionGrantScope::Turn,
                            strict_auto_review: false,
                        }
                    }
                };
                let response = self.unless_stale_grant(response, entry.shown_under);
                let recorded = self.record_granted_request_permissions_for_turn(
                    &response,
                    &entry.environment.selection.environment_id,
                    &entry.turn_context,
                    entry.shown_under,
                );
                entry
                    .tx_response
                    .send(Self::as_recorded(recorded, response))
                    .ok();
            }
            None => {
                warn!("No pending request_permissions found for call_id: {call_id}");
            }
        }
    }

    /// #945: a grant answered after a security transition was for a stale
    /// request: nothing is granted (the model may ask again under the new
    /// level).
    fn unless_stale_grant(
        &self,
        response: RequestPermissionsResponse,
        shown_under: Option<(u64, u64)>,
    ) -> RequestPermissionsResponse {
        if response.permissions.is_empty() || !self.services.transition_since(shown_under) {
            return response;
        }
        warn!("request_permissions answered after a security transition; nothing granted");
        RequestPermissionsResponse {
            permissions: RequestPermissionProfile::default(),
            scope: PermissionGrantScope::Turn,
            strict_auto_review: false,
        }
    }

    fn normalize_request_permissions_response(
        requested_permissions: RequestPermissionProfile,
        response: RequestPermissionsResponse,
        context: &FileSystemSandboxPolicyContext<'_>,
    ) -> RequestPermissionsResponse {
        if response.strict_auto_review && matches!(response.scope, PermissionGrantScope::Session) {
            return RequestPermissionsResponse {
                permissions: RequestPermissionProfile::default(),
                scope: PermissionGrantScope::Turn,
                strict_auto_review: false,
            };
        }

        if response.permissions.is_empty() {
            return response;
        }

        RequestPermissionsResponse {
            permissions: intersect_permission_profiles_with_context(
                requested_permissions.into(),
                response.permissions.into(),
                context,
            )
            .into(),
            scope: response.scope,
            strict_auto_review: response.strict_auto_review,
        }
    }

    /// #584: under Aggressive no answer grants a protected path or a
    /// read-only one (`ReadOnlyScope::for_grant`; #520: the project's
    /// `.codex` and `.agents` included), so those entries are left out before
    /// anyone is asked, and the person is told why. `Err` with why when
    /// nothing would be left to grant: then nobody is asked.
    async fn grantable_request(
        &self,
        turn_context: &TurnContext,
        requested: RequestPermissionProfile,
        cwd: &AbsolutePathBuf,
    ) -> Result<RequestPermissionProfile, String> {
        if requested.is_empty() {
            return Ok(requested);
        }
        let Some(level @ codex_security_policy::SecurityLevel::Aggressive) =
            crate::security::protected_additional::protected_level(self)
        else {
            return Ok(requested);
        };
        let (codex_home, sqlite_home, configs) = self.grant_check_configs(Some(cwd)).await;
        let (kept, refusal) = crate::security::protected_additional::strip_grant_off_thread(
            level,
            codex_home,
            sqlite_home,
            configs,
            requested.clone().into(),
        )
        .await;
        let Some(refusal) = refusal else {
            return Ok(requested);
        };
        let unchecked = refusal == crate::security::protected_additional::unchecked_refusal(level);
        crate::security::inspection::record_refusal(
            self.thread_id(),
            "a permission request naming a protected path",
            match kept {
                Some(_) => "those paths were left out before anyone was asked",
                None if unchecked => "refused before anyone was asked; it could not be checked",
                None => "refused before anyone was asked; never granted under Aggressive",
            },
        );
        crate::security::protected_reads::notify(self, turn_context, refusal.clone()).await;
        kept.map(Into::into).ok_or(refusal)
    }

    /// Issue #245: once the protected-path rules apply, a permission grant
    /// never names a protected path. Such entries are dropped from the answer
    /// the model gets and from what is recorded, and the person is told why.
    async fn without_protected_grant(
        &self,
        response: RequestPermissionsResponse,
        cwd: &AbsolutePathBuf,
        turn_context: Option<&TurnContext>,
    ) -> RequestPermissionsResponse {
        if response.permissions.is_empty() {
            return response;
        }
        let Some(level) = crate::security::protected_additional::protected_level(self) else {
            return response;
        };
        let (codex_home, sqlite_home, configs) = self.grant_check_configs(Some(cwd)).await;
        let (kept, refusal) = crate::security::protected_additional::strip_grant_off_thread(
            level,
            codex_home,
            sqlite_home,
            configs,
            response.permissions.clone().into(),
        )
        .await;
        let Some(refusal) = refusal else {
            return response;
        };
        // #622: listed under the inspector's recent denials too.
        crate::security::inspection::record_refusal(
            self.thread_id(),
            "a permission grant naming a protected path",
            "those paths were left out of the grant",
        );
        match turn_context {
            Some(turn_context) => {
                crate::security::protected_reads::notify(self, turn_context, refusal).await;
            }
            None => {
                if let Some((turn_context, _)) =
                    self.active_turn_context_and_cancellation_token().await
                {
                    crate::security::protected_reads::notify(self, &turn_context, refusal).await;
                }
            }
        }
        RequestPermissionsResponse {
            permissions: kept.map(Into::into).unwrap_or_default(),
            ..response
        }
    }

    /// Records a grant answered to a request shown under `granted_under`;
    /// whether it was recorded (nothing to record counts as recorded).
    /// D51 (#1001), D520 (a): it holds only while no security transition is
    /// committed (see [`crate::state::GrantedPermissions`]).
    pub(crate) fn record_granted_request_permissions_for_turn(
        &self,
        response: &RequestPermissionsResponse,
        environment_id: &str,
        turn_context: &TurnContext,
        granted_under: Option<(u64, u64)>,
    ) -> bool {
        if response.permissions.is_empty() {
            return true;
        }
        let permissions: AdditionalPermissionProfile = response.permissions.clone().into();
        match response.scope {
            PermissionGrantScope::Turn => turn_context.record_granted_permissions(
                environment_id,
                permissions,
                response.strict_auto_review,
                granted_under,
            ),
            PermissionGrantScope::Session => {
                self.services
                    .record_granted_permissions(environment_id, permissions, granted_under)
            }
        }
    }

    /// The answer the model gets: nothing granted when the grant was not
    /// recorded (a security transition committed in between, D51), so the
    /// model is never told it holds a grant it does not have.
    fn as_recorded(
        recorded: bool,
        response: RequestPermissionsResponse,
    ) -> RequestPermissionsResponse {
        if recorded {
            return response;
        }
        warn!("request_permissions grant not recorded after a security transition");
        RequestPermissionsResponse {
            permissions: RequestPermissionProfile::default(),
            scope: PermissionGrantScope::Turn,
            strict_auto_review: false,
        }
    }

    #[expect(
        clippy::await_holding_invalid_type,
        reason = "active turn reads must stay consistent with the matching turn state"
    )]
    pub(crate) async fn active_turn_context_and_strict_auto_review(
        &self,
    ) -> Option<(
        Arc<TurnContext>,
        Arc<ResolvedStepSettings>,
        TurnEnvironmentSnapshot,
        bool,
    )> {
        let active = self.active_turn.lock().await;
        let active = active.as_ref()?;
        let task = active.task.as_ref()?;
        let turn_context = Arc::clone(&task.turn_context);
        let settings = turn_context.next_step_settings.load_full();
        let strict_auto_review = turn_context.strict_auto_review_enabled();
        let environments = self.services.turn_environments.snapshot_now();
        Some((turn_context, settings, environments, strict_auto_review))
    }

    async fn active_task_turn_context(&self) -> Option<Arc<TurnContext>> {
        let active = self.active_turn.lock().await;
        let task = active.as_ref()?.task.as_ref()?;
        Some(Arc::clone(&task.turn_context))
    }

    /// The running turn's own grants for `environment_id`, as later commands
    /// and file tools may use them (#245; D51: none after a security transition).
    pub(crate) async fn granted_turn_permissions(
        &self,
        environment_id: &str,
    ) -> Option<AdditionalPermissionProfile> {
        let granted = self
            .active_task_turn_context()
            .await?
            .granted_turn_permissions(environment_id);
        self.usable_grant(granted).await
    }

    /// Issue #245: a stored grant as later commands and file tools may use
    /// it: without protected paths while the protected-path rules apply. A
    /// grant recorded before they applied (the level raised mid-session)
    /// cannot reopen one either.
    async fn usable_grant(
        &self,
        granted: Option<AdditionalPermissionProfile>,
    ) -> Option<AdditionalPermissionProfile> {
        let granted = granted?;
        let Some(level) = crate::security::protected_additional::protected_level(self) else {
            return Some(granted);
        };
        let (codex_home, sqlite_home, configs) = self.grant_check_configs(/*cwd*/ None).await;
        crate::security::protected_additional::strip_grant_off_thread(
            level,
            codex_home,
            sqlite_home,
            configs,
            granted,
        )
        .await
        .0
    }

    /// Issue #524: the Corbanu home and its database folder (#611), and what
    /// a grant is checked against:
    /// `cwd` (or the session's original folder) with the original profile,
    /// and the session's current folder and profile, which may have changed
    /// since, each with the current workspace roots.
    async fn grant_check_configs(
        &self,
        cwd: Option<&AbsolutePathBuf>,
    ) -> (
        std::path::PathBuf,
        std::path::PathBuf,
        Vec<crate::security::protected_additional::GrantCheckConfig>,
    ) {
        use crate::security::protected_additional::GrantCheckConfig;
        let state = self.state.lock().await;
        let current = &state.session_configuration;
        let config = &current.original_config_do_not_use;
        // Both the primary environment's roots (what the turn runs with) and the runtime
        // roots: a protected `.codex`/`.agents` under either is checked.
        let mut workspace_roots =
            crate::environment_selection::ThreadEnvironments::primary_workspace_roots_for(
                &current.environments,
            );
        for root in &current.runtime_workspace_roots {
            if !workspace_roots.contains(root) {
                workspace_roots.push(root.clone());
            }
        }
        let mut configs = vec![GrantCheckConfig {
            cwd: cwd.unwrap_or(&config.cwd).clone(),
            workspace_roots: workspace_roots.clone(),
            profile: config.permissions.permission_profile().clone(),
        }];
        let now = GrantCheckConfig {
            cwd: current.cwd().clone(),
            workspace_roots,
            profile: current.permission_profile(),
        };
        if !configs.contains(&now) {
            configs.push(now);
        }
        (
            config.codex_home.to_path_buf(),
            config.sqlite.home().to_path_buf(),
            configs,
        )
    }

    /// Whether strict auto-review is on for the running turn under the policy
    /// in force (D51: not after a security transition).
    pub(crate) async fn strict_auto_review_enabled_for_turn(&self) -> bool {
        self.active_task_turn_context()
            .await
            .is_some_and(|turn_context| turn_context.strict_auto_review_enabled())
    }

    /// The session's grants for `environment_id`, as later commands and file
    /// tools may use them (#245; D51: none after a security transition).
    pub(crate) async fn granted_session_permissions(
        &self,
        environment_id: &str,
    ) -> Option<AdditionalPermissionProfile> {
        let granted = self.services.session_grants.get(environment_id);
        self.usable_grant(granted).await
    }

    #[expect(
        clippy::await_holding_invalid_type,
        reason = "active turn checks and turn state updates must remain atomic"
    )]
    pub async fn notify_dynamic_tool_response(&self, call_id: &str, response: DynamicToolResponse) {
        let entry = {
            let mut active = self.active_turn.lock().await;
            match active.as_mut() {
                Some(at) => {
                    let mut ts = at.turn_state.lock().await;
                    ts.remove_pending_dynamic_tool(call_id)
                }
                None => None,
            }
        };
        match entry {
            Some(tx_response) => {
                tx_response.send(response).ok();
            }
            None => {
                warn!("No pending dynamic tool call found for call_id: {call_id}");
            }
        }
    }

    #[expect(
        clippy::await_holding_invalid_type,
        reason = "active turn checks and turn state updates must remain atomic"
    )]
    async fn take_pending_approval(
        &self,
        approval_id: &str,
        expected_turn_id: Option<&str>,
    ) -> Option<(PendingApproval, Option<String>)> {
        let active = self.active_turn.lock().await;
        let turn = active.as_ref()?;
        // A live Cancel interrupts the turn servicing the prompt, which can differ from its origin.
        let turn_id = turn.task.as_ref().map(|task| &task.turn_context.sub_id);
        let pending = turn
            .turn_state
            .lock()
            .await
            .remove_pending_approval(approval_id, expected_turn_id)?;
        (!pending.sender.is_closed()).then(|| (pending, turn_id.cloned()))
    }

    pub async fn notify_approval(&self, approval_id: &str, decision: ReviewDecision) {
        if let Some((pending, _)) = self
            .take_pending_approval(approval_id, /*expected_turn_id*/ None)
            .await
        {
            let decision = pending.validate(decision);
            let _ = pending.sender.send(decision);
        } else {
            warn!("No pending approval found for call_id: {approval_id}");
        }
    }

    pub(crate) fn response_item_create_time() -> serde_json::Number {
        let create_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        serde_json::Number::from_f64(create_time.as_secs_f64())
            .unwrap_or_else(|| serde_json::Number::from(create_time.as_secs()))
    }

    /// Adds the turn ID and creation time to newly created conversation items.
    pub(crate) fn stamp_response_item_for_history(item: &mut ResponseItem, turn_id: &str) {
        item.set_turn_id_if_missing(turn_id);
        item.set_create_time_if_missing(Self::response_item_create_time());
    }

    /// Prepares media using the originating model and preserves existing item identity.
    pub(crate) async fn prepare_conversation_items_for_history<'a>(
        &self,
        turn_context: &TurnContext,
        model_info: &ModelInfo,
        items: &'a [ResponseItem],
    ) -> (Cow<'a, [ResponseItem]>, Vec<ImagePreparationMetadata>) {
        let mut items = items.to_vec();
        let image_preparation_mode =
            if unified_image_budget_enabled(&turn_context.config.features, model_info) {
                ImagePreparationMode::UnifiedBudget
            } else {
                ImagePreparationMode::DetailBased
            };
        let image_resize_notice_mode = if turn_context
            .config
            .features
            .enabled(Feature::ImageResizeNotice)
        {
            ImageResizeNoticeMode::Enabled
        } else {
            ImageResizeNoticeMode::Disabled
        };
        // Keep nested image-upload futures out of every caller's future frame.
        let image_preparations = Box::pin(prepare_image_response_items(
            &self.thread_id.to_string(),
            turn_context.config.ephemeral,
            &mut items,
            image_preparation_mode,
            image_resize_notice_mode,
            self.services.image_store.as_ref(),
        ))
        .await;
        prepare_audio_response_items(&mut items);
        // Most response items get their passthrough turn ID at the durable history boundary.
        for item in &mut items {
            Self::stamp_response_item_for_history(item, &turn_context.sub_id);
        }
        let items = Cow::Owned(items);
        (
            Self::assign_missing_response_item_ids(items),
            image_preparations,
        )
    }

    fn assign_missing_response_item_ids(items: Cow<'_, [ResponseItem]>) -> Cow<'_, [ResponseItem]> {
        if items
            .iter()
            .all(|item| item.id().is_some_and(|id| !id.is_empty()))
        {
            return items;
        }
        let mut items = items;
        for item in items.to_mut() {
            Self::assign_missing_response_item_id(item);
        }
        items
    }

    pub(crate) fn assign_missing_response_item_id(item: &mut ResponseItem) {
        if item.id().is_some_and(|id| !id.is_empty()) {
            return;
        }
        let Some(prefix) = item.id_prefix() else {
            return;
        };
        item.set_id(Some(ResponseItemId::new(prefix)));
    }

    fn assign_missing_rollout_response_item_ids(items: &mut [RolloutItem]) {
        for item in items {
            if let RolloutItem::ResponseItem(response_item) = item {
                Self::assign_missing_response_item_id(&mut response_item.item);
            }
        }
    }

    pub(crate) fn response_item_from_user_input(&self, input: Vec<UserInput>) -> ResponseItem {
        self.response_item_from_user_input_with_image_positions(input, &mut HashMap::new())
    }

    fn response_item_from_user_input_with_image_positions(
        &self,
        input: Vec<UserInput>,
        user_image_content_indices: &mut HashMap<usize, usize>,
    ) -> ResponseItem {
        let mut item = ResponseItem::from(ResponseInputItem::from_user_input(
            input,
            LocalImagePreparation::Defer,
            user_image_content_indices,
        ));
        if let ResponseItem::Message {
            content,
            internal_chat_message_metadata_passthrough,
            ..
        } = &mut item
        {
            let content_item_kinds = content
                .iter()
                .map(|content| {
                    ContentItemKind(
                        match content {
                            ContentItem::InputText { .. } | ContentItem::OutputText { .. } => {
                                "user.text"
                            }
                            ContentItem::InputImage { .. } => "user.image",
                            ContentItem::InputAudio { .. } => "user.audio",
                        }
                        .to_string(),
                    )
                })
                .collect();
            *internal_chat_message_metadata_passthrough =
                Some(InternalChatMessageMetadataPassthrough {
                    content_item_kinds: Some(content_item_kinds),
                    content_item_metadata: Some(vec![ContentItemMetadata::user(); content.len()]),
                    ..Default::default()
                });
        }
        item
    }

    /// Appends to history, persists the prepared items, then notifies raw-item observers.
    /// Execution callers supply their captured model; standalone callers need no tool runtime.
    #[tracing::instrument(level = "trace", skip_all, fields(item_count = items.len()))]
    pub(crate) async fn record_conversation_items(
        &self,
        turn_context: &TurnContext,
        model_info: &ModelInfo,
        items: &[ResponseItem],
    ) {
<<<<<<< HEAD
        self.record_conversation_items_from(turn_context, items, /*origin*/ None)
            .await;
    }

    /// Record items whose messages have a host-known origin. Unattributed
    /// messages (`None`) reach a protected provider only as labelled data.
    pub(crate) async fn record_conversation_items_from(
        &self,
        turn_context: &TurnContext,
        items: &[ResponseItem],
        origin: Option<crate::security::ingress::MessageOrigin>,
    ) {
        let items = self.prepare_conversation_items_for_history(turn_context, items);
        // PF-28-S01: tool results and model output are gated before history,
        // rollout and clients see them.
        let gated = crate::security::disclosure_gate::gate_values(
            codex_secret_broker::output_gate::OutputSink::ToolResult,
            items.as_ref(),
        );
        let items = gated.as_deref().unwrap_or(items.as_ref());
        if let Some(origin) = origin {
            self.services
                .model_client()
                .register_message_origin(items, origin);
        }
        self.services.model_client().note_recorded_for_taint(items);
        self.services.model_client().observe_native_ingress(items);
||||||| base
        let items = self.prepare_conversation_items_for_history(turn_context, items);
        let items = items.as_ref();
=======
        let (items, image_preparations) = self
            .prepare_conversation_items_for_history(turn_context, model_info, items)
            .await;
        let items = items
            .into_owned()
            .into_iter()
            .map(ResponseItemEnvelope::new)
            .collect();
        self.record_prepared_conversation_items(
            turn_context,
            model_info,
            items,
            image_preparations,
        )
        .await;
    }

    async fn record_prepared_conversation_items(
        &self,
        turn_context: &TurnContext,
        model_info: &ModelInfo,
        mut items: Vec<ResponseItemEnvelope>,
        image_preparations: Vec<ImagePreparationMetadata>,
    ) {
        // Save the originating history budget for replay.
        // Preserve any existing tool-specific override.
        let policy: codex_utils_output_truncation::TruncationPolicy =
            model_info.truncation_policy.into();
        for envelope in &mut items {
            if matches!(
                envelope.item,
                ResponseItem::FunctionCallOutput { .. } | ResponseItem::CustomToolCallOutput { .. }
            ) {
                envelope
                    .metadata
                    .get_or_insert_default()
                    .history_truncation_token_limit
                    .get_or_insert_with(|| with_serialization_allowance(policy).token_budget());
            }
        }
        // Last-N-turn forks retain a suffix starting at a user turn boundary. Repeat the
        // cumulative checkpoint there so the suffix remains self-contained even when the fork
        // happens mid-turn; dirty checkpoints preserve new sources between boundaries.
        let force_mcp_checkpoint = items
            .iter()
            .any(|envelope| crate::context_manager::is_user_turn_boundary(&envelope.item));
        let mcp_revision = self
            .services
            .executed_tool_calls
            .mcp_attribution_checkpoint(force_mcp_checkpoint)
            .and_then(|(attribution, revision)| {
                let first_persisted = items
                    .iter()
                    .position(|envelope| should_persist_response_item(&envelope.item))?;
                for (index, envelope) in items.iter_mut().enumerate() {
                    // Rollout batches may be partially written. Checkpoint the first persisted
                    // item, and repeat the checkpoint at turn boundaries retained by forks.
                    if index == first_persisted
                        || crate::context_manager::is_user_turn_boundary(&envelope.item)
                    {
                        envelope.metadata.get_or_insert_default().mcp_attribution =
                            Some(attribution.clone());
                    }
                }
                Some(revision)
            });
        let response_items = items
            .iter()
            .map(|envelope| envelope.item.clone())
            .collect::<Vec<_>>();
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        {
            let mut state = self.state.lock().await;
            state
                .current_time_reminder
                .note_recorded_items(&response_items);
            let pending_orders = turn_context
                .extension_data
                .get::<retained_context::PendingAssistantMessageOrders>();
            for envelope in &mut items {
                if envelope
                    .metadata
                    .as_ref()
                    .is_some_and(|metadata| metadata.compaction_output)
                {
                    continue;
                }
                if matches!(&envelope.item, ResponseItem::Message { role, .. } if role == "assistant")
                    || matches!(&envelope.item, ResponseItem::FunctionCall { .. })
                    || crate::context::is_user_authorization_message(&envelope.item)
                {
                    let message_order = pending_orders.as_ref().and_then(|orders| {
                        orders
                            .0
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner)
                            .remove(envelope.item.id()?.as_str())
                    });
                    // Preserve input acceptance and source-message start order.
                    // Synthetic messages still receive their order here.
                    envelope
                        .metadata
                        .get_or_insert_default()
                        .user_input_order
                        .get_or_insert_with(|| {
                            message_order.unwrap_or_else(|| state.history.reserve_input_order())
                        });
                }
            }
            state
                .history
                .record_annotated_items(&mut items, model_info.truncation_policy.into());
        }
<<<<<<< HEAD
        let mut rollout_items: Vec<RolloutItem> = items
            .iter()
            .cloned()
            .map(RolloutItem::ResponseItem)
            .collect();
        // Origins follow the items they describe so a resume restores them.
        if let Some(record) = self.services.model_client().take_source_origin_record() {
            rollout_items.push(RolloutItem::SourceOrigin(record));
        }
        self.persist_rollout_items(&rollout_items).await;
        self.send_raw_response_items(turn_context, items).await;
||||||| base
        self.persist_rollout_response_items(items).await;
        self.send_raw_response_items(turn_context, items).await;
=======
        for image in image_preparations {
            self.services
                .analytics_events_client
                .track_image_preparation(ImagePreparationFact {
                    turn_id: turn_context.sub_id.clone(),
                    metadata: image,
                });
        }
        let settings_guard =
            if force_mcp_checkpoint && self.reference_context_item().await.is_none() {
                Some(thread_settings::acquire_persistence_lock(self).await)
            } else {
                None
            };
        let initial_settings = if settings_guard.is_some()
            && self.reference_context_item().await.is_none()
            && self.state.lock().await.take_initial_settings_pending()
        {
            Some(thread_settings::applied_event(self).await)
        } else {
            None
        };
        let rollout_items = initial_settings
            .into_iter()
            .map(RolloutItem::EventMsg)
            .chain(items.into_iter().map(RolloutItem::ResponseItem))
            .collect::<Vec<_>>();
        let persisted = self.persist_rollout_items(&rollout_items).await;
        drop(settings_guard);
        if persisted && let Some(revision) = mcp_revision {
            self.services
                .executed_tool_calls
                .mark_mcp_attribution_persisted(revision);
        }
        if turn_context.config.memories.disable_on_external_context
            && let Some(item) = response_items
                .iter()
                .find(|item| matches!(item, ResponseItem::FunctionCallOutput { call_id: None, .. }))
        {
            mark_thread_memory_mode_polluted_if_external_context(self, turn_context, item).await;
        }
        self.send_raw_response_items(turn_context, &response_items)
            .await;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
    }

    /// Persist origin registrations made outside `record_conversation_items_from`
    /// (compaction installs its replacement history directly).
    async fn persist_source_origins(&self) {
        if let Some(record) = self.services.model_client().take_source_origin_record() {
            self.persist_rollout_items(&[RolloutItem::SourceOrigin(record)])
                .await;
        }
    }

    pub(crate) async fn record_step_world_state_if_changed(
        &self,
        step_context: &step_context::StepContext,
    ) -> CodexResult<Arc<WorldState>> {
        let turn_context = step_context.turn.as_ref();
        // Render model-visible state from the same step used to build and run tools.
        let world_state = self
            .build_world_state_for_step(step_context, /*new_window*/ false)
            .await?;
        let (world_state_snapshot, fragments, world_state_item) = self
            .state
            .lock()
            .await
            .history
            .render_step_world_state(&world_state);
        let items = crate::context_manager::updates::merge_world_state_updates(fragments);
        if !items.is_empty() {
<<<<<<< HEAD
            self.record_conversation_items_from(
                turn_context,
                &items,
                Some(crate::security::ingress::MessageOrigin::Host),
            )
            .await;
||||||| base
            self.record_conversation_items(turn_context, &items).await;
=======
            self.record_conversation_items(turn_context, &step_context.settings.model_info, &items)
                .await;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        }

        self.state
            .lock()
            .await
            .history
            .set_world_state_baseline(world_state_snapshot);
        // Record the patch after the context it describes is present in model history.
        if let Some(world_state_item) = world_state_item {
            self.persist_rollout_items(&[RolloutItem::WorldState(world_state_item)])
                .await;
        }
        Ok(Arc::new(world_state))
    }

    /// Retains the step captured for execution.
    pub(crate) async fn set_last_known_step_context(&self, step_context: &Arc<StepContext>) {
        let turn_state = {
            let active_turn = self.active_turn.lock().await;
            active_turn.as_ref().and_then(|active_turn| {
                active_turn
                    .task
                    .as_ref()
                    .filter(|task| task.turn_context.sub_id == step_context.turn.sub_id)
                    .map(|_| Arc::clone(&active_turn.turn_state))
            })
        };
        if let Some(turn_state) = turn_state {
            turn_state.lock().await.last_known_step_context = Some(Arc::clone(step_context));
        }
    }

    /// Captures one request-scoped view of dynamic state and retains it for the active turn.
    ///
    /// This may refresh filesystem-derived state. Normal turns should call it only from
    /// `run_turn` and pass the result down; standalone request or history boundaries may capture
    /// their own step. Use speculative capture for a step that may not execute.
    pub(crate) async fn capture_step_context(
        self: &Arc<Self>,
        turn_context: Arc<TurnContext>,
        cancellation_token: &CancellationToken,
    ) -> CodexResult<Arc<StepContext>> {
        self.capture_step_context_with_required_mcp_servers(
            turn_context,
            cancellation_token,
            /*required_servers*/ &[],
            /*required_plugins*/ &HashSet::new(),
        )
        .await
    }

    /// Captures and retains a step with the required MCP startup dependencies.
    pub(crate) async fn capture_step_context_with_required_mcp_servers(
        self: &Arc<Self>,
        turn_context: Arc<TurnContext>,
        cancellation_token: &CancellationToken,
        required_servers: &[String],
        required_plugins: &HashSet<String>,
    ) -> CodexResult<Arc<StepContext>> {
        let step_context = self
            .capture_step_context_inner(
                turn_context,
                cancellation_token,
                required_servers,
                required_plugins,
            )
            .await?;
        self.set_last_known_step_context(&step_context).await;
        Ok(step_context)
    }

    /// Prepares a candidate step without replacing the active turn's retained context.
    /// The caller must retain it explicitly if it is selected for execution.
    pub(crate) async fn capture_speculative_step_context(
        self: &Arc<Self>,
        turn_context: Arc<TurnContext>,
        cancellation_token: &CancellationToken,
    ) -> CodexResult<Arc<StepContext>> {
        self.capture_step_context_inner(
            turn_context,
            cancellation_token,
            /*required_servers*/ &[],
            /*required_plugins*/ &HashSet::new(),
        )
        .await
    }

    #[expect(
        clippy::await_holding_invalid_type,
        reason = "step settings and both environment snapshots must be captured under active_turn then state"
    )]
    #[tracing::instrument(name = "step_context.capture", level = "info", skip_all)]
    async fn capture_step_context_inner(
        self: &Arc<Self>,
        turn_context: Arc<TurnContext>,
        cancellation_token: &CancellationToken,
        required_servers: &[String],
        required_plugins: &HashSet<String>,
    ) -> CodexResult<Arc<StepContext>> {
        // Read the step's model and record its environments together so an update cannot split them.
        // Match apply_turn_settings and activate_turn_environments: active_turn before state.
        // Wait for executor startup below, after releasing both locks.
        let (mut settings, environments, inherited_environments) = {
            let _active = self
                .active_turn
                .lock()
                .or_cancel(cancellation_token)
                .await?;
            let _state = self.state.lock().or_cancel(cancellation_token).await?;
            (
                turn_context.next_step_settings.load_full(),
                self.services.turn_environments.snapshot(),
                self.features
                    .enabled(Feature::SubagentThreadEnvironments)
                    .then(|| self.services.turn_environments.configured_snapshot_now()),
            )
        };
        if matches!(
            turn_context.session_source,
            SessionSource::SubAgent(SubAgentSource::ThreadSpawn { .. })
        ) {
            let root_service_tier = self.services.agent_control.service_tier();
            if settings.selected().service_tier != root_service_tier {
                let mut selected = settings.selected().clone();
                selected.service_tier = root_service_tier;
                let mut inherited_settings = ResolvedStepSettings::new(
                    Arc::new(selected),
                    Arc::clone(&settings.model_info),
                    &self.features,
                );
                inherited_settings.mcp_approvals_reviewer_override =
                    settings.mcp_approvals_reviewer_override;
                settings = Arc::new(inherited_settings);
            }
<<<<<<< HEAD
        } else if !turn_context
            .config
            .permissions
            .file_system_sandbox_policy()
            .has_full_disk_read_access()
        {
            let sandbox_contexts = environments
                .turn_environments()
                .map(|environment| {
                    (
                        environment.environment_id.clone(),
                        // PF-23-S01: post-taint read denials, as for file tools.
                        crate::security::protected_surface::protect_file_tool_context(
                            self,
                            &turn_context,
                            turn_context.file_system_sandbox_context(
                                /*additional_permissions*/ None,
                                environment,
                            ),
                        ),
                    )
                })
                .collect::<HashMap<_, _>>();
            extension_data.insert(sandbox_contexts);
||||||| base
        } else if !turn_context
            .config
            .permissions
            .file_system_sandbox_policy()
            .has_full_disk_read_access()
        {
            let sandbox_contexts = environments
                .turn_environments()
                .map(|environment| {
                    (
                        environment.environment_id.clone(),
                        turn_context.file_system_sandbox_context(
                            /*additional_permissions*/ None,
                            environment,
                        ),
                    )
                })
                .collect::<HashMap<_, _>>();
            extension_data.insert(sandbox_contexts);
=======
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        }
        let dynamic_tools = self.dynamic_tools().await;
        let token_budget = token_budget::resolve_token_budget(
            turn_context.configured_token_budget.as_ref(),
            turn_context.use_model_token_budget_defaults,
            settings.model_info.as_ref(),
        );
        let session_telemetry = settings.telemetry(&turn_context.session_telemetry);
        let environments = environments.or_cancel(cancellation_token).await?;
        let inherited_environments = inherited_environments.unwrap_or_else(|| environments.clone());
        // Keep both preparation futures off caller stacks while they are live together.
        let load_agents_md = Box::pin(async {
            let (loaded_agents_md, warnings) = self
                .services
                .agents_md_manager
                .refresh(&turn_context.config, &environments)
                .or_cancel(cancellation_token)
                .await?;
            self.emit_instruction_warnings(warnings).await;
            loaded_agents_md
        });
        let prepare_tools = Box::pin(async {
            let selected_capability_roots = self
                .resolve_selected_capability_roots_for_step(&environments)
                .await;
            let ready_selected_capability_roots =
                Self::ready_selected_capability_roots(&selected_capability_roots);
            let (executor_capability_discovery_v2, inventory_changed) = self
                .executor_capability_discovery_for_step_v2(&turn_context.config, &environments)
                .await;
            let v2_snapshots = executor_capability_discovery_v2.as_ref();
            let (executor_capability_discovery, discovery_recovered) = if v2_snapshots.is_some() {
                (None, false)
            } else {
                self.executor_capability_discovery_for_step(
                    &turn_context.config,
                    &selected_capability_roots,
                    &environments,
                    executor_capability_discovery_v2.as_deref(),
                )
                .await
            };
            if inventory_changed || discovery_recovered {
                self.mark_mcp_runtime_dirty();
            }
            let extension_data =
                codex_extension_api::ExtensionData::new(turn_context.sub_id.clone());
            extension_data.insert_arc(Arc::clone(&settings.model_info.model));
            // Standalone and hosted search must use the same turn-scoped permission mode.
            extension_data.insert(turn_context.config.web_search_mode.value());
            if let Some(messages) = settings
                .model_info
                .model_messages
                .as_ref()
                .and_then(|messages| messages.tools.as_ref())
                .and_then(|tools| tools.multi_agent.as_ref())
            {
                extension_data.insert(messages.clone());
            }
            let model_messages = ResolvedModelMessages::from_model(&settings.model_info);
            if let Some(messages) = model_messages.history() {
                extension_data.insert(messages.clone());
            }
            if let Some(messages) = model_messages.notes() {
                extension_data.insert(messages.clone());
            }
            let (mcp, prepared_recommendations) = tokio::join!(
                // MCP refresh can be large; keep it off the sampling request's stack.
                Box::pin(self.mcp_runtime_for_step(
                    turn_context.as_ref(),
                    &selected_capability_roots,
                    required_servers,
                    required_plugins,
                )),
                turn::prepare_tool_recommendations(self.as_ref(), turn_context.as_ref()),
            );
            extension_data.insert(selected_capability_roots);
            // A step keeps the plugins from the environments it captured, even if shared MCP
            // moves on to another environment. Its skill tools and the model use this same copy.
            // Policy follows this step's MCP binding, which may have refreshed mid-turn.
            let selected_plugins = self
                .services
                .mcp_manager
                .selected_plugins_for_step(
                    codex_extension_api::McpServerContributionContext::for_step(
                        turn_context.config.as_ref(),
                        &self.services.mcp_thread_init,
                        &self.services.thread_extension_data,
                        &turn_context.originator,
                        &ready_selected_capability_roots,
                        executor_capability_discovery.as_deref(),
                        executor_capability_discovery_v2.as_deref(),
                    )
                    .with_session_source(&turn_context.session_source)
                    .with_selected_environments(&environments.all_selections()),
                    &mcp.config().plugins,
                    &turn_context.disabled_plugin_ids,
                )
                .await;
            for contributor in self.services.extensions.tool_contributors() {
                contributor
                    .prepare_for_step(codex_extension_api::StepPreparationInput {
                        turn_id: &turn_context.sub_id,
                        ready_selected_capability_roots: &ready_selected_capability_roots,
                        executor_capability_discovery: executor_capability_discovery.as_deref(),
                        executor_capability_discovery_v2: executor_capability_discovery_v2
                            .as_deref(),
                        selected_plugins: &selected_plugins,
                        session_store: &self.services.session_extension_data,
                        thread_store: &self.services.thread_extension_data,
                        turn_store: turn_context.extension_data.as_ref(),
                        step_store: &extension_data,
                    })
                    .or_cancel(cancellation_token)
                    .await?;
            }
            let executor_hook_sources = crate::hook_runtime::prepare_executor_hook_sources(
                &mcp,
                &selected_plugins,
                executor_capability_discovery.as_deref(),
                executor_capability_discovery_v2.as_deref(),
            );
            extension_data.insert(selected_plugins);
            let tool_router = turn::built_tools(
                self.as_ref(),
                turn_context.as_ref(),
                &settings.model_info,
                &environments,
                &mcp,
                &extension_data,
                prepared_recommendations,
                &dynamic_tools,
            )
            .await?;
            Ok::<_, CodexErr>((executor_hook_sources, mcp, tool_router, extension_data))
        });
        // Returned warnings must finish delivery even if tools fail or preparation is cancelled.
        let (loaded_agents_md, prepared_tools) =
            tokio::join!(load_agents_md, prepare_tools.or_cancel(cancellation_token));
        let loaded_agents_md = loaded_agents_md?;
        if cancellation_token.is_cancelled() {
            return Err(CodexErr::TurnAborted);
        }
        let (executor_hook_sources, mcp, tool_router, extension_data) = prepared_tools??;
        Ok(Arc::new(StepContext {
            serving_assessment: None,
            preempt: turn_context
                .config
                .features
                .enabled(Feature::InstantInterrupt)
                .then(CancellationToken::new),
            realtime: self.conversation.snapshot().await,
            settings,
            token_budget,
            session_telemetry,
            turn: turn_context,
            extension_data: Arc::new(extension_data),
            environments,
            inherited_environments,
            executor_hook_sources,
            mcp,
            tool_router,
            loaded_agents_md,
        }))
    }

    pub(crate) async fn record_inter_agent_communication(
        &self,
        turn_context: &TurnContext,
        model_info: &ModelInfo,
        communication: impl Into<crate::PendingAgentMessage>,
    ) {
<<<<<<< HEAD
        communication.set_turn_id_if_missing(&turn_context.sub_id);
        if communication.id.as_ref().is_none_or(|id| id.is_empty()) {
            communication.id = Some(ResponseItemId::new("amsg"));
        }
        let message_id = communication.message_id.clone();
        if let Some(message_id) = message_id.as_ref()
            && !self
                .applied_agent_message_ids
                .lock()
                .await
                .insert(message_id.clone())
        {
            return;
        }
        let response_item = communication.to_model_input_item();
        let items = self.prepare_conversation_items_for_history(
            turn_context,
            std::slice::from_ref(&response_item),
        );
||||||| base
        communication.set_turn_id_if_missing(&turn_context.sub_id);
        let response_item = communication.to_model_input_item();
        let items = self.prepare_conversation_items_for_history(
            turn_context,
            std::slice::from_ref(&response_item),
        );
=======
        let communication = communication.into();
        // Agent-message dependencies are not captured by direct-tool timing.
        if turn_context
            .config
            .features
            .enabled(Feature::CriticalPathTiming)
        {
            turn_context.turn_timing_state.invalidate_critical_path();
        }
        let mut response_item = communication.to_model_input_item();
        if let ResponseItem::AgentMessage {
            content,
            internal_chat_message_metadata_passthrough,
            ..
        } = &mut response_item
        {
            internal_chat_message_metadata_passthrough
                .get_or_insert_default()
                .content_item_metadata
                .get_or_insert_with(|| {
                    vec![
                        ContentItemMetadata::tool(Some(
                            turn_context
                                .config
                                .multi_agent_v2
                                .tool_namespace
                                .clone()
                                .map(ContentItemNamespace::from)
                                .unwrap_or(ContentItemNamespace::Functions),
                        ));
                        content.len()
                    ]
                });
        }
        let (items, _) = self
            .prepare_conversation_items_for_history(
                turn_context,
                model_info,
                std::slice::from_ref(&response_item),
            )
            .await;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        let items = items.as_ref();
<<<<<<< HEAD
        let response_item = items[0].clone();
        // Agent mail is data without standing (PF-30-S03 taint).
        self.services.model_client().note_recorded_for_taint(items);
||||||| base
        let response_item = items[0].clone();
=======
        let mut response_item = ResponseItemEnvelope::new(items[0].clone());
        // A send confirmed after the pending snapshot must not reach the rollout
        // before this boundary; older readers assign deliveries by physical order.
        let boundary = self
            .code_mode_message_tasks
            .communication_boundary
            .acquire()
            .await
            .unwrap_or_else(|_| unreachable!("communication boundary remains open"));
        // Older readers assign delivered assistant messages to the next physical
        // communication boundary, so persist earlier confirmed sends first.
        let (order, pending) = {
            let mut state = self.state.lock().await;
            (
                state.history.reserve_input_order(),
                self.pending_code_mode_message_recordings(),
            )
        };
        response_item
            .metadata
            .get_or_insert_default()
            .user_input_order = Some(order);
        for mut recording in pending {
            let _ = recording.changed().await;
        }
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        {
            let mut state = self.state.lock().await;
            state.current_time_reminder.note_recorded_items(items);
            state.history.record_annotated_items(
                std::slice::from_mut(&mut response_item),
                model_info.truncation_policy.into(),
            );
        }
<<<<<<< HEAD
        self.persist_rollout_items(&[RolloutItem::InterAgentCommunication(communication)])
            .await;
        if let Some(message_id) = message_id
            && let Some(state_db) = self.state_db()
        {
            match self.flush_rollout().await {
                Ok(()) => {
                    if self
                        .track_agent_message_for_turn(&turn_context.sub_id, message_id.clone())
                        .await
                    {
                        match state_db
                            .transition_agent_message(
                                &message_id,
                                codex_state::AgentMailboxPhase::Submitted,
                                codex_state::AgentMailboxPhase::ProviderRunning,
                                crate::turn_timing::now_unix_timestamp_ms(),
                            )
                            .await
                        {
                            Ok(true) => {}
                            Ok(false) => warn!(
                                %message_id,
                                "agent mailbox message could not enter provider_running from its current state"
                            ),
                            Err(err) => warn!(
                                %err,
                                %message_id,
                                "failed to mark agent mailbox message provider_running"
                            ),
                        }
                    } else if let Err(err) = state_db
                        .mark_agent_message_completed(
                            &message_id,
                            crate::turn_timing::now_unix_timestamp_ms(),
                        )
                        .await
                    {
                        warn!(%err, %message_id, "failed to complete locally applied mailbox message");
                    }
                }
                Err(err) => warn!(
                    %err,
                    %message_id,
                    "leaving agent mailbox message recoverable because its rollout did not flush"
                ),
            }
        }
||||||| base
        self.persist_rollout_items(&[
            RolloutItem::InterAgentCommunicationMetadata {
                trigger_turn: communication.trigger_turn,
            },
            RolloutItem::ResponseItem(response_item),
        ])
        .await;
=======
        if let Some(delivery) = &communication.delivery {
            delivery.consumed(self.thread_id);
        }
        self.persist_rollout_items(&[
            RolloutItem::InterAgentCommunicationMetadata {
                trigger_turn: communication.trigger_turn,
            },
            RolloutItem::ResponseItem(response_item),
        ])
        .await;
        drop(boundary);
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        self.send_raw_response_items(turn_context, items).await;
        // Plaintext inter-agent controls are operator-visible transcript content. Raw response
        // notifications preserve provider compatibility, but app-server clients render the
        // item lifecycle; emit it here just as we do for ordinary recorded response items.
        if let Some(item) = parse_turn_item(&response_item) {
            self.emit_turn_item_started(turn_context, &item).await;
            self.emit_turn_item_completed(turn_context, item).await;
        }
    }

    pub(crate) async fn has_applied_agent_message_id(&self, message_id: &str) -> bool {
        self.applied_agent_message_ids
            .lock()
            .await
            .contains(message_id)
    }

    async fn track_agent_message_for_turn(&self, sub_id: &str, message_id: String) -> bool {
        let Some(turn_state) = self
            .input_queue
            .turn_state_for_sub_id(&self.active_turn, sub_id)
            .await
        else {
            return false;
        };
        turn_state.lock().await.track_mailbox_message(message_id);
        true
    }

    async fn maybe_warn_on_server_model_mismatch(
        self: &Arc<Self>,
        step_context: &StepContext,
        server_model: String,
    ) -> bool {
        let turn_context = &step_context.turn;
        let requested_model = step_context.settings.model_info.slug.clone();
        let server_model_normalized = server_model.to_ascii_lowercase();
        let requested_model_normalized = requested_model.to_ascii_lowercase();
        if server_model_normalized == requested_model_normalized {
            info!("server reported model {server_model} (matches requested model)");
            return false;
        }

        warn!("server reported model {server_model} while requested model was {requested_model}");

        if !turn_context.config.model_provider.is_openai() {
            info!(
                provider = %turn_context.config.model_provider_id,
                "skipping model reroute warning for non-OpenAI provider"
            );
            return false;
        }

        let warning_message = format!(
            "Your account was flagged for potentially high-risk cyber activity and this request was routed to gpt-5.2 as a fallback. To regain access to gpt-5.3-codex, apply for trusted access: {CYBER_VERIFY_URL} or learn more: {CYBER_SAFETY_URL}"
        );

        self.send_event(
            turn_context,
            EventMsg::ModelReroute(ModelRerouteEvent {
                from_model: requested_model.clone(),
                to_model: server_model.clone(),
                reason: ModelRerouteReason::HighRiskCyberActivity,
            }),
        )
        .await;

        self.send_event(
            turn_context,
            EventMsg::Warning(WarningEvent {
                message: warning_message.clone(),
            }),
        )
        .await;
        true
    }

    fn record_server_model_identity(
        self: &Arc<Self>,
        turn_context: &Arc<TurnContext>,
        server_model: &str,
    ) {
        let requested_model = turn_context.model_info.slug.as_str();
        if server_model.eq_ignore_ascii_case(requested_model) {
            info!("server reported model {server_model} (matches requested model)");
            return;
        }
        info!(
            model_provider = %turn_context.config.model_provider_id,
            requested_model,
            reported_model = server_model,
            "provider reported a different model identity; recording it without inferring a reroute reason"
        );
    }

    pub(crate) async fn emit_model_verification(
        self: &Arc<Self>,
        turn_context: &Arc<TurnContext>,
        verifications: Vec<ModelVerification>,
    ) {
        self.send_event(
            turn_context,
            EventMsg::ModelVerification(ModelVerificationEvent { verifications }),
        )
        .await;
    }

    pub(crate) async fn emit_turn_moderation_metadata(
        self: &Arc<Self>,
        turn_context: &Arc<TurnContext>,
        metadata: TurnModerationMetadataEvent,
    ) {
        self.send_event(turn_context, EventMsg::TurnModerationMetadata(metadata))
            .await;
    }

    #[cfg(test)]
    pub(crate) async fn replace_history(
        &self,
        items: Vec<ResponseItem>,
        reference_context_item: Option<TurnContextItem>,
    ) {
        let mut state = self.state.lock().await;
        state.replace_history(items, reference_context_item);
    }

    pub(crate) async fn replace_compacted_history<'a>(
        &self,
<<<<<<< HEAD
        items: Vec<ResponseItem>,
        reference_context_item: Option<TurnContextItem>,
        world_state_baseline: Option<Arc<WorldState>>,
        metadata: CompactedHistoryMetadata,
    ) {
        let items = Self::assign_missing_response_item_ids(Cow::Owned(items)).into_owned();
        // Resumes that start at this checkpoint (paginated threads, referenced
        // forks) never read earlier records, so restate the current origins.
        self.services.model_client().journal_current_origins(&items);
        let compacted_item = CompactedItem {
            message: metadata.message,
            replacement_history: Some(items.clone()),
            window_number: Some(metadata.window_number),
            first_window_id: Some(metadata.window_ids.first_window_id.to_string()),
            previous_window_id: metadata
                .window_ids
                .previous_window_id
                .map(|id| id.to_string()),
            window_id: Some(metadata.window_ids.window_id.to_string()),
||||||| base
        items: Vec<ResponseItem>,
        reference_context_item: Option<TurnContextItem>,
        world_state_baseline: Option<Arc<WorldState>>,
        metadata: CompactedHistoryMetadata,
    ) {
        let items = Self::assign_missing_response_item_ids(Cow::Owned(items)).into_owned();
        let compacted_item = CompactedItem {
            message: metadata.message,
            replacement_history: Some(items.clone()),
            window_number: Some(metadata.window_number),
            first_window_id: Some(metadata.window_ids.first_window_id.to_string()),
            previous_window_id: metadata
                .window_ids
                .previous_window_id
                .map(|id| id.to_string()),
            window_id: Some(metadata.window_ids.window_id.to_string()),
=======
        mut items: Vec<ResponseItemEnvelope>,
        reference_context_item: TurnContextItem,
        mut world_state_baseline: WorldStateSnapshot,
        installation: impl Into<CompactedHistoryInstallation<'a>>,
    ) -> bool {
        let installation = installation.into();
        let compaction_model_hash = match &installation {
            CompactedHistoryInstallation::Complete(metadata) => &metadata.compaction_model_hash,
            CompactedHistoryInstallation::Prefix { model_info, .. } => &model_info.comp_hash,
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        };
        let mcp_revision = self
            .services
            .executed_tool_calls
            .mcp_attribution_checkpoint(/*force*/ true)
            .and_then(|(attribution, revision)| {
                items.last_mut().map(|envelope| {
                    envelope.metadata.get_or_insert_default().mcp_attribution = Some(attribution);
                    revision
                })
            });
        if let Some(checkpoint) = items.iter_mut().rev().find(|envelope| {
            matches!(
                envelope.item,
                ResponseItem::Compaction { .. } | ResponseItem::ContextCompaction { .. }
            )
        }) {
            checkpoint
                .metadata
                .get_or_insert_default()
                .compaction_model_hash = compaction_model_hash.clone();
        }
        // Wait for accepted updates to finish persisting, then keep later updates from
        // overtaking the current settings snapshot while its checkpoint is written.
        let _settings_guard = thread_settings::acquire_persistence_lock(self).await;
        let (compacted_item, started_item) = {
            let mut state = self.state.lock().await;
            let complete_replacement =
                matches!(&installation, CompactedHistoryInstallation::Complete(_));
            let (metadata, started_item) = match installation {
                CompactedHistoryInstallation::Complete(metadata) => (metadata, None),
                CompactedHistoryInstallation::Prefix {
                    prefix,
                    prefix_context,
                    initial_context,
                    model_info,
                    compaction_response_id,
                    reviewer_compaction_hash,
                    turn_context,
                    started_item,
                } => {
                    // Validate immediately before slicing and replacing the captured prefix.
                    if state.history.history_version() != prefix.generation
                        || state.history.annotated_items().len() < prefix.len
                        || self.services.model_client.auth_owner_generation()
                            != prefix.auth_owner_generation
                    {
                        return false;
                    }
                    let current = state.history.annotated_items();
                    let suffix = &current[prefix.len..];
                    if suffix.iter().any(|envelope| {
                        crate::compact_retained_tool_outputs::is_retained_tool_output(envelope)
                            || compact::is_real_user_or_agent_message(&envelope.item)
                    }) {
                        items.extend(initial_context);
                    } else {
                        items = compact::insert_initial_context_before_last_real_user_or_summary(
                            items,
                            initial_context,
                        );
                    }
                    items.splice(
                        0..0,
                        prefix_context.into_iter().map(ResponseItemEnvelope::new),
                    );
                    items.extend_from_slice(suffix);
                    // The unchanged suffix already contains updates through this baseline.
                    if let Some(checkpoint) = state.history.world_state_checkpoint() {
                        world_state_baseline = WorldStateSnapshot::from(&checkpoint.state);
                    }
                    // Prefix replacement already retains all edits in the live suffix.
                    let input_goal_ids = Default::default();
                    let (window_number, window_ids) = state.advance_auto_compact_window();
                    (
                        CompactedHistoryMetadata {
                            input_goal_ids,
                            message: String::new(),
                            window_number,
                            window_ids,
                            compaction_response_id: Some(compaction_response_id),
                            compaction_model_hash: model_info.comp_hash.clone(),
                            reviewer_compaction_hash,
                        },
                        started_item.map(|item| (turn_context, item)),
                    )
                }
            };
            for envelope in &mut items {
                Self::assign_missing_response_item_id(&mut envelope.item);
            }
            // Goal edits are published outside the running task. Keep edits accepted after
            // the compaction input snapshot, in their original order, after its replacement.
            if complete_replacement {
                let replacement_goal_ids = crate::context::UserGoalUpdate::message_ids(
                    items.iter().map(|envelope| &envelope.item),
                );
                items.extend(
                    state
                        .history
                        .annotated_items()
                        .iter()
                        .filter(|envelope| {
                            crate::context::UserGoalUpdate::message_text(&envelope.item).is_some()
                                && envelope.item.id().is_some_and(|id| {
                                    !metadata.input_goal_ids.contains(id)
                                        && !replacement_goal_ids.contains(id)
                                })
                        })
                        .cloned(),
                );
            }
            let replacement_history = items.clone();
            state.replace_annotated_history(
                items,
                Some(reference_context_item.clone()),
                HistoryReplacement::Compaction {
                    reviewer_compaction_hash: metadata.reviewer_compaction_hash,
                },
            );
            state.reasoning_effort_pin = ReasoningEffortPin::Compacted;
            state
                .history
                .set_world_state_baseline(world_state_baseline.clone());
            let compacted_item = CompactedItem {
                message: metadata.message,
                replacement_history: Some(replacement_history),
                guardian_history: state.history.guardian_history_checkpoint(),
                retained_context: Some(state.history.retained_context().clone()),
                mcp_resource_origins: self.services.mcp_runtime.resource_origin_checkpoint(),
                window_number: Some(metadata.window_number),
                first_window_id: Some(metadata.window_ids.first_window_id.to_string()),
                previous_window_id: metadata
                    .window_ids
                    .previous_window_id
                    .map(|id| id.to_string()),
                window_id: Some(metadata.window_ids.window_id.to_string()),
                compaction_response_id: metadata.compaction_response_id,
                latest_token_usage_record: state.latest_token_usage_record.clone(),
                resume_metadata: Some(CompactionResumeMetadata {
                    dynamic_tools: Some(Box::new(DynamicToolsSnapshot {
                        thread_id: self.thread_id(),
                        tools: state.persisted_dynamic_tools.to_vec(),
                    })),
                    multi_agent_version: self.multi_agent_version(),
                    last_started_turn_id: state.last_started_turn_id.clone(),
                    turn_attribution: state.turn_attribution.clone(),
                    previous_turn_settings: state.previous_turn_settings(),
                }),
            };
            (compacted_item, started_item)
        };

<<<<<<< HEAD
        self.persist_rollout_items(&[RolloutItem::Compacted(compacted_item)])
            .await;
        // Origins registered for the replacement history follow it.
        self.persist_source_origins().await;
        // Persist the baseline after the replacement history that established it.
        if let Some(world_state_item) = world_state_item {
            self.persist_rollout_items(&[RolloutItem::WorldState(world_state_item)])
                .await;
||||||| base
        self.persist_rollout_items(&[RolloutItem::Compacted(compacted_item)])
            .await;
        // Persist the baseline after the replacement history that established it.
        if let Some(world_state_item) = world_state_item {
            self.persist_rollout_items(&[RolloutItem::WorldState(world_state_item)])
                .await;
=======
        if let Some((turn_context, item)) = started_item {
            // The prefix is now known to be valid; announce the install before checkpoint I/O.
            self.emit_turn_item_started(turn_context, item).await;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        }
        let world_state_item = WorldStateItem::full(world_state_baseline.into_object());
        let rollout_items = [
            RolloutItem::Compacted(Box::new(compacted_item)),
            RolloutItem::WorldState(world_state_item),
            RolloutItem::TurnContext(reference_context_item),
            // The frozen turn context must not override current settings in persisted metadata.
            RolloutItem::EventMsg(thread_settings::applied_event(self).await),
        ];
        if self.persist_rollout_items(&rollout_items).await
            && let Some(revision) = mcp_revision
        {
            self.services
                .executed_tool_calls
                .mark_mcp_attribution_persisted(revision);
        }
        {
            let mut state = self.state.lock().await;
            state.queue_pending_session_start_source(codex_hooks::SessionStartSource::Compact);
        }
        true
    }

<<<<<<< HEAD
||||||| base
    async fn persist_rollout_response_items(&self, items: &[ResponseItem]) {
        let rollout_items: Vec<RolloutItem> = items
            .iter()
            .cloned()
            .map(RolloutItem::ResponseItem)
            .collect();
        self.persist_rollout_items(&rollout_items).await;
    }

=======
    fn preserve_parent_tool_declarations(&self) -> bool {
        // Only prediction forks and thread/sample set this marker when they disable MCP.
        self.enabled(Feature::PredictionPreserveParentTools)
            && self
                .services
                .thread_extension_data
                .get::<crate::McpServersDisabled>()
                .is_some()
    }

>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
    pub fn enabled(&self, feature: Feature) -> bool {
        self.features.enabled(feature)
    }

    pub(crate) fn features(&self) -> ManagedFeatures {
        self.features.clone()
    }

    pub(crate) async fn collaboration_mode(&self) -> CollaborationMode {
        let state = self.state.lock().await;
        state
            .session_configuration
            .step_settings
            .collaboration_mode
            .clone()
    }

    pub(crate) fn multi_agent_version(&self) -> Option<MultiAgentVersion> {
        self.multi_agent_version.get().copied()
    }

    pub(crate) fn set_multi_agent_version_if_unset(
        &self,
        multi_agent_version: MultiAgentVersion,
    ) -> MultiAgentVersion {
        *self.multi_agent_version.get_or_init(|| multi_agent_version)
    }

    pub(crate) fn resolve_multi_agent_version_for_model(
        &self,
        model_info: &ModelInfo,
        config: &Config,
    ) -> MultiAgentVersion {
        if let Some(multi_agent_version) = self.multi_agent_version() {
            return config.multi_agent_version_for_model(Some(multi_agent_version));
        }

        let selected = config.multi_agent_version_for_model(model_info.multi_agent_version);

        self.set_multi_agent_version_if_unset(selected)
    }

    #[tracing::instrument(level = "trace", skip_all, fields(item_count = items.len()))]
    async fn send_raw_response_items(&self, turn_context: &TurnContext, items: &[ResponseItem]) {
        for item in items {
            self.send_event(
                turn_context,
                EventMsg::RawResponseItem(RawResponseItemEvent { item: item.clone() }),
            )
            .await;
        }
    }

    /// `source_envelopes`: one developer message per stored-data fragment
    /// (memories, MCP-supplied hints), registered as external data before any
    /// caller records the surrounding context as host. The first registration
    /// wins, so it is never upgraded and reaches a protected request labelled
    /// with its real source (`source=memory`, `source=mcp`).
    fn push_stored_data_messages(
        &self,
        items: &mut Vec<ResponseItem>,
        sections: Vec<String>,
        kind: codex_protocol::provenance::SourceKind,
    ) {
        for section in sections {
            if let Some(message) =
                crate::context_manager::updates::build_developer_update_item(vec![section])
            {
                self.services.model_client().register_message_origin(
                    std::slice::from_ref(&message),
                    crate::security::ingress::MessageOrigin::External(kind),
                );
                items.push(message);
            }
        }
    }

    async fn build_turn_context_contribution_items(
        &self,
        step_context: &StepContext,
    ) -> Vec<ResponseItem> {
        let turn_context = step_context.turn.as_ref();
        let mut developer_sections = Vec::new();
        let context_contributors = self.services.extensions.context_contributors().to_vec();
        let separate_stored_data = self.services.model_client().source_envelopes_enabled();
        let mut stored_data_sections = Vec::new();

        for contributor in &context_contributors {
            for fragment in contributor
                .contribute_turn_context(TurnContextContributionInput {
                    thread_id: self.thread_id(),
                    turn_id: turn_context.sub_id.as_str(),
                    session_store: &self.services.session_extension_data,
                    thread_store: &self.services.thread_extension_data,
                    turn_store: turn_context.extension_data.as_ref(),
                    model_context_window: step_context.settings.model_info.usable_context_window(),
                })
                .await
            {
<<<<<<< HEAD
                if separate_stored_data && fragment.is_stored_data() {
                    stored_data_sections.push(fragment.text().to_string());
                    continue;
                }
                push_prompt_fragment(
                    fragment,
                    &mut developer_sections,
                    &mut contextual_user_sections,
                    &mut separate_developer_sections,
                );
||||||| base
                push_prompt_fragment(
                    fragment,
                    &mut developer_sections,
                    &mut contextual_user_sections,
                    &mut separate_developer_sections,
                );
=======
                developer_sections.push(fragment.into());
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
            }
        }

<<<<<<< HEAD
        let mut items = Vec::with_capacity(3);
        if let Some(developer_message) =
            crate::context_manager::updates::build_developer_update_item(developer_sections)
        {
            items.push(developer_message);
        }
        for section in separate_developer_sections {
            if let Some(developer_message) =
                crate::context_manager::updates::build_developer_update_item(vec![section])
            {
                items.push(developer_message);
            }
        }
        self.push_stored_data_messages(
            &mut items,
            stored_data_sections,
            codex_protocol::provenance::SourceKind::Memory,
        );
        if let Some(contextual_user_message) =
            crate::context_manager::updates::build_contextual_user_message(contextual_user_sections)
        {
            items.push(contextual_user_message);
        }
        items
||||||| base
        let mut items = Vec::with_capacity(3);
        if let Some(developer_message) =
            crate::context_manager::updates::build_developer_update_item(developer_sections)
        {
            items.push(developer_message);
        }
        for section in separate_developer_sections {
            if let Some(developer_message) =
                crate::context_manager::updates::build_developer_update_item(vec![section])
            {
                items.push(developer_message);
            }
        }
        if let Some(contextual_user_message) =
            crate::context_manager::updates::build_contextual_user_message(contextual_user_sections)
        {
            items.push(contextual_user_message);
        }
        items
=======
        crate::context_manager::updates::build_rendered_message(developer_sections)
            .into_iter()
            .collect()
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
    }

    /// Normally both inputs come from the same step. Async compaction uses the prefix's
    /// world state so later state updates can stay in history unchanged.
    /// Returns rendered items with placement preserved for the caller's history layout.
    pub(crate) async fn build_initial_context_with_world_state(
        &self,
        step_context: &StepContext,
        world_state: &WorldState,
<<<<<<< HEAD
    ) -> Vec<ResponseItem> {
        let mut developer_sections = Vec::<ContextSection>::with_capacity(8);
        let mut contextual_user_sections = Vec::<String>::with_capacity(2);
        let mut separate_developer_sections = Vec::<ContextSection>::new();
||||||| base
    ) -> Vec<ResponseItem> {
        let mut developer_sections = Vec::<String>::with_capacity(8);
        let mut contextual_user_sections = Vec::<String>::with_capacity(2);
        let mut separate_developer_sections = Vec::<String>::new();
=======
    ) -> (Vec<WorldStateUpdate>, WorldStateSnapshot) {
        let turn_context = step_context.turn.as_ref();
        let mut developer_sections = Vec::<RenderedFragment>::with_capacity(8);
        let mut contextual_user_sections = Vec::<RenderedFragment>::with_capacity(2);
        let mut separate_developer_messages = Vec::<ResponseItem>::new();
        let mut context_window_hints = Vec::new();
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        let (session_source, auto_compact_window_ids) = {
            let state = self.state.lock().await;
            (
                state.session_configuration.session_source.clone(),
                state.auto_compact_window_ids(),
            )
        };
        let separate_guardian_developer_message =
            crate::guardian::is_basic_session_source(&session_source);
        // Keep the guardian policy prompt out of the aggregated developer bundle so it
        // stays isolated as its own top-level developer message for guardian subagents.
        if !separate_guardian_developer_message
            && let Some(developer_instructions) = turn_context.developer_instructions.as_deref()
            && !developer_instructions.is_empty()
        {
<<<<<<< HEAD
            developer_sections.push(developer_instructions.to_string().into());
        }
        if turn_context.config.include_skill_instructions
            && turn_context
                .extension_data
                .get::<HostSkillsCatalogInWorldState>()
                .is_none()
        {
            let available_skills = build_available_skills(
                turn_context.turn_skills.snapshot.outcome(),
                default_skill_metadata_budget(turn_context.model_info.context_window),
                SkillRenderSideEffects::ThreadStart {
                    session_telemetry: &self.services.session_telemetry,
                },
            );
            if let Some(available_skills) = available_skills {
                let warning_message = available_skills.warning_message.clone();
                let skills_instructions = AvailableSkillsInstructions::from_available_skills(
                    &available_skills,
                    turn_context.model_info.include_skills_usage_instructions,
                );
                if let Some(warning_message) = warning_message {
                    self.send_event_raw(Event {
                        id: String::new(),
                        msg: EventMsg::Warning(WarningEvent {
                            message: warning_message,
                        }),
                    })
                    .await;
                }
                developer_sections.push(ContextSection::attributed(
                    skills_instructions.render(),
                    HOST_SKILLS_CONTEXT_SOURCE_ID,
                ));
            }
||||||| base
            developer_sections.push(developer_instructions.to_string());
        }
        if turn_context.config.include_skill_instructions
            && turn_context
                .extension_data
                .get::<HostSkillsCatalogInWorldState>()
                .is_none()
        {
            let available_skills = build_available_skills(
                turn_context.turn_skills.snapshot.outcome(),
                default_skill_metadata_budget(turn_context.model_info.context_window),
                SkillRenderSideEffects::ThreadStart {
                    session_telemetry: &self.services.session_telemetry,
                },
            );
            if let Some(available_skills) = available_skills {
                let warning_message = available_skills.warning_message.clone();
                let skills_instructions = AvailableSkillsInstructions::from_available_skills(
                    &available_skills,
                    turn_context.model_info.include_skills_usage_instructions,
                );
                if let Some(warning_message) = warning_message {
                    self.send_event_raw(Event {
                        id: String::new(),
                        msg: EventMsg::Warning(WarningEvent {
                            message: warning_message,
                        }),
                    })
                    .await;
                }
                developer_sections.push(skills_instructions.render());
            }
=======
            developer_sections
                .push(DeveloperInstructions::new(developer_instructions).render_fragment());
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        }
        let loaded_plugins = self
            .services
            .plugins_manager
            .plugins_for_config(&turn_context.config.plugins_config_input())
            .await
            .without_plugins(&turn_context.disabled_plugin_ids);
        let recommended_plugin_candidates = if turn_context
            .config
            .features
            .plugin_recommendations_enabled()
        {
            let auth = self.services.auth_manager().auth().await;
            let plugins_config = turn_context.config.plugins_config_input();
            self.services
                .plugins_manager
                .recommended_plugin_candidates_for_config(RecommendedPluginCandidatesInput {
                    plugins_config: &plugins_config,
                    loaded_plugins: &loaded_plugins,
                    auth: auth.as_ref(),
                    disabled_tools: &turn_context.config.tool_suggest.disabled_tools,
                    app_server_client_name: turn_context.app_server_client_name.as_deref(),
                })
                .await
        } else {
            None
        };
        let context_contributors = self.services.extensions.context_contributors().to_vec();
        // `source_envelopes`: stored-data fragments (memories) get their own
        // message so they can be labelled without labelling host policy.
        let separate_stored_data = self.services.model_client().source_envelopes_enabled();
        let mut stored_data_sections = Vec::<String>::new();
        let mut mcp_hint_sections = Vec::<String>::new();
        for contributor in &context_contributors {
            for fragment in contributor
                .contribute_thread_context(
                    &self.services.session_extension_data,
                    &self.services.thread_extension_data,
                )
                .await
            {
<<<<<<< HEAD
                if separate_stored_data && fragment.is_stored_data() {
                    stored_data_sections.push(fragment.text().to_string());
                    continue;
                }
                push_prompt_fragment(
                    fragment,
                    &mut developer_sections,
                    &mut contextual_user_sections,
                    &mut separate_developer_sections,
                );
||||||| base
                push_prompt_fragment(
                    fragment,
                    &mut developer_sections,
                    &mut contextual_user_sections,
                    &mut separate_developer_sections,
                );
=======
                match fragment.slot() {
                    PromptSlot::ContextWindow => {
                        context_window_hints.push(fragment.into_content());
                    }
                    PromptSlot::DeveloperPolicy | PromptSlot::DeveloperCapabilities => {
                        developer_sections.push(fragment.into());
                    }
                }
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
            }
        }
        for contributor in &context_contributors {
            for fragment in contributor
                .contribute_turn_context(TurnContextContributionInput {
                    thread_id: self.thread_id(),
                    turn_id: turn_context.sub_id.as_str(),
                    session_store: &self.services.session_extension_data,
                    thread_store: &self.services.thread_extension_data,
                    turn_store: turn_context.extension_data.as_ref(),
                    model_context_window: step_context.settings.model_info.usable_context_window(),
                })
                .await
            {
<<<<<<< HEAD
                if separate_stored_data && fragment.is_stored_data() {
                    stored_data_sections.push(fragment.text().to_string());
                    continue;
                }
                push_prompt_fragment(
                    fragment,
                    &mut developer_sections,
                    &mut contextual_user_sections,
                    &mut separate_developer_sections,
                );
||||||| base
                push_prompt_fragment(
                    fragment,
                    &mut developer_sections,
                    &mut contextual_user_sections,
                    &mut separate_developer_sections,
                );
=======
                developer_sections.push(fragment.into());
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
            }
        }
        // This is full-context metadata. Steady-state context diffs should not re-emit it.
        if turn_context.config.features.enabled(Feature::TokenBudget)
            && step_context
                .settings
                .model_info
                .resolved_context_window()
                .is_some()
        {
<<<<<<< HEAD
            let mcp_result = self
                .services
                .mcp_runtime
                .latest_call_tool(
                    "notes",
                    "thread_hint",
                    /*arguments*/ None,
                    Some(serde_json::json!({
                        "threadId": self.thread_id().to_string(),
                    })),
                )
                .await
                .ok()
                .and_then(|result| {
                    let text = result
                        .content
                        .iter()
                        .filter_map(|content| {
                            content.get("text").and_then(serde_json::Value::as_str)
                        })
                        .filter(|text| !text.is_empty())
                        .collect::<Vec<_>>()
                        .join("\n");
                    (!text.is_empty()).then_some(text)
                });
            // `source_envelopes`: the MCP server's hint is external text, not
            // host policy; send it as its own `source=mcp` data message.
            let (mcp_result, data_hint) =
                token_budget_hint_placement(separate_stored_data, mcp_result);
            mcp_hint_sections.extend(data_hint);
            developer_sections.push(
||||||| base
            let mcp_result = self
                .services
                .mcp_runtime
                .latest_call_tool(
                    "notes",
                    "thread_hint",
                    /*arguments*/ None,
                    Some(serde_json::json!({
                        "threadId": self.thread_id().to_string(),
                    })),
                )
                .await
                .ok()
                .and_then(|result| {
                    let text = result
                        .content
                        .iter()
                        .filter_map(|content| {
                            content.get("text").and_then(serde_json::Value::as_str)
                        })
                        .filter(|text| !text.is_empty())
                        .collect::<Vec<_>>()
                        .join("\n");
                    (!text.is_empty()).then_some(text)
                });
            developer_sections.push(
=======
            // Keep the legacy bridge hint when native Notes is disabled. A failed
            // native request must not fall back to the bridge.
            if !turn_context
                .config
                .token_budget
                .as_ref()
                .is_some_and(|config| config.use_history_notes_extension)
                && let Some((text, source_tool_namespace)) = self
                    .services
                    .mcp_runtime
                    .latest_call_tool(
                        "notes",
                        "thread_hint",
                        /*environment_id*/ None,
                        /*arguments*/ None,
                        Some(serde_json::json!({
                            "threadId": self.thread_id().to_string(),
                        })),
                        /*requested_timeout*/ None,
                        /*wait_for_server*/ true,
                    )
                    .await
                    .ok()
                    .and_then(|call| {
                        let text = call
                            .result
                            .content
                            .iter()
                            .filter_map(|content| {
                                content.get("text").and_then(serde_json::Value::as_str)
                            })
                            .filter(|text| !text.is_empty())
                            .collect::<Vec<_>>()
                            .join("\n");
                        (!text.is_empty()).then_some((text, call.source_tool_namespace))
                    })
            {
                context_window_hints.push(codex_context_fragments::AnnotatedContent::text(
                    text,
                    ContentItemKind("notes.thread_hint".to_string()),
                    ContentItemMetadata::tool(source_tool_namespace.map(Into::into)),
                ));
            }
            separate_developer_messages.push(
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
                crate::context::TokenBudgetContext::new(
                    session_source
                        .get_agent_path()
                        .unwrap_or_else(codex_protocol::AgentPath::root),
                    auto_compact_window_ids.first_window_id,
                    auto_compact_window_ids.previous_window_id,
                    auto_compact_window_ids.window_id,
                )
<<<<<<< HEAD
                .render()
                .into(),
||||||| base
                .render(),
=======
                .render_with_hints(context_window_hints),
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
            );
        }
<<<<<<< HEAD
        // Render the active mode after the usage hint so it can override that hint.
        let mut initial_multi_agent_mode = None;
        for fragment in world_state.render_full() {
            match fragment.role() {
                "developer"
                    if fragment.markers().0 == ModelSwitchInstructions::type_markers().0 =>
                {
                    // New-model instructions must precede the rest of the developer context.
                    developer_sections.insert(0, ContextSection::from_fragment(fragment.as_ref()));
                }
                "developer" if fragment.markers().0 == MULTI_AGENT_MODE_OPEN_TAG => {
                    initial_multi_agent_mode = Some(fragment);
                }
                "developer" => {
                    developer_sections.push(ContextSection::from_fragment(fragment.as_ref()))
                }
                "user" => contextual_user_sections.push(fragment.render()),
                _ => {}
            }
        }

        let multi_agent_v2_usage_hint_text =
            multi_agents::usage_hint_text(turn_context, &session_source);

||||||| base
        // Render the active mode after the usage hint so it can override that hint.
        let mut initial_multi_agent_mode = None;
        for fragment in world_state.render_full() {
            match fragment.role() {
                "developer"
                    if fragment.markers().0 == ModelSwitchInstructions::type_markers().0 =>
                {
                    // New-model instructions must precede the rest of the developer context.
                    developer_sections.insert(0, fragment.render());
                }
                "developer" if fragment.markers().0 == MULTI_AGENT_MODE_OPEN_TAG => {
                    initial_multi_agent_mode = Some(fragment);
                }
                "developer" => developer_sections.push(fragment.render()),
                "user" => contextual_user_sections.push(fragment.render()),
                _ => {}
            }
        }

        let multi_agent_v2_usage_hint_text =
            multi_agents::usage_hint_text(turn_context, &session_source);

=======
        let (snapshot, updates) = world_state.render_full();
        let (mut prefix, context) = split_prefix_updates(updates);
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        let mut items = Vec::with_capacity(4);
        let mut context = context.into_iter();
        // Keep the initial-context grouping within each fragment run, but never across an item.
        loop {
            let mut next_item = None;
            // Render the active mode after the usage hint so it can override that hint.
            let mut initial_multi_agent_mode = None;
            let mut managed_developer_instructions = None;
            for update in context.by_ref() {
                let fragment = match update.content {
                    WorldStateUpdateContent::Fragment(fragment) => fragment,
                    WorldStateUpdateContent::Item(item) => {
                        next_item = Some(*item);
                        break;
                    }
                };
                match fragment.role() {
                    "developer"
                        if fragment.markers().0 == ModelSwitchInstructions::type_markers().0 =>
                    {
                        // New-model instructions must precede the rest of the developer context.
                        developer_sections.insert(0, fragment.render_fragment());
                    }
                    "developer" if fragment.markers().0 == MULTI_AGENT_MODE_OPEN_TAG => {
                        initial_multi_agent_mode = Some(fragment);
                    }
                    "developer"
                        if fragment.markers().0
                            == ManagedDeveloperInstructions::type_markers().0 =>
                    {
                        managed_developer_instructions = Some(fragment);
                    }
                    "developer"
                        if fragment.markers().0 == MultiAgentRoleInstructions::type_markers().0 =>
                    {
                        separate_developer_messages.push(fragment.render_fragment().into());
                    }
                    "developer"
                        if update.placement == Placement::Standalone
                            && fragment.markers().0.is_empty() =>
                    {
                        separate_developer_messages.push(fragment.render_fragment().into());
                    }
                    "developer" => developer_sections.push(fragment.render_fragment()),
                    "user" => contextual_user_sections.push(fragment.render_fragment()),
                    _ => {}
                }
            }

            if next_item.is_none()
                && let Some(recommended_plugins) = recommended_plugin_candidates
                    .as_deref()
                    .and_then(RecommendedPluginsInstructions::from_plugins)
            {
                developer_sections.push(recommended_plugins.render_fragment());
            }

            if let Some(developer_message) = crate::context_manager::updates::build_rendered_message(
                std::mem::take(&mut developer_sections),
            ) {
                items.push(developer_message);
            }
<<<<<<< HEAD
        }
        self.push_stored_data_messages(
            &mut items,
            stored_data_sections,
            codex_protocol::provenance::SourceKind::Memory,
        );
        self.push_stored_data_messages(
            &mut items,
            mcp_hint_sections,
            codex_protocol::provenance::SourceKind::Mcp,
        );
        if let Some(usage_hint_text) = multi_agent_v2_usage_hint_text
            && let Some(usage_hint_message) =
                crate::context_manager::updates::build_developer_update_item(vec![
                    usage_hint_text.to_string(),
                ])
        {
            items.push(usage_hint_message);
        }
        if let Some(initial_multi_agent_mode) = initial_multi_agent_mode {
            items.push(initial_multi_agent_mode.into_boxed_response_item());
        }
        if let Some(contextual_user_message) =
            crate::context_manager::updates::build_contextual_user_message(contextual_user_sections)
        {
            items.push(contextual_user_message);
        }
        // Emit the guardian policy prompt as a separate developer item so the guardian
        // subagent sees a distinct, easy-to-audit instruction block.
        if separate_guardian_developer_message
            && let Some(developer_instructions) = turn_context.developer_instructions.as_deref()
            && !developer_instructions.is_empty()
            && let Some(guardian_developer_message) =
                crate::context_manager::updates::build_developer_update_item(vec![
                    developer_instructions.to_string(),
                ])
        {
            items.push(guardian_developer_message);
||||||| base
        }
        if let Some(usage_hint_text) = multi_agent_v2_usage_hint_text
            && let Some(usage_hint_message) =
                crate::context_manager::updates::build_developer_update_item(vec![
                    usage_hint_text.to_string(),
                ])
        {
            items.push(usage_hint_message);
        }
        if let Some(initial_multi_agent_mode) = initial_multi_agent_mode {
            items.push(initial_multi_agent_mode.into_boxed_response_item());
        }
        if let Some(contextual_user_message) =
            crate::context_manager::updates::build_contextual_user_message(contextual_user_sections)
        {
            items.push(contextual_user_message);
        }
        // Emit the guardian policy prompt as a separate developer item so the guardian
        // subagent sees a distinct, easy-to-audit instruction block.
        if separate_guardian_developer_message
            && let Some(developer_instructions) = turn_context.developer_instructions.as_deref()
            && !developer_instructions.is_empty()
            && let Some(guardian_developer_message) =
                crate::context_manager::updates::build_developer_update_item(vec![
                    developer_instructions.to_string(),
                ])
        {
            items.push(guardian_developer_message);
=======
            items.append(&mut separate_developer_messages);
            if let Some(initial_multi_agent_mode) = initial_multi_agent_mode
                && let Some(message) =
                    crate::context_manager::updates::build_rendered_message(vec![
                        initial_multi_agent_mode.render_fragment(),
                    ])
            {
                items.push(message);
            }
            if let Some(contextual_user_message) =
                crate::context_manager::updates::build_rendered_message(std::mem::take(
                    &mut contextual_user_sections,
                ))
            {
                items.push(contextual_user_message);
            }
            // Emit the guardian policy prompt as a separate developer item so the guardian
            // subagent sees a distinct, easy-to-audit instruction block.
            if next_item.is_none()
                && separate_guardian_developer_message
                && let Some(developer_instructions) = turn_context.developer_instructions.as_deref()
                && !developer_instructions.is_empty()
                && let Some(guardian_developer_message) =
                    crate::context_manager::updates::build_rendered_message(vec![
                        GuardianPolicy::new(developer_instructions).render_fragment(),
                    ])
            {
                items.push(guardian_developer_message);
            }
            if let Some(managed_developer_instructions) = managed_developer_instructions
                && let Some(message) =
                    crate::context_manager::updates::build_rendered_message(vec![
                        managed_developer_instructions.render_fragment(),
                    ])
            {
                items.push(message);
            }

            if let Some(item) = next_item {
                items.push(item);
            } else {
                break;
            }
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        }
        // New context windows and compaction install these items directly into replacement history.
        for item in prefix.iter_mut().chain(&mut items) {
            item.set_turn_id_if_missing(&turn_context.sub_id);
        }
        let updates = prefix
            .into_iter()
            .map(WorldStateUpdate::prefix_item)
            .chain(items.into_iter().map(|item| WorldStateUpdate {
                placement: Placement::Standalone,
                content: WorldStateUpdateContent::Item(Box::new(item)),
            }))
            .collect();
        (updates, snapshot)
    }

    #[tracing::instrument(level = "trace", skip_all, fields(item_count = items.len()))]
<<<<<<< HEAD
    pub(crate) async fn persist_rollout_items(&self, items: &[RolloutItem]) {
        let gated = crate::security::disclosure_gate::gate_values(
            codex_secret_broker::output_gate::OutputSink::Transcript,
            items,
        );
        let items = gated.as_deref().unwrap_or(items);
        let Some(live_thread) = self.live_thread() else {
            return;
        };
        let Err(e) = live_thread.append_items(items).await else {
            // #616: only an append that wrote something shows the home works.
            if items.iter().any(|item| {
                codex_rollout::is_persisted_rollout_item(item, live_thread.history_mode())
            }) {
                crate::home_access::note_rollout_written(self.thread_id());
            }
            return;
        };
        error!("failed to record rollout items: {e:#}");
        // #616: the writer keeps unwritten items for its next flush; exec
        // waits for their folder before it saves the session at exit. Record
        // why the rollout could not be written.
        let message = format!("{e:#}");
        if crate::home_access::is_operation_not_permitted(&message)
            && let Ok(Some(rollout)) = live_thread.local_rollout_path().await
||||||| base
    pub(crate) async fn persist_rollout_items(&self, items: &[RolloutItem]) {
        if let Some(live_thread) = self.live_thread()
            && let Err(e) = live_thread.append_items(items).await
=======
    pub(crate) async fn persist_rollout_items(&self, items: &[RolloutItem]) -> bool {
        if let Some(live_thread) = self.live_thread()
            && let Err(e) = live_thread.append_items(items).await
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        {
<<<<<<< HEAD
            if let Some(dir) = rollout.parent() {
                crate::home_access::note_rollout_unwritten(self.thread_id(), dir.to_path_buf());
            }
            tokio::task::spawn_blocking(move || {
                crate::home_access::record_home_access_failure("rollout write", &rollout, &message)
            });
||||||| base
            error!("failed to record rollout items: {e:#}");
=======
            error!("failed to record rollout items: {e:#}");
            return false;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        }
        true
    }

    /// See `ModelClient::agent_handoff_origin`: the standing of text this
    /// session's model hands to another agent, from its current history.
    pub(crate) async fn agent_handoff_origin(&self) -> crate::security::ingress::MessageOrigin {
        let state = self.state.lock().await;
        self.services
            .model_client()
            .agent_handoff_origin(state.history.raw_items())
    }

    pub(crate) async fn clone_history(&self) -> ContextManager {
        let state = self.state.lock().await;
        state.clone_history()
    }

<<<<<<< HEAD
    /// Drops each tool output whose call already has an output in history.
    pub(crate) async fn retain_unrecorded_tool_outputs(&self, outputs: &mut Vec<ResponseItem>) {
        fn output_call_id(item: &ResponseItem) -> Option<&str> {
            match item {
                ResponseItem::FunctionCallOutput { call_id, .. }
                | ResponseItem::CustomToolCallOutput { call_id, .. } => Some(call_id),
                ResponseItem::ToolSearchOutput { call_id, .. } => call_id.as_deref(),
                _ => None,
            }
        }
        let state = self.state.lock().await;
        let recorded: HashSet<&str> = state
            .history
            .raw_items()
            .iter()
            .filter_map(output_call_id)
            .collect();
        outputs.retain(|item| output_call_id(item).is_none_or(|id| !recorded.contains(id)));
    }

    pub(crate) async fn current_window_id(&self) -> String {
||||||| base
    pub(crate) async fn current_window_id(&self) -> String {
=======
    /// Captures the history lifetime without exposing mutable session state.
    pub(crate) async fn history_reset(&self) -> (u64, CancellationToken) {
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        let state = self.state.lock().await;
        (
            state.history.reset_version,
            state.history_reset.child_token(),
        )
    }

    pub(crate) async fn conversation_history_snapshot(
        &self,
    ) -> Arc<dyn ConversationHistorySnapshot> {
        let state = self.state.lock().await;
        state.history.conversation_history_snapshot()
    }

    pub(crate) async fn current_window_id(&self) -> String {
        self.current_window().await.0
    }

    pub(crate) async fn current_window(&self) -> (String, u64, Uuid) {
        let state = self.state.lock().await;
        self.current_window_from_state(&state)
    }

    pub(crate) fn current_window_from_state(&self, state: &SessionState) -> (String, u64, Uuid) {
        let thread_id = self.thread_id;
        let window_number = state.auto_compact_window_number();
        let context_window_id = state.auto_compact_window_ids().window_id;
        (
            format!("{thread_id}:{window_number}"),
            window_number,
            context_window_id,
        )
    }

    pub(crate) async fn advance_auto_compact_window(&self) -> (u64, AutoCompactWindowIds) {
        let mut state = self.state.lock().await;
        state.advance_auto_compact_window()
    }

    pub(crate) async fn request_new_context_window(&self) {
        let mut state = self.state.lock().await;
        state.request_new_context_window();
    }

    pub(crate) async fn take_new_context_window_request(&self) -> bool {
        let mut state = self.state.lock().await;
        state.take_new_context_window_request()
    }

    pub(crate) async fn start_new_context_window(
        &self,
        step_context: &StepContext,
        world_state: Arc<WorldState>,
    ) {
        let turn_context = step_context.turn.as_ref();
        let history = self.clone_history().await;
        let input_goal_ids = crate::context::UserGoalUpdate::message_ids(history.raw_items());
        let retained_client_items =
            crate::compact_remote_v2::truncate_retained_messages_for_remote_compaction(
                history
                    .annotated_items()
                    .iter()
                    .filter(|item| {
                        (self.enabled(Feature::RetainClientToolOutputs)
                            && crate::compact_retained_tool_outputs::is_retained_tool_output(item))
                            || (self.enabled(Feature::RetainClientDeveloperMessages)
                                && crate::compact_remote_v2::is_client_authored_developer_message(
                                    item,
                                ))
                    })
                    .cloned()
                    .collect(),
                crate::compact_remote_v2::RETAINED_MESSAGE_TOKEN_BUDGET,
            );
        let window = {
            let mut state = self.state.lock().await;
            state.start_new_context_window()
        };
        let (window_number, window_ids) = window;
        let (context_updates, world_state_snapshot) = self
            .build_initial_context_with_world_state(step_context, world_state.as_ref())
            .await;
<<<<<<< HEAD
        // Host-built context; stored-data messages were registered first.
        self.services.model_client().register_message_origin(
            &context_items,
            crate::security::ingress::MessageOrigin::Host,
        );
        let turn_context_item = turn_context.to_turn_context_item();
||||||| base
        let turn_context_item = turn_context.to_turn_context_item();
=======
        let context_items =
            crate::context_manager::updates::merge_world_state_updates(context_updates)
                .into_iter()
                .map(ResponseItemEnvelope::new)
                .chain(retained_client_items)
                .collect();
        let turn_context_item = step_context.to_turn_context_item();
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        self.replace_compacted_history(
            context_items,
            turn_context_item,
            world_state_snapshot,
            CompactedHistoryMetadata {
                input_goal_ids,
                message: String::new(),
                window_number,
                window_ids,
                compaction_response_id: None,
                compaction_model_hash: None,
                reviewer_compaction_hash: None,
            },
        )
        .await;
        self.recompute_token_usage(turn_context).await;
    }

    pub(crate) async fn reference_context_item(&self) -> Option<TurnContextItem> {
        let state = self.state.lock().await;
        state.reference_context_item()
    }

    /// Persist the latest turn context snapshot for the first real user turn and for
    /// steady-state turns that emit model-visible context updates.
    ///
    /// When the reference snapshot is missing, this injects full initial context. Otherwise, it
    /// emits only context diffs.
    ///
    /// If full context is injected and a model switch occurred, this prepends the
    /// `<model_switch>` developer message so model-specific instructions are not lost.
    ///
    /// This is the normal runtime path that establishes a new `reference_context_item`.
    /// Compaction also establishes that reference when it installs full initial context
    /// into replacement history. Live world-state changes may
    /// independently advance their in-memory baseline within a turn.
    #[instrument(level = "trace", skip_all)]
    pub(crate) async fn record_context_updates_and_set_reference_context_item(
        &self,
        step_context: &StepContext,
    ) -> CodexResult<Arc<WorldState>> {
        let turn_context = step_context.turn.as_ref();
        let reference_context_item = {
            let state = self.state.lock().await;
            state.reference_context_item()
        };
<<<<<<< HEAD
        let mut turn_context_item = turn_context.to_turn_context_item();
        // PF-23-S03: record the level in force, including a stricter one
        // committed during this session; memory work treats it as a floor.
        if let Ok(policy) = self
            .services
            .agent_control
            .effective_security_policy()
            .snapshot_for_agent(self.thread_id)
        {
            turn_context_item.security_level =
                turn_context_item.security_level.max(Some(policy.level));
        }
        // The recorded level is not model-visible context: a change in it
        // (or its absence in older rollouts) alone does not re-inject.
        let turn_context_changed = reference_context_item.as_ref().is_none_or(|reference| {
            let mut reference = reference.clone();
            reference.security_level = turn_context_item.security_level;
            reference != turn_context_item
        });
        let security_level_changed = reference_context_item
            .as_ref()
            .is_some_and(|reference| reference.security_level != turn_context_item.security_level);
        let world_state = Arc::new(self.build_world_state_for_step(step_context).await?);
        let reinject_host_context = self.services.model_client().take_host_context_reinjection();
        let should_inject_full_context = reference_context_item.is_none() || reinject_host_context;
||||||| base
        let turn_context_item = turn_context.to_turn_context_item();
        let turn_context_changed = reference_context_item.as_ref() != Some(&turn_context_item);
        let should_inject_full_context = reference_context_item.is_none();
        let world_state = Arc::new(self.build_world_state_for_step(step_context).await?);
=======
        let turn_context_item = step_context.to_turn_context_item();
        let turn_context_changed = reference_context_item.as_ref() != Some(&turn_context_item);
        let should_inject_full_context = reference_context_item.is_none();
        let world_state = self
            .build_world_state_for_step(step_context, should_inject_full_context)
            .await?;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        // Full initial context resets the baseline; later turns persist only its changes.
        let (mut context_items, world_state_item) = if should_inject_full_context {
            let (context_updates, snapshot) = self
                .build_initial_context_with_world_state(step_context, &world_state)
                .await;
            let context_items =
                crate::context_manager::updates::merge_world_state_updates(context_updates);
            self.state
                .lock()
                .await
                .history
                .set_world_state_baseline(snapshot.clone());
            (
                context_items,
                Some(WorldStateItem::full(snapshot.into_object())),
            )
        } else {
            let (world_state_items, world_state_item) = {
                let mut state = self.state.lock().await;
                let (fragments, rollout_item) = state.history.update_world_state(&world_state);
                (
                    crate::context_manager::updates::merge_world_state_updates(fragments),
                    rollout_item,
                )
            };
            (world_state_items, world_state_item)
        };
        if !should_inject_full_context && turn_context_changed {
            context_items.extend(
                self.build_turn_context_contribution_items(step_context)
                    .await,
            );
        }
        // A snapshot can change without producing model-visible or TurnContext updates.
        let only_world_state_changed =
            !turn_context_changed && !security_level_changed && context_items.is_empty();
        if only_world_state_changed && world_state_item.is_none() {
            return Ok(Arc::new(world_state));
        }
        if !context_items.is_empty() {
<<<<<<< HEAD
            self.record_conversation_items_from(
                turn_context,
                &context_items,
                Some(crate::security::ingress::MessageOrigin::Host),
||||||| base
            self.record_conversation_items(turn_context, &context_items)
                .await;
=======
            self.record_conversation_items(
                turn_context,
                &step_context.settings.model_info,
                &context_items,
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
            )
            .await;
        }
        // Persist state only after any model-visible context generated from it.
        if let Some(world_state_item) = world_state_item {
            self.persist_rollout_items(&[RolloutItem::WorldState(world_state_item)])
                .await;
        }
        // A snapshot-only change does not require a duplicate TurnContext record.
        if only_world_state_changed {
            return Ok(Arc::new(world_state));
        }
        // Persist one `TurnContextItem` per real user turn so resume/lazy replay can recover the
        // latest durable baseline even when this turn emitted no model-visible context diffs.
        // Checkpoint current thread-owned settings after the first turn's context:
        // the step may predate an acknowledged update or use executor-owned permissions.
        // Serialize with settings updates so an older checkpoint cannot supersede a newer one.
        let _settings_guard = if should_inject_full_context {
            Some(thread_settings::acquire_persistence_lock(self).await)
        } else {
            None
        };
        let mut rollout_items = vec![RolloutItem::TurnContext(turn_context_item.clone())];
        if should_inject_full_context {
            rollout_items.push(RolloutItem::EventMsg(
                thread_settings::applied_event(self).await,
            ));
        }
        self.persist_rollout_items(&rollout_items).await;

        // Advance the persisted-settings baseline even when this turn emitted no model-visible
        // context items.
        let mut state = self.state.lock().await;
        state.set_reference_context_item(Some(turn_context_item));
        Ok(Arc::new(world_state))
    }

    /// Record reported spending without changing the context baseline or estimating usage.
    pub(crate) async fn record_incomplete_response_usage(
        &self,
        turn_context: &TurnContext,
        settings: &ResolvedStepSettings,
        incomplete: &ResponseIncompleteError,
    ) {
        if let Some(response_id) = &incomplete.response_id {
            self.record_response_token_usage(
                turn_context,
                response_id,
                incomplete.token_usage.as_ref(),
            )
            .await;
        }
        // Accounting may exhaust the budget; keep the original incomplete-response reason.
        let _ = self
            .record_token_usage_info(
                turn_context,
                settings,
                incomplete.token_usage.as_ref(),
                TokenUsageContext::SpendingOnly,
            )
            .await;
    }

    pub(crate) async fn update_token_usage_info(
        &self,
        turn_context: &TurnContext,
        token_usage: Option<&TokenUsage>,
        includes_reasoning: bool,
    ) -> CodexResult<()> {
        let result = self
            .record_token_usage_info(
                turn_context,
                &turn_context.initial_settings,
                token_usage,
                TokenUsageContext::Sampling {
                    includes_reasoning,
                    input_item_count: 0,
                },
            )
            .await;
        self.send_token_count_event(turn_context).await;
        result
    }

    pub(crate) async fn record_observed_response_completed(
        &self,
        turn_context: &TurnContext,
        response_id: &str,
        usage: Option<&TokenUsage>,
        usage_metadata: Option<&ResponseUsageMetadata>,
    ) {
        self.send_event(
            turn_context,
            EventMsg::RawResponseCompleted(RawResponseCompletedEvent {
                response_id: response_id.to_string(),
                token_usage: usage.cloned(),
                usage_metadata: usage_metadata.cloned(),
            }),
        )
        .await;
        self.record_response_token_usage(turn_context, response_id, usage)
            .await;
    }

    pub(crate) async fn record_response_token_usage(
        &self,
        turn_context: &TurnContext,
        response_id: &str,
        usage: Option<&TokenUsage>,
    ) {
        let Some(usage) = usage else {
            return;
        };
        let record = self.state.lock().await.record_token_usage(
            self.thread_id,
            &turn_context.sub_id,
            self.session_id(),
            turn_context
                .turn_metadata_state
                .root_turn_id()
                .unwrap_or_else(|| turn_context.sub_id.clone()),
            response_id.to_string(),
            usage,
        );
        self.persist_rollout_items(&[RolloutItem::TokenUsageRecord(record)])
            .await;
    }

    pub(crate) async fn record_token_usage_info(
        &self,
        turn_context: &TurnContext,
        settings: &ResolvedStepSettings,
        token_usage: Option<&TokenUsage>,
        context: TokenUsageContext,
    ) -> CodexResult<()> {
        if let Some(token_usage) = token_usage {
            let token_info = {
                let mut state = self.state.lock().await;
<<<<<<< HEAD
                state
                    .update_token_info_from_usage(token_usage, turn_context.model_context_window());
                // PF-84: attribute the usage to the account that served it, as the
                // accounting ledger does (provider id and account name only).
                if turn_context.config.features.enabled(Feature::NamedAccounts) {
                    let account = turn_context
                        .provider
                        .info()
                        .account
                        .as_ref()
                        .map_or(codex_vault::DEFAULT_PROVIDER_ACCOUNT, |account| {
                            account.name.as_str()
                        });
                    state.record_account_token_usage(
                        &turn_context.config.model_provider_id,
                        account,
                        token_usage,
                    );
                }
                if matches!(
                    turn_context.config.model_auto_compact_token_limit_scope,
                    AutoCompactTokenLimitScope::BodyAfterPrefix
                ) {
                    state.ensure_auto_compact_window_server_prefill_from_usage(token_usage);
||||||| base
                state
                    .update_token_info_from_usage(token_usage, turn_context.model_context_window());
                if matches!(
                    turn_context.config.model_auto_compact_token_limit_scope,
                    AutoCompactTokenLimitScope::BodyAfterPrefix
                ) {
                    state.ensure_auto_compact_window_server_prefill_from_usage(token_usage);
=======
                match context {
                    TokenUsageContext::Sampling {
                        includes_reasoning,
                        input_item_count,
                    } => {
                        state.update_token_info_from_usage(
                            token_usage,
                            turn_context.model_context_window(),
                            includes_reasoning,
                            input_item_count,
                        );
                        if matches!(
                            turn_context.config.model_auto_compact_token_limit_scope,
                            AutoCompactTokenLimitScope::BodyAfterPrefix
                        ) {
                            state.ensure_auto_compact_window_server_prefill_from_usage(token_usage);
                        }
                        state.token_info()
                    }
                    TokenUsageContext::SpendingOnly => {
                        let mut info = state.token_info().unwrap_or(TokenUsageInfo {
                            total_token_usage: TokenUsage::default(),
                            last_usage_includes_reasoning: None,
                            last_token_usage: TokenUsage::default(),
                            model_context_window: turn_context.model_context_window(),
                        });
                        info.total_token_usage.add_assign(token_usage);
                        state.set_token_info(Some(info.clone()));

                        // Contributors receive the reported usage; stored context stays unchanged.
                        info.last_token_usage = token_usage.clone();
                        info.last_usage_includes_reasoning = None;
                        info.model_context_window = turn_context
                            .model_context_window()
                            .or(info.model_context_window);
                        Some(info)
                    }
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
                }
            };
            let turn_state = self
                .input_queue
                .turn_state_for_sub_id(&self.active_turn, &turn_context.sub_id)
                .await;
            if let Some(turn_state) = turn_state {
                turn_state.lock().await.token_usage_by_model.record(
                    settings.selected_collaboration_mode().model(),
                    settings.telemetry(&turn_context.session_telemetry),
                    token_usage,
                );
            }
            let budget_result = self.record_rollout_budget_usage(token_usage).await;
            if let Some(token_info) = token_info.as_ref() {
                for contributor in self.services.extensions.token_usage_contributors() {
                    contributor
                        .on_token_usage(
                            &self.services.session_extension_data,
                            &self.services.thread_extension_data,
                            turn_context.extension_data.as_ref(),
                            token_info,
                        )
                        .await;
                }
            }
            budget_result?;
        }
        Ok(())
    }

    pub(crate) async fn recompute_token_usage(&self, turn_context: &TurnContext) {
        let history = self.clone_history().await;
        let base_instructions = self.get_base_instructions().await;
        let Some(estimated_total_tokens) =
            history.estimate_token_count_with_base_instructions(&base_instructions)
        else {
            return;
        };
        {
            let mut state = self.state.lock().await;
            let mut info = state.token_info().unwrap_or(TokenUsageInfo {
                total_token_usage: TokenUsage::default(),
                last_usage_includes_reasoning: None,
                last_token_usage: TokenUsage::default(),
                model_context_window: None,
            });

            info.last_token_usage = TokenUsage {
                input_tokens: 0,
                cached_input_tokens: 0,
                cache_write_input_tokens: 0,
                output_tokens: 0,
                reasoning_output_tokens: 0,
                total_tokens: estimated_total_tokens.max(0),
                codex_rollout_budget_units: None,
            };

            if let Some(model_context_window) = turn_context.model_context_window() {
                info.model_context_window = Some(model_context_window);
            }

            // The local estimate already includes every retained reasoning item.
            info.last_usage_includes_reasoning = Some(true);
            state
                .history
                .set_token_info_with_boundary(Some(info), history.raw_items().len());
        }
        self.set_auto_compact_window_estimated_prefill_for_scope(
            turn_context,
            estimated_total_tokens,
        )
        .await;
        self.send_token_count_event(turn_context).await;
    }

    pub(crate) async fn update_rate_limits(
        &self,
        turn_context: &TurnContext,
        new_rate_limits: RateLimitSnapshot,
    ) {
        self.record_rate_limits_info(new_rate_limits).await;
        self.send_token_count_event(turn_context).await;
    }

    pub(crate) async fn record_rate_limits_info(&self, new_rate_limits: RateLimitSnapshot) {
        {
            let mut state = self.state.lock().await;
            state.set_rate_limits(new_rate_limits);
        }
    }

    pub(crate) async fn mcp_dependency_prompted(&self) -> HashSet<String> {
        let state = self.state.lock().await;
        state.mcp_dependency_prompted()
    }

    pub(crate) async fn record_mcp_dependency_prompted<I>(&self, names: I)
    where
        I: IntoIterator<Item = String>,
    {
        let mut state = self.state.lock().await;
        state.record_mcp_dependency_prompted(names);
    }

    pub(crate) async fn send_token_count_event(&self, turn_context: &TurnContext) {
        let (info, rate_limits, account_usage) = {
            let state = self.state.lock().await;
            let (info, rate_limits) = state.token_info_and_rate_limits();
            (info, rate_limits, state.account_token_usage())
        };
        let event = EventMsg::TokenCount(TokenCountEvent {
            info,
            rate_limits,
            account_usage,
        });
        self.send_event(turn_context, event).await;
    }

    pub(crate) async fn set_total_tokens_full(&self, turn_context: &TurnContext) {
        if let Some(context_window) = turn_context.model_context_window() {
            let mut state = self.state.lock().await;
            state.set_token_usage_full(context_window);
        }
        self.send_token_count_event(turn_context).await;
    }

    pub(crate) async fn record_response_item_and_emit_turn_item(
        &self,
        turn_context: &TurnContext,
        model_info: &ModelInfo,
        response_item: ResponseItem,
    ) {
        // Add to conversation history and persist response item to rollout.
        self.record_conversation_items(
            turn_context,
            model_info,
            std::slice::from_ref(&response_item),
        )
        .await;

        // Derive a turn item and emit lifecycle events if applicable.
        if let Some(item) = parse_turn_item(&response_item) {
            self.emit_turn_item_started(turn_context, &item).await;
            self.emit_turn_item_completed(turn_context, item).await;
        }
    }

    pub(crate) async fn record_user_prompt_and_emit_turn_item(
        &self,
        turn_context: &TurnContext,
        model_info: &ModelInfo,
        input: &[UserInput],
        client_id: Option<String>,
        metadata: UserInputMetadata,
        persist_context: PersistContext,
    ) {
        // Persist the user message to history, but emit the turn item from `UserInput` so
        // UI-only `text_elements` are preserved. `ResponseItem::Message` does not carry
        // those spans, and `record_response_item_and_emit_turn_item` would drop them.
<<<<<<< HEAD
        let response_item = self.response_item_from_user_input(input.to_vec());
        // Input another agent submitted keeps that agent's standing (PF-30-S02).
        let origin = self
            .services
            .model_client()
            .take_agent_input_origin(input)
            .unwrap_or(crate::security::ingress::MessageOrigin::Human);
        self.record_conversation_items_from(
            turn_context,
            std::slice::from_ref(&response_item),
            Some(origin),
        )
        .await;
||||||| base
        let response_item = self.response_item_from_user_input(input.to_vec());
        self.record_conversation_items(turn_context, std::slice::from_ref(&response_item))
            .await;
=======
        let mut user_image_content_indices = HashMap::new();
        let mut response_item = self.response_item_from_user_input_with_image_positions(
            input.to_vec(),
            &mut user_image_content_indices,
        );
        if let Some(uuid) = metadata.response_item_uuid {
            response_item.set_id(Some(ResponseItemId::for_admitted_user_input(uuid)));
        }
        if metadata.origin == codex_history::UserInputOrigin::Heartbeat
            && let ResponseItem::Message {
                content,
                internal_chat_message_metadata_passthrough: Some(metadata),
                ..
            } = &mut response_item
            && matches!(content.as_slice(), [ContentItem::InputText { .. }])
        {
            metadata.content_item_kinds = Some(vec![ContentItemKind(
                codex_history::HEARTBEAT_CONTENT_KIND.to_owned(),
            )]);
        }
        let (prepared_items, image_preparations) = self
            .prepare_annotated_conversation_items_for_history(
                turn_context,
                model_info,
                vec![ResponseItemEnvelope {
                    item: response_item,
                    metadata: metadata.acceptance_order.map(|order| CodexHarnessMetadata {
                        user_input_order: Some(order),
                        ..Default::default()
                    }),
                }],
            )
            .await;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        let mut user_message_item = UserMessageItem::new(input);
        apply_prepared_image_file_ids(
            &mut user_message_item,
            &prepared_items,
            &user_image_content_indices,
        );
        self.record_prepared_conversation_items(
            turn_context,
            model_info,
            prepared_items,
            image_preparations,
        )
        .await;
        user_message_item.client_id = client_id;
        let turn_item = TurnItem::UserMessage(user_message_item);
        self.emit_turn_item_started(turn_context, &turn_item).await;
        self.emit_turn_item_completed(turn_context, turn_item).await;
        self.ensure_rollout_materialized(persist_context).await;
    }

    pub(crate) async fn notify_stream_error(
        &self,
        turn_context: &TurnContext,
        message: impl Into<String>,
        codex_error: CodexErr,
    ) {
        let additional_details = codex_error.to_string();
        let codex_error_info = CodexErrorInfo::ResponseStreamDisconnected {
            http_status_code: codex_error.http_status_code_value(),
        };
        let event = EventMsg::StreamError(StreamErrorEvent {
            message: message.into(),
            codex_error_info: Some(codex_error_info),
            additional_details: Some(additional_details),
        });
        self.send_event(turn_context, event).await;
    }

<<<<<<< HEAD
    // `authorization_matches` moved next to `pending_approval_shown_under` (S2a, sync-s2a3):
    // it now compares `TurnAuthority` snapshots.

    /// Inject additional user input into the currently active turn.
    ///
    /// Returns the active turn id when accepted.
    #[expect(
        clippy::await_holding_invalid_type,
        reason = "active turn checks and turn state updates must remain atomic"
    )]
    pub async fn steer_input(
        &self,
        input: Vec<UserInput>,
        additional_context: BTreeMap<String, AdditionalContextEntry>,
        expected_turn_id: Option<&str>,
        client_user_message_id: Option<String>,
        responsesapi_client_metadata: Option<HashMap<String, String>>,
    ) -> Result<String, SteerInputError> {
        let mut active = self.active_turn.lock().await;
        let Some(active_turn) = active.as_mut() else {
            return Err(SteerInputError::NoActiveTurn(input));
        };

        let Some(active_task) = active_turn.task.as_ref() else {
            return Err(SteerInputError::NoActiveTurn(input));
        };
        let active_turn_id = &active_task.turn_context.sub_id;

        if let Some(expected_turn_id) = expected_turn_id
            && expected_turn_id != active_turn_id
        {
            return Err(SteerInputError::ExpectedTurnMismatch {
                expected: expected_turn_id.to_string(),
                actual: active_turn_id.clone(),
            });
        }

        match active_task.kind {
            crate::state::TaskKind::Regular => {}
            crate::state::TaskKind::Review => {
                return Err(SteerInputError::ActiveTurnNotSteerable {
                    turn_kind: NonSteerableTurnKind::Review,
                });
            }
            crate::state::TaskKind::Compact => {
                return Err(SteerInputError::ActiveTurnNotSteerable {
                    turn_kind: NonSteerableTurnKind::Compact,
                });
            }
        }

        if input.is_empty() {
            return Err(SteerInputError::EmptyInput);
        }

        // Serialize authorization comparison and admission with settings updates.
        // A new context cannot re-sandbox existing exec sessions: steered work
        // could still send them instructions through write_stdin. Refuse admission
        // instead of changing running work or revoking an already granted approval.
        let mut state = self.state.lock().await;
        if !Self::authorization_matches(&state.session_configuration, &active_task.turn_context) {
            return Err(SteerInputError::AuthorizationChanged(input));
        }
        let additional_context_input = state.additional_context.merge(additional_context);

        if let Some(responsesapi_client_metadata) = responsesapi_client_metadata {
            active_task
                .turn_context
                .turn_metadata_state
                .set_responsesapi_client_metadata(responsesapi_client_metadata);
        }

        let mut pending_input = additional_context_input
            .into_iter()
            .map(ResponseItem::from)
            .map(TurnInput::ResponseItem)
            .collect::<Vec<_>>();
        pending_input.push(TurnInput::UserInput {
            content: input,
            client_id: client_user_message_id,
        });
        self.input_queue
            .extend_pending_input_and_accept_mailbox_delivery_for_turn_state(
                active_turn.turn_state.as_ref(),
                pending_input,
            )
            .await;
        drop(state);
        Ok(active_turn_id.clone())
    }

    #[expect(
        clippy::await_holding_invalid_type,
        reason = "active turn checks and turn state updates must remain atomic"
    )]
    pub(crate) async fn defer_user_input_until_active_turn_finished(
        &self,
        input: Vec<UserInput>,
        additional_context: BTreeMap<String, AdditionalContextEntry>,
        client_user_message_id: Option<String>,
        final_output_json_schema: Option<Value>,
    ) -> Result<String, SteerInputError> {
        let mut active = self.active_turn.lock().await;
        let Some(active_turn) = active.as_mut() else {
            return Err(SteerInputError::NoActiveTurn(input));
        };

        let Some(active_task) = active_turn.task.as_ref() else {
            return Err(SteerInputError::NoActiveTurn(input));
        };

        if input.is_empty() {
            return Err(SteerInputError::EmptyInput);
        }

        let additional_context_input = {
            let mut state = self.state.lock().await;
            state.additional_context.merge(additional_context)
        };

        let mut pending_input = additional_context_input
            .into_iter()
            .map(ResponseItem::from)
            .map(TurnInput::ResponseItem)
            .collect::<Vec<_>>();
        pending_input.push(TurnInput::UserInput {
            content: input,
            client_id: client_user_message_id,
        });
        self.input_queue
            .defer_input_for_turn_state(
                active_turn.turn_state.as_ref(),
                pending_input,
                final_output_json_schema,
            )
            .await;
        Ok(active_task.turn_context.sub_id.clone())
    }

    pub(crate) async fn submit_internal_follow_up(
        &self,
        op: Op,
        client_user_message_id: Option<String>,
    ) -> CodexResult<()> {
        let sub = Submission {
            id: Uuid::now_v7().to_string(),
            op,
            parent_turn_id: None,
            client_user_message_id,
            trace: current_span_w3c_trace_context(),
        };
        self.tx_sub
            .send(sub)
            .await
            .map_err(|_| CodexErr::InternalAgentDied)
    }

||||||| base
    /// Inject additional user input into the currently active turn.
    ///
    /// Returns the active turn id when accepted.
    #[expect(
        clippy::await_holding_invalid_type,
        reason = "active turn checks and turn state updates must remain atomic"
    )]
    pub async fn steer_input(
        &self,
        input: Vec<UserInput>,
        additional_context: BTreeMap<String, AdditionalContextEntry>,
        expected_turn_id: Option<&str>,
        client_user_message_id: Option<String>,
        responsesapi_client_metadata: Option<HashMap<String, String>>,
    ) -> Result<String, SteerInputError> {
        let mut active = self.active_turn.lock().await;
        let Some(active_turn) = active.as_mut() else {
            return Err(SteerInputError::NoActiveTurn(input));
        };

        let Some(active_task) = active_turn.task.as_ref() else {
            return Err(SteerInputError::NoActiveTurn(input));
        };
        let active_turn_id = &active_task.turn_context.sub_id;

        if let Some(expected_turn_id) = expected_turn_id
            && expected_turn_id != active_turn_id
        {
            return Err(SteerInputError::ExpectedTurnMismatch {
                expected: expected_turn_id.to_string(),
                actual: active_turn_id.clone(),
            });
        }

        match active_task.kind {
            crate::state::TaskKind::Regular => {}
            crate::state::TaskKind::Review => {
                return Err(SteerInputError::ActiveTurnNotSteerable {
                    turn_kind: NonSteerableTurnKind::Review,
                });
            }
            crate::state::TaskKind::Compact => {
                return Err(SteerInputError::ActiveTurnNotSteerable {
                    turn_kind: NonSteerableTurnKind::Compact,
                });
            }
        }

        if input.is_empty() {
            return Err(SteerInputError::EmptyInput);
        }

        let additional_context_input = {
            let mut state = self.state.lock().await;
            state.additional_context.merge(additional_context)
        };

        if let Some(responsesapi_client_metadata) = responsesapi_client_metadata {
            active_task
                .turn_context
                .turn_metadata_state
                .set_responsesapi_client_metadata(responsesapi_client_metadata);
        }

        let mut pending_input = additional_context_input
            .into_iter()
            .map(ResponseItem::from)
            .map(TurnInput::ResponseItem)
            .collect::<Vec<_>>();
        pending_input.push(TurnInput::UserInput {
            content: input,
            client_id: client_user_message_id,
        });
        self.input_queue
            .extend_pending_input_and_accept_mailbox_delivery_for_turn_state(
                active_turn.turn_state.as_ref(),
                pending_input,
            )
            .await;
        Ok(active_turn_id.clone())
    }

=======
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
    pub(crate) async fn record_memory_citation_for_turn(&self, sub_id: &str) {
        let turn_state = self
            .input_queue
            .turn_state_for_sub_id(&self.active_turn, sub_id)
            .await;
        let Some(turn_state) = turn_state else {
            return;
        };
        turn_state.lock().await.has_memory_citation = true;
    }

    pub async fn interrupt_task(self: &Arc<Self>) {
        info!("interrupt received: abort current task, if any");
        let had_active_turn = self.active_turn.lock().await.is_some();
        self.abort_all_tasks(TurnAbortReason::Interrupted).await;
        if !had_active_turn {
            self.cancel_mcp_startup();
        }
    }

    pub(crate) fn hooks(&self) -> Arc<Hooks> {
        self.services.hooks.load_full()
    }

    pub(crate) fn user_shell(&self) -> Arc<shell::Shell> {
        Arc::clone(&self.services.user_shell)
    }

    pub(crate) async fn current_rollout_path(&self) -> anyhow::Result<Option<PathBuf>> {
        let Some(live_thread) = self.live_thread() else {
            return Ok(None);
        };
        live_thread.local_rollout_path().await.map_err(Into::into)
    }

    pub(crate) async fn hook_transcript_path(&self) -> Option<PathBuf> {
        let rollout_path = match self.current_rollout_path().await {
            Ok(Some(path)) => path,
            Ok(None) => return None,
            Err(err) => {
                warn!("{err}");
                return None;
            }
        };
        self.ensure_rollout_materialized(PersistContext::Standard)
            .await;
        Some(rollout_path)
    }

    pub(crate) async fn take_pending_session_start_source(
        &self,
    ) -> Option<codex_hooks::SessionStartSource> {
        let mut state = self.state.lock().await;
        state.take_pending_session_start_source()
    }

    fn show_raw_agent_reasoning(&self) -> bool {
        self.services.show_raw_agent_reasoning
    }
}

fn apply_prepared_image_file_ids(
    user_message_item: &mut UserMessageItem,
    prepared_items: &[ResponseItemEnvelope],
    user_image_content_indices: &HashMap<usize, usize>,
) {
    // Preparation replaces image slots in place; resize notices are separate messages.
    // Read the references from the original message while retaining UI-only input spans.
    if let Some(ResponseItemEnvelope {
        item: ResponseItem::Message { content, .. },
        ..
    }) = prepared_items.first()
    {
        for (&input_index, &content_index) in user_image_content_indices {
            if let Some(ContentItem::InputImage {
                image: ImageReference::File { file_id },
                ..
            }) = content.get(content_index)
                && let UserInput::Image { detail, .. } | UserInput::LocalImage { detail, .. } =
                    &user_message_item.content[input_index]
            {
                user_message_item.content[input_index] = UserInput::Image {
                    image: ImageReference::File {
                        file_id: file_id.clone(),
                    },
                    detail: *detail,
                };
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn emit_subagent_session_started(
    analytics_events_client: &AnalyticsEventsClient,
    client_metadata: AppServerClientMetadata,
    session_id: SessionId,
    thread_id: ThreadId,
    parent_thread_id: Option<ThreadId>,
    thread_config: ThreadConfigSnapshot,
    subagent_source: SubAgentSource,
    resumed_created_at: Option<DateTime<Utc>>,
) {
    let initialization_mode = if resumed_created_at.is_some() {
        ThreadInitializationMode::Resumed
    } else {
        ThreadInitializationMode::New
    };
    let created_at =
        u64::try_from(resumed_created_at.unwrap_or_else(Utc::now).timestamp()).unwrap_or_default();
    let AppServerClientMetadata {
        client_name,
        client_version,
    } = client_metadata;
    if (client_name.is_none() || client_version.is_none())
        && subagent_source.kind() != crate::guardian::GUARDIAN_REVIEWER_NAME
    {
        tracing::warn!("skipping subagent thread analytics: missing inherited client metadata");
        return;
    }
    let input = SubAgentThreadStartedInput {
        session_id: session_id.to_string(),
        thread_id: thread_id.to_string(),
        parent_thread_id: parent_thread_id.map(|thread_id| thread_id.to_string()),
        forked_from_thread_id: thread_config
            .forked_from_thread_id
            .map(|thread_id| thread_id.to_string()),
        product_client_id: thread_config.originator.clone(),
        client_name,
        client_version,
        model: thread_config.model,
        ephemeral: thread_config.ephemeral,
        thread_source: thread_config.thread_source,
        subagent_source,
        initialization_mode,
        created_at,
    };
    analytics_events_client.track_subagent_thread_started(input);
}

/// Builds hook configuration for one config snapshot, including any enabled plugin hooks.
async fn build_hooks_config(
    config: &Config,
    plugins_manager: &PluginsManager,
    environment: Option<&TurnEnvironment>,
    disabled_plugin_ids: &[String],
) -> HooksConfig {
    let (hook_shell_program, hook_shell_argv) = environment
        .and_then(|environment| environment.shell.as_ref())
        .map(|shell| {
            let mut argv = shell.derive_exec_args("", /*use_login_shell*/ false);
            let program = argv.remove(0);
            let _ = argv.pop();
            (Some(program), argv)
        })
        .unwrap_or_default();
    let plugins_input = config.plugins_config_input();
    let plugin_outcome = plugins_manager
        .plugins_for_config(&plugins_input)
        .await
        .without_plugins(disabled_plugin_ids);
    let plugin_hook_sources = plugin_outcome.effective_plugin_hook_sources();
    let plugin_hook_load_warnings = plugin_outcome.effective_plugin_hook_warnings();
    HooksConfig {
        legacy_notify_argv: config.notify.clone(),
        feature_enabled: config.features.enabled(Feature::CodexHooks),
        bypass_hook_trust: config.bypass_hook_trust,
        config_layer_stack: Some(config.config_layer_stack.clone()),
        plugin_hook_sources,
        plugin_hook_load_warnings,
        shell_program: hook_shell_program,
        shell_args: hook_shell_argv,
    }
}

#[cfg(test)]
#[path = "elicitation_holders_tests.rs"]
mod elicitation_holders_tests;

#[cfg(test)]
pub(crate) mod tests;

#[cfg(test)]
<<<<<<< HEAD
#[path = "pending_mailbox_wake_tests.rs"]
mod pending_mailbox_wake_tests;

#[cfg(test)]
#[path = "account_switch_tests.rs"]
mod account_switch_tests;

#[cfg(test)]
#[path = "home_access_session_tests.rs"]
mod home_access_session_tests;

#[cfg(test)]
#[path = "stale_approval_tests.rs"]
mod stale_approval_tests;

#[cfg(test)]
#[path = "withdrawn_approval_tests.rs"]
mod withdrawn_approval_tests;
||||||| base
=======
#[path = "world_state_item_tests.rs"]
mod world_state_item_tests;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
