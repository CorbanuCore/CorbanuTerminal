use super::step_settings::ResolvedStepSettings;
use super::token_budget::has_explicit_settings;
use super::token_budget::resolve_token_budget;
use super::*;
use crate::config::TokenBudgetConfig;
use crate::cyber_access_program;
use crate::environment_selection::EnvironmentConfigOrigin;
use crate::environment_selection::TurnEnvironmentSnapshot;
use crate::exec_policy::AllowPrefixRules;
use crate::shell_snapshot::ShellSnapshot;
use crate::shell_snapshot::ShellSnapshotFile;
use crate::shell_snapshot::ShellSnapshotSandbox;
use crate::tools::sandboxing::configured_windows_sandbox_selection;
use crate::tools::sandboxing::executor_windows_sandbox_selection;
use arc_swap::ArcSwap;
use codex_core_plugins::PluginCommandAttribution;
use codex_core_plugins::ResolvedPluginMetricsOperation;
use codex_core_plugins::TrustedPluginRoots;
use codex_exec_server::ExecutorFileSystem;
use codex_extension_api::SelectedPluginSnapshot;
use codex_file_system::EnvironmentAccess;
use codex_file_system::FileSystemEnvironmentAccessor;
use codex_file_system::FileSystemSandboxContext;
use codex_model_provider::SharedModelProvider;
use codex_protocol::SessionId;
use codex_protocol::ThreadId;
use codex_protocol::config_types::ShellEnvironmentPolicy;
use codex_protocol::models::AdditionalPermissionProfile;
use codex_protocol::openai_models::CatalogModel;
use codex_protocol::openai_models::MODEL_SPECIALTY_CYBER;
use codex_protocol::openai_models::ModelOrigin;
use codex_protocol::permissions::RawFileSystemSandboxPolicy;
use codex_protocol::protocol::EnvironmentConfig;
use codex_protocol::protocol::EnvironmentConfigState;
use codex_protocol::protocol::ErrorEvent;
use codex_protocol::protocol::MultiAgentVersion;
use codex_protocol::protocol::ThreadHistoryMode;
use codex_protocol::protocol::TurnEnvironmentSelection;
use codex_protocol::sandbox::SandboxOverride;
use codex_protocol::turn_input::CyberAccessProgram;
use codex_sandboxing::policy_transforms::effective_permission_profile;
use codex_sandboxing::policy_transforms::merge_permission_profiles;
use codex_skills_extension::HostSkillsSnapshot;
use codex_skills_extension::SkillLoadOutcome;
use codex_utils_path_uri::PathUri;
use codex_utils_plugins::PluginIdentity;
use futures::FutureExt;
use futures::future::BoxFuture;
use futures::future::Shared;
use std::path::PathBuf;
use std::sync::Mutex as StdMutex;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicU8;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use tracing::instrument;

pub(crate) type ShellSnapshotTask = Shared<BoxFuture<'static, Option<Arc<ShellSnapshotFile>>>>;
pub(crate) type ShellSnapshotCache =
    Arc<Mutex<HashMap<ShellSnapshotCacheKey, Arc<ShellSnapshotFile>>>>;

#[derive(Eq, Hash, PartialEq)]
pub(crate) struct ShellSnapshotCacheKey {
    cwd: AbsolutePathBuf,
    shell_path: PathBuf,
    allow_login_shell: bool,
    sandbox: String,
}

const STRICT_APPLY_PATCH_ONE_GRAMMAR_FAILURE: u8 = 1;
const STRICT_APPLY_PATCH_FALLBACK_ACTIVE: u8 = 2;

#[derive(Debug, Default)]
pub(crate) struct ModelEditProtocolState {
    strict_apply_patch_grammar_state: AtomicU8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PatchFallbackTransition {
    Unchanged { consecutive_failures: u8 },
    Activated { consecutive_failures: u8 },
}

impl PatchFallbackTransition {
    pub(crate) fn consecutive_failures(self) -> u8 {
        match self {
            Self::Unchanged {
                consecutive_failures,
            }
            | Self::Activated {
                consecutive_failures,
            } => consecutive_failures,
        }
    }
}

impl ModelEditProtocolState {
    fn record_grammar_failure(&self) -> PatchFallbackTransition {
        let mut current = self
            .strict_apply_patch_grammar_state
            .load(Ordering::Acquire);
        loop {
            if current >= STRICT_APPLY_PATCH_FALLBACK_ACTIVE {
                return PatchFallbackTransition::Unchanged {
                    consecutive_failures: STRICT_APPLY_PATCH_FALLBACK_ACTIVE,
                };
            }
            let next = current.saturating_add(1);
            match self.strict_apply_patch_grammar_state.compare_exchange(
                current,
                next,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) if next == STRICT_APPLY_PATCH_FALLBACK_ACTIVE => {
                    return PatchFallbackTransition::Activated {
                        consecutive_failures: next,
                    };
                }
                Ok(_) => {
                    return PatchFallbackTransition::Unchanged {
                        consecutive_failures: next,
                    };
                }
                Err(observed) => current = observed,
            }
        }
    }

    fn record_parse_success(&self) {
        let mut current = self
            .strict_apply_patch_grammar_state
            .load(Ordering::Acquire);
        while current == STRICT_APPLY_PATCH_ONE_GRAMMAR_FAILURE {
            match self.strict_apply_patch_grammar_state.compare_exchange(
                current,
                0,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => return,
                Err(observed) => current = observed,
            }
        }
    }

    fn fallback_enabled(&self) -> bool {
        self.strict_apply_patch_grammar_state
            .load(Ordering::Acquire)
            >= STRICT_APPLY_PATCH_FALLBACK_ACTIVE
    }
}

#[derive(Debug, Default)]
pub(crate) struct ExplicitToolBudgetState {
    shell_command_limit: AtomicU64,
    shell_commands_used: AtomicU64,
}

#[derive(Debug, Default)]
pub(crate) struct MalformedToolCallState {
    counts: Mutex<HashMap<String, u8>>,
}

impl MalformedToolCallState {
    async fn record(&self, signature: String) -> u8 {
        let mut counts = self.counts.lock().await;
        let count = counts.entry(signature).or_insert(0);
        *count = count.saturating_add(1);
        *count
    }
}

/// Tracks repeated direct tool calls across every model request in a turn.
/// Each sampling request builds a fresh tool runtime, so a model that re-issues
/// one call per request, or cycles through the same few calls, is only visible
/// at turn scope.
#[derive(Debug, Default)]
pub(crate) struct RepeatedToolCallState {
    streak: Mutex<Option<(String, u32)>>,
    /// Per call signature: every result identity it has returned this turn,
    /// and how many times it has returned an already-seen result since the
    /// turn last learned something new.
    results: Mutex<HashMap<String, (HashSet<String>, u32)>>,
    refusals: AtomicU32,
}

/// How a direct call repeats earlier ones in the same turn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ToolCallRepetition {
    /// Times in a row this exact call has now been made.
    pub(crate) consecutive_calls: u32,
    /// Times this exact call has already returned its latest result.
    pub(crate) identical_results: u32,
}

impl RepeatedToolCallState {
    async fn record_call(&self, signature: &str) -> ToolCallRepetition {
        let consecutive_calls = {
            let mut streak = self.streak.lock().await;
            let count = match streak.as_ref() {
                Some((last, count)) if last == signature => count.saturating_add(1),
                _ => 1,
            };
            *streak = Some((signature.to_string(), count));
            count
        };
        let identical_results = self
            .results
            .lock()
            .await
            .get(signature)
            .map_or(0, |(_, count)| *count);
        ToolCallRepetition {
            consecutive_calls,
            identical_results,
        }
    }

    /// Records a call's result. A result this call has never returned before
    /// in the turn is progress (for example the output of a new edit, or a
    /// test that now reports something new), so every repeat count restarts.
    /// Results seen before, including ones a model oscillates between by
    /// toggling an edit back and forth, are not progress.
    async fn record_result(&self, signature: String, identity: String) {
        let mut results = self.results.lock().await;
        if let Some((seen, count)) = results.get_mut(&signature)
            && seen.contains(&identity)
        {
            *count = count.saturating_add(1);
            return;
        }
        for (_, count) in results.values_mut() {
            *count = 0;
        }
        let (seen, count) = results.entry(signature).or_default();
        seen.insert(identity);
        *count = 1;
    }

    fn record_refusal(&self) -> u32 {
        self.refusals
            .fetch_add(1, Ordering::AcqRel)
            .saturating_add(1)
    }
}

#[derive(Clone)]
pub(crate) struct TurnEnvironment {
    pub(crate) selection: TurnEnvironmentSelection,
    pub(crate) config_origin: EnvironmentConfigOrigin,
    pub(crate) environment: Arc<Environment>,
    /// Cached from the selected executor; `None` means it did not report one.
    pub(crate) user_home_dir: Option<PathUri>,
    /// Cached from the selected executor; `None` means it did not report them.
    pub(crate) temporary_directories: Option<Vec<PathUri>>,
    pub(crate) shell: Option<shell::Shell>,
    /// OS reported by the selected executor; `None` for legacy executors.
    pub(crate) executor_platform_os: Option<String>,
    pub(crate) shell_snapshot: ShellSnapshotTask,
    pub(crate) shell_snapshot_builder: Option<Box<ShellSnapshot>>,
    pub(crate) shell_snapshot_cache: ShellSnapshotCache,
    pub(crate) shell_snapshot_v2_supported: bool,
}

impl TurnEnvironment {
    pub(crate) fn new(
        selection: TurnEnvironmentSelection,
        config_origin: EnvironmentConfigOrigin,
        environment: Arc<Environment>,
        shell: Option<shell::Shell>,
    ) -> Self {
        debug_assert!(matches!(selection.config, EnvironmentConfigState::Ready(_)));
        Self {
            selection,
            config_origin,
            environment,
            user_home_dir: None,
            temporary_directories: None,
            shell,
            executor_platform_os: None,
            shell_snapshot: futures::future::ready(None).boxed().shared(),
            shell_snapshot_builder: None,
            shell_snapshot_cache: Arc::default(),
            shell_snapshot_v2_supported: false,
        }
    }

    pub(crate) fn config(&self) -> &EnvironmentConfig {
        let EnvironmentConfigState::Ready(config) = &self.selection.config else {
            unreachable!("ready turn environments always carry resolved configuration")
        };
        config
    }

    pub(crate) fn shell_environment_policy(&self) -> &ShellEnvironmentPolicy {
        &self.config().shell_environment_policy
    }

    #[cfg(test)]
    pub(crate) fn config_mut(&mut self) -> &mut EnvironmentConfig {
        let EnvironmentConfigState::Ready(config) = &mut self.selection.config else {
            unreachable!("ready turn environments always carry resolved configuration")
        };
        config
    }

    pub(crate) async fn shell_snapshot(
        &self,
        cwd: &AbsolutePathBuf,
        command: &[String],
        shell: &shell::Shell,
        config: &Config,
        sandbox: Option<ShellSnapshotSandbox>,
    ) -> Option<Arc<ShellSnapshotFile>> {
        let credential_broker_enabled = config
            .permissions
            .network
            .as_ref()
            .is_some_and(|spec| spec.enabled() && spec.credential_broker_enabled());
        let cwd_matches_selection = self.selection.cwd == PathUri::from_abs_path(cwd);
        if !config.features.enabled(Feature::ShellSnapshot)
            || (!cwd_matches_selection && !credential_broker_enabled)
            || command.len() < 3
            || !matches!(command[1].as_str(), "-lc" | "-c")
            || (command[1] == "-c" && !credential_broker_enabled)
        {
            return None;
        }
        if credential_broker_enabled {
            let sandbox = sandbox?;
            let use_login_shell = command[1] == "-lc";
            let shell_snapshot_builder = self.shell_snapshot_builder.as_ref()?;
            let key = ShellSnapshotCacheKey {
                cwd: cwd.clone(),
                shell_path: shell.shell_path.clone(),
                allow_login_shell: use_login_shell,
                sandbox: sandbox
                    .cache_key()
                    .inspect_err(|err| {
                        tracing::warn!("Failed to identify shell snapshot sandbox: {err:?}");
                    })
                    .ok()?,
            };
            if let Some(snapshot) = self
                .shell_snapshot_cache
                .lock()
                .await
                .get(&key)
                .filter(|snapshot| {
                    snapshot.path().as_path().is_file()
                        && snapshot.is_brokered_for(self.shell_environment_policy())
                })
                .cloned()
            {
                return Some(snapshot);
            }
            // Captures keep this command's cancellation token. Retry once if brokerage
            // changes during startup, without retrying failed or cancelled captures.
            for _ in 0..2 {
                let snapshot = shell_snapshot_builder
                    .as_ref()
                    .clone()
                    .build(
                        Arc::clone(&self.environment),
                        PathUri::from_abs_path(cwd),
                        Some(shell.clone()),
                        use_login_shell,
                        self.shell_environment_policy().clone(),
                        Some(sandbox.clone()),
                    )
                    .await?;
                if !snapshot.is_brokered_for(self.shell_environment_policy()) {
                    continue;
                }
                let mut snapshots = self.shell_snapshot_cache.lock().await;
                if snapshots.len() >= 32 && !snapshots.contains_key(&key) {
                    snapshots.clear();
                }
                snapshots.insert(key, Arc::clone(&snapshot));
                return Some(snapshot);
            }
            None
        } else {
            self.shell_snapshot.peek()?.clone().filter(|snapshot| {
                &snapshot.shell_environment_policy == self.shell_environment_policy()
            })
        }
    }

    pub(crate) fn cwd(&self) -> &PathUri {
        &self.selection.cwd
    }

    pub(crate) fn workspace_roots(&self) -> &[PathUri] {
        &self.config().workspace_roots
    }

    pub(crate) fn permission_profile(&self) -> &PermissionProfile {
        self.config().permission_profile.permission_profile()
    }

    /// Borrows this snapshot's filesystem using its permissions and any additional grants.
    pub(crate) fn fs_accessor(
        &self,
        additional_permissions: Option<AdditionalPermissionProfile>,
    ) -> impl EnvironmentAccess + '_ {
        FileSystemEnvironmentAccessor::new(
            self.environment.filesystem_ref(),
            self.sandbox_context(additional_permissions),
        )
    }

    /// Sandbox context for this environment, including any additional permission grants.
    pub(crate) fn sandbox_context(
        &self,
        additional_permissions: Option<AdditionalPermissionProfile>,
    ) -> FileSystemSandboxContext {
        let config = self.config();
        // Grant-adjusted permissions take precedence over the environment's baseline;
        // paths and sandbox backend settings remain environment-owned.
        let permissions = effective_permission_profile(
            self.permission_profile(),
            additional_permissions.as_ref(),
        );
        FileSystemSandboxContext {
            sandbox_override: SandboxOverride::NoOverride,
            permissions,
            cwd: self.cwd().clone(),
            workspace_roots: self.workspace_roots().to_vec(),
            user_home_dir: self.user_home_dir.clone(),
            temporary_directories: self.temporary_directories.clone(),
            windows_sandbox_selection: executor_windows_sandbox_selection(
                config.windows_sandbox_type,
                config.windows_sandbox_level,
                self.cwd(),
            ),
            windows_sandbox_proxy_settings_mode: None,
            use_legacy_landlock: config.use_legacy_landlock,
        }
    }

    pub(crate) fn windows_sandbox_selection_for_turn_metadata(
        &self,
    ) -> codex_file_system::WindowsSandboxSelection {
        let config = self.config();
        if self.environment.is_remote() {
            executor_windows_sandbox_selection(
                config.windows_sandbox_type,
                config.windows_sandbox_level,
                self.cwd(),
            )
        } else {
            configured_windows_sandbox_selection(
                config.windows_sandbox_type,
                config.windows_sandbox_level,
                self.cwd(),
            )
        }
    }

    pub(crate) fn active_permission_profile(&self) -> Option<ActivePermissionProfile> {
        self.config().permission_profile.active_permission_profile()
    }

    pub(crate) fn permission_profile_with_workspace_roots(&self) -> PermissionProfile {
        self.permission_profile()
            .clone()
            .materialize_project_roots_with_path_uris(self.workspace_roots())
    }

    pub(crate) fn selection(&self) -> TurnEnvironmentSelection {
        self.config_origin
            .into_input_selection(self.selection.clone())
    }
}

impl std::fmt::Debug for TurnEnvironment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TurnEnvironment")
            .field("environment_id", &self.selection.environment_id)
            .field("environment", &self.environment)
            .field("cwd", &self.selection.cwd)
            .field("workspace_roots", &self.config().workspace_roots)
            .field("user_home_dir", &self.user_home_dir)
            .field("temporary_directories", &self.temporary_directories)
            .field("shell", &self.shell)
            .field("executor_platform_os", &self.executor_platform_os)
            .field("config", self.config())
            .field("config_origin", &self.config_origin)
            .finish_non_exhaustive()
    }
}

/// Request options for one turn, separate from persistent thread settings.
#[derive(Default)]
pub(crate) struct NewTurnContextOptions {
    pub(crate) final_output_json_schema: Option<Value>,
    pub(crate) cyber_access_program: Option<CyberAccessProgram>,
}

/// The context needed for a single turn of the thread.
#[derive(Debug)]
pub struct TurnContext {
    pub(crate) sub_id: String,
    pub(crate) trace_id: Option<String>,
    /// Call state at turn creation; model requests use the `StepContext` snapshot.
    pub(crate) realtime_active: bool,
    pub(crate) code_mode_available: bool,
    /// Turn-scoped configuration. Read step-specific settings such as service tier and
    /// approvals reviewer from the corresponding `StepContext` instead.
    pub config: Arc<Config>,
    /// Preferences captured before token-budget defaults from the turn's initial model.
    pub(crate) configured_token_budget: Option<TokenBudgetConfig>,
    /// Captured once so later steps do not re-read config layers to detect user preferences.
    pub(crate) use_model_token_budget_defaults: bool,
    pub(crate) auth_manager: Option<Arc<AuthManager>>,
    /// Frozen settings used to construct this context. Legacy turn consumers
    /// keep this view even when later steps use different settings.
    pub(crate) initial_settings: Arc<ResolvedStepSettings>,
    /// Thread-owned plugin selection captured when this turn was admitted.
    pub(crate) disabled_plugin_ids: Vec<String>,
    pub(super) active_host_plugin_identities: Option<Vec<PluginIdentity>>,
    /// Settings for the next step; environments are owned by `ThreadEnvironments`.
    pub(super) next_step_settings: ArcSwap<ResolvedStepSettings>,
    /// Turn-wide telemetry; model-attributed step work should use `StepContext::session_telemetry`.
    pub(crate) session_telemetry: SessionTelemetry,
    pub(crate) provider: SharedModelProvider,
    pub(crate) session_source: SessionSource,
    pub(crate) history_mode: ThreadHistoryMode,
    pub(crate) parent_thread_id: Option<ThreadId>,
    pub(crate) originator: String,
    /// Initial selection retained for legacy turn consumers. Step work uses StepContext.
    // TODO(sayan): Migrate remaining step consumers to their StepContext's environments.
    pub(crate) initial_environments: TurnEnvironmentSnapshot,
    /// The session's absolute working directory. All relative paths provided
    /// by the model as well as sandbox policies are resolved against this path
    /// instead of `std::env::current_dir()`.
    #[deprecated(note = "use the selected turn environment cwd instead")]
    pub(crate) cwd: AbsolutePathBuf,
    pub(crate) current_date: Option<String>,
    pub(crate) timezone: Option<String>,
    pub(crate) app_server_client_name: Option<String>,
    pub(crate) developer_instructions: Option<String>,
    pub(crate) multi_agent_version: MultiAgentVersion,
    pub(crate) network: Option<NetworkProxy>,
    // TODO(anp): Reconcile this parallel turn snapshot with TurnEnvironment::sandbox_context
    // so owner-provided environment settings govern the remaining sandbox decisions.
    pub(crate) windows_sandbox_level: WindowsSandboxLevel,
    pub(crate) available_models: Vec<ModelPreset>,
    pub(crate) final_output_json_schema: Option<Value>,
    pub(crate) turn_metadata_state: Arc<TurnMetadataState>,
    pub(crate) extension_data: Arc<codex_extension_api::ExtensionData>,
    pub(crate) turn_timing_state: Arc<TurnTimingState>,
    /// Whether this turn was admitted from collaboration work that requested a model turn.
    /// Direct operator turns in a managed child pane must not report their result to the
    /// child's structural parent.
    pub(crate) parent_completion_expected: Arc<AtomicBool>,
    pub(crate) terminal_error: Arc<Mutex<Option<ErrorEvent>>>,
<<<<<<< HEAD
    pub(crate) model_edit_protocol_state: Arc<ModelEditProtocolState>,
    pub(crate) explicit_tool_budget_state: Arc<ExplicitToolBudgetState>,
    pub(crate) malformed_tool_call_state: Arc<MalformedToolCallState>,
    pub(crate) repeated_tool_call_state: Arc<RepeatedToolCallState>,
    pub(crate) pending_tool_calls: Arc<crate::tools::parallel::PendingToolCalls>,
||||||| 413492cd6c
=======
    /// Shares the session-owned store; session approvals remain live across turns.
    session_grants: Arc<StdMutex<HashMap<String, AdditionalPermissionProfile>>>,
    /// Retained by background cells so a later active turn cannot replace their grants.
    turn_grants: Arc<StdMutex<TurnGrants>>,
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
    pub(crate) server_model_warning_emitted: AtomicBool,
    pub(crate) provider_cache_pressure_warning_emitted: AtomicBool,
    pub(crate) model_verification_emitted: AtomicBool,
<<<<<<< HEAD
    /// The turn already warned that developer accounting stopped recording it.
    pub(crate) accounting_gap_warning_emitted: AtomicBool,
||||||| 413492cd6c
=======
    /// Effective cyber treatment for this turn, including any child-agent inheritance.
    pub(crate) cyber_access_program: Option<CyberAccessProgram>,
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
}

/// Selects which preparation is needed when building a turn context.
#[derive(Debug)]
enum TurnContextBuildMode {
    /// Resolves and stores model/multi-agent metadata and performs normal skill discovery.
    /// Used for execution and initial context creation.
    Full,

    /// Prepares startup resources with normal skill discovery, without updating
    /// shared model/multi-agent metadata.
    StartupPrewarm,

    /// Captures recording settings without updating shared model/multi-agent metadata
    /// or discovering skills. Must not initialize context or capture an execution step.
    InjectItems,
}

impl TurnContext {
    /// Returns the resolved causal root for this admitted turn.
    pub(crate) fn root_turn_id(&self) -> String {
        self.turn_metadata_state
            .root_turn_id()
            .unwrap_or_else(|| self.sub_id.clone())
    }

    /// Builds a review turn with shared session grants and fresh turn-local state.
    pub(super) fn for_review(
        &self,
        sub_id: String,
        config: Arc<Config>,
        step_settings: Arc<ResolvedStepSettings>,
        available_models: Vec<ModelPreset>,
        turn_metadata_state: Arc<TurnMetadataState>,
    ) -> Self {
        let session_telemetry = step_settings.telemetry(&self.session_telemetry);
        let extension_data = Arc::new(codex_extension_api::ExtensionData::new(sub_id.clone()));
        extension_data.insert(self.skills_snapshot().as_ref().clone());

        Self {
            sub_id,
            trace_id: current_span_trace_id(),
            realtime_active: self.realtime_active,
            code_mode_available: self.code_mode_available,
            configured_token_budget: config.token_budget.clone(),
            use_model_token_budget_defaults: config.features.enabled(Feature::TokenBudget)
                && !has_explicit_settings(&config),
            config,
            auth_manager: self.auth_manager.clone(),
            initial_settings: Arc::clone(&step_settings),
            disabled_plugin_ids: self.disabled_plugin_ids.clone(),
            active_host_plugin_identities: None,
            next_step_settings: ArcSwap::from(step_settings),
            session_telemetry,
            provider: self.provider.clone(),
            session_source: self.session_source.clone(),
            history_mode: self.history_mode,
            parent_thread_id: self.parent_thread_id,
            originator: self.originator.clone(),
            initial_environments: self.initial_environments.clone(),
            available_models,
            current_date: self.current_date.clone(),
            timezone: self.timezone.clone(),
            app_server_client_name: self.app_server_client_name.clone(),
            developer_instructions: None,
            multi_agent_version: MultiAgentVersion::Disabled,
            network: self.network.clone(),
            windows_sandbox_level: self.windows_sandbox_level,
            #[allow(deprecated)]
            cwd: self.cwd.clone(),
            final_output_json_schema: None,
            turn_metadata_state,
            extension_data,
            turn_timing_state: Arc::new(TurnTimingState::default()),
            terminal_error: Arc::new(Mutex::new(None)),
            session_grants: Arc::clone(&self.session_grants),
            turn_grants: Arc::default(),
            server_model_warning_emitted: AtomicBool::new(false),
            model_verification_emitted: AtomicBool::new(false),
            cyber_access_program: None,
        }
    }

    /// Returns current session and turn grants for this environment.
    #[expect(
        clippy::expect_used,
        reason = "poisoned grant state must not authorize further operations"
    )]
    pub(crate) fn granted_permissions(
        &self,
        environment_id: &str,
    ) -> Option<AdditionalPermissionProfile> {
        let session_grants = self
            .session_grants
            .lock()
            .expect("session permission grants lock poisoned")
            .get(environment_id)
            .cloned();
        let turn_grants = self
            .turn_grants
            .lock()
            .expect("turn permission grants lock poisoned")
            .granted_permissions_by_environment_id
            .get(environment_id)
            .cloned();
        merge_permission_profiles(session_grants.as_ref(), turn_grants.as_ref())
    }

    #[expect(
        clippy::expect_used,
        reason = "poisoned grant state must not authorize further operations"
    )]
    pub(crate) fn record_granted_permissions(
        &self,
        environment_id: &str,
        permissions: AdditionalPermissionProfile,
        strict_auto_review: bool,
    ) {
        let mut grants = self
            .turn_grants
            .lock()
            .expect("turn permission grants lock poisoned");
        let granted_permissions = merge_permission_profiles(
            grants
                .granted_permissions_by_environment_id
                .get(environment_id),
            Some(&permissions),
        );
        if let Some(granted_permissions) = granted_permissions {
            grants
                .granted_permissions_by_environment_id
                .insert(environment_id.to_string(), granted_permissions);
        }
        grants.strict_auto_review_enabled |= strict_auto_review;
    }

    #[expect(
        clippy::expect_used,
        reason = "poisoned grant state must not authorize further operations"
    )]
    pub(crate) fn strict_auto_review_enabled(&self) -> bool {
        self.turn_grants
            .lock()
            .expect("turn permission grants lock poisoned")
            .strict_auto_review_enabled
    }

    pub(crate) fn attribution(&self) -> codex_history::TurnAttribution {
        codex_history::TurnAttribution {
            turn_id: self.sub_id.clone(),
            turn_trigger: self.turn_metadata_state.current_turn_trigger(),
            parent_turn_id: self.turn_metadata_state.parent_turn_id(),
            initiating_agent_path: self.turn_metadata_state.initiating_agent_path().cloned(),
            root_turn_id: self.turn_metadata_state.root_turn_id(),
        }
    }

    /// Captures current model metadata without preparing a step.
    pub(crate) fn capture_current_model_info(&self) -> Arc<CatalogModel> {
        Arc::clone(&self.next_step_settings.load().model_info)
    }

    /// Legacy: returns the frozen initial-turn model metadata.
    /// Step-scoped consumers should use their captured `StepContext::settings`.
    pub(crate) fn model_info(&self) -> &Arc<CatalogModel> {
        &self.initial_settings.model_info
    }

    /// Legacy: returns the frozen initial-turn reasoning-effort selection.
    /// Step-scoped consumers should use their captured `StepContext::settings`.
    pub(crate) fn reasoning_effort(&self) -> Option<&ReasoningEffortConfig> {
        self.initial_settings.reasoning_effort()
    }

    /// Legacy: returns the frozen initial-turn effective reasoning summary.
    /// Step-scoped consumers should use their captured `StepContext::settings`.
    pub(crate) fn reasoning_summary(&self) -> ReasoningSummaryConfig {
        self.initial_settings.reasoning_summary
    }

    /// Legacy: returns the frozen initial-turn collaboration mode kind.
    /// Step-scoped consumers should use their captured `StepContext::settings`.
    pub(crate) fn mode(&self) -> ModeKind {
        self.initial_settings.selected_collaboration_mode().mode
    }

    /// Legacy: returns the frozen initial-turn personality.
    /// Step-scoped consumers should use their captured `StepContext::settings`.
    pub(crate) fn personality(&self) -> Option<Personality> {
        self.initial_settings.personality()
    }

    pub(crate) fn skills_snapshot(&self) -> Arc<HostSkillsSnapshot> {
        let Some(snapshot) = self.extension_data.get::<HostSkillsSnapshot>() else {
            unreachable!("every turn has a host skills snapshot");
        };
        snapshot
    }

    /// Legacy: returns the frozen initial-turn collaboration mode with the resolved model slug.
    /// Step-scoped consumers should use their captured `StepContext::settings`.
    pub(crate) fn collaboration_mode(&self) -> CollaborationMode {
        self.initial_settings.effective_collaboration_mode()
    }

    /// Combines setup-time host identities with the first step's ready selected packages.
    /// Keeps the complete observation or marks it unknown; never truncates membership.
    pub(super) fn active_plugin_ids_for_telemetry(
        &self,
        selected: Option<&SelectedPluginSnapshot>,
    ) -> Option<Vec<String>> {
        const MAX_TELEMETRY_PLUGIN_IDS: usize = 512;
        const MAX_TELEMETRY_PLUGIN_ID_BYTES: usize = 128;

        let mut identities = self.active_host_plugin_identities.clone()?;
        // Selected catalogs preserve local package keys or canonical remote identities.
        if let Some(selected) = selected {
            identities.extend(
                selected
                    .plugins
                    .iter()
                    .map(|plugin| match &plugin.plugin_id {
                        codex_core_plugins::PluginIdentity::Local { plugin_id } => PluginIdentity {
                            plugin_id: plugin_id.clone(),
                            remote_plugin_id: None,
                        },
                        codex_core_plugins::PluginIdentity::Remote { remote_plugin_id } => {
                            PluginIdentity {
                                plugin_id: String::new(),
                                remote_plugin_id: Some(remote_plugin_id.clone()),
                            }
                        }
                    }),
            );
        }
        let mut ids = Vec::with_capacity(identities.len());
        for identity in identities {
            let id = match identity.remote_plugin_id {
                Some(remote_id) => {
                    if !codex_core_plugins::remote::is_valid_remote_plugin_id(&remote_id) {
                        return None;
                    }
                    remote_id
                }
                None => {
                    if codex_plugin::PluginId::parse(&identity.plugin_id).is_err() {
                        return None;
                    }
                    identity.plugin_id
                }
            };
            if id.len() > MAX_TELEMETRY_PLUGIN_ID_BYTES {
                return None;
            }
            ids.push(id);
        }
        ids.sort_unstable();
        ids.dedup();
        (ids.len() <= MAX_TELEMETRY_PLUGIN_IDS).then_some(ids)
    }

    pub(crate) fn plugin_attribution_for_command(
        &self,
        command: &[String],
        cwd: &AbsolutePathBuf,
    ) -> Option<PluginCommandAttribution> {
        self.extension_data
            .get::<TrustedPluginRoots>()?
            .resolve_attribution(command, cwd)
    }

    pub(crate) async fn plugin_attribution_for_executor_command(
        &self,
        command: &[String],
        cwd: &PathUri,
        file_system: &dyn ExecutorFileSystem,
    ) -> Option<PluginCommandAttribution> {
        self.extension_data
            .get::<TrustedPluginRoots>()?
            .resolve_executor_attribution(command, cwd, file_system)
            .await
    }

    /// Legacy: returns the frozen initial-turn approval policy.
    /// Step-scoped consumers should use their captured `StepContext::settings`.
    pub(crate) fn approval_policy(&self) -> AskForApproval {
        self.config.permissions.approval_policy.value()
    }

    /// Legacy: returns the frozen initial-turn prefix-rule policy.
    /// Step-scoped consumers should use their captured `StepContext::settings`.
    pub(crate) fn allow_prefix_rules(&self) -> AllowPrefixRules {
        let ignore_rules = self
            .config
            .config_layer_stack
            .requirements_toml()
            .auto_review
            .as_ref()
            .and_then(|auto_review| auto_review.ignore_rules.as_ref())
            .is_some_and(|models| models.contains(&self.model_info().slug));
        if self.model_info().model_specialty.as_deref() == Some(MODEL_SPECIALTY_CYBER)
            || ignore_rules
        {
            AllowPrefixRules::IgnoreForCyberModel
        } else {
            AllowPrefixRules::Honor
        }
    }

    pub(crate) async fn plugin_metrics_operation_for_command(
        &self,
        command: &[String],
        cwd: &PathUri,
        environment: &Environment,
    ) -> Option<ResolvedPluginMetricsOperation> {
        let trusted_roots = self.extension_data.get::<TrustedPluginRoots>()?;
        if environment.is_remote() {
            trusted_roots
                .resolve_metrics_operation_in_filesystem(
                    command,
                    cwd,
                    environment.get_filesystem().as_ref(),
                )
                .await
        } else {
            trusted_roots.resolve_metrics_operation(command, &cwd.to_abs_path().ok()?)
        }
    }

    /// Returns the selected environment's permissions, or the thread's permissions when none is ready.
    pub(crate) fn permission_profile(&self) -> PermissionProfile {
        self.initial_environments
            .permission_profile_or_else(|| self.config.permissions.effective_permission_profile())
    }

    /// Uses this selection's permissions, or the turn's thread defaults for these folders.
    pub(crate) fn permission_profile_for_environments(
        &self,
        environments: &TurnEnvironmentSnapshot,
    ) -> PermissionProfile {
        environments.permission_profile_or_else(|| {
            self.config
                .permissions
                .permission_profile()
                .clone()
                .materialize_project_roots_with_path_uris(
                    environments.primary_workspace_root_uris(),
                )
        })
    }

    pub(crate) fn file_system_sandbox_policy(&self) -> FileSystemSandboxPolicy {
        self.permission_profile().file_system_sandbox_policy()
    }

    pub(crate) fn network_sandbox_policy(&self) -> NetworkSandboxPolicy {
        self.permission_profile().network_sandbox_policy()
    }

    pub(crate) fn sandbox_policy(&self) -> SandboxPolicy {
        #[allow(deprecated)]
        codex_sandboxing::compatibility_sandbox_policy_for_permission_profile(
            &self.permission_profile(),
            &self.cwd,
        )
    }

    /// Legacy: returns the frozen initial-turn reasoning effort, including the initial model default.
    /// Step-scoped consumers should use their captured `StepContext::settings`.
    pub(crate) fn effective_reasoning_effort(&self) -> Option<ReasoningEffortConfig> {
        self.initial_settings.effective_reasoning_effort()
    }

    /// Legacy: returns the frozen initial-turn reasoning-effort label for tracing.
    /// Step-scoped consumers should use their captured `StepContext::settings`.
    pub(crate) fn effective_reasoning_effort_for_tracing(&self) -> String {
        self.effective_reasoning_effort()
            .map(|effort| effort.to_string())
            .unwrap_or_else(|| "default".to_string())
    }

    /// Legacy: returns the frozen initial-turn model context window.
    /// Step-scoped consumers should use their captured `StepContext::settings`.
    pub(crate) fn model_context_window(&self) -> Option<i64> {
        self.model_info().usable_context_window()
    }

    pub(crate) fn apps_enabled(&self) -> bool {
        let uses_codex_backend = self
            .auth_manager
            .as_deref()
            .is_some_and(AuthManager::current_auth_uses_codex_backend);
        self.config
            .features
            .apps_enabled_for_auth(uses_codex_backend)
            && self.config.orchestrator_mcp_enabled
    }

    pub(crate) fn record_strict_apply_patch_grammar_failure(&self) -> PatchFallbackTransition {
        self.model_edit_protocol_state.record_grammar_failure()
    }

    pub(crate) fn record_strict_apply_patch_parse_success(&self) {
        self.model_edit_protocol_state.record_parse_success();
    }

    pub(crate) fn structured_edit_fallback_enabled(&self) -> bool {
        self.model_edit_protocol_state.fallback_enabled()
    }

    pub(crate) async fn record_malformed_tool_call(&self, signature: String) -> u8 {
        self.malformed_tool_call_state.record(signature).await
    }

    pub(crate) async fn record_direct_tool_call(&self, signature: &str) -> ToolCallRepetition {
        self.repeated_tool_call_state.record_call(signature).await
    }

    pub(crate) async fn record_direct_tool_result(&self, signature: String, identity: String) {
        self.repeated_tool_call_state
            .record_result(signature, identity)
            .await
    }

    /// Returns how many repeated calls this turn has refused, including this one.
    pub(crate) fn record_repeated_tool_call_refusal(&self) -> u32 {
        self.repeated_tool_call_state.record_refusal()
    }

    pub(crate) fn set_explicit_shell_command_budget(&self, limit: u64) {
        if limit == 0 {
            return;
        }
        let mut current = self
            .explicit_tool_budget_state
            .shell_command_limit
            .load(Ordering::Acquire);
        loop {
            if current != 0 && current <= limit {
                return;
            }
            match self
                .explicit_tool_budget_state
                .shell_command_limit
                .compare_exchange(current, limit, Ordering::AcqRel, Ordering::Acquire)
            {
                Ok(_) => return,
                Err(next) => current = next,
            }
        }
    }

    pub(crate) fn record_shell_command_against_explicit_budget(
        &self,
    ) -> Result<Option<(u64, u64)>, u64> {
        let limit = self
            .explicit_tool_budget_state
            .shell_command_limit
            .load(Ordering::Acquire);
        if limit == 0 {
            return Ok(None);
        }

        let mut used = self
            .explicit_tool_budget_state
            .shell_commands_used
            .load(Ordering::Acquire);
        loop {
            if used >= limit {
                return Err(limit);
            }
            match self
                .explicit_tool_budget_state
                .shell_commands_used
                .compare_exchange(
                    used,
                    used.saturating_add(1),
                    Ordering::AcqRel,
                    Ordering::Acquire,
                ) {
                Ok(_) => return Ok(Some((used.saturating_add(1), limit))),
                Err(next) => used = next,
            }
        }
    }

    pub(crate) async fn with_model(
        &self,
        model: String,
        models_manager: &SharedModelsManager,
    ) -> Self {
        let mut config = (*self.config).clone();
        config.model = Some(model.clone());
        let model_info =
            crate::model_info_for_provider::model_info_for_config(models_manager, &model, &config)
                .await;
        let supported_reasoning_levels = model_info
            .supported_reasoning_levels
            .iter()
            .map(|preset| preset.effort.clone())
            .collect::<Vec<_>>();
        let reasoning_effort = if let Some(current_reasoning_effort) = self.reasoning_effort() {
            if supported_reasoning_levels.contains(current_reasoning_effort) {
                Some(current_reasoning_effort.clone())
            } else {
                supported_reasoning_levels
                    .get(supported_reasoning_levels.len().saturating_sub(1) / 2)
                    .cloned()
                    .or_else(|| model_info.default_reasoning_level.clone())
            }
        } else {
            supported_reasoning_levels
                .get(supported_reasoning_levels.len().saturating_sub(1) / 2)
                .cloned()
                .or_else(|| model_info.default_reasoning_level.clone())
        };
        config.model_reasoning_effort = reasoning_effort.clone();

        let available_models = models_manager
            .list_models(
                RefreshStrategy::OnlineIfUncached,
                config.http_client_factory(),
            )
            .await;
        let model_info = Arc::new(model_info);
        let mut selected = self.initial_settings.selected().clone();
        selected.collaboration_mode = selected.collaboration_mode.with_updates(
            Some(model),
            Some(reasoning_effort),
            /*developer_instructions*/ None,
        );
        let step_settings = Arc::new(ResolvedStepSettings::new(
            Arc::new(selected),
            model_info,
            &config.features,
        ));
        config.service_tier = step_settings.service_tier.clone();
        let session_telemetry = step_settings.telemetry(&self.session_telemetry);

        Self {
            sub_id: self.sub_id.clone(),
            trace_id: self.trace_id.clone(),
            realtime_active: self.realtime_active,
            code_mode_available: self.code_mode_available,
            config: Arc::new(config),
            configured_token_budget: self.configured_token_budget.clone(),
            use_model_token_budget_defaults: self.use_model_token_budget_defaults,
            auth_manager: self.auth_manager.clone(),
            initial_settings: Arc::clone(&step_settings),
            disabled_plugin_ids: self.disabled_plugin_ids.clone(),
            active_host_plugin_identities: self.active_host_plugin_identities.clone(),
            next_step_settings: ArcSwap::from(step_settings),
            session_telemetry,
            provider: self.provider.clone(),
            session_source: self.session_source.clone(),
            history_mode: self.history_mode,
            parent_thread_id: self.parent_thread_id,
            originator: self.originator.clone(),
            initial_environments: self.initial_environments.clone(),
            #[allow(deprecated)]
            cwd: self.cwd.clone(),
            current_date: self.current_date.clone(),
            timezone: self.timezone.clone(),
            app_server_client_name: self.app_server_client_name.clone(),
            developer_instructions: self.developer_instructions.clone(),
            multi_agent_version: self.multi_agent_version,
            network: self.network.clone(),
            windows_sandbox_level: self.windows_sandbox_level,
            available_models,
            final_output_json_schema: self.final_output_json_schema.clone(),
            turn_metadata_state: self.turn_metadata_state.clone(),
            extension_data: Arc::clone(&self.extension_data),
            turn_timing_state: Arc::clone(&self.turn_timing_state),
            parent_completion_expected: Arc::clone(&self.parent_completion_expected),
            terminal_error: Arc::clone(&self.terminal_error),
<<<<<<< HEAD
            model_edit_protocol_state: Arc::clone(&self.model_edit_protocol_state),
            explicit_tool_budget_state: Arc::clone(&self.explicit_tool_budget_state),
            malformed_tool_call_state: Arc::clone(&self.malformed_tool_call_state),
            repeated_tool_call_state: Arc::clone(&self.repeated_tool_call_state),
            pending_tool_calls: Arc::clone(&self.pending_tool_calls),
||||||| 413492cd6c
=======
            session_grants: Arc::clone(&self.session_grants),
            turn_grants: Arc::clone(&self.turn_grants),
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
            server_model_warning_emitted: AtomicBool::new(
                self.server_model_warning_emitted.load(Ordering::Relaxed),
            ),
            provider_cache_pressure_warning_emitted: AtomicBool::new(
                self.provider_cache_pressure_warning_emitted
                    .load(Ordering::Relaxed),
            ),
            model_verification_emitted: AtomicBool::new(
                self.model_verification_emitted.load(Ordering::Relaxed),
            ),
<<<<<<< HEAD
            accounting_gap_warning_emitted: AtomicBool::new(
                self.accounting_gap_warning_emitted.load(Ordering::Relaxed),
            ),
||||||| 413492cd6c
=======
            cyber_access_program: self.cyber_access_program,
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        }
    }

<<<<<<< HEAD
    pub(crate) fn file_system_sandbox_context(
        &self,
        additional_permissions: Option<AdditionalPermissionProfile>,
        environment: &TurnEnvironment,
    ) -> FileSystemSandboxContext {
        let mut permissions = effective_permission_profile(
            self.config.permissions.permission_profile(),
            additional_permissions.as_ref(),
        );
        // PF-27-S02: in-process file tools get the protected profile, or no
        // file access when the turn's permissions cannot be protected.
        if let Some(contract) = crate::security::launch_contract::active() {
            let cwd = environment.cwd().to_abs_path().ok();
            let sandboxed = cwd.is_some() && cfg!(any(target_os = "macos", target_os = "linux"));
            let cwd = cwd.map(|cwd| cwd.to_path_buf()).unwrap_or_default();
            permissions = contract.file_tool_permissions(&permissions, sandboxed, &cwd);
        }
        FileSystemSandboxContext {
            permissions: permissions.into(),
            cwd: Some(environment.cwd().clone()),
            workspace_roots: environment.workspace_roots().to_vec(),
            windows_sandbox_level: self.windows_sandbox_level,
            windows_sandbox_private_desktop: self
                .config
                .permissions
                .windows_sandbox_private_desktop,
            windows_sandbox_proxy_settings_mode: None,
            use_legacy_landlock: self.config.features.use_legacy_landlock(),
            // #1976 (D491): refused where the host has no platform sandbox.
            unsandboxed_refusal: crate::security::windows_unsandboxed::applies(
                self.config.security_level,
                self.windows_sandbox_level,
            )
            .then(|| {
                crate::security::windows_unsandboxed::file_refusal(self.config.security_level)
            }),
        }
    }

    fn non_legacy_file_system_sandbox_policy(&self) -> Option<FileSystemSandboxPolicy> {
||||||| 413492cd6c
    pub(crate) fn file_system_sandbox_context(
        &self,
        additional_permissions: Option<AdditionalPermissionProfile>,
        environment: &TurnEnvironment,
    ) -> FileSystemSandboxContext {
        let permissions = effective_permission_profile(
            self.config.permissions.permission_profile(),
            additional_permissions.as_ref(),
        );
        FileSystemSandboxContext {
            permissions: permissions.into(),
            cwd: Some(environment.cwd().clone()),
            workspace_roots: environment.workspace_roots().to_vec(),
            windows_sandbox_level: self.windows_sandbox_level,
            windows_sandbox_private_desktop: self
                .config
                .permissions
                .windows_sandbox_private_desktop,
            windows_sandbox_proxy_settings_mode: None,
            use_legacy_landlock: self.config.features.use_legacy_landlock(),
        }
    }

    fn non_legacy_file_system_sandbox_policy(&self) -> Option<FileSystemSandboxPolicy> {
=======
    fn non_legacy_file_system_sandbox_policy(&self) -> Option<RawFileSystemSandboxPolicy> {
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        // Omit the derived split filesystem policy when it is equivalent to
        // the legacy sandbox policy. This keeps turn-context payloads stable
        // while both fields exist; once callers consume only the split policy,
        // this comparison and the legacy projection should go away.
        let legacy_file_system_sandbox_policy =
            FileSystemSandboxPolicy::from_legacy_sandbox_policy_for_cwd(
                &self.sandbox_policy(),
                #[allow(deprecated)]
                &self.cwd,
            );
        let file_system_sandbox_policy = self.file_system_sandbox_policy();
        // `permission_profile` below is authoritative and serializes the same
        // runtime entries, so this compatibility field may omit an unrenderable policy.
        (file_system_sandbox_policy != legacy_file_system_sandbox_policy)
            .then(|| file_system_sandbox_policy.try_into().ok())
            .flatten()
    }

    /// #578: the provider (and named account) recorded as the producer of
    /// this turn's model output items: `<provider id>`, or
    /// `<provider id>:<account>` for a named account.
    pub(crate) fn output_producer(&self) -> String {
        crate::context_manager::provider_items::producer_key(
            &self.config.model_provider_id,
            self.provider.info().account.as_ref(),
        )
    }

    pub(crate) fn to_turn_context_item(&self) -> TurnContextItem {
        #[allow(deprecated)]
        let cwd = self.cwd.clone();
        TurnContextItem {
            turn_id: Some(self.sub_id.clone()),
            root_turn_id: self.turn_metadata_state.root_turn_id(),
            disabled_plugin_ids: Some(self.disabled_plugin_ids.clone()),
            cwd,
            approval_policy: self.approval_policy(),
            approvals_reviewer: Some(self.config.approvals_reviewer),
            sandbox_policy: self.sandbox_policy(),
            permission_profile: Some(self.permission_profile()),
            active_permission_profile: self.initial_environments.primary().map_or_else(
                || self.config.permissions.active_permission_profile(),
                TurnEnvironment::active_permission_profile,
            ),
            file_system_sandbox_policy: self.non_legacy_file_system_sandbox_policy(),
<<<<<<< HEAD
            model: self.model_info.slug.clone(),
            model_provider: Some(self.config.model_provider_id.clone()),
            provider_account: self
                .config
                .features
                .enabled(Feature::NamedAccounts)
                .then(|| {
                    // #1129: an account borrowed from another provider (same
                    // API key, see `with_account_from`) keeps its owner.
                    self.provider.info().account.as_ref().map_or_else(
                        || "default".to_string(),
                        |account| {
                            if account.provider_id == self.config.model_provider_id {
                                account.name.clone()
                            } else {
                                account.to_string()
                            }
                        },
                    )
                }),
            comp_hash: self.model_info.comp_hash.clone(),
            personality: self.personality,
||||||| 413492cd6c
            model: self.model_info.slug.clone(),
            comp_hash: self.model_info.comp_hash.clone(),
            personality: self.personality,
=======
            model: self.model_info().slug.clone(),
            comp_hash: self.model_info().comp_hash.clone(),
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
            collaboration_mode: Some(self.collaboration_mode()),
            multi_agent_version: Some(self.multi_agent_version),
            realtime_active: Some(self.realtime_active),
<<<<<<< HEAD
            effort: self.reasoning_effort.clone(),
            security_level: Some(self.config.security_level),
            summary: ReasoningSummaryConfig::Auto,
||||||| 413492cd6c
            effort: self.reasoning_effort.clone(),
            summary: ReasoningSummaryConfig::Auto,
=======
            cyber_access_program: self.cyber_access_program,
            effort: self.reasoning_effort().cloned(),
            summary: Some(ReasoningSummaryConfig::None),
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        }
    }
}

/// Grants and their review requirement share a lock so readers cannot see a new grant before
/// its strict-review requirement.
#[derive(Debug, Default)]
struct TurnGrants {
    granted_permissions_by_environment_id: HashMap<String, AdditionalPermissionProfile>,
    strict_auto_review_enabled: bool,
}

fn local_time_context() -> (String, String) {
    match iana_time_zone::get_timezone() {
        Ok(timezone) => (Local::now().format("%Y-%m-%d").to_string(), timezone),
        Err(_) => (
            Utc::now().format("%Y-%m-%d").to_string(),
            "Etc/UTC".to_string(),
        ),
    }
}

impl Session {
    /// Don't expand the number of mutated arguments on config. We are in the process of getting rid of it.
    pub(crate) fn build_per_turn_config(
        &self,
        session_configuration: &SessionConfiguration,
        local_environment: Option<&TurnEnvironmentSelection>,
    ) -> Config {
        // todo(aibrahim): store this state somewhere else so we don't need to mut config
        let config = session_configuration.original_config_do_not_use.clone();
        let mut per_turn_config = (*config).clone();
<<<<<<< HEAD
        per_turn_config.model = Some(session_configuration.collaboration_mode.model().to_string());
        per_turn_config.cwd = cwd;
        let workspace_roots = session_configuration.primary_workspace_roots();
||||||| 413492cd6c
        per_turn_config.cwd = cwd;
        let workspace_roots = session_configuration.primary_workspace_roots();
=======
        // Config is consumed by host plugins and MCP processes. Executor paths and
        // permissions remain on their captured environment attachments.
        per_turn_config.cwd = local_environment
            .and_then(|environment| environment.cwd.to_abs_path().ok())
            .unwrap_or_else(|| session_configuration.cwd().clone());
        let workspace_roots = local_environment.map_or_else(
            || session_configuration.runtime_workspace_roots.clone(),
            |environment| {
                environment
                    .workspace_roots
                    .iter()
                    .filter_map(|root| root.to_abs_path().ok())
                    .collect()
            },
        );
        per_turn_config.permissions.approval_policy =
            session_configuration.step_settings.approval_policy.clone();
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        per_turn_config.workspace_roots = workspace_roots.clone();
        per_turn_config
            .permissions
            .set_workspace_roots(workspace_roots);
<<<<<<< HEAD
        per_turn_config.model_reasoning_effort =
            session_configuration.collaboration_mode.reasoning_effort();
        per_turn_config.model_reasoning_summary = session_configuration.model_reasoning_summary;
        if let Some(runtime_limit) = session_configuration.runtime_model_context_window {
            per_turn_config.model_context_window = Some(
                per_turn_config
                    .model_context_window
                    .map_or(runtime_limit, |configured| configured.min(runtime_limit)),
            );
        }
        per_turn_config.service_tier = session_configuration.service_tier.clone();
        per_turn_config.personality = session_configuration.personality;
        per_turn_config.approvals_reviewer = session_configuration.approvals_reviewer;
||||||| 413492cd6c
        per_turn_config.model_reasoning_effort =
            session_configuration.collaboration_mode.reasoning_effort();
        per_turn_config.model_reasoning_summary = session_configuration.model_reasoning_summary;
        per_turn_config.service_tier = session_configuration.service_tier.clone();
        per_turn_config.personality = session_configuration.personality;
        per_turn_config.approvals_reviewer = session_configuration.approvals_reviewer;
=======
        per_turn_config.model_reasoning_effort = session_configuration
            .step_settings
            .collaboration_mode
            .reasoning_effort();
        per_turn_config.model_reasoning_summary =
            session_configuration.step_settings.reasoning_summary;
        per_turn_config.service_tier = session_configuration.step_settings.service_tier.clone();
        per_turn_config.personality = session_configuration.step_settings.personality;
        per_turn_config.approvals_reviewer = session_configuration.step_settings.approvals_reviewer;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        session_configuration
            .apply_permission_profile_to_permissions(&mut per_turn_config.permissions);
        let permission_profile = session_configuration.permission_profile();
        let resolved_web_search_mode = resolve_web_search_mode_for_turn(
            &per_turn_config.web_search_mode,
            &permission_profile,
            session_configuration.provider.capabilities(),
        );
        if let Err(err) = per_turn_config
            .web_search_mode
            .set(resolved_web_search_mode)
        {
            let fallback_value = per_turn_config.web_search_mode.value();
            tracing::warn!(
                error = %err,
                ?resolved_web_search_mode,
                ?fallback_value,
                "resolved web_search_mode is disallowed by requirements; keeping constrained value"
            );
        }
        per_turn_config.features = config.features.clone();
        per_turn_config
    }

    pub(crate) fn build_effective_session_config(
        &self,
        session_configuration: &SessionConfiguration,
    ) -> Config {
        let mut config = self.build_per_turn_config(
            session_configuration,
            session_configuration
                .environments
                .iter()
                .find(|environment| {
                    environment.environment_id == codex_exec_server::LOCAL_ENVIRONMENT_ID
                        && !matches!(environment.config, EnvironmentConfigState::Failed(_))
                }),
        );
        config.model = Some(
            session_configuration
                .step_settings
                .collaboration_mode
                .model()
                .to_string(),
        );
        config
    }

    #[allow(clippy::too_many_arguments)]
    #[instrument(name = "turn_context.make", level = "trace", skip_all)]
    pub(crate) fn make_turn_context(
        thread_id: ThreadId,
        session_id: SessionId,
        session_grants: Arc<StdMutex<HashMap<String, AdditionalPermissionProfile>>>,
        auth_manager: Option<Arc<AuthManager>>,
        session_telemetry: &SessionTelemetry,
        provider: SharedModelProvider,
        session_configuration: &SessionConfiguration,
        multi_agent_version: MultiAgentVersion,
        per_turn_config: Config,
        step_settings: Arc<ResolvedStepSettings>,
        models_manager: &SharedModelsManager,
        network: Option<NetworkProxy>,
        environments: TurnEnvironmentSnapshot,
        cwd: AbsolutePathBuf,
        sub_id: String,
        skills_snapshot: HostSkillsSnapshot,
    ) -> TurnContext {
        let model_info = &step_settings.model_info;
        let session_telemetry_for_context = step_settings.telemetry(session_telemetry);
        let session_source = session_configuration.session_source.clone();
<<<<<<< HEAD
        let auth_manager_for_context = auth_manager.clone();
        let provider_for_context = create_model_provider(provider, auth_manager);
        let session_telemetry_for_context = session_telemetry;
        let mut available_models = models_manager.try_list_models().unwrap_or_default();
        let runtime_provider_id = per_turn_config.model_provider_id.clone();
        if runtime_provider_id.starts_with("gpu-")
            && !available_models
                .iter()
                .any(|preset| preset.model == model_info.slug)
        {
            let mut preset = ModelPreset::from(model_info.clone());
            preset.provider_id = Some(runtime_provider_id.clone());
            preset.orchestration = Some(ModelOrchestrationMetadata::Eligible {
                provider_id: runtime_provider_id,
                capability: ModelCapabilityTier::Balanced,
                billing: ModelBilling::Local,
            });
            available_models.push(preset);
        }
        let unified_exec_shell_mode = UnifiedExecShellMode::for_session(
            codex_tools::unified_exec_feature_mode_for_features(per_turn_config.features.get()),
            crate::tools::tool_user_shell_type(user_shell),
            shell_zsh_path,
            main_execve_wrapper_exe,
        );
||||||| 413492cd6c
        let auth_manager_for_context = auth_manager.clone();
        let provider_for_context = create_model_provider(provider, auth_manager);
        let session_telemetry_for_context = session_telemetry;
        let available_models = models_manager.try_list_models().unwrap_or_default();
        let unified_exec_shell_mode = UnifiedExecShellMode::for_session(
            codex_tools::unified_exec_feature_mode_for_features(per_turn_config.features.get()),
            crate::tools::tool_user_shell_type(user_shell),
            shell_zsh_path,
            main_execve_wrapper_exe,
        );
=======
        let available_models = models_manager.try_list_models().unwrap_or_default();
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a

        let mut per_turn_config = per_turn_config;
        let configured_token_budget = per_turn_config.token_budget.clone();
        let use_model_token_budget_defaults =
            per_turn_config.features.enabled(Feature::TokenBudget)
                && !has_explicit_settings(&per_turn_config);
        per_turn_config.token_budget = resolve_token_budget(
            configured_token_budget.as_ref(),
            use_model_token_budget_defaults,
            model_info,
        );
        if per_turn_config
            .features
            .persistent_execution_enabled(step_settings.reasoning_effort())
        {
            super::time_reminder::apply_persistent_defaults(&mut per_turn_config);
        }
        per_turn_config.service_tier = step_settings.service_tier.clone();
        let permission_profile = environments.permission_profile_or_else(|| {
            per_turn_config.permissions.effective_permission_profile()
        });
        let auto_review_enabled = crate::guardian::routes_approval_policy_to_guardian(
            per_turn_config.permissions.approval_policy.value(),
            per_turn_config.approvals_reviewer,
        );
        let windows_sandbox_selection = environments
            .primary()
            .map(TurnEnvironment::windows_sandbox_selection_for_turn_metadata)
            .unwrap_or_else(|| {
                configured_windows_sandbox_selection(
                    session_configuration.windows_sandbox_type,
                    session_configuration.windows_sandbox_level,
                    &PathUri::from_abs_path(&cwd),
                )
            });
        let per_turn_config = Arc::new(per_turn_config);
        let turn_metadata_state = Arc::new(TurnMetadataState::new(
            session_id.to_string(),
            thread_id.to_string(),
            session_configuration.forked_from_thread_id,
            session_configuration.parent_thread_id,
            &session_configuration.session_source,
            session_configuration.thread_source.clone(),
            sub_id.clone(),
            cwd.clone(),
            &permission_profile,
            windows_sandbox_selection,
            network.is_some(),
            auto_review_enabled,
            model_info,
        ));
        turn_metadata_state
            .set_responses_api_metadata(per_turn_config.responses_api_metadata.clone());
        let (current_date, timezone) = local_time_context();
        let extension_data = Arc::new(codex_extension_api::ExtensionData::new_with_init(
            sub_id.clone(),
            session_configuration.turn_extension_init.clone(),
        ));
        extension_data.insert(skills_snapshot);
        TurnContext {
            sub_id,
            trace_id: current_span_trace_id(),
            realtime_active: false,
            code_mode_available: true,
            config: per_turn_config,
            configured_token_budget,
            use_model_token_budget_defaults,
            auth_manager,
            initial_settings: Arc::clone(&step_settings),
            disabled_plugin_ids: session_configuration.disabled_plugin_ids.clone(),
            active_host_plugin_identities: None,
            next_step_settings: ArcSwap::from(step_settings),
            session_telemetry: session_telemetry_for_context,
            provider,
            session_source,
            history_mode: session_configuration.history_mode,
            parent_thread_id: session_configuration.parent_thread_id,
            originator: session_configuration.originator.clone(),
            initial_environments: environments,
            #[allow(deprecated)]
            cwd,
            current_date: Some(current_date),
            timezone: Some(timezone),
            app_server_client_name: session_configuration.app_server_client_name.clone(),
            developer_instructions: session_configuration.developer_instructions.clone(),
            multi_agent_version,
            network,
            windows_sandbox_level: session_configuration.windows_sandbox_level,
            available_models,
            final_output_json_schema: None,
            turn_metadata_state,
            extension_data,
            turn_timing_state: Arc::new(TurnTimingState::default()),
            parent_completion_expected: Arc::new(AtomicBool::new(false)),
            terminal_error: Arc::new(Mutex::new(None)),
<<<<<<< HEAD
            model_edit_protocol_state: Arc::new(ModelEditProtocolState::default()),
            explicit_tool_budget_state: Arc::new(ExplicitToolBudgetState::default()),
            malformed_tool_call_state: Arc::new(MalformedToolCallState::default()),
            repeated_tool_call_state: Arc::new(RepeatedToolCallState::default()),
            pending_tool_calls: Arc::default(),
||||||| 413492cd6c
=======
            session_grants,
            turn_grants: Arc::default(),
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
            server_model_warning_emitted: AtomicBool::new(false),
            provider_cache_pressure_warning_emitted: AtomicBool::new(false),
            model_verification_emitted: AtomicBool::new(false),
<<<<<<< HEAD
            accounting_gap_warning_emitted: AtomicBool::new(false),
||||||| 413492cd6c
=======
            cyber_access_program: None,
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        }
    }

    pub(crate) async fn new_turn_with_sub_id(
        &self,
        sub_id: String,
<<<<<<< HEAD
        mut updates: SessionSettingsUpdate,
    ) -> CodexResult<Arc<TurnContext>> {
        // GPU endpoints are process-local controller transports and can change while a
        // session remains open. Refresh the selected runtime provider before every turn so
        // recovery does not leave the session's ModelClient pinned to a dead local port.
        self.ensure_runtime_model_provider(&mut updates).await;
        let notify_config_contributors = !self.services.extensions.config_contributors().is_empty();
        let update_result: CodexResult<_> = {
            let mut state = self.state.lock().await;
            match state.session_configuration.clone().apply(&updates) {
                Ok(next) => {
                    let mcp_inputs_changed = state.session_configuration.mcp_inputs_differ(&next);
                    let previous_permission_profile =
                        state.session_configuration.permission_profile();
                    let next_permission_profile = next.permission_profile();
                    let permission_profile_changed =
                        previous_permission_profile != next_permission_profile;
                    let previous_config = notify_config_contributors.then(|| {
                        Self::build_effective_session_config(&state.session_configuration)
                    });
                    let new_config = notify_config_contributors
                        .then(|| Self::build_effective_session_config(&next));
                    let model_provider_changed = state
                        .session_configuration
                        .original_config_do_not_use
                        .model_provider_id
                        != next.original_config_do_not_use.model_provider_id
                        || state.session_configuration.provider != next.provider;
                    // PF-84: as in update_settings, the account follows first.
                    if model_provider_changed {
                        self.follow_provider_account(&next);
                    }
                    let follows_model_list =
                        model_provider_changed && self.model_list_follows_this_session(&next);
                    if model_provider_changed {
                        // Match update_settings: frame guards must see this provider
                        // before its configuration is published, under the same writer lock.
                        self.services
                            .replace_model_client(self.build_model_client_for_configuration(&next));
                    }
                    let stale_startup_prewarm = if model_provider_changed {
                        state.take_session_startup_prewarm()
                    } else {
                        None
                    };
                    if updates.environments.is_some() {
                        self.services
                            .turn_environments
                            .update_selections(next.environment_selections());
                    }
                    if mcp_inputs_changed {
                        self.mark_mcp_runtime_dirty();
                    }
                    state.session_configuration = next.clone();
                    Ok((
                        next,
                        mcp_inputs_changed,
                        permission_profile_changed,
                        previous_config,
                        new_config,
                        stale_startup_prewarm,
                        follows_model_list,
                    ))
                }
                Err(err) => Err(CodexErr::InvalidRequest(err.to_string())),
            }
||||||| 413492cd6c
        updates: SessionSettingsUpdate,
    ) -> CodexResult<Arc<TurnContext>> {
        let notify_config_contributors = !self.services.extensions.config_contributors().is_empty();
        let update_result: CodexResult<_> = {
            let mut state = self.state.lock().await;
            match state.session_configuration.clone().apply(&updates) {
                Ok(next) => {
                    let mcp_inputs_changed = state.session_configuration.mcp_inputs_differ(&next);
                    let previous_permission_profile =
                        state.session_configuration.permission_profile();
                    let next_permission_profile = next.permission_profile();
                    let permission_profile_changed =
                        previous_permission_profile != next_permission_profile;
                    let previous_config = notify_config_contributors.then(|| {
                        Self::build_effective_session_config(&state.session_configuration)
                    });
                    let new_config = notify_config_contributors
                        .then(|| Self::build_effective_session_config(&next));
                    if updates.environments.is_some() {
                        self.services
                            .turn_environments
                            .update_selections(next.environment_selections());
                    }
                    if mcp_inputs_changed {
                        self.mark_mcp_runtime_dirty();
                    }
                    state.session_configuration = next.clone();
                    Ok((
                        next,
                        mcp_inputs_changed,
                        permission_profile_changed,
                        previous_config,
                        new_config,
                    ))
                }
                Err(err) => Err(CodexErr::InvalidRequest(err.to_string())),
            }
=======
        updates: SessionSettingsUpdate,
        options: NewTurnContextOptions,
    ) -> CodexResult<(Arc<TurnContext>, ThreadSettingsSnapshot)> {
        let Some((turn_context, snapshot)) = self
            .new_turn_with_sub_id_if(sub_id, updates, options, |_, _| true)
            .await?
        else {
            unreachable!("unconditional turn construction must accept valid settings");
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        };
        Ok((turn_context, snapshot))
    }

<<<<<<< HEAD
        let (
            session_configuration,
            mcp_inputs_changed,
            permission_profile_changed,
            previous_config,
            new_config,
            stale_startup_prewarm,
            follows_model_list,
        ) = match update_result {
            Ok(update) => update,
            Err(err) => {
                let message = err.to_string();
||||||| 413492cd6c
        let (
            session_configuration,
            mcp_inputs_changed,
            permission_profile_changed,
            previous_config,
            new_config,
        ) = match update_result {
            Ok(update) => update,
            Err(err) => {
                let message = err.to_string();
=======
    /// Commits accepted settings atomically, then constructs the turn without holding
    /// the state lock. The caller owns admission policy.
    ///
    /// `should_start` runs under the state lock against the current and validated
    /// proposed configurations. It must be fast and side-effect-free, and must not
    /// block, acquire other locks, or call back into `Session`.
    pub(super) async fn new_turn_with_sub_id_if(
        &self,
        sub_id: String,
        updates: SessionSettingsUpdate,
        options: NewTurnContextOptions,
        should_start: impl FnOnce(&SessionConfiguration, &SessionConfiguration) -> bool + Send,
    ) -> CodexResult<Option<(Arc<TurnContext>, ThreadSettingsSnapshot)>> {
        let service_tier_for_turn = updates.service_tier_for_turn.clone();
        let commit = match self.update_settings_if(updates, should_start).await {
            Ok(Some(commit)) => commit,
            Ok(None) => return Ok(None),
            Err(error) => {
                let message = CodexErr::InvalidRequest(error.to_string()).to_string();
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
                self.send_event_raw(Event {
                    id: sub_id,
                    msg: EventMsg::Error(ErrorEvent {
                        misalignment: None,
                        message: message.clone(),
                        codex_error_info: Some(CodexErrorInfo::BadRequest),
                    }),
                })
                .await;
                return Err(CodexErr::InvalidRequest(message));
            }
        };
        let mut configuration = commit.configuration;
        // Apply the override only to the turn's copy, after persisting thread settings.
        if let Some(service_tier) = service_tier_for_turn {
            Arc::make_mut(&mut configuration.step_settings).service_tier = Some(service_tier);
        }
<<<<<<< HEAD
        if follows_model_list {
            self.follow_model_list_account().await;
        }

        if let Some(startup_prewarm) = stale_startup_prewarm {
            startup_prewarm.abort().await;
        }

        if permission_profile_changed {
            self.refresh_managed_network_proxy_for_current_permission_profile()
||||||| 413492cd6c

        if permission_profile_changed {
            self.refresh_managed_network_proxy_for_current_permission_profile()
=======
        if !crate::guardian::is_basic_session_source(&configuration.session_source) {
            self.services
                .models_manager
                .refresh_after_auth_change(
                    self.build_effective_session_config(&configuration)
                        .http_client_factory(),
                )
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
                .await;
        }
        let turn_environments = self.activate_turn_environments(&configuration).await;
        let turn_context = self
            .new_turn_from_configuration(sub_id, configuration, turn_environments, options)
            .await;
        Ok(Some((turn_context, commit.snapshot)))
    }

    /// Builds a context from the caller's chosen settings and environments without starting work.
    async fn new_turn_from_configuration(
        &self,
        sub_id: String,
        session_configuration: SessionConfiguration,
        turn_environments: TurnEnvironmentSnapshot,
        options: NewTurnContextOptions,
    ) -> Arc<TurnContext> {
        // Keep context construction out of the enclosing turn and history futures.
        Box::pin(self.new_turn_context_from_configuration(
            sub_id,
            session_configuration,
            turn_environments,
            options,
            TurnContextBuildMode::Full,
            self.git_enrichment_policy,
        ))
        .await
    }

    async fn new_startup_prewarm_turn_from_configuration(
        &self,
        sub_id: String,
        session_configuration: SessionConfiguration,
    ) -> Arc<TurnContext> {
        let turn_environments = self.services.turn_environments.snapshot().await;
        Box::pin(self.new_turn_context_from_configuration(
            sub_id,
            session_configuration,
            turn_environments,
            NewTurnContextOptions::default(),
            TurnContextBuildMode::StartupPrewarm,
            GitEnrichmentPolicy::Skip,
        ))
        .await
    }

    #[instrument(name = "turn_context.build", level = "trace", skip_all, fields(?build_mode))]
    async fn new_turn_context_from_configuration(
        &self,
        sub_id: String,
        session_configuration: SessionConfiguration,
        turn_environments: TurnEnvironmentSnapshot,
        options: NewTurnContextOptions,
        build_mode: TurnContextBuildMode,
        git_enrichment_policy: GitEnrichmentPolicy,
    ) -> Arc<TurnContext> {
        let primary_turn_environment = turn_environments.primary();
        let per_turn_config =
            self.build_per_turn_config(&session_configuration, turn_environments.local_selection());
        // Legacy turn consumers still refer to the primary executor's repository.
        let cwd = primary_turn_environment
            .and_then(|environment| environment.cwd().to_abs_path().ok())
            .unwrap_or_else(|| session_configuration.cwd().clone());
<<<<<<< HEAD
        let per_turn_config = Self::build_per_turn_config(&session_configuration, cwd.clone());
        // PF-84 (#572): Bedrock's own metadata after a switch to Bedrock.
        let model_info = crate::model_info_for_provider::model_info_for_config(
            &self.services.models_manager,
            session_configuration.collaboration_mode.model(),
            &per_turn_config,
        )
        .await;
        self.services
            .thread_extension_data
            .insert(model_info.clone());

        let multi_agent_version = match multi_agent_runtime {
            TurnMultiAgentRuntime::ResolveAndStore => {
||||||| 413492cd6c
        let per_turn_config = Self::build_per_turn_config(&session_configuration, cwd.clone());
        let model_info = self
            .services
            .models_manager
            .get_model_info(
                session_configuration.collaboration_mode.model(),
                &per_turn_config.to_models_manager_config(),
            )
            .await;
        self.services
            .thread_extension_data
            .insert(model_info.clone());

        let multi_agent_version = match multi_agent_runtime {
            TurnMultiAgentRuntime::ResolveAndStore => {
=======
        let network_permission_profile = primary_turn_environment
            .map(TurnEnvironment::permission_profile)
            .cloned()
            .unwrap_or_else(|| session_configuration.permission_profile());
        let model_info = session_configuration
            .step_settings
            .resolve_model_info(
                self.services.models_manager.as_ref(),
                &session_configuration.model_info_overrides,
            )
            .await;
        let multi_agent_version = match build_mode {
            TurnContextBuildMode::Full => {
                // Only execution and initial context creation publish model metadata.
                self.services
                    .thread_extension_data
                    .insert_arc(Arc::clone(&model_info.model));
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
                self.resolve_multi_agent_version_for_model(&model_info, &per_turn_config)
            }
            TurnContextBuildMode::StartupPrewarm | TurnContextBuildMode::InjectItems => {
                per_turn_config.multi_agent_version_for_model(
                    self.multi_agent_version()
                        .or(model_info.multi_agent_version),
                )
            }
        };
        let plugins_input = per_turn_config.plugins_config_input();
        let plugin_outcome = self
            .services
            .plugins_manager
            .plugins_for_config(&plugins_input)
            .await
            .without_plugins(&session_configuration.disabled_plugin_ids);
        let trusted_plugin_roots = TrustedPluginRoots::from_plugin_load_outcome(
            &plugin_outcome,
            per_turn_config.codex_home.as_path(),
        );
        let skills_snapshot = if matches!(build_mode, TurnContextBuildMode::InjectItems)
            || crate::guardian::is_basic_session_source(&session_configuration.session_source)
            || (per_turn_config
                .features
                .enabled(Feature::SkipHostSkillDiscovery)
                && !self.services.extensions.requires_host_skill_discovery())
        {
            // Executor and orchestrator catalogs are supplied independently of host skills.
            HostSkillsSnapshot::new(Arc::new(SkillLoadOutcome::default()))
        } else {
            let effective_skill_roots = plugin_outcome.effective_plugin_skill_roots();
            let (skills_input, fs) = crate::skills::skills_load_input_for_environments(
                &per_turn_config,
                &self.services.plugins_manager,
                effective_skill_roots,
                &turn_environments,
            );
            self.services
                .skills_service
                .snapshot_for_config(&skills_input, fs)
                .await
        };
        let step_settings = Arc::new(ResolvedStepSettings::new(
            Arc::clone(&session_configuration.step_settings),
            Arc::new(model_info),
            &self.features,
        ));
        let mut turn_context: TurnContext = Self::make_turn_context(
            self.thread_id(),
            self.session_id(),
<<<<<<< HEAD
            Some(self.services.auth_manager()),
||||||| 413492cd6c
            Some(Arc::clone(&self.services.auth_manager)),
=======
            Arc::clone(&self.services.granted_permissions_by_environment_id),
            Some(Arc::clone(&self.services.auth_manager)),
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
            &self.services.session_telemetry,
            session_configuration.provider.clone(),
            &session_configuration,
            multi_agent_version,
            per_turn_config,
            step_settings,
            &self.services.models_manager,
            self.services
                .network_proxy
                .load_full()
                .as_ref()
                .and_then(|started_proxy| {
                    Self::managed_network_proxy_active_for_permission_profile(
                        &network_permission_profile,
                    )
                    .then(|| started_proxy.proxy())
                }),
            turn_environments,
            cwd,
            sub_id,
            skills_snapshot,
        );
        turn_context.code_mode_available = self.services.code_mode_service.is_available();
        turn_context.extension_data.insert(trusted_plugin_roots);
        turn_context.active_host_plugin_identities = Some(
            plugin_outcome
                .plugins()
                .iter()
                .filter(|plugin| plugin.is_active())
                .map(|plugin| PluginIdentity {
                    plugin_id: plugin.config_name.clone(),
                    remote_plugin_id: plugin.remote_plugin_id.clone(),
                })
                .collect(),
        );
        turn_context.realtime_active = self.conversation.running_state().await.is_some();

        turn_context.final_output_json_schema = options.final_output_json_schema;
        turn_context.cyber_access_program =
            cyber_access_program::for_provider(&turn_context.config, options.cyber_access_program);
        // Tool observers retain this turn's selection even after a later turn starts.
        if let Some(program) = turn_context.cyber_access_program {
            turn_context.extension_data.insert(program);
        }
        let turn_context = Arc::new(turn_context);
        if git_enrichment_policy == GitEnrichmentPolicy::Fresh
            && turn_context
                .initial_environments
                .single_local_environment_cwd()
                .is_some()
        {
            turn_context
                .turn_metadata_state
                .spawn_git_enrichment_task(Arc::clone(&self.services.git_root_discovery));
        }
        turn_context
    }

    pub(crate) async fn maybe_emit_model_warnings_for_turn(&self, tc: &TurnContext) {
        if tc.model_info().origin == ModelOrigin::Synthetic {
            self.send_event(
                tc,
                EventMsg::Warning(WarningEvent {
                    message: format!(
                        "Model metadata for `{}` not found. Defaulting to fallback metadata; this can degrade performance and cause issues.",
                        tc.model_info().slug
                    ),
                }),
            )
            .await;
        }

        if !tc.code_mode_available
            && matches!(
                crate::tools::requested_tool_mode(tc, tc.model_info()),
                codex_protocol::openai_models::ToolMode::CodeMode
                    | codex_protocol::openai_models::ToolMode::CodeModeOnly
            )
            && let Some(message) = self
                .services
                .code_mode_service
                .take_unavailable_warning(crate::tools::effective_tool_mode(tc, tc.model_info()))
        {
            self.send_event(tc, EventMsg::Warning(WarningEvent { message }))
                .await;
        }

        if let Some(message) =
            unsupported_code_mode_warning(tc.model_info(), tc.config.features.get())
        {
            self.send_event(tc, EventMsg::Warning(WarningEvent { message }))
                .await;
        }
    }

    /// Builds a context without starting work or changing the current environments.
    pub(crate) async fn new_default_turn(&self) -> Arc<TurnContext> {
        self.new_default_turn_for(TurnContextBuildMode::Full).await
    }

    /// Captures current recording settings without discovering skills.
    /// This context must not be used to capture or execute a step.
    pub(crate) async fn new_inject_items_context(&self) -> Arc<TurnContext> {
        self.new_default_turn_for(TurnContextBuildMode::InjectItems)
            .await
    }

    async fn new_default_turn_for(&self, build_mode: TurnContextBuildMode) -> Arc<TurnContext> {
        let session_configuration = self.default_turn_configuration().await;
        let turn_environments = self.services.turn_environments.snapshot().await;
        Box::pin(self.new_turn_context_from_configuration(
            self.next_internal_sub_id(),
            session_configuration,
            turn_environments,
            NewTurnContextOptions::default(),
            build_mode,
            self.git_enrichment_policy,
        ))
        .await
    }

    /// Prepares new work on the saved environments; context-only callers keep the current ones.
    pub(crate) async fn new_turn_with_default_settings(
        &self,
        sub_id: String,
        options: NewTurnContextOptions,
    ) -> Arc<TurnContext> {
        let session_configuration = self.default_turn_configuration().await;
        self.new_turn_with_configuration(sub_id, session_configuration, options)
            .await
    }

    /// Builds new work from the exact configuration selected by turn admission.
    pub(crate) async fn new_turn_with_configuration(
        &self,
        sub_id: String,
        session_configuration: SessionConfiguration,
        options: NewTurnContextOptions,
    ) -> Arc<TurnContext> {
        let turn_environments = self
            .activate_turn_environments(&session_configuration)
            .await;
        self.new_turn_from_configuration(sub_id, session_configuration, turn_environments, options)
            .await
    }

    pub(crate) async fn new_startup_prewarm_turn_with_sub_id(
        &self,
        sub_id: String,
    ) -> Arc<TurnContext> {
        let session_configuration = self.default_turn_configuration().await;
        self.new_startup_prewarm_turn_from_configuration(sub_id, session_configuration)
            .await
    }

    async fn default_turn_configuration(&self) -> SessionConfiguration {
        let state = self.state.lock().await;
        state.session_configuration.clone()
    }
}

#[cfg(test)]
<<<<<<< HEAD
#[path = "turn_context_tests.rs"]
||||||| 413492cd6c
=======
#[path = "active_plugin_inventory_tests.rs"]
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
mod tests;
