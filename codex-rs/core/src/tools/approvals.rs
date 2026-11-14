//! Central approval policy-stage execution and reviewer routing.
//! Managed human checkpoints cannot be satisfied by hooks, automated review, or reusable approvals.

use crate::command_canonicalization::canonicalize_command_for_approval;
use crate::exec_policy::prompt_is_rejected_by_policy;
use crate::guardian::GuardianNetworkAccessTrigger;
use crate::guardian::GuardianReviewContext;
use crate::guardian::GuardianReviewOptions;
use crate::guardian::decide_approval;
use crate::guardian::new_guardian_review_id;
use crate::guardian::spawn_approval_decision;
use crate::hook_runtime::run_permission_request_hooks;
use crate::mcp_tool_call::request_mcp_tool_user_approval;
use crate::sandboxing::SandboxPermissions;
use crate::session::NetworkApprovalTurn;
use crate::session::session::Session;
use crate::tools::flat_tool_name;
use crate::tools::hook_names::HookToolName;
use crate::tools::runtimes::apply_patch::ApplyPatchApprovalKey;
use crate::tools::runtimes::unified_exec::UnifiedExecApprovalKey;
use crate::tools::sandboxing::ApprovalRequestReasons;
use crate::tools::sandboxing::ExecApprovalRequirement;
use crate::tools::sandboxing::PermissionRequestPayload;
use crate::tools::sandboxing::ToolCtx;
use crate::tools::sandboxing::ToolError;
use crate::tools::sandboxing::ToolRuntime;
use crate::tools::sandboxing::asking_person;
use crate::tools::sandboxing::with_cached_approval;
use codex_analytics::GuardianApprovalRequestSource;
use codex_config::types::AppToolApproval;
use codex_hooks::PermissionRequestDecision;
use codex_otel::ToolDecisionSource;
use codex_prompts::ResolvedModelMessages;
use codex_protocol::approvals::ExecApprovalKind;
use codex_protocol::approvals::ExecPolicyAmendment;
use codex_protocol::approvals::NetworkApprovalContext;
use codex_protocol::approvals::NetworkApprovalProtocol;
use codex_protocol::config_types::ApprovalsReviewer;
use codex_protocol::error::CodexErr;
use codex_protocol::mcp_approval_meta::McpApprovalCancellationReason;
use codex_protocol::models::AdditionalPermissionProfile;
use codex_protocol::openai_models::ModelInfo;
use codex_protocol::protocol::AskForApproval;
use codex_protocol::protocol::FileChange;
use codex_protocol::protocol::NetworkPolicyRuleAction;
use codex_protocol::protocol::ReviewDecision;
use codex_protocol::request_permissions::RequestPermissionProfile;
use codex_tools::ToolName;
use codex_utils_absolute_path::AbsolutePathBuf;
use codex_utils_path_uri::PathConvention;
use codex_utils_path_uri::PathUri;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;
use tracing::error;
use tracing::warn;

#[derive(Clone)]
pub(crate) struct ApprovalContext {
    /// Preserve the command's policy result through review and retries. Other actions
    /// use their existing reviewer routing without an execution policy requirement.
    pub(crate) exec_requirement: Option<ExecApprovalRequirement>,
    pub(crate) review_context: GuardianReviewContext,
    pub(crate) cancellation_token: Option<CancellationToken>,
    pub(crate) call_id: String,
    pub(crate) tool_name: ToolName,
    pub(crate) strict_auto_review: bool,
    pub(crate) approval_reason: Option<String>,
    pub(crate) retry_reason: Option<String>,
    pub(crate) network_approval_context: Option<NetworkApprovalContext>,
}

impl ApprovalContext {
    fn requires_human_approval(&self) -> bool {
        matches!(
            self.exec_requirement,
            Some(ExecApprovalRequirement::NeedsHumanApproval { .. })
        )
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(tag = "tool", rename_all = "snake_case")]
pub(crate) enum ApprovalAction {
    ExecCommand {
        id: String,
        environment_id: String,
        command: Vec<String>,
        #[serde(skip_serializing)]
        hook_command: String,
        cwd: PathUri,
        sandbox_permissions: SandboxPermissions,
        additional_permissions: Option<AdditionalPermissionProfile>,
        justification: Option<String>,
        tty: bool,
        proposed_execpolicy_amendment: Option<ExecPolicyAmendment>,
    },
    WriteStdin {
        id: String,
        approval_id: String,
        environment_id: String,
        process_id: i32,
        original_command: String,
        input: String,
        cwd: PathUri,
        tty: bool,
        sandbox_permissions: SandboxPermissions,
        additional_permissions: Option<AdditionalPermissionProfile>,
        #[serde(skip)]
        additional_permissions_preapproved: bool,
    },
    ApplyPatch {
        id: String,
        environment_id: String,
        cwd: PathUri,
        files: Vec<PathUri>,
        patch: String,
        #[serde(skip_serializing)]
        changes: Arc<HashMap<PathBuf, FileChange>>,
        permissions_preapproved: bool,
    },
    McpToolCall {
        id: String,
        environment_id: String,
        server: String,
        tool_name: String,
        arguments: Option<serde_json::Value>,
        connector_id: Option<String>,
        connector_name: Option<String>,
        connector_description: Option<String>,
        connected_account_email: Option<String>,
        tool_title: Option<String>,
        tool_description: Option<String>,
        annotations: Option<crate::guardian::GuardianMcpAnnotations>,
        #[serde(skip_serializing)]
        hook_tool_name: HookToolName,
        approval_policy: AskForApproval,
        reviewer: ApprovalsReviewer,
        approval_mode: AppToolApproval,
        allow_session_remember: bool,
        allow_persistent_approval: bool,
    },
    NetworkAccess {
        id: String,
        turn_id: String,
        environment_id: String,
        target: String,
        host: String,
        protocol: NetworkApprovalProtocol,
        port: u16,
        trigger: Option<GuardianNetworkAccessTrigger>,
        #[serde(skip_serializing)]
        hook_command: String,
        #[serde(skip_serializing)]
        hook_run_id: String,
        command: Vec<String>,
        cwd: AbsolutePathBuf,
    },
    RequestPermissions {
        id: String,
        environment_id: String,
        turn_id: String,
        reason: Option<String>,
        permissions: RequestPermissionProfile,
    },
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, serde::Serialize)]
#[serde(untagged)]
pub(crate) enum ApprovalCacheKey {
    ExecCommand(UnifiedExecApprovalKey),
    ApplyPatch(ApplyPatchApprovalKey),
}

impl ApprovalAction {
    /// #722 (D246): a permission hook may not allow a remote network request
    /// without a prompt; the automatic reviewer or the person decides it.
    fn permission_hook_may_allow(
        &self,
        environments: &crate::environment_selection::TurnEnvironmentSnapshot,
    ) -> bool {
        !matches!(self, Self::NetworkAccess { environment_id, .. }
            if crate::tools::network_approval::is_remote_environment(environments, environment_id))
    }

    fn hook_environment_id(&self) -> &str {
        match self {
            Self::ExecCommand { environment_id, .. }
            | Self::WriteStdin { environment_id, .. }
            | Self::ApplyPatch { environment_id, .. }
            | Self::McpToolCall { environment_id, .. }
            | Self::NetworkAccess { environment_id, .. }
            | Self::RequestPermissions { environment_id, .. } => environment_id,
        }
    }

    pub(crate) fn permission_request_payload(&self) -> PermissionRequestPayload {
        let mut payload = match self {
            Self::ExecCommand {
                hook_command,
                justification,
                cwd,
                ..
            } => PermissionRequestPayload::bash(
                hook_command.clone(),
                justification.clone(),
                cwd.clone(),
            ),
            Self::WriteStdin {
                id,
                approval_id,
                environment_id,
                process_id,
                original_command: _,
                input,
                cwd,
                tty,
                sandbox_permissions,
                additional_permissions,
                additional_permissions_preapproved: _,
            } => PermissionRequestPayload {
                tool_name: HookToolName::new("write_stdin"),
                tool_cwd: Some(cwd.clone()),
                tool_input: serde_json::json!({
                    "session_id": process_id,
                    "chars": input,
                    "parent_call_id": id,
                    "approval_id": approval_id,
                    "environment_id": environment_id,
                    "cwd": cwd,
                    "tty": tty,
                    "sandbox_permissions": sandbox_permissions,
                    "additional_permissions": additional_permissions,
                }),
            },
            Self::ApplyPatch { patch, .. } => PermissionRequestPayload {
                tool_name: HookToolName::apply_patch(),
                tool_cwd: None,
                tool_input: serde_json::json!({ "command": patch }),
            },
            Self::McpToolCall {
                hook_tool_name,
                arguments,
                ..
            } => PermissionRequestPayload {
                tool_name: hook_tool_name.clone(),
                tool_cwd: None,
                tool_input: arguments
                    .clone()
                    .unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::new())),
            },
            Self::NetworkAccess {
                hook_command,
                target,
                cwd,
                trigger,
                ..
            } => PermissionRequestPayload::bash(
                hook_command.clone(),
                Some(format!("network-access {target}")),
                trigger.as_ref().map_or_else(
                    || PathUri::from_abs_path(cwd),
                    |trigger| trigger.cwd.clone(),
                ),
            ),
            Self::RequestPermissions {
                reason,
                permissions,
                ..
            } => PermissionRequestPayload {
                tool_name: HookToolName::new("request_permissions"),
                tool_cwd: None,
                tool_input: serde_json::json!({
                    "reason": reason,
                    "permissions": permissions,
                }),
            },
        };
        // Command hooks run on the host. Keep remote paths in the action's
        // original tool input, but never use them as the hook's top-level cwd.
        if matches!(
            self,
            Self::ExecCommand { environment_id, .. }
                | Self::WriteStdin { environment_id, .. }
                | Self::NetworkAccess { environment_id, .. }
                if environment_id != codex_exec_server::LOCAL_ENVIRONMENT_ID
        ) {
            payload.tool_cwd = None;
        }
        payload
    }

    pub(crate) fn cache_keys(&self) -> Vec<ApprovalCacheKey> {
        match self {
            Self::ExecCommand {
                environment_id,
                command,
                cwd,
                tty,
                sandbox_permissions,
                additional_permissions,
                ..
            } => vec![ApprovalCacheKey::ExecCommand(UnifiedExecApprovalKey {
                environment_id: environment_id.clone(),
                executable: command.first().cloned(),
                command: canonicalize_command_for_approval(command),
                cwd: cwd.clone(),
                tty: *tty,
                sandbox_permissions: *sandbox_permissions,
                additional_permissions: additional_permissions.clone(),
            })],
            Self::McpToolCall { .. }
            | Self::NetworkAccess { .. }
            | Self::RequestPermissions { .. }
            | Self::WriteStdin { .. } => Vec::new(),
            Self::ApplyPatch {
                environment_id,
                files,
                ..
            } => files
                .iter()
                .cloned()
                .map(|path| {
                    ApprovalCacheKey::ApplyPatch(ApplyPatchApprovalKey {
                        environment_id: environment_id.clone(),
                        path,
                    })
                })
                .collect(),
        }
    }

    pub(crate) fn into_guardian_request(
        self,
        exec_command_cwd_convention: Option<PathConvention>,
    ) -> std::io::Result<crate::guardian::GuardianApprovalRequest> {
        Ok(match self {
            Self::ExecCommand {
                id,
                environment_id,
                command,
                cwd,
                sandbox_permissions,
                additional_permissions,
                justification,
                tty,
                ..
            } => crate::guardian::GuardianApprovalRequest::ExecCommand {
                id,
                environment_id,
                command,
                guardian_cwd: codex_utils_path_uri::LegacyAppPathString::from_path_uri(
                    &cwd,
                    exec_command_cwd_convention.ok_or_else(|| {
                        std::io::Error::other("missing exec command cwd convention")
                    })?,
                )
                .map_err(std::io::Error::other)?,
                cwd,
                sandbox_permissions,
                additional_permissions,
                justification,
                tty,
            },
            Self::WriteStdin {
                id,
                approval_id,
                environment_id,
                process_id,
                original_command,
                input,
                cwd,
                tty,
                sandbox_permissions,
                additional_permissions,
                additional_permissions_preapproved: _,
            } => crate::guardian::GuardianApprovalRequest::WriteStdin {
                id,
                approval_id,
                environment_id,
                process_id,
                original_command,
                input,
                cwd,
                tty,
                sandbox_permissions,
                additional_permissions,
            },
            Self::ApplyPatch {
                id,
                environment_id,
                cwd,
                files,
                patch,
                ..
            } => crate::guardian::GuardianApprovalRequest::ApplyPatch {
                id,
                environment_id,
                cwd,
                files,
                patch,
            },
            Self::McpToolCall {
                id,
                server,
                tool_name,
                arguments,
                connector_id,
                connector_name,
                connector_description,
                connected_account_email,
                tool_title,
                tool_description,
                annotations,
                ..
            } => crate::guardian::GuardianApprovalRequest::McpToolCall {
                id,
                server,
                tool_name,
                arguments,
                connector_id,
                connector_name,
                connector_description,
                connected_account_email,
                tool_title,
                tool_description,
                annotations,
            },
            Self::NetworkAccess {
                id,
                turn_id,
                environment_id,
                target,
                host,
                protocol,
                port,
                trigger,
                ..
            } => crate::guardian::GuardianApprovalRequest::NetworkAccess {
                id,
                turn_id,
                environment_id,
                target,
                host,
                protocol,
                port,
                trigger,
            },
            Self::RequestPermissions {
                id,
                environment_id,
                turn_id,
                reason,
                permissions,
            } => crate::guardian::GuardianApprovalRequest::RequestPermissions {
                id,
                environment_id,
                turn_id,
                reason,
                permissions,
            },
        })
    }
}

/// Who answered an approval.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ApprovalResolutionSource {
    Hook,
    Guardian,
    /// A person, asked about this request now.
    User,
    /// Answered without asking anyone: an earlier "don't ask again" answer
    /// (the approval cache) or a stored permission grant (issue #513).
    Remembered,
}

/// Preserves the approval path even when the decision becomes a tool error.
/// The source identifies the reviewer, not the actor that caused cancellation.
#[must_use]
pub(crate) struct ApprovalResult {
    pub(crate) result: Result<ReviewDecision, ToolError>,
    pub(crate) source: Option<ApprovalResolutionSource>,
    pub(crate) cancellation_reason: Option<McpApprovalCancellationReason>,
    /// The answer as given (before it became `result`), when a hook, the
    /// automatic reviewer or a person answered (#437, #466).
    pub(crate) decision: Option<ReviewDecision>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ApprovalResolution {
    pub(crate) decision: ReviewDecision,
    pub(crate) source: ApprovalResolutionSource,
    pub(crate) cancellation_reason: Option<McpApprovalCancellationReason>,
}

impl ApprovalResolution {
    pub(super) fn user(decision: ReviewDecision) -> Self {
        Self {
            decision,
            source: ApprovalResolutionSource::User,
            cancellation_reason: None,
        }
    }

    fn into_tool_result(self, model_info: &ModelInfo) -> Result<ReviewDecision, ToolError> {
        let source = self.source;
        match self.decision {
            ReviewDecision::ApprovedMcpPolicyAmendment => {
                error!("Tool approval received ApprovedMcpPolicyAmendment");
                Err(ToolError::Rejected(
                    "Error while requesting approval".to_string(),
                ))
            }
            ReviewDecision::NetworkPolicyAmendment {
                network_policy_amendment,
            } if network_policy_amendment.action == NetworkPolicyRuleAction::Deny => {
                let rejection = match source {
                    ApprovalResolutionSource::Hook => "rejected by configuration",
                    ApprovalResolutionSource::Guardian => {
                        "automatic approval review denied the action"
                    }
                    ApprovalResolutionSource::User | ApprovalResolutionSource::Remembered => {
                        "rejected by user"
                    }
                };
                Err(ToolError::Declined(rejection.to_string()))
            }
            // #822/#873: an approval request that resolved without permission
            // is a decline, not a policy refusal.
            ReviewDecision::Denied { rejection } => Err(ToolError::Declined(rejection)),
            ReviewDecision::TimedOut => Err(ToolError::Declined(
                ResolvedModelMessages::from_model(model_info)
                    .auto_review()
                    .timeout_instructions
                    .to_string(),
            )),
            // Upstream: aborting an approval ends the turn.
            ReviewDecision::Abort => Err(ToolError::Codex(CodexErr::TurnAborted)),
            decision => Ok(decision),
        }
    }
}

impl Session {
    async fn check_approval_requirement(&self, ctx: &ApprovalContext) -> Result<(), ToolError> {
        if ctx.requires_human_approval() {
            if let Some(reason) = prompt_is_rejected_by_policy(
                ctx.review_context.approval_policy,
                /*prompt_is_rule*/ true,
            ) {
                return Err(ToolError::Rejected(reason.to_string()));
            }
            let config = self.get_config().await;
            let requirements = config.config_layer_stack.requirements();
            if ctx.strict_auto_review
                || ctx.review_context.turn().strict_auto_review_enabled()
                || requirements
                    .approvals_reviewer
                    .can_set(&ApprovalsReviewer::User)
                    .is_err()
                || requirements.auto_review_required_for_model(&ctx.review_context.model_info.slug)
            {
                return Err(ToolError::Rejected("Managed command policy requires human approval, but this session requires automatic review.".to_string()));
            }
        }
        Ok(())
    }

    pub(crate) async fn request_approval(
        self: &Arc<Self>,
        action: ApprovalAction,
        ctx: ApprovalContext,
    ) -> ApprovalResult {
        self.request_approval_with_user_prompt(
            &action,
            &ctx,
            self.request_user_approval(&action, &ctx),
        )
        .await
    }

    pub(super) async fn request_approval_with_user_prompt(
        self: &Arc<Self>,
        action: &ApprovalAction,
        ctx: &ApprovalContext,
        user_prompt: impl Future<Output = ApprovalResolution>,
    ) -> ApprovalResult {
        self.request_approval_with_authority(
            action,
            ctx,
            user_prompt,
            /*fresh_human_authority*/ false,
        )
        .await
    }

    /// [`Self::request_approval_with_user_prompt`]; with
    /// `fresh_human_authority` (PF-30-S03) only the person decides: a
    /// permission hook can still refuse but not allow, and no automatic
    /// reviewer is asked.
    async fn request_approval_with_authority(
        self: &Arc<Self>,
        action: &ApprovalAction,
        ctx: &ApprovalContext,
        user_prompt: impl Future<Output = ApprovalResolution>,
        fresh_human_authority: bool,
    ) -> ApprovalResult {
        let human_only = fresh_human_authority || ctx.requires_human_approval();
        if let Err(err) = self.check_approval_requirement(ctx).await {
            return ApprovalResult {
                result: Err(err),
                source: None,
                cancellation_reason: None,
                decision: None,
            };
        }
        // Retained authority outside current grants needs a new sandbox approval.
        // Current grants permit review of the input; they never approve the input itself.
        let policy = ctx.review_context.turn().approval_policy();
        if matches!(action, ApprovalAction::WriteStdin { sandbox_permissions, additional_permissions_preapproved, .. }
            if sandbox_permissions.requests_sandbox_override()
                && (!additional_permissions_preapproved || matches!(policy, AskForApproval::Never)))
            && !(ctx.strict_auto_review && matches!(policy, AskForApproval::Never))
            && let Some(reason) =
                prompt_is_rejected_by_policy(policy, /*prompt_is_rule*/ false)
        {
            return ApprovalResult {
                result: Err(ToolError::Rejected(reason.to_string())),
                source: None,
                cancellation_reason: None,
                decision: None,
            };
        }
        let is_mcp_tool_call = matches!(action, ApprovalAction::McpToolCall { .. });
        let is_network_approval = matches!(action, ApprovalAction::NetworkAccess { .. });
        let permission_request_run_id = match action {
            ApprovalAction::NetworkAccess { hook_run_id, .. } => hook_run_id.clone(),
            _ if ctx.retry_reason.is_some() => format!("{}:retry", ctx.call_id),
            _ => ctx.call_id.clone(),
        };

        let hook_may_allow =
            !human_only && action.permission_hook_may_allow(ctx.review_context.environments());
        // Hook denials always win. Managed checkpoints require a person even
        // when a hook allows the action or the session uses automatic review.
        let mut resolution = match run_permission_request_hooks(
            self,
            &ctx.review_context,
            &permission_request_run_id,
            action.permission_request_payload(),
            action.hook_environment_id(),
        )
        .await
        {
            // A hook can still refuse a post-taint action, but only the person
            // can allow it (PF-30-S03).
            Some(PermissionRequestDecision::Allow) if hook_may_allow => ApprovalResolution {
                decision: ReviewDecision::Approved,
                source: ApprovalResolutionSource::Hook,
                cancellation_reason: None,
            },
            Some(PermissionRequestDecision::Deny { message }) => ApprovalResolution {
                decision: ReviewDecision::denied(message),
                source: ApprovalResolutionSource::Hook,
                cancellation_reason: None,
            },
            Some(PermissionRequestDecision::Allow) | None => {
                self.request_reviewer_approval(action, ctx, user_prompt, human_only)
                    .await
            }
        };
        if resolution.decision == ReviewDecision::Approved {
            if let Err(ToolError::Rejected(reason)) = self.check_approval_requirement(ctx).await {
                resolution.decision = ReviewDecision::denied(reason);
            }
            if human_only
                && ctx
                    .cancellation_token
                    .as_ref()
                    .is_some_and(CancellationToken::is_cancelled)
            {
                resolution.decision = ReviewDecision::Abort;
            }
        }
        // Network approvals record their final telemetry after validation and persistence.
        if !is_network_approval {
            record_resolution(ctx, &resolution);
        }
        let source = Some(resolution.source);
        let cancellation_reason = resolution.cancellation_reason;
        let decision = Some(resolution.decision.clone());
        if is_mcp_tool_call && resolution.decision == ReviewDecision::ApprovedMcpPolicyAmendment {
            return ApprovalResult {
                result: Ok(resolution.decision),
                source,
                cancellation_reason,
                decision,
            };
        }
        if is_network_approval {
            match (&resolution.decision, resolution.source) {
                (
                    ReviewDecision::NetworkPolicyAmendment {
                        network_policy_amendment,
                    },
                    _,
                ) if network_policy_amendment.action == NetworkPolicyRuleAction::Deny => {
                    return ApprovalResult {
                        result: Ok(resolution.decision),
                        source,
                        cancellation_reason,
                        decision,
                    };
                }
                (ReviewDecision::Abort, ApprovalResolutionSource::Guardian) => {
                    return ApprovalResult {
                        result: Err(ToolError::Rejected(
                            "automatic approval review was cancelled".to_string(),
                        )),
                        source,
                        cancellation_reason,
                        decision,
                    };
                }
                _ => {}
            }
        }
        ApprovalResult {
            result: resolution.into_tool_result(ctx.review_context.turn().model_info()),
            source,
            cancellation_reason,
            decision,
        }
    }

    async fn request_reviewer_approval(
        self: &Arc<Self>,
        action: &ApprovalAction,
        ctx: &ApprovalContext,
        user_prompt: impl Future<Output = ApprovalResolution>,
        human_only: bool,
    ) -> ApprovalResolution {
        if human_only {
            ask_person(action, user_prompt).await
        } else if let Some(decision) = self.request_guardian_approval(action.clone(), ctx).await {
            ApprovalResolution {
                cancellation_reason: (decision == ReviewDecision::Abort)
                    .then_some(McpApprovalCancellationReason::Interrupted),
                decision,
                source: ApprovalResolutionSource::Guardian,
            }
        } else {
            ask_person(action, user_prompt).await
        }
    }

    pub(crate) async fn request_guardian_approval(
        self: &Arc<Self>,
        action: ApprovalAction,
        ctx: &ApprovalContext,
    ) -> Option<ReviewDecision> {
        let mut context = ctx.review_context.clone();
        if let ApprovalAction::McpToolCall {
            approval_policy,
            reviewer,
            ..
        } = &action
        {
            context.approval_policy = *approval_policy;
            context.approvals_reviewer = *reviewer;
        }
        let is_network_approval = matches!(&action, ApprovalAction::NetworkAccess { .. });
        let review_id = new_guardian_review_id();
        let action = crate::guardian::from_approval_action(action, &context);

        let reasons = ApprovalRequestReasons {
            approval: ctx.approval_reason.clone(),
            retry: ctx.retry_reason.clone(),
        };
        let options = GuardianReviewOptions {
            require_guardian: ctx.strict_auto_review,
            plugin_attribution_override: None,
            approval_request_source: GuardianApprovalRequestSource::MainTurn,
            external_cancel: ctx.cancellation_token.clone(),
            require_synchronous_review: false,
        };
        if ctx.cancellation_token.is_some() {
            spawn_approval_decision(
                Arc::clone(self),
                context,
                review_id,
                action,
                reasons,
                options,
            )
            .await
            .unwrap_or_else(|_| {
                Some(ReviewDecision::denied(
                    "automatic approval review could not complete",
                ))
            })
        } else if is_network_approval {
            let review_cancel = CancellationToken::new();
            let review_cancel_guard = review_cancel.clone().drop_guard();
            let review = tokio::spawn(decide_approval(
                Arc::clone(self),
                context,
                review_id,
                action,
                reasons,
                GuardianReviewOptions {
                    external_cancel: Some(review_cancel),
                    ..options
                },
            ));
            let decision = review.await.unwrap_or_else(|err| {
                warn!("network Guardian review task failed: {err}");
                Some(ReviewDecision::denied(
                    "automatic approval review could not complete",
                ))
            });
            drop(review_cancel_guard.disarm());
            decision
        } else {
            decide_approval(
                Arc::clone(self),
                context,
                review_id,
                action,
                reasons,
                options,
            )
            .await
        }
    }

    pub(super) async fn request_user_approval(
        &self,
        action: &ApprovalAction,
        ctx: &ApprovalContext,
    ) -> ApprovalResolution {
        self.request_user_approval_with_authority(action, ctx, /*fresh_human_authority*/ false)
            .await
    }

    /// [`Self::request_user_approval`]; with `fresh_human_authority`
    /// (PF-30-S03) the person decides this exact action: no answer remembered
    /// from earlier, no stored grant, no "don't ask again" rule, and a "for
    /// session" answer counts for this one call only.
    pub(super) async fn request_user_approval_with_authority(
        &self,
        action: &ApprovalAction,
        ctx: &ApprovalContext,
        fresh_human_authority: bool,
    ) -> ApprovalResolution {
        let decision = match action {
            ApprovalAction::ExecCommand {
                environment_id,
                command,
                cwd,
                additional_permissions,
                justification,
                proposed_execpolicy_amendment,
                ..
            } => {
                let reason = ctx
                    .retry_reason
                    .clone()
                    .or_else(|| ctx.approval_reason.clone())
                    .or_else(|| justification.clone());
                let human_once = ctx.requires_human_approval();
                let request = || async {
                    self.request_command_approval(
                        ctx.review_context.turn(),
                        if human_once {
                            ExecApprovalKind::HumanCommand
                        } else {
                            ExecApprovalKind::Command
                        },
                        ctx.review_context.model_context(),
                        ctx.call_id.clone(),
                        Some(&ctx.call_id),
                        human_once.then(|| uuid::Uuid::new_v4().to_string()),
                        Some(environment_id.clone()),
                        command.clone(),
                        cwd.clone(),
                        reason,
                        if human_once {
                            None
                        } else {
                            ctx.network_approval_context
                                .clone()
                                .map(|context| (context, NetworkApprovalTurn::AnyActive))
                        },
                        // A post-taint approval covers this action only: no
                        // "don't ask again" rule (PF-30-S03).
                        if human_once || fresh_human_authority {
                            None
                        } else {
                            proposed_execpolicy_amendment.clone()
                        },
                        additional_permissions.clone(),
                        human_once.then(|| {
                            vec![
                                ReviewDecision::Approved,
                                ReviewDecision::denied("rejected by user"),
                                ReviewDecision::Abort,
                            ]
                        }),
                        /*plugin_attribution_override*/ None,
                    )
                    .await
                };
                if human_once {
                    crate::tools::sandboxing::note_person_asked();
                    request().await
                } else {
                    let tool_name = "unified_exec";
                    let policy_fingerprint = ctx
                        .review_context
                        .environments()
                        .turn_environments()
                        .find(|environment| environment.selection.environment_id == *environment_id)
                        .and_then(|environment| environment.config().exec_policy.as_ref())
                        .map(codex_execpolicy::RequirementsExecPolicy::fingerprint);
                    let cache_keys = action
                        .cache_keys()
                        .into_iter()
                        .map(|key| (key, &policy_fingerprint))
                        .collect();
                    with_cached_approval(
                        &self.services,
                        tool_name,
                        cache_keys,
                        fresh_human_authority,
                        request,
                    )
                    .await
                }
            }
            ApprovalAction::WriteStdin {
                id,
                approval_id,
                environment_id,
                process_id,
                input,
                cwd,
                additional_permissions,
                ..
            } => {
                crate::tools::sandboxing::note_person_asked();
                self.request_command_approval(
                    ctx.review_context.turn(),
                    ExecApprovalKind::WriteStdin,
                    ctx.review_context.model_context(),
                    id.clone(),
                    Some(&ctx.call_id),
                    Some(approval_id.clone()),
                    Some(environment_id.clone()),
                    vec![
                        "write_stdin".to_string(),
                        "--session-id".to_string(),
                        process_id.to_string(),
                        input.clone(),
                    ],
                    cwd.clone(),
                    ctx.approval_reason.clone(),
                    /*network_approval*/ None,
                    /*proposed_execpolicy_amendment*/ None,
                    additional_permissions.clone(),
                    Some(vec![ReviewDecision::Approved, ReviewDecision::Abort]),
                    /*plugin_attribution_override*/ None,
                )
                .await
            }
            ApprovalAction::ApplyPatch {
                changes,
                permissions_preapproved,
                ..
            } => {
                let reason = ctx
                    .retry_reason
                    .clone()
                    .or_else(|| ctx.approval_reason.clone());
                if *permissions_preapproved && reason.is_none() && !fresh_human_authority {
                    ReviewDecision::Approved
                } else if reason.is_some() {
                    crate::tools::sandboxing::note_person_asked();
                    match self
                        .request_patch_approval(
                            ctx.review_context.turn(),
                            ctx.call_id.clone(),
                            changes.as_ref().clone(),
                            reason,
                            /*grant_root*/ None,
                        )
                        .await
                    {
                        ReviewDecision::ApprovedForSession if fresh_human_authority => {
                            ReviewDecision::Approved
                        }
                        decision => decision,
                    }
                } else {
                    with_cached_approval(
                        &self.services,
                        "apply_patch",
                        action.cache_keys(),
                        fresh_human_authority,
                        || async {
                            self.request_patch_approval(
                                ctx.review_context.turn(),
                                ctx.call_id.clone(),
                                changes.as_ref().clone(),
                                /*reason*/ None,
                                /*grant_root*/ None,
                            )
                            .await
                        },
                    )
                    .await
                }
            }
            ApprovalAction::McpToolCall { .. } => {
                return request_mcp_tool_user_approval(
                    self,
                    ctx.review_context.turn(),
                    &ctx.call_id,
                    action,
                )
                .await;
            }
            ApprovalAction::NetworkAccess {
                environment_id,
                command,
                cwd,
                trigger,
                ..
            } => {
                self.request_command_approval(
                    ctx.review_context.turn(),
                    ExecApprovalKind::Command,
                    ctx.review_context.model_context(),
                    ctx.call_id.clone(),
                    trigger.as_ref().map(|trigger| trigger.call_id.as_str()),
                    /*approval_id*/ None,
                    Some(environment_id.clone()),
                    command.clone(),
                    cwd.clone().into(),
                    ctx.approval_reason.clone(),
                    ctx.network_approval_context
                        .clone()
                        .map(|context| (context, NetworkApprovalTurn::Originating)),
                    /*proposed_execpolicy_amendment*/ None,
                    /*additional_permissions*/ None,
                    /*available_decisions*/ None,
                    /*plugin_attribution_override*/ None,
                )
                .await
            }
            ApprovalAction::RequestPermissions { .. } => {
                unreachable!("permission requests are routed directly to Guardian")
            }
        };
        ApprovalResolution::user(decision)
    }
}

fn record_resolution(ctx: &ApprovalContext, resolution: &ApprovalResolution) {
    let source = match resolution.source {
        ApprovalResolutionSource::Hook => ToolDecisionSource::Config,
        ApprovalResolutionSource::Guardian => ToolDecisionSource::AutomatedReviewer,
        ApprovalResolutionSource::User | ApprovalResolutionSource::Remembered => {
            ToolDecisionSource::User
        }
    };
    ctx.review_context.turn().session_telemetry.tool_decision(
        &ctx.tool_name,
        &ctx.call_id,
        &resolution.decision,
        Some(source),
    );
}

/// Issue #513: awaits a person's prompt for `action` and says whether a
/// person was asked during it, as opposed to an answer remembered from
/// earlier (the approval cache) or a stored grant. Tracked for the tool
/// runtimes' actions (commands, typing, patches); unmarked counts as not
/// asked (fail closed).
async fn ask_person(
    action: &ApprovalAction,
    user_prompt: impl Future<Output = ApprovalResolution>,
) -> ApprovalResolution {
    let tracked = matches!(
        action,
        ApprovalAction::ExecCommand { .. }
            | ApprovalAction::WriteStdin { .. }
            | ApprovalAction::ApplyPatch { .. }
    );
    let (mut resolution, asked) = asking_person(|| user_prompt).await;
    if tracked && !asked && resolution.source == ApprovalResolutionSource::User {
        resolution.source = ApprovalResolutionSource::Remembered;
    }
    resolution
}

/// Who an approval is expected to go to, for what the prompt says (notes
/// meant for a person). Who really answered is the result's
/// [`ApprovalResolutionSource`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ApprovalReviewer {
    Guardian,
    User,
}

impl ApprovalReviewer {
    pub(super) fn for_context(ctx: &ApprovalContext) -> Self {
        let review = &ctx.review_context;
        if ctx.requires_human_approval() {
            Self::User
        } else if ctx.strict_auto_review
            || (matches!(
                review.approval_policy,
                AskForApproval::OnRequest | AskForApproval::Granular(_)
            ) && review.approvals_reviewer == ApprovalsReviewer::AutoReview)
        {
            Self::Guardian
        } else {
            Self::User
        }
    }
}

/// Who lists a refused approval under the inspector's recent denials (#437).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DeclineRecord {
    /// [`resolve_tool_approval`] does.
    Here,
    /// The caller records the outcome itself (protected actions after
    /// untrusted content).
    ByCaller,
}

/// Corbanu's approval of a tool runtime's request (commands, patches):
/// [`Session::request_approval`] with PF-30-S03 `fresh_human_authority`
/// (only the person decides this exact action), the D50 note when it is
/// asked again, the inspector's pending and declined approvals (#437) and
/// the runtime's review note (#466).
pub(super) async fn resolve_tool_approval<Rq, Out, T>(
    tool: &mut T,
    req: &Rq,
    ctx: ApprovalContext,
    tool_ctx: &ToolCtx,
    fresh_human_authority: bool,
    decline_record: DeclineRecord,
    asked_again: bool,
) -> ApprovalResult
where
    T: ToolRuntime<Rq, Out>,
{
    let action = match tool.approval_action(req, &ctx.call_id) {
        Ok(action) => action,
        Err(err) => {
            return ApprovalResult {
                result: Err(ToolError::Rejected(format!(
                    "could not prepare approval action: {err}"
                ))),
                source: None,
                cancellation_reason: None,
                decision: None,
            };
        }
    };
    resolve_tool_action_approval::<Rq, Out, T>(
        tool,
        action,
        ctx,
        tool_ctx,
        fresh_human_authority,
        decline_record,
        asked_again,
    )
    .await
}

/// [`resolve_tool_approval`] of an `action` the caller prepared from the
/// runtime's request (a retry outside the sandbox reviews the authority it
/// actually runs with).
pub(super) async fn resolve_tool_action_approval<Rq, Out, T>(
    tool: &mut T,
    action: ApprovalAction,
    mut ctx: ApprovalContext,
    tool_ctx: &ToolCtx,
    fresh_human_authority: bool,
    decline_record: DeclineRecord,
    asked_again: bool,
) -> ApprovalResult
where
    T: ToolRuntime<Rq, Out>,
{
    let what = approval_kind(&flat_tool_name(&tool_ctx.tool_name));
    let thread = tool_ctx.session.thread_id();
    // An interrupted wait is "declined by you" only when the person was
    // asked (not a hook or the automatic reviewer).
    let asks_person =
        fresh_human_authority || ApprovalReviewer::for_context(&ctx) == ApprovalReviewer::User;
    // D50: only a person is told why they are asked again.
    if asked_again && asks_person {
        asked_again_reason(&mut ctx, &action);
    }
    let mut pending = (decline_record == DeclineRecord::Here)
        .then(|| crate::security::inspection::PendingApproval::new(thread, what, asks_person));
    let session = &tool_ctx.session;
    let result = session
        .request_approval_with_authority(
            &action,
            &ctx,
            session.request_user_approval_with_authority(&action, &ctx, fresh_human_authority),
            fresh_human_authority,
        )
        .await;
    if let Some(pending) = pending.as_mut() {
        pending.answered();
        if let (Some(decision), Some(source)) = (result.decision.as_ref(), result.source)
            && let Some(outcome) = decline_outcome(decision, source)
        {
            crate::security::inspection::record_declined_approval(thread, what, outcome);
        }
    }
    // #466: a permission hook shows nothing.
    if let (Some(decision), Some(source)) = (result.decision.as_ref(), result.source)
        && source != ApprovalResolutionSource::Hook
    {
        tool.note_review_decision(decision, ctx.network_approval_context.is_some());
    }
    result
}

/// D50 (#1001): the note an approval asked again carries, because the one
/// shown before a security transition went stale.
pub(crate) const ASKED_AGAIN_NOTE: &str = "The security level changed while the earlier \
     prompt was open, so it is asked again under the new level.";

/// D50: puts [`ASKED_AGAIN_NOTE`] ahead of the reason the prompt shows (the
/// retry reason, else the approval reason, else the model's justification).
/// A retry keeps its retry reason; otherwise the note joins the approval
/// reason, so the permission hooks' run id stays the call's.
fn asked_again_reason(ctx: &mut ApprovalContext, action: &ApprovalAction) {
    let with_note = |why: Option<String>| {
        Some(match why {
            Some(why) => format!("{ASKED_AGAIN_NOTE} {why}"),
            None => ASKED_AGAIN_NOTE.to_string(),
        })
    };
    if ctx.retry_reason.is_some() {
        ctx.retry_reason = with_note(ctx.retry_reason.take());
    } else {
        let justification = match action {
            ApprovalAction::ExecCommand { justification, .. } => justification.clone(),
            _ => None,
        };
        ctx.approval_reason = with_note(ctx.approval_reason.take().or(justification));
    }
}

/// What the inspector says about a refused approval, or `None` when it was
/// approved.
fn decline_outcome(
    decision: &ReviewDecision,
    source: ApprovalResolutionSource,
) -> Option<&'static str> {
    let refused = match decision {
        ReviewDecision::NetworkPolicyAmendment {
            network_policy_amendment,
        } => network_policy_amendment.action == NetworkPolicyRuleAction::Deny,
        ReviewDecision::Denied { .. } | ReviewDecision::Abort => true,
        ReviewDecision::TimedOut => return Some("not answered in time"),
        _ => false,
    };
    refused.then_some(match source {
        ApprovalResolutionSource::User | ApprovalResolutionSource::Remembered => "declined by you",
        ApprovalResolutionSource::Hook => "declined by a permission hook",
        ApprovalResolutionSource::Guardian => "declined by the automatic reviewer",
    })
}

fn approval_kind(tool_name: &str) -> &'static str {
    match tool_name {
        "exec_command" | "shell" | "shell_command" | "local_shell" | "container.exec" => {
            "command approval"
        }
        "apply_patch" => "file edit approval",
        _ => "tool approval",
    }
}

#[cfg(all(test, unix))]
#[path = "approvals_tests.rs"]
mod tests;

#[cfg(all(test, windows))]
#[path = "approvals_windows_tests.rs"]
mod windows_tests;

#[cfg(test)]
#[path = "refusal_diagnostics_tests.rs"]
mod refusal_diagnostics_tests;
