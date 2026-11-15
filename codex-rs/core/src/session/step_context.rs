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
use codex_utils_path_uri::PathUri;
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
    /// Each context is built once, so every operation first checks that no security
    /// transition was committed since (#2183 review): one made during the step refuses the
    /// step's file access instead of using permissions decided under the earlier policy.
    pub(crate) async fn environments<'a>(
        &'a self,
        session: &'a Session,
    ) -> Vec<(&'a TurnEnvironment, impl EnvironmentAccess + 'a)> {
        // Taken before the contexts are built, so a transition while they are built counts.
        let decided_under = session.services.authority_marker();
        let mut environments = Vec::new();
        for environment in self.environments.turn_environments() {
            let context = self.file_access_context(session, environment).await;
            environments.push((
                environment,
                StepFileAccess {
                    inner: FileSystemEnvironmentAccessor::new(
                        environment.environment.filesystem_ref(),
                        context,
                    ),
                    session,
                    decided_under,
                },
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
            environment,
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

/// A step's file access in one environment: its context was built under `decided_under`
/// (D50's policy marker); after a security transition every operation is refused.
struct StepFileAccess<'a> {
    inner: FileSystemEnvironmentAccessor<'a>,
    session: &'a Session,
    decided_under: Option<(u64, u64)>,
}

/// Why a step's file access was refused after a security transition.
pub(crate) const STEP_FILE_ACCESS_STALE: &str = "file access refused: the security policy \
     changed during this step; it is decided again on the next step";

impl StepFileAccess<'_> {
    fn stale(&self) -> Option<std::io::Error> {
        self.session
            .services
            .transition_since(self.decided_under)
            .then(|| {
                std::io::Error::new(std::io::ErrorKind::PermissionDenied, STEP_FILE_ACCESS_STALE)
            })
    }
}

/// Forwards one operation to the step's accessor unless a transition made it stale.
macro_rules! checked {
    ($self:ident, $call:expr) => {
        match $self.stale() {
            Some(error) => Box::pin(async move { Err(error) }),
            None => $call,
        }
    };
}

impl EnvironmentAccess for StepFileAccess<'_> {
    fn cache_key(&self) -> codex_file_system::EnvironmentAccessKey {
        self.inner.cache_key()
    }

    fn canonicalize<'b>(
        &'b self,
        path: &'b PathUri,
    ) -> codex_file_system::ExecutorFileSystemFuture<'b, PathUri> {
        checked!(self, self.inner.canonicalize(path))
    }

    fn read_file<'b>(
        &'b self,
        path: &'b PathUri,
        options: codex_file_system::ReadFileOptions,
    ) -> codex_file_system::ExecutorFileSystemFuture<'b, Vec<u8>> {
        checked!(self, self.inner.read_file(path, options))
    }

    fn read_file_stream<'b>(
        &'b self,
        path: &'b PathUri,
    ) -> codex_file_system::ExecutorFileSystemFuture<'b, codex_file_system::FileSystemReadStream>
    {
        checked!(self, self.inner.read_file_stream(path))
    }

    fn write_file_stream<'b>(
        &'b self,
        path: &'b PathUri,
        contents: futures::stream::BoxStream<
            'b,
            codex_file_system::FileSystemResult<codex_file_system::Bytes>,
        >,
    ) -> codex_file_system::ExecutorFileSystemFuture<'b, u64> {
        checked!(self, self.inner.write_file_stream(path, contents))
    }

    fn write_file<'b>(
        &'b self,
        path: &'b PathUri,
        contents: Vec<u8>,
        options: codex_file_system::WriteFileOptions,
    ) -> codex_file_system::ExecutorFileSystemFuture<'b, ()> {
        checked!(self, self.inner.write_file(path, contents, options))
    }

    fn create_directory<'b>(
        &'b self,
        path: &'b PathUri,
        options: codex_file_system::CreateDirectoryOptions,
    ) -> codex_file_system::ExecutorFileSystemFuture<'b, ()> {
        checked!(self, self.inner.create_directory(path, options))
    }

    fn get_metadata<'b>(
        &'b self,
        path: &'b PathUri,
        options: codex_file_system::GetMetadataOptions,
    ) -> codex_file_system::ExecutorFileSystemFuture<'b, codex_file_system::FileMetadata> {
        checked!(self, self.inner.get_metadata(path, options))
    }

    fn read_directory<'b>(
        &'b self,
        path: &'b PathUri,
    ) -> codex_file_system::ExecutorFileSystemFuture<'b, Vec<codex_file_system::ReadDirectoryEntry>>
    {
        checked!(self, self.inner.read_directory(path))
    }

    fn walk<'b>(
        &'b self,
        path: &'b PathUri,
        options: codex_file_system::WalkOptions,
    ) -> codex_file_system::ExecutorFileSystemFuture<'b, codex_file_system::WalkOutcome> {
        checked!(self, self.inner.walk(path, options))
    }

    fn remove<'b>(
        &'b self,
        path: &'b PathUri,
        options: codex_file_system::RemoveOptions,
    ) -> codex_file_system::ExecutorFileSystemFuture<'b, ()> {
        checked!(self, self.inner.remove(path, options))
    }

    fn copy<'b>(
        &'b self,
        source_path: &'b PathUri,
        destination_path: &'b PathUri,
        options: codex_file_system::CopyOptions,
    ) -> codex_file_system::ExecutorFileSystemFuture<'b, ()> {
        checked!(
            self,
            self.inner.copy(source_path, destination_path, options)
        )
    }
}

#[cfg(test)]
#[path = "step_context_tests.rs"]
mod tests;
