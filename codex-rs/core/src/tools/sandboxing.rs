//! Shared approvals and sandboxing traits used by tool runtimes.
//!
//! Consolidates the approval flow primitives (`ApprovalDecision`, `ApprovalStore`,
//! `Approvable`) together with the sandbox orchestration traits
//! and helpers (`Sandboxable`, `ToolRuntime`, `SandboxAttempt`, etc.).

use crate::sandboxing::ExecOptions;
use crate::sandboxing::SandboxPermissions;
use crate::session::session::Session;
use crate::session::step_context::StepContext;
use crate::session::turn_context::TurnEnvironment;
use crate::state::SessionServices;
use crate::tools::hook_names::HookToolName;
use crate::tools::network_approval::NetworkApprovalSpec;
use codex_file_system::FileSystemSandboxContext;
use codex_file_system::WindowsSandboxSelection;
use codex_network_proxy::NetworkProxy;
use codex_protocol::approvals::ExecPolicyAmendment;
use codex_protocol::config_types::WindowsSandboxLevel;
use codex_protocol::error::CodexErr;
use codex_protocol::permissions::FileSystemSandboxKind;
use codex_protocol::permissions::FileSystemSandboxPolicy;
use codex_protocol::protocol::AskForApproval;
use codex_protocol::protocol::ReviewDecision;
pub(crate) use codex_protocol::sandbox::SandboxOverride;
use codex_sandboxing::SandboxCommand;
use codex_sandboxing::SandboxManager;
use codex_sandboxing::SandboxTransformRequest;
use codex_sandboxing::SandboxType;
use codex_sandboxing::SandboxablePreference;
use codex_sandboxing::policy_transforms::effective_permission_profile;
use codex_tools::ToolName;
use codex_utils_path_uri::PathConvention;
use codex_utils_path_uri::PathUri;
use futures::Future;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

#[derive(Clone, Default, Debug)]
pub(crate) struct ApprovalStore {
    // Store serialized keys for generic caching across requests.
    map: HashMap<String, ReviewDecision>,
    /// Policy epoch and revocation generation the entries were made under.
    policy: Option<(u64, u64)>,
}

impl ApprovalStore {
    /// PF-23-S03: a committed security transition drops every cached
    /// decision.
    pub(crate) fn fence(&mut self, policy: Option<(u64, u64)>) {
        if self.policy != policy {
            self.map.clear();
            self.policy = policy;
        }
    }

    /// The policy the entries were made under.
    pub(crate) fn policy(&self) -> Option<(u64, u64)> {
        self.policy
    }

    pub fn get<K>(&self, key: &K) -> Option<ReviewDecision>
    where
        K: Serialize,
    {
        let s = serde_json::to_string(key).ok()?;
        self.map.get(&s).cloned()
    }

    pub fn put<K>(&mut self, key: K, value: ReviewDecision)
    where
        K: Serialize,
    {
        if let Ok(s) = serde_json::to_string(&key) {
            self.map.insert(s, value);
        }
    }
}

tokio::task_local! {
    /// Issue #513: set once a person is asked about the approval being
    /// resolved (see [`asking_person`]).
    static PERSON_ASKED: std::cell::Cell<bool>;
}

/// Issue #513: runs `approval` (a runtime's `start_approval_async`) and says
/// whether a person was asked during it, as opposed to an answer remembered
/// from earlier (the approval cache) or a stored grant. Unmarked counts as
/// not asked (fail closed).
pub(crate) async fn asking_person<Fut>(approval: impl FnOnce() -> Fut) -> (ReviewDecision, bool)
where
    Fut: Future<Output = ReviewDecision>,
{
    // The whole call, its synchronous part included, runs in the scope.
    PERSON_ASKED
        .scope(std::cell::Cell::new(false), async {
            let decision = approval().await;
            (decision, PERSON_ASKED.with(std::cell::Cell::get))
        })
        .await
}

/// Issue #513: a person is being asked about this approval now.
pub(crate) fn note_person_asked() {
    let _ = PERSON_ASKED.try_with(|asked| asked.set(true));
}

/// Takes a vector of approval keys and returns a ReviewDecision.
/// There will be one key in most cases, but apply_patch can modify multiple files at once.
///
/// - If all keys are already approved for session, we skip prompting.
/// - If the user approves for session, we store the decision for each key individually
///   so future requests touching any subset can also skip prompting.
///
/// With `fresh_human_authority` (PF-30-S03 post-taint actions) the cache is
/// neither read nor written: the human decides this exact action, and a
/// "for session" answer counts for this one call only.
pub(crate) async fn with_cached_approval<K, F, Fut>(
    services: &SessionServices,
    // Name of the tool, used for metrics collection.
    tool_name: &str,
    keys: Vec<K>,
    fresh_human_authority: bool,
    fetch: F,
) -> ReviewDecision
where
    K: Serialize,
    F: FnOnce() -> Fut,
    Fut: Future<Output = ReviewDecision>,
{
    if fresh_human_authority {
        note_person_asked();
        return match fetch().await {
            ReviewDecision::ApprovedForSession => ReviewDecision::Approved,
            decision => decision,
        };
    }
    // To be defensive here, don't bother with checking the cache if keys are empty.
    if keys.is_empty() {
        note_person_asked();
        return fetch().await;
    }

    let (already_approved, decided_under) = {
        let store = services.approval_cache().await;
        (
            keys.iter()
                .all(|key| matches!(store.get(key), Some(ReviewDecision::ApprovedForSession))),
            store.policy(),
        )
    };

    if already_approved {
        return ReviewDecision::ApprovedForSession;
    }

    note_person_asked();
    let decision = fetch().await;

    services.session_telemetry.counter(
        "codex.approval.requested",
        /*inc*/ 1,
        &[
            ("tool", tool_name),
            ("approved", decision.to_opaque_string()),
        ],
    );

    if matches!(decision, ReviewDecision::ApprovedForSession) {
        services.remember_approvals(keys, decided_under).await;
    }

    decision
}

<<<<<<< HEAD
#[derive(Clone)]
pub(crate) struct ApprovalCtx<'a> {
    pub session: &'a Arc<Session>,
    pub turn: &'a Arc<TurnContext>,
    pub call_id: &'a str,
    pub retry_reason: Option<String>,
    pub network_approval_context: Option<NetworkApprovalContext>,
    /// PF-30-S03: ask the human about this exact action; no cache, no preapproval.
    pub fresh_human_authority: bool,
||||||| 413492cd6c
#[derive(Clone)]
pub(crate) struct ApprovalCtx<'a> {
    pub session: &'a Arc<Session>,
    pub turn: &'a Arc<TurnContext>,
    pub call_id: &'a str,
    pub retry_reason: Option<String>,
    pub network_approval_context: Option<NetworkApprovalContext>,
=======
#[derive(Clone, Debug, Default)]
pub(crate) struct ApprovalRequestReasons {
    pub(crate) approval: Option<String>,
    pub(crate) retry: Option<String>,
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
}

pub(crate) use super::approvals::ApprovalAction;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PermissionRequestPayload {
    pub tool_name: HookToolName,
    pub tool_input: serde_json::Value,
    pub tool_cwd: Option<PathUri>,
}

impl PermissionRequestPayload {
    pub(crate) fn bash(command: String, description: Option<String>, cwd: PathUri) -> Self {
        let mut tool_input = serde_json::Map::new();
        tool_input.insert("command".to_string(), serde_json::Value::String(command));
        if let Some(description) = description {
            tool_input.insert(
                "description".to_string(),
                serde_json::Value::String(description),
            );
        }

        Self {
            tool_name: HookToolName::bash(),
            tool_input: serde_json::Value::Object(tool_input),
            tool_cwd: Some(cwd),
        }
    }
}

// Specifies what tool orchestrator should do with a given tool call.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ExecApprovalRequirement {
    /// No approval required for this tool call.
    Skip {
        /// The first attempt should skip sandboxing (e.g., when explicitly
        /// greenlit by policy).
        bypass_sandbox: bool,
        /// Proposed execpolicy amendment to skip future approvals for similar commands
        /// Only applies if the command fails to run in sandbox and codex prompts the user to run outside the sandbox.
        proposed_execpolicy_amendment: Option<ExecPolicyAmendment>,
    },
    /// Approval required for this tool call.
    NeedsApproval {
        reason: Option<String>,
        /// Proposed execpolicy amendment to skip future approvals for similar commands
        /// See core/src/exec_policy.rs for more details on how proposed_execpolicy_amendment is determined.
        proposed_execpolicy_amendment: Option<ExecPolicyAmendment>,
    },
    /// A fresh human decision is required for each execution attempt.
    NeedsHumanApproval { reason: Option<String> },
    /// Execution forbidden for this tool call.
    Forbidden { reason: String },
}

impl ExecApprovalRequirement {
    pub fn proposed_execpolicy_amendment(&self) -> Option<&ExecPolicyAmendment> {
        match self {
            Self::NeedsApproval {
                proposed_execpolicy_amendment: Some(prefix),
                ..
            } => Some(prefix),
            Self::Skip {
                proposed_execpolicy_amendment: Some(prefix),
                ..
            } => Some(prefix),
            _ => None,
        }
    }
}

/// - Never: do not ask
/// - OnRequest: ask unless filesystem access is unrestricted
/// - Granular: ask unless filesystem access is unrestricted, but auto-reject
///   when granular sandbox approval is disabled.
/// - UnlessTrusted: always ask
pub(crate) fn default_exec_approval_requirement(
    policy: AskForApproval,
    file_system_sandbox_policy: &FileSystemSandboxPolicy,
) -> ExecApprovalRequirement {
    let needs_approval = match policy {
        AskForApproval::Never => false,
        AskForApproval::OnRequest | AskForApproval::Granular(_) => {
            matches!(
                file_system_sandbox_policy.kind,
                FileSystemSandboxKind::Restricted
            )
        }
        AskForApproval::UnlessTrusted => true,
    };

    if needs_approval
        && matches!(
            policy,
            AskForApproval::Granular(granular_config)
                if !granular_config.allows_sandbox_approval()
        )
    {
        ExecApprovalRequirement::Forbidden {
            reason: "approval policy disallowed sandbox approval prompt".to_string(),
        }
    } else if needs_approval {
        ExecApprovalRequirement::NeedsApproval {
            reason: None,
            proposed_execpolicy_amendment: None,
        }
    } else {
        ExecApprovalRequirement::Skip {
            bypass_sandbox: false,
            proposed_execpolicy_amendment: None,
        }
    }
}

pub(crate) fn ensure_native_sandbox(
    sandbox_override: SandboxOverride,
    sandbox: SandboxType,
) -> Result<(), ToolError> {
    if sandbox_override == SandboxOverride::EscalatedSandboxWithRestrictions
        && sandbox == SandboxType::None
    {
        return Err(ToolError::Rejected(
            "command escalation with denied reads requires an available filesystem sandbox"
                .to_string(),
        ));
    }
    Ok(())
}

pub(crate) fn sandbox_override_for_first_attempt(
    sandbox_permissions: SandboxPermissions,
    exec_approval_requirement: &ExecApprovalRequirement,
    file_system_sandbox_policy: &FileSystemSandboxPolicy,
    already_approved: bool,
) -> SandboxOverride {
    // Only actual approval of an explicit escalation may widen the filesystem;
    // a command allow rule does not authorize removing filesystem restrictions.
    if !unsandboxed_execution_allowed(file_system_sandbox_policy) {
        return if sandbox_permissions.requires_escalated_permissions() && already_approved {
            SandboxOverride::EscalatedSandboxWithRestrictions
        } else {
            SandboxOverride::NoOverride
        };
    }

    // ExecPolicy `Allow` can intentionally imply full trust (Skip + bypass_sandbox=true),
    // which supersedes `with_additional_permissions` sandboxed execution hints.
    if matches!(
        exec_approval_requirement,
        ExecApprovalRequirement::Skip {
            bypass_sandbox: true,
            ..
        }
    ) {
        return SandboxOverride::BypassSandboxFirstAttempt;
    }

    if sandbox_permissions.requires_escalated_permissions() {
        SandboxOverride::BypassSandboxFirstAttempt
    } else {
        SandboxOverride::NoOverride
    }
}

/// Returns true when the active filesystem policy can be represented by
/// running without a filesystem sandbox.
///
/// Denied reads only exist inside the sandbox. If a policy contains any
/// denied-read paths, bypassing the sandbox would silently grant those reads,
/// so escalation must keep the command sandboxed with the denied reads intact.
/// #807: so does keeping the macOS keychain services away (#758), by itself
/// (it denies no file read).
pub(crate) fn unsandboxed_execution_allowed(
    file_system_sandbox_policy: &FileSystemSandboxPolicy,
) -> bool {
    !file_system_sandbox_policy.has_denied_read_restrictions()
        && !(cfg!(target_os = "macos") && file_system_sandbox_policy.denies_keychain_services())
}

pub(crate) fn sandbox_permissions_preserving_denied_reads(
    sandbox_permissions: SandboxPermissions,
    file_system_sandbox_policy: &FileSystemSandboxPolicy,
) -> SandboxPermissions {
    if sandbox_permissions.requires_escalated_permissions()
        && !unsandboxed_execution_allowed(file_system_sandbox_policy)
    {
        // `RequireEscalated` normally asks the executor to bypass the sandbox.
        // When denied reads are active, that would drop the only mechanism that
        // enforces them, so fall back to the default sandboxed attempt instead.
        SandboxPermissions::UseDefault
    } else {
        sandbox_permissions
    }
}

/// [`sandbox_permissions_preserving_denied_reads`] for a run in `session`:
/// under security level Moderate (#747, decision D3), and under Permissive
/// with protected launch armed (#739), a request to run outside the sandbox
/// runs inside it too.
pub(crate) fn sandbox_permissions_for_session(
    sandbox_permissions: SandboxPermissions,
    file_system_sandbox_policy: &FileSystemSandboxPolicy,
    session: &crate::session::session::Session,
) -> SandboxPermissions {
    if sandbox_permissions.requires_escalated_permissions()
        && crate::security::protected_additional::keeps_sandbox_always(session)
    {
        SandboxPermissions::UseDefault
    } else {
        sandbox_permissions_preserving_denied_reads(sandbox_permissions, file_system_sandbox_policy)
    }
}

pub(crate) fn managed_network_for_sandbox_permissions(
    network: Option<&NetworkProxy>,
    sandbox_permissions: SandboxPermissions,
) -> Option<&NetworkProxy> {
    if sandbox_permissions.requires_escalated_permissions() {
        None
    } else {
        network
    }
}

pub(crate) trait Approvable<Req> {
    /// Return per-request sandbox permissions for first-attempt sandbox
    /// selection. Most tools use the ambient sandbox policy unchanged.
    fn sandbox_permissions(&self, _req: &Req) -> SandboxPermissions {
        SandboxPermissions::UseDefault
    }

    /// The permissions this request adds to the sandbox for its run (issue
    /// #245: checked against the protected paths before it runs). Required,
    /// so a new runtime cannot skip the check by accident.
    fn additional_permissions<'a>(
        &self,
        req: &'a Req,
    ) -> Option<&'a codex_protocol::models::AdditionalPermissionProfile>;

    fn should_bypass_approval(&self, policy: AskForApproval, already_approved: bool) -> bool {
        if already_approved {
            // We do not ask one more time
            return true;
        }
        matches!(policy, AskForApproval::Never)
    }

    /// Return `Some(_)` to specify a custom exec approval requirement, or `None`
    /// to fall back to policy-based default.
    fn exec_approval_requirement(&self, _req: &Req) -> Option<ExecApprovalRequirement> {
        None
    }

<<<<<<< HEAD
    /// Called with each decision the person or the automatic reviewer gave
    /// on this request (not permission hooks, which show nothing). Clients
    /// show the command as running while it is reviewed (#466).
    fn note_review_decision(&mut self, _decision: &ReviewDecision, _ctx: &ApprovalCtx<'_>) {}

    /// Return hook input for approval-time policy hooks when this runtime wants
    /// hook evaluation to run before guardian or user approval.
    fn permission_request_payload(&self, _req: &Req) -> Option<PermissionRequestPayload> {
        None
    }

||||||| 413492cd6c
    /// Return hook input for approval-time policy hooks when this runtime wants
    /// hook evaluation to run before guardian or user approval.
    fn permission_request_payload(&self, _req: &Req) -> Option<PermissionRequestPayload> {
        None
    }

=======
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
    /// Decide we can request an approval for no-sandbox execution.
    fn wants_no_sandbox_approval(&self, policy: AskForApproval) -> bool {
        match policy {
            AskForApproval::UnlessTrusted => true,
            AskForApproval::Never => false,
            AskForApproval::OnRequest => false,
            AskForApproval::Granular(granular_config) => granular_config.sandbox_approval,
        }
    }

    fn approval_action(&self, req: &Req, call_id: &str) -> std::io::Result<ApprovalAction>;
}

pub(crate) trait Sandboxable {
    fn sandbox_preference(&self) -> SandboxablePreference;
    fn escalate_on_failure(&self) -> bool {
        true
    }
}

pub(crate) struct ToolCtx {
    pub session: Arc<Session>,
    pub step_context: Arc<StepContext>,
    pub cancellation_token: CancellationToken,
    pub call_id: String,
    pub tool_name: ToolName,
}

#[derive(Debug)]
pub(crate) enum ToolError {
    /// Refused by policy or setup, before execution or (a network-policy
    /// denial, #873) while the command ran; no approval decision is implied.
    Rejected(String),
    /// An approval request resolved without permission to proceed.
    Declined(String),
    Codex(CodexErr),
}

pub(crate) trait ToolRuntime<Req, Out>: Approvable<Req> + Sandboxable {
    fn turn_environment<'a>(&self, req: &'a Req) -> &'a TurnEnvironment;

    fn uses_executor_managed_process_sandbox(&self, _req: &Req) -> bool {
        false
    }

    fn network_approval_spec(&self, _req: &Req, _ctx: &ToolCtx) -> Option<NetworkApprovalSpec> {
        None
    }

    fn sandbox_cwd<'a>(&self, _req: &'a Req) -> Option<&'a PathUri> {
        None
    }

    /// #943 item 11: the folder the command itself runs in (the model's
    /// `workdir`), for the post-command `.git` check only.
    fn command_cwd(&self, _req: &Req) -> Option<codex_utils_absolute_path::AbsolutePathBuf> {
        None
    }

    /// D48 (c): the `PATH` the command runs with, where it runs on this host
    /// (a program named without a folder is looked up there); `None` where
    /// unknown or remote.
    fn local_search_path<'a>(&self, _req: &'a Req) -> Option<&'a str> {
        None
    }

    async fn run(
        &mut self,
        req: &Req,
        attempt: &SandboxAttempt<'_>,
        ctx: &ToolCtx,
    ) -> Result<Out, ToolError>;
}

pub(crate) struct SandboxAttempt<'a> {
    pub sandbox_override: SandboxOverride,
    pub sandbox: SandboxType,
    /// Whether policy requested sandboxing, independent of this host's concrete wrapper.
    pub sandbox_requested: bool,
    pub permissions: &'a codex_protocol::models::PermissionProfile,
    /// Canonical permissions before this host materializes workspace roots.
    pub exec_server_permissions: &'a codex_protocol::models::PermissionProfile,
    pub enforce_managed_network: bool,
    pub(crate) manager: &'a SandboxManager,
    pub(crate) sandbox_cwd: &'a PathUri,
    pub(crate) workspace_roots: &'a [PathUri],
    pub sandbox_exe: Option<&'a std::path::PathBuf>,
    // TODO(anp): Reconcile these attempt settings with TurnEnvironment::sandbox_context
    // so process execution and patch writes honor the selected environment's backend.
    pub use_legacy_landlock: bool,
    pub windows_sandbox_type: SandboxType,
    pub windows_sandbox_level: codex_protocol::config_types::WindowsSandboxLevel,
    pub network_denial_cancellation_token: Option<CancellationToken>,
    pub(crate) network_proxy: Option<&'a NetworkProxy>,
}

pub(crate) fn executor_windows_sandbox_level(
    windows_sandbox_type: SandboxType,
    windows_sandbox_level: WindowsSandboxLevel,
    cwd: &PathUri,
) -> WindowsSandboxLevel {
    if windows_sandbox_type != SandboxType::WindowsMxc
        && windows_sandbox_level == WindowsSandboxLevel::Disabled
        && cwd.infer_path_convention() == Some(PathConvention::Windows)
    {
        WindowsSandboxLevel::RestrictedToken
    } else {
        windows_sandbox_level
    }
}

pub(crate) fn executor_windows_sandbox_selection(
    windows_sandbox_type: SandboxType,
    windows_sandbox_level: WindowsSandboxLevel,
    cwd: &PathUri,
) -> WindowsSandboxSelection {
    configured_windows_sandbox_selection(
        windows_sandbox_type,
        executor_windows_sandbox_level(windows_sandbox_type, windows_sandbox_level, cwd),
        cwd,
    )
}

pub(crate) fn configured_windows_sandbox_selection(
    windows_sandbox_type: SandboxType,
    windows_sandbox_level: WindowsSandboxLevel,
    cwd: &PathUri,
) -> WindowsSandboxSelection {
    if cwd.infer_path_convention() != Some(PathConvention::Windows) {
        return WindowsSandboxSelection::Disabled;
    }

    if windows_sandbox_type == SandboxType::WindowsMxc {
        WindowsSandboxSelection::Mxc
    } else {
        windows_sandbox_level.into()
    }
}

impl<'a> SandboxAttempt<'a> {
    /// Whether this attempt bypasses sandboxing required by its ambient policy.
    /// Use the requested policy, not the controller's wrapper, for remote executors.
    pub(crate) fn is_escalated(&self) -> bool {
        !self.sandbox_requested
            && self.manager.should_sandbox(
                self.permissions,
                SandboxablePreference::Auto,
                self.enforce_managed_network,
            )
    }

    pub(crate) fn network_proxy<'b>(
        &'b self,
        fallback: Option<&'b NetworkProxy>,
    ) -> Option<&'b NetworkProxy> {
        // Execution-only proxies need no fallback; offline attempts must not revive one.
        if self.enforce_managed_network {
            self.network_proxy.or(fallback)
        } else {
            None
        }
    }

<<<<<<< HEAD
    /// PF-27-S02: when the secretless launch contract is armed, refuses
    /// launches it cannot contain, scrubs the command's environment and
    /// returns `permissions` with the protected paths denied.
    /// `proxy_enforced`: the launch goes through the managed network proxy,
    /// which on Windows selects the elevated sandbox backend.
    fn protect_launch(
        &self,
        contract: Option<&crate::security::launch_contract::LaunchContract>,
        command: &mut SandboxCommand,
        exec_server: bool,
        proxy_enforced: bool,
    ) -> Result<Option<codex_protocol::models::PermissionProfile>, CodexErr> {
        let Some(contract) = contract else {
            return Ok(None);
        };
        let cwd = self.sandbox_cwd.to_abs_path()?;
        let windows_elevated = codex_sandboxing::windows_sandbox_uses_elevated_backend(
            self.windows_sandbox_level,
            proxy_enforced,
        );
        let protected = contract
            .protect_launch(
                self.sandbox,
                self.sandbox_requested,
                exec_server,
                windows_elevated,
                command,
                self.permissions,
                cwd.as_path(),
            )
            .map_err(|denied| CodexErr::UnsupportedOperation(denied.to_string()))?;
        #[cfg(windows)]
        if windows_elevated {
            contract
                .protect_new_codex_home_files()
                .inspect_err(crate::security::inspection::record_launch_denial)
                .map_err(|denied| CodexErr::UnsupportedOperation(denied.to_string()))?;
        }
        Ok(Some(protected))
    }

    pub fn env_for(
||||||| 413492cd6c
    pub fn env_for(
=======
    pub async fn env_for(
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        &self,
        command: SandboxCommand,
        options: ExecOptions,
        network: Option<&NetworkProxy>,
        environment_id: Option<&str>,
    ) -> Result<crate::sandboxing::ExecRequest, CodexErr> {
        self.env_for_with_contract(
            crate::security::launch_contract::active(),
            command,
            options,
            network,
            environment_id,
        )
    }

    /// [`Self::env_for`] with an explicit PF-27-S02 launch contract.
    pub(crate) fn env_for_with_contract(
        &self,
        contract: Option<&crate::security::launch_contract::LaunchContract>,
        mut command: SandboxCommand,
        options: ExecOptions,
        network: Option<&NetworkProxy>,
        environment_id: Option<&str>,
    ) -> Result<crate::sandboxing::ExecRequest, CodexErr> {
        let network = self.network_proxy(network);
<<<<<<< HEAD
        let protected = self.protect_launch(
            contract,
            &mut command,
            /*exec_server*/ false,
            /*proxy_enforced*/ network.is_some(),
        )?;
        let request = self
||||||| 413492cd6c
        let request = self
=======
        let mut request = self
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
            .manager
            .transform(SandboxTransformRequest {
                command,
                permissions: protected.as_ref().unwrap_or(self.permissions),
                sandbox: self.sandbox,
                enforce_managed_network: self.enforce_managed_network,
                environment_id,
                network,
                sandbox_policy_cwd: self.sandbox_cwd,
                sandbox_exe: self.sandbox_exe.map(std::path::PathBuf::as_path),
                use_legacy_landlock: self.use_legacy_landlock,
                windows_sandbox_level: self.windows_sandbox_level,
            })
            .map_err(CodexErr::from)?;
        request.sandbox_override = self.sandbox_override;
        let workspace_roots = self
            .workspace_roots
            .iter()
            .map(PathUri::to_abs_path)
            .collect::<std::io::Result<Vec<_>>>()?;
<<<<<<< HEAD
        let mut exec_request = crate::sandboxing::ExecRequest::from_sandbox_exec_request(
            request,
            options,
            workspace_roots,
        );
        if exec_request.sandbox == SandboxType::WindowsRestrictedToken {
            let sandbox_cwd = self.sandbox_cwd.to_abs_path()?;
            if codex_sandboxing::windows_sandbox_uses_elevated_backend(
                exec_request.windows_sandbox_level,
                exec_request.network.is_some(),
            ) {
                // #294: without this the elevated Windows sandbox applies none
                // of the profile's deny entries, including the contract's.
                crate::exec::attach_windows_sandbox_filesystem_overrides(
                    &mut exec_request,
                    &sandbox_cwd,
                    contract,
                )?;
            } else {
                // #300: the unelevated backend cannot block reads, so a
                // profile that denies some is refused, as in
                // `process_exec_tool_call`, rather than run with its
                // protected files readable.
                codex_sandboxing::refuse_unenforceable_windows_read_restrictions(
                    exec_request.sandbox,
                    &exec_request.permission_profile,
                )
                .map_err(CodexErr::UnsupportedOperation)?;
            }
        }
        Ok(exec_request)
||||||| 413492cd6c
        Ok(crate::sandboxing::ExecRequest::from_sandbox_exec_request(
            request,
            options,
            workspace_roots,
        ))
=======
        crate::sandboxing::ExecRequest::from_sandbox_exec_request(request, options, workspace_roots)
            .await
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
    }

    pub async fn env_for_exec_server(
        &self,
        command: SandboxCommand,
        options: ExecOptions,
    ) -> Result<crate::sandboxing::ExecRequest, CodexErr> {
        self.env_for_exec_server_with_contract(
            crate::security::launch_contract::active(),
            command,
            options,
        )
    }

    /// [`Self::env_for_exec_server`] with an explicit launch contract. A
    /// remote environment rebuilds the child environment on its own host, so
    /// an armed contract refuses it (PF-27-S02).
    pub(crate) fn env_for_exec_server_with_contract(
        &self,
        contract: Option<&crate::security::launch_contract::LaunchContract>,
        mut command: SandboxCommand,
        options: ExecOptions,
    ) -> Result<crate::sandboxing::ExecRequest, CodexErr> {
        let managed_network = command.managed_network.clone();
        self.protect_launch(
            contract,
            &mut command,
            /*exec_server*/ true,
            /*proxy_enforced*/ false,
        )?;
        let exec_server_permissions = effective_permission_profile(
            self.exec_server_permissions,
            command.additional_permissions.as_ref(),
        );
        let mut request = self
            .manager
            .transform(SandboxTransformRequest {
                command,
                permissions: self.permissions,
                // The exec-server must receive the native command, not this host's wrapper.
                sandbox: SandboxType::None,
                enforce_managed_network: self.enforce_managed_network,
                environment_id: None,
                network: None,
                sandbox_policy_cwd: self.sandbox_cwd,
                sandbox_exe: None,
                use_legacy_landlock: self.use_legacy_landlock,
                windows_sandbox_level: self.windows_sandbox_level,
            })
            .map_err(CodexErr::from)?;
        request.sandbox_override = self.sandbox_override;
        let mut exec_request =
            crate::sandboxing::ExecRequest::from_sandbox_exec_request(request, options, Vec::new())
                .await?;
        exec_request.exec_server_managed_network = managed_network;
        if self.sandbox_requested {
            exec_request.exec_server_sandbox = Some(FileSystemSandboxContext {
                permissions: exec_server_permissions,
                sandbox_override: self.sandbox_override,
                cwd: exec_request.windows_sandbox_policy_cwd.clone(),
                workspace_roots: self.workspace_roots.to_vec(),
                user_home_dir: None,
                temporary_directories: None,
                windows_sandbox_selection: executor_windows_sandbox_selection(
                    self.windows_sandbox_type,
                    self.windows_sandbox_level,
                    self.sandbox_cwd,
                ),
                windows_sandbox_proxy_settings_mode: None,
                use_legacy_landlock: self.use_legacy_landlock,
            });
            exec_request.exec_server_enforce_managed_network = self.enforce_managed_network;
        }
        Ok(exec_request)
    }
}

#[cfg(test)]
#[path = "sandboxing_tests.rs"]
mod tests;

#[cfg(all(test, windows))]
#[path = "sandboxing_windows_tests.rs"]
mod windows_tests;
