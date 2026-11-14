//! Request-scoped settings and capabilities, with live grants bound to the originating turn.

use std::sync::Arc;

use crate::agents_md::LoadedAgentsMd;
use crate::config::TokenBudgetConfig;
use crate::environment_selection::TurnEnvironmentSnapshot;
use crate::realtime_conversation::RealtimeConversationSnapshot;
use crate::session::session::Session;
use crate::session::step_settings::ResolvedStepSettings;
use crate::session::turn_context::TurnContext;
use crate::session::turn_context::TurnEnvironment;
use crate::tools::router::ToolRouter;
use codex_extension_api::ExtensionData;
use codex_file_system::EnvironmentAccess;
use codex_file_system::FileSystemEnvironmentAccessor;
use codex_file_system::FileSystemSandboxContext;
use codex_mcp::McpBinding;
use codex_otel::SessionTelemetry;
use codex_plugin::ExecutorPluginHookSource;
use codex_protocol::items::ModelInvocationContext;
use codex_protocol::models::AdditionalPermissionProfile;
use codex_protocol::protocol::TurnContextItem;
use tokio_util::sync::CancellationToken;

/// Request-scoped state that may change between model sampling requests.
#[derive(Clone)]
pub(crate) struct StepContext {
    /// Final serving value for this invocation; retained by its code cell across waits.
    pub(crate) serving_assessment: Option<Arc<crate::guardian::ServingAssessment>>,
    pub(crate) turn: Arc<TurnContext>,
    /// Preempts this request and yields its code-mode observations when user input arrives.
    pub(crate) preempt: Option<CancellationToken>,
    /// Realtime call activity and instructions captured for this sampling request.
    pub(crate) realtime: RealtimeConversationSnapshot,
    /// One immutable settings version captured before request preparation.
    pub(crate) settings: Arc<ResolvedStepSettings>,
    /// Frozen turn preferences resolved against this step's captured model.
    pub(crate) token_budget: Option<TokenBudgetConfig>,
    /// Telemetry context tagged with this sampling request's model.
    pub(crate) session_telemetry: SessionTelemetry,
    pub(crate) environments: TurnEnvironmentSnapshot,
    /// Child bindings captured with this step; configured-only when opted in.
    pub(crate) inherited_environments: TurnEnvironmentSnapshot,
    /// Admitted executor hooks bound to this step's discovery, plugin policy, and MCP routing.
    pub(crate) executor_hook_sources: Vec<ExecutorPluginHookSource>,
    /// Keeps the extension inputs used to build this step's tools for its World State as well.
    pub(crate) extension_data: Arc<ExtensionData>,
    /// The exact MCP connections, configuration, and catalog captured for this step.
    pub(crate) mcp: Arc<McpBinding>,
    /// The finalized tool plan advertised and executed for this exact sampling request.
    pub(crate) tool_router: Arc<ToolRouter>,
    /// The canonical AGENTS.md value observed with this environment snapshot.
    pub(crate) loaded_agents_md: Option<Arc<LoadedAgentsMd>>,
}

impl StepContext {
    /// Whether new context windows should record tool declarations in history.
    pub(crate) fn incremental_tools_enabled(&self) -> bool {
        self.settings.model_info.use_responses_lite
            && self
                .turn
                .config
                .features
                .enabled(codex_features::Feature::IncrementalTools)
    }

    /// Pairs the step's environments with access, each through [`Self::file_access_context`].
    pub(crate) async fn environments(
        &self,
        session: &Session,
    ) -> Vec<(&TurnEnvironment, impl EnvironmentAccess + '_)> {
        let mut environments = Vec::new();
        for environment in self.environments.turn_environments() {
            let context = self.file_access_context(session, environment).await;
            environments.push((
                environment,
                FileSystemEnvironmentAccessor::new(
                    environment.environment.filesystem_ref(),
                    context,
                ),
            ));
        }
        environments
    }

    /// File access in `environment` with the current session and originating-turn grants
    /// ([`Self::usable_grants`]), through Corbanu's file-tool sandbox context (PF-27-S02 launch
    /// contract, D491 refusal where the host has no platform sandbox) and PF-23-S01's read
    /// denials once they apply, as main's extension file access had.
    pub(crate) async fn file_access_context(
        &self,
        session: &Session,
        environment: &TurnEnvironment,
    ) -> FileSystemSandboxContext {
        let grants = self
            .usable_grants(session, &environment.selection.environment_id)
            .await;
        crate::security::protected_surface::protect_file_tool_context(
            session,
            &self.turn,
            self.turn.file_system_sandbox_context(grants, environment),
        )
    }

    /// The session's and the originating turn's grants for `environment_id`, fenced by the
    /// policy marker (D51, D520 (a)), as file access may use them: #245, read through the
    /// session, which strips protected paths while the protected-path rules apply.
    pub(crate) async fn usable_grants(
        &self,
        session: &Session,
        environment_id: &str,
    ) -> Option<AdditionalPermissionProfile> {
        session
            .usable_grant(self.turn.granted_permissions(environment_id))
            .await
    }

    /// Persist the context captured for this request, even after a live update.
    pub(crate) fn to_turn_context_item(&self) -> TurnContextItem {
        let mut item = self.turn.to_turn_context_item();
        item.realtime_active = Some(self.realtime.active);
        item
    }

    pub(crate) fn model_context(&self) -> ModelInvocationContext {
        ModelInvocationContext {
            model_slug: self.settings.model_info.slug.clone(),
            reasoning_effort: self
                .settings
                .effective_reasoning_effort()
                .map(|effort| effort.to_string()),
        }
    }
}

#[cfg(test)]
#[path = "step_context_tests.rs"]
mod tests;
