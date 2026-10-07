/*
Module: orchestrator

Central place for approvals + sandbox selection + retry semantics. Drives a
simple sequence for any ToolRuntime: approval → select sandbox → attempt →
retry with an escalated sandbox strategy on denial (no re‑approval thanks to
caching).
*/
use super::approvals::ApprovalReviewer;
use super::approvals::resolve_tool_apporval;
use crate::network_policy_decision::network_approval_context_from_payload;
use crate::tools::flat_tool_name;
use crate::tools::network_approval::ActiveNetworkApproval;
use crate::tools::network_approval::DeferredNetworkApproval;
use crate::tools::network_approval::NetworkApprovalMode;
use crate::tools::network_approval::begin_network_approval;
use crate::tools::network_approval::finish_deferred_network_approval;
use crate::tools::network_approval::finish_immediate_network_approval;
use crate::tools::sandboxing::ApprovalAction;
use crate::tools::sandboxing::ApprovalCtx;
use crate::tools::sandboxing::ExecApprovalRequirement;
use crate::tools::sandboxing::SandboxAttempt;
use crate::tools::sandboxing::SandboxOverride;
use crate::tools::sandboxing::ToolCtx;
use crate::tools::sandboxing::ToolError;
use crate::tools::sandboxing::ToolRuntime;
use crate::tools::sandboxing::default_exec_approval_requirement;
use crate::tools::sandboxing::sandbox_override_for_first_attempt;
use crate::tools::sandboxing::unsandboxed_execution_allowed;
use codex_otel::ToolDecisionSource;
use codex_protocol::error::CodexErrorDetails;
use codex_protocol::error::SandboxErr;
use codex_protocol::exec_output::ExecToolCallOutput;
use codex_protocol::protocol::AskForApproval;
use codex_protocol::protocol::ReviewDecision;
use codex_sandboxing::SandboxManager;
use codex_sandboxing::SandboxType;
use codex_utils_path_uri::PathUri;
use std::time::Instant;

pub(crate) struct ToolOrchestrator {
    sandbox: SandboxManager,
}

pub(crate) struct OrchestratorRunResult<Out> {
    pub output: Out,
    pub deferred_network_approval: Option<DeferredNetworkApproval>,
}

impl ToolOrchestrator {
    pub fn new() -> Self {
        Self {
            sandbox: SandboxManager::new(),
        }
    }

    async fn run_attempt<Rq, Out, T>(
        tool: &mut T,
        req: &Rq,
        tool_ctx: &ToolCtx,
        attempt: &SandboxAttempt<'_>,
        managed_network_active: bool,
    ) -> (Result<Out, ToolError>, Option<DeferredNetworkApproval>)
    where
        T: ToolRuntime<Rq, Out>,
    {
        let network_approval = match begin_network_approval(
            &tool_ctx.session,
            &tool_ctx.turn.sub_id,
            managed_network_active,
            tool.network_approval_spec(req, tool_ctx),
        )
        .await
        {
            Ok(network_approval) => network_approval,
            Err(err) => return (Err(err), None),
        };

        let attempt_tool_ctx = ToolCtx {
            session: tool_ctx.session.clone(),
            turn: tool_ctx.turn.clone(),
            call_id: tool_ctx.call_id.clone(),
            tool_name: tool_ctx.tool_name.clone(),
        };
        let attempt_with_network_approval = SandboxAttempt {
            sandbox: attempt.sandbox,
            sandbox_requested: attempt.sandbox_requested,
            permissions: attempt.permissions,
            exec_server_permissions: attempt.exec_server_permissions,
            enforce_managed_network: attempt.enforce_managed_network,
            manager: attempt.manager,
            sandbox_cwd: attempt.sandbox_cwd,
            workspace_roots: attempt.workspace_roots,
            codex_linux_sandbox_exe: attempt.codex_linux_sandbox_exe,
            use_legacy_landlock: attempt.use_legacy_landlock,
            windows_sandbox_level: attempt.windows_sandbox_level,
            windows_sandbox_private_desktop: attempt.windows_sandbox_private_desktop,
            network_denial_cancellation_token: network_approval
                .as_ref()
                .map(ActiveNetworkApproval::cancellation_token),
            network_proxy: network_approval
                .as_ref()
                .map(ActiveNetworkApproval::execution_proxy),
        };
        let run_result = tool
            .run(req, &attempt_with_network_approval, &attempt_tool_ctx)
            .await;

        let Some(network_approval) = network_approval else {
            return (run_result, None);
        };

        match network_approval.mode() {
            NetworkApprovalMode::Immediate => {
                let finalize_result =
                    finish_immediate_network_approval(&tool_ctx.session, network_approval).await;
                if let Err(err) = finalize_result {
                    return (Err(err), None);
                }
                (run_result, None)
            }
            NetworkApprovalMode::Deferred => {
                let deferred = network_approval.into_deferred();
                if run_result.is_err() {
                    let finalize_result =
                        finish_deferred_network_approval(&tool_ctx.session, deferred).await;
                    if let Err(err) = finalize_result {
                        return (Err(err), None);
                    }
                    return (run_result, None);
                }
                (run_result, deferred)
            }
        }
    }

    pub async fn run<Rq, Out, T>(
        &mut self,
        tool: &mut T,
        req: &Rq,
        tool_ctx: &ToolCtx,
        turn_ctx: &crate::session::turn_context::TurnContext,
        approval_policy: AskForApproval,
    ) -> Result<OrchestratorRunResult<Out>, ToolError>
    where
        T: ToolRuntime<Rq, Out>,
    {
        let otel = turn_ctx.session_telemetry.clone();
        let otel_tn = flat_tool_name(&tool_ctx.tool_name).into_owned();
        let otel_ci = &tool_ctx.call_id;
        let strict_auto_review = tool_ctx.session.strict_auto_review_enabled_for_turn().await;
        // 1) Approval
        let mut already_approved = false;

        let workspace_roots = tool.workspace_roots(req);
        let permission_profile = turn_ctx.config.permissions.permission_profile();
        let materialized_workspace_roots = workspace_roots
            .iter()
            .filter_map(|workspace_root| workspace_root.to_abs_path().ok())
            .collect::<Vec<_>>();
        let permissions = permission_profile
            .clone()
            .materialize_project_roots_with_workspace_roots(&materialized_workspace_roots);
        let file_system_sandbox_policy = permissions.file_system_sandbox_policy();
        let requirement = tool.exec_approval_requirement(req).unwrap_or_else(|| {
            default_exec_approval_requirement(approval_policy, &file_system_sandbox_policy)
        });
        // PF-23-S02: under Aggressive, the grant digest of this exact command.
        // PF-25-S01: while its approval is open, the human may grant it once
        // in the TUI; the offer ends with the approval.
        let grant_operation = aggressive_grant_operation(tool, req, tool_ctx);
        // Registered just before the user (not an automatic reviewer) is
        // asked. A cached approval or a permission hook may still answer
        // without them; then no review is shown and nothing is confirmed.
        let mut grant_offer = None;
        // PF-30-S03: a protected action after untrusted content needs fresh,
        // exact human approval, whatever the requirement above would allow.
        let post_taint = post_taint_action(tool, req, tool_ctx, &requirement).await;
        if let Some(action) = &post_taint {
            if let Some(refusal) = action.refused_up_front() {
                post_taint_outcome(
                    action,
                    tool_ctx,
                    "refused_kill_switch",
                    /*waited*/ None,
                );
                return Err(ToolError::Rejected(refusal));
            }
            if approval_policy == AskForApproval::Never {
                post_taint_outcome(
                    action,
                    tool_ctx,
                    "refused_approvals_off",
                    /*waited*/ None,
                );
                return Err(ToolError::Rejected(action.approvals_off_rejection()));
            }
            let approval_ctx = ApprovalCtx {
                session: &tool_ctx.session,
                turn: &tool_ctx.turn,
                call_id: &tool_ctx.call_id,
                retry_reason: Some(action.approval_reason()),
                network_approval_context: None,
                fresh_human_authority: true,
            };
            let asked = Instant::now();
            grant_offer = aggressive_grant_offer(grant_operation.as_ref(), tool_ctx);
            let decision = resolve_tool_apporval(
                tool,
                req,
                tool_ctx.call_id.as_str(),
                approval_ctx,
                tool_ctx,
                ApprovalReviewer::User,
                &otel,
            )
            .await;
            post_taint_outcome(
                action,
                tool_ctx,
                if decision.is_ok() {
                    "approved"
                } else {
                    "declined"
                },
                Some(asked.elapsed()),
            );
            decision?;
            post_taint_recheck(action, tool_ctx)?;
            already_approved = true;
        }
        // Issue #239: under Moderate before untrusted content, a command that
        // asks to run outside the sandbox needs the human (no cache, hook or
        // automatic reviewer), also where it would run unasked; that answer
        // lifts the protected-path rules. With approvals off it stays refused.
        let ask_human_to_escalate = post_taint.is_none()
            && !matches!(requirement, ExecApprovalRequirement::Forbidden { .. })
            && match approval_policy {
                AskForApproval::Never => false,
                AskForApproval::Granular(granular) => granular.allows_sandbox_approval(),
                AskForApproval::UnlessTrusted | AskForApproval::OnRequest => true,
            }
            && moderate_escalation_before_taint(tool, req, tool_ctx);
        let mut escalation_approved_by_human = false;
        match &requirement {
            _ if post_taint.is_some() => {}
            _ if ask_human_to_escalate => {
                let note =
                    "Approving also lifts the Moderate credential protection for this command.";
                let mut approval_ctx = ApprovalCtx {
                    session: &tool_ctx.session,
                    turn: &tool_ctx.turn,
                    call_id: &tool_ctx.call_id,
                    retry_reason: None,
                    network_approval_context: None,
                    fresh_human_authority: true,
                };
                // Keep why the command runs (the requirement's reason, else the
                // model's justification) ahead of the note.
                let why = match &requirement {
                    ExecApprovalRequirement::NeedsApproval {
                        reason: Some(reason),
                        ..
                    } => Some(reason.clone()),
                    _ => match tool.approval_action(req, &approval_ctx) {
                        Ok(
                            ApprovalAction::Shell { justification, .. }
                            | ApprovalAction::ExecCommand { justification, .. },
                        ) => justification,
                        _ => None,
                    },
                };
                approval_ctx.retry_reason = Some(match why {
                    Some(why) => format!("{why} {note}"),
                    None => note.to_string(),
                });
                resolve_tool_apporval(
                    tool,
                    req,
                    tool_ctx.call_id.as_str(),
                    approval_ctx,
                    tool_ctx,
                    ApprovalReviewer::User,
                    &otel,
                )
                .await?;
                already_approved = true;
                escalation_approved_by_human = true;
            }
            ExecApprovalRequirement::Skip { .. } => {
                if strict_auto_review {
                    let approval_ctx = ApprovalCtx {
                        session: &tool_ctx.session,
                        turn: &tool_ctx.turn,
                        call_id: &tool_ctx.call_id,
                        retry_reason: None,
                        network_approval_context: None,
                        fresh_human_authority: false,
                    };
                    resolve_tool_apporval(
                        tool,
                        req,
                        tool_ctx.call_id.as_str(),
                        approval_ctx,
                        tool_ctx,
                        ApprovalReviewer::Guardian,
                        &otel,
                    )
                    .await?;
                    already_approved = true;
                } else {
                    otel.tool_decision(
                        &otel_tn,
                        otel_ci,
                        &ReviewDecision::Approved,
                        ToolDecisionSource::Config,
                    );
                }
            }
            ExecApprovalRequirement::Forbidden { reason } => {
                return Err(ToolError::Rejected(reason.clone()));
            }
            ExecApprovalRequirement::NeedsApproval { reason, .. } => {
                let approval_ctx = ApprovalCtx {
                    session: &tool_ctx.session,
                    turn: &tool_ctx.turn,
                    call_id: &tool_ctx.call_id,
                    retry_reason: reason.clone(),
                    network_approval_context: None,
                    fresh_human_authority: false,
                };
                let reviewer = if strict_auto_review {
                    ApprovalReviewer::Guardian
                } else {
                    ApprovalReviewer::for_turn(turn_ctx)
                };
                if reviewer == ApprovalReviewer::User {
                    grant_offer = aggressive_grant_offer(grant_operation.as_ref(), tool_ctx);
                }
                resolve_tool_apporval(
                    tool,
                    req,
                    tool_ctx.call_id.as_str(),
                    approval_ctx,
                    tool_ctx,
                    reviewer,
                    &otel,
                )
                .await?;
                already_approved = true;
            }
        }

        // The approval came back approved (a refusal returned above): the
        // grant the person confirmed for it, if any (PF-25-S01).
        let confirmed_grant = grant_offer
            .as_ref()
            .and_then(crate::security::grant_offer::OfferGuard::take_confirmed);
        drop(grant_offer);

        // PF-23-S01 slice 3: after untrusted content the sandbox itself denies
        // credential and Corbanu home reads, whatever the command text says.
        // What stays readable comes from the turn, never from the command's
        // own (model-chosen) working folder.
        #[allow(deprecated)]
        let turn_cwd = turn_ctx.cwd.clone();
        // The taint generation a human approval of this run was given under.
        let human_approved_at = match &post_taint {
            Some(action) => Some(action.state.taint_generation),
            None => escalation_approved_by_human.then_some(0),
        };
        let denied = post_taint_read_denials(
            tool_ctx,
            human_approved_at,
            grant_operation
                .as_ref()
                .map(|(operation, _)| operation.as_str()),
            confirmed_grant,
            turn_cwd,
            &materialized_workspace_roots,
            permission_profile,
            &permissions,
        );
        let (denied_exec_server_permissions, permissions) = match denied {
            Some((exec_server, materialized)) => (Some(exec_server), materialized),
            None => (None, permissions),
        };
        let permission_profile = denied_exec_server_permissions
            .as_ref()
            .unwrap_or(permission_profile);
        let file_system_sandbox_policy = permissions.file_system_sandbox_policy();

        // 2) First attempt under the selected sandbox.
        let sandbox_override = sandbox_override_for_first_attempt(
            tool.sandbox_permissions(req),
            &requirement,
            &file_system_sandbox_policy,
        );
        let managed_network_active = turn_ctx.network.is_some();
        let sandbox_preference = tool.sandbox_preference();
        let sandbox_requested = match sandbox_override {
            SandboxOverride::BypassSandboxFirstAttempt => false,
            SandboxOverride::NoOverride => self.sandbox.should_sandbox(
                &permissions,
                sandbox_preference,
                managed_network_active,
            ),
        };
        let initial_sandbox = if sandbox_requested {
            self.sandbox.select_initial(
                &permissions,
                sandbox_preference,
                turn_ctx.windows_sandbox_level,
                managed_network_active,
            )
        } else {
            SandboxType::None
        };

        // Platform-specific flag gating is handled by SandboxManager::select_initial.
        let use_legacy_landlock = turn_ctx.config.features.use_legacy_landlock();
        #[allow(deprecated)]
        let sandbox_policy_cwd = tool
            .sandbox_cwd(req)
            .cloned()
            .unwrap_or_else(|| PathUri::from_abs_path(&turn_ctx.cwd));
        let initial_attempt = SandboxAttempt {
            sandbox: initial_sandbox,
            sandbox_requested,
            permissions: &permissions,
            exec_server_permissions: permission_profile,
            enforce_managed_network: managed_network_active,
            manager: &self.sandbox,
            sandbox_cwd: &sandbox_policy_cwd,
            workspace_roots,
            codex_linux_sandbox_exe: turn_ctx.config.codex_linux_sandbox_exe.as_ref(),
            use_legacy_landlock,
            windows_sandbox_level: turn_ctx.windows_sandbox_level,
            windows_sandbox_private_desktop: turn_ctx
                .config
                .permissions
                .windows_sandbox_private_desktop,
            network_denial_cancellation_token: None,
            network_proxy: None,
        };

        let initial_attempt_start = Instant::now();
        let (first_result, first_deferred_network_approval) = Self::run_attempt(
            tool,
            req,
            tool_ctx,
            &initial_attempt,
            managed_network_active,
        )
        .await;
        let initial_duration = initial_attempt_start.elapsed();
        match first_result {
            Ok(out) => {
                // We have a successful initial result
                Ok(OrchestratorRunResult {
                    output: out,
                    deferred_network_approval: first_deferred_network_approval,
                })
            }
            Err(ToolError::Codex(err)) => {
                let CodexErrorDetails::Sandbox(SandboxErr::Denied {
                    output,
                    network_policy_decision,
                }) = err.details()
                else {
                    let err = ToolError::Codex(err);
                    if let Some(outcome) = sandbox_outcome_from_tool_error(&err) {
                        otel.sandbox_outcome(
                            &otel_tn,
                            otel_ci,
                            outcome,
                            initial_duration,
                            /*escalated_duration*/ None,
                        );
                    }
                    return Err(err);
                };
                let network_approval_context = if managed_network_active {
                    network_policy_decision
                        .as_ref()
                        .and_then(network_approval_context_from_payload)
                } else {
                    None
                };
                if network_policy_decision.is_some() && network_approval_context.is_none() {
                    otel.sandbox_outcome(
                        &otel_tn,
                        otel_ci,
                        "denied",
                        initial_duration,
                        /*escalated_duration*/ None,
                    );
                    return Err(ToolError::Codex(err));
                }
                if !tool.escalate_on_failure() {
                    otel.sandbox_outcome(
                        &otel_tn,
                        otel_ci,
                        "denied",
                        initial_duration,
                        /*escalated_duration*/ None,
                    );
                    return Err(ToolError::Codex(err));
                }
                let sandbox_startup_failure =
                    is_sandbox_startup_failure(initial_sandbox, output.as_ref());
                let unsandboxed_allowed =
                    unsandboxed_execution_allowed(&file_system_sandbox_policy);
                // Under `Never` or `OnRequest`, do not retry without sandbox;
                // surface a concise sandbox denial that preserves the original
                // output. Exception: if the platform sandbox itself failed to
                // start after an explicit OnRequest approval, the approval is
                // enough to retry unsandboxed when doing so would not drop
                // denied-read enforcement.
                if !tool.wants_no_sandbox_approval(approval_policy) {
                    let allow_on_request_network_prompt =
                        matches!(approval_policy, AskForApproval::OnRequest)
                            && network_approval_context.is_some()
                            && matches!(
                                default_exec_approval_requirement(
                                    approval_policy,
                                    &file_system_sandbox_policy
                                ),
                                ExecApprovalRequirement::NeedsApproval { .. }
                            );
                    let allow_on_request_startup_retry = allow_on_request_sandbox_startup_retry(
                        approval_policy,
                        already_approved,
                        sandbox_startup_failure,
                    );
                    if !allow_on_request_network_prompt && !allow_on_request_startup_retry {
                        otel.sandbox_outcome(
                            &otel_tn,
                            otel_ci,
                            "denied",
                            initial_duration,
                            /*escalated_duration*/ None,
                        );
                        return Err(ToolError::Codex(err));
                    }
                }
                if !unsandboxed_allowed && network_approval_context.is_none() {
                    otel.sandbox_outcome(
                        &otel_tn,
                        otel_ci,
                        "denied",
                        initial_duration,
                        /*escalated_duration*/ None,
                    );
                    return Err(ToolError::Codex(err));
                }
                let retry_reason =
                    if let Some(network_approval_context) = network_approval_context.as_ref() {
                        format!(
                            "Network access to \"{}\" is blocked by policy.",
                            network_approval_context.host
                        )
                    } else if sandbox_startup_failure {
                        "sandbox failed to start; retry without sandbox?".to_string()
                    } else {
                        build_denial_reason_from_output(output.as_ref())
                    };

                // Strict auto-review approval covers the sandboxed attempt only;
                // retrying without the sandbox requires a fresh guardian review.
                // A post-taint approval covered the sandboxed attempt only.
                let bypass_retry_approval = !strict_auto_review
                    && post_taint.is_none()
                    && (tool.should_bypass_approval(approval_policy, already_approved)
                        || allow_on_request_sandbox_startup_retry(
                            approval_policy,
                            already_approved,
                            sandbox_startup_failure,
                        ))
                    && network_approval_context.is_none();
                if !bypass_retry_approval {
                    let approval_ctx = ApprovalCtx {
                        session: &tool_ctx.session,
                        turn: &tool_ctx.turn,
                        call_id: &tool_ctx.call_id,
                        retry_reason: Some(match &post_taint {
                            Some(action) => format!("{} {retry_reason}", action.approval_reason()),
                            None => retry_reason,
                        }),
                        network_approval_context: network_approval_context.clone(),
                        fresh_human_authority: post_taint.is_some(),
                    };

                    let permission_request_run_id = format!("{}:retry", tool_ctx.call_id);
                    resolve_tool_apporval(
                        tool,
                        req,
                        &permission_request_run_id,
                        approval_ctx,
                        tool_ctx,
                        if strict_auto_review {
                            ApprovalReviewer::Guardian
                        } else {
                            ApprovalReviewer::for_turn(turn_ctx)
                        },
                        &otel,
                    )
                    .await?;
                    if let Some(action) = &post_taint {
                        post_taint_recheck(action, tool_ctx)?;
                    }
                }

                let retry_sandbox_requested = !unsandboxed_allowed
                    && self.sandbox.should_sandbox(
                        &permissions,
                        sandbox_preference,
                        managed_network_active,
                    );
                let retry_sandbox = if retry_sandbox_requested {
                    self.sandbox.select_initial(
                        &permissions,
                        sandbox_preference,
                        turn_ctx.windows_sandbox_level,
                        managed_network_active,
                    )
                } else {
                    SandboxType::None
                };
                let retry_codex_linux_sandbox_exe = if unsandboxed_allowed {
                    None
                } else {
                    turn_ctx.config.codex_linux_sandbox_exe.as_ref()
                };
                let retry_attempt = SandboxAttempt {
                    sandbox: retry_sandbox,
                    sandbox_requested: retry_sandbox_requested,
                    permissions: &permissions,
                    exec_server_permissions: permission_profile,
                    enforce_managed_network: managed_network_active,
                    manager: &self.sandbox,
                    sandbox_cwd: &sandbox_policy_cwd,
                    workspace_roots,
                    codex_linux_sandbox_exe: retry_codex_linux_sandbox_exe,
                    use_legacy_landlock,
                    windows_sandbox_level: turn_ctx.windows_sandbox_level,
                    windows_sandbox_private_desktop: turn_ctx
                        .config
                        .permissions
                        .windows_sandbox_private_desktop,
                    network_denial_cancellation_token: None,
                    network_proxy: None,
                };

                // Second attempt.
                let escalated_attempt_start = Instant::now();
                let (retry_result, retry_deferred_network_approval) =
                    Self::run_attempt(tool, req, tool_ctx, &retry_attempt, managed_network_active)
                        .await;
                let escalated_duration = escalated_attempt_start.elapsed();
                match retry_result {
                    Ok(output) => {
                        otel.sandbox_outcome(
                            &otel_tn,
                            otel_ci,
                            "escalated",
                            initial_duration,
                            Some(escalated_duration),
                        );
                        Ok(OrchestratorRunResult {
                            output,
                            deferred_network_approval: retry_deferred_network_approval,
                        })
                    }
                    Err(err) => {
                        if let Some(outcome) = sandbox_outcome_from_tool_error(&err) {
                            otel.sandbox_outcome(
                                &otel_tn,
                                otel_ci,
                                outcome,
                                initial_duration,
                                Some(escalated_duration),
                            );
                        }
                        Err(err)
                    }
                }
            }
            Err(err) => {
                if let Some(outcome) = sandbox_outcome_from_tool_error(&err) {
                    otel.sandbox_outcome(
                        &otel_tn,
                        otel_ci,
                        outcome,
                        initial_duration,
                        /*escalated_duration*/ None,
                    );
                }
                Err(err)
            }
        }
    }
}

fn sandbox_outcome_from_tool_error(err: &ToolError) -> Option<&'static str> {
    match err {
        ToolError::Codex(err) => match err.details() {
            CodexErrorDetails::Sandbox(SandboxErr::Denied { .. }) => Some("denied"),
            CodexErrorDetails::Sandbox(SandboxErr::Timeout { .. }) => Some("timed_out"),
            CodexErrorDetails::Sandbox(SandboxErr::Signal(_)) => Some("signal"),
            _ => None,
        },
        ToolError::Rejected(_) => None,
    }
}

/// PF-30-S03: how long classifying one post-taint action may take.
const POST_TAINT_CLASSIFY_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(3);

/// PF-30-S03: the protected action this request performs, when post-taint
/// checks apply and the session already holds content without standing.
async fn post_taint_action<Rq, Out, T>(
    tool: &T,
    req: &Rq,
    tool_ctx: &ToolCtx,
    requirement: &ExecApprovalRequirement,
) -> Option<crate::security::tainted_action::PostTaintAction>
where
    T: ToolRuntime<Rq, Out>,
{
    if matches!(requirement, ExecApprovalRequirement::Forbidden { .. }) {
        return None;
    }
    let state = tool_ctx
        .session
        .services
        .model_client()
        .post_taint_state()
        .filter(|state| state.taint_generation > 0)?;
    let ctx = ApprovalCtx {
        session: &tool_ctx.session,
        turn: &tool_ctx.turn,
        call_id: &tool_ctx.call_id,
        retry_reason: None,
        network_approval_context: None,
        fresh_human_authority: true,
    };
    // An action the host cannot describe cannot be shown to be ordinary.
    let started = Instant::now();
    let kind = match tool.approval_action(req, &ctx) {
        Ok(action) => {
            // Classification may look at the filesystem: keep it off the
            // async workers and bounded in time; a stall fails closed.
            let codex_home = tool_ctx.turn.config.codex_home.to_path_buf();
            let classified = tokio::time::timeout(
                POST_TAINT_CLASSIFY_TIMEOUT,
                tokio::task::spawn_blocking(move || {
                    crate::security::tainted_action::classify(&action, &codex_home)
                }),
            )
            .await;
            match classified {
                Ok(Ok(kind)) => kind?,
                Ok(Err(_)) | Err(_) => {
                    crate::security::tainted_action::ProtectedActionKind::UnseenCode
                }
            }
        }
        Err(_) => crate::security::tainted_action::ProtectedActionKind::UnseenCode,
    };
    tracing::info!(
        target: "codex_core::security::tainted_action",
        kind = ?kind,
        taint_generation = state.taint_generation,
        classify_us = started.elapsed().as_micros() as u64,
        call_id = %tool_ctx.call_id,
        "post-taint protected action needs fresh human approval"
    );
    Some(crate::security::tainted_action::PostTaintAction { kind, state })
}

/// Issue #239: under Moderate before untrusted content, whether this command
/// asks to run outside the sandbox (a human approval then lifts the
/// protected-path rules for it, as after untrusted content).
fn moderate_escalation_before_taint<Rq, Out, T>(tool: &T, req: &Rq, tool_ctx: &ToolCtx) -> bool
where
    T: ToolRuntime<Rq, Out>,
{
    tool.sandbox_permissions(req)
        .requires_escalated_permissions()
        && tool_ctx
            .session
            .services
            .model_client()
            .post_taint_state()
            .is_some_and(|state| state.taint_generation == 0 && state.moderate_bound())
}

/// The command and folder a grant offer shows (PF-25-S01).
type GrantShown = Option<(Vec<String>, String)>;

/// PF-23-S02: under Aggressive, the grant operation naming this exact
/// command or patch in its folder, and for a command what a grant offer
/// shows (PF-25-S01). `None` under any other level.
fn aggressive_grant_operation<Rq, Out, T>(
    tool: &T,
    req: &Rq,
    tool_ctx: &ToolCtx,
) -> Option<(String, GrantShown)>
where
    T: ToolRuntime<Rq, Out>,
{
    use crate::security::aggressive::command_operation;
    tool_ctx
        .session
        .services
        .model_client()
        .post_taint_state()
        .filter(|state| state.level == codex_security_policy::SecurityLevel::Aggressive)?;
    let ctx = ApprovalCtx {
        session: &tool_ctx.session,
        turn: &tool_ctx.turn,
        call_id: &tool_ctx.call_id,
        retry_reason: None,
        network_approval_context: None,
        fresh_human_authority: true,
    };
    Some(match tool.approval_action(req, &ctx).ok()? {
        ApprovalAction::Shell {
            command,
            cwd,
            sandbox_permissions,
            additional_permissions,
            ..
        }
        | ApprovalAction::ExecCommand {
            command,
            cwd,
            sandbox_permissions,
            additional_permissions,
            ..
        } => {
            let shown_cwd = cwd.to_path_buf().display().to_string();
            let cwd = cwd.to_string();
            let permissions = format!("{sandbox_permissions:?} {additional_permissions:?}");
            let mut parts = vec!["command", cwd.as_str(), permissions.as_str()];
            parts.extend(command.iter().map(String::as_str));
            let operation = command_operation(&parts);
            (operation, Some((command, shown_cwd)))
        }
        ApprovalAction::ApplyPatch { cwd, patch, .. } => (
            command_operation(&["patch", &cwd.to_string(), &patch]),
            None,
        ),
    })
}

/// PF-25-S01: offer the human a grant for this exact command while its
/// approval is open (only under a live Aggressive policy).
fn aggressive_grant_offer(
    operation: Option<&(String, GrantShown)>,
    tool_ctx: &ToolCtx,
) -> Option<crate::security::grant_offer::OfferGuard> {
    let (operation, Some((command, cwd))) = operation? else {
        return None;
    };
    let state = tool_ctx
        .session
        .services
        .model_client()
        .post_taint_state()?;
    Some(crate::security::grant_offer::offer_for_approval(
        &tool_ctx.session,
        &state,
        &tool_ctx.call_id,
        operation.clone(),
        command.clone(),
        cwd.clone(),
    ))
}

/// PF-23-S01 slice 3 / PF-23-S02 / issue #239: the exec-server and
/// materialized profiles with credential and Corbanu home reads denied and
/// persistence files made read-only under Moderate or Aggressive, from the
/// start of the session (`PostTaintState::protected_paths_apply`).
///
/// Under Moderate a fresh human approval lifts the rules for this run: after
/// untrusted content, of this exact protected command; before it, of this
/// command's request to run outside the sandbox. `human_approved_at` is the
/// taint generation that approval was given under; new taint since voids it.
/// Under Aggressive an approval never does; only a matching human grant for
/// this exact command (`security::aggressive`). Lifting only leaves these rules out: every
/// denial of the profile itself stays. An external sandbox cannot take the
/// rules and keeps its own.
#[allow(clippy::too_many_arguments)]
fn post_taint_read_denials(
    tool_ctx: &ToolCtx,
    human_approved_at: Option<u64>,
    grant_operation: Option<&str>,
    confirmed_grant: Option<crate::security::grant_offer::Confirmed>,
    cwd: codex_utils_absolute_path::AbsolutePathBuf,
    workspace_roots: &[codex_utils_absolute_path::AbsolutePathBuf],
    exec_server: &codex_protocol::models::PermissionProfile,
    materialized: &codex_protocol::models::PermissionProfile,
) -> Option<(
    codex_protocol::models::PermissionProfile,
    codex_protocol::models::PermissionProfile,
)> {
    use crate::security::aggressive;
    use crate::security::protected_surface::ReadDenials;
    let thread = tool_ctx.session.thread_id();
    let state = tool_ctx
        .session
        .services
        .model_client()
        .post_taint_state()
        .filter(crate::security::tainted_action::PostTaintState::protected_paths_apply)?;
    if state.human_approval_lifts_rules(human_approved_at) {
        tracing::info!(
            target: "codex_core::security::tainted_action",
            taint_generation = state.taint_generation,
            call_id = %tool_ctx.call_id,
            "protected-path rules lifted by the human approval of this command"
        );
        return None;
    }
    if let (Some(operation), Some(confirmed)) = (grant_operation, confirmed_grant) {
        match crate::security::grant_offer::apply(
            thread,
            &state,
            confirmed,
            operation,
            aggressive::now_unix_seconds(),
        ) {
            Ok(grant_id) => {
                tracing::info!(
                    target: "codex_core::security::tainted_action",
                    grant_id = grant_id.as_str(),
                    call_id = %tool_ctx.call_id,
                    "protected-path rules lifted by the grant the human confirmed for this command"
                );
                return None;
            }
            Err(reason) => tracing::warn!(
                target: "codex_core::security::tainted_action",
                call_id = %tool_ctx.call_id,
                reason,
                "the confirmed grant does not apply; the rules stay"
            ),
        }
    }
    if let Some(operation) = grant_operation
        && let Some(grant_id) = aggressive::admit(
            thread,
            &state,
            aggressive::Surface::UnprotectedCommand,
            operation,
            aggressive::now_unix_seconds(),
        )
    {
        tracing::info!(
            target: "codex_core::security::tainted_action",
            grant_id = grant_id.as_str(),
            call_id = %tool_ctx.call_id,
            "protected-path rules lifted by an Aggressive grant for this command"
        );
        return None;
    }
    let denials = ReadDenials::for_turn(
        tool_ctx.turn.config.codex_home.as_path(),
        &cwd,
        workspace_roots,
        materialized,
    );
    let (Some(exec_server), Some(materialized)) =
        (denials.apply(exec_server), denials.apply(materialized))
    else {
        tracing::warn!(
            target: "codex_core::security::tainted_action",
            call_id = %tool_ctx.call_id,
            "protected-path rules cannot be added to an external sandbox"
        );
        return None;
    };
    tracing::info!(
        target: "codex_core::security::tainted_action",
        taint_generation = state.taint_generation,
        denied = denials.paths().count(),
        read_only = denials.read_only_paths().count(),
        skipped = denials.skipped.len(),
        call_id = %tool_ctx.call_id,
        "protected-path rules applied"
    );
    Some((exec_server, materialized))
}

/// PF-30-S03: refuse when the taint or the effective policy changed while
/// the human was deciding; they approved under what they saw.
fn post_taint_recheck(
    action: &crate::security::tainted_action::PostTaintAction,
    tool_ctx: &ToolCtx,
) -> Result<(), ToolError> {
    let now = tool_ctx.session.services.model_client().post_taint_state();
    action.recheck(now.as_ref()).map_err(|refusal| {
        post_taint_outcome(action, tool_ctx, "refused_stale", /*waited*/ None);
        ToolError::Rejected(refusal)
    })
}

/// PF-30-S03 / PF-26: one log line per post-taint decision, with how long
/// the human took, so research workflows can count prompts and their cost.
fn post_taint_outcome(
    action: &crate::security::tainted_action::PostTaintAction,
    tool_ctx: &ToolCtx,
    outcome: &'static str,
    waited: Option<std::time::Duration>,
) {
    tracing::info!(
        target: "codex_core::security::tainted_action",
        kind = ?action.kind,
        taint_generation = action.state.taint_generation,
        outcome,
        waited_ms = waited.map(|waited| waited.as_millis() as u64),
        call_id = %tool_ctx.call_id,
        "post-taint decision"
    );
}

fn build_denial_reason_from_output(_output: &ExecToolCallOutput) -> String {
    // Keep approval reason terse and stable for UX/tests, but accept the
    // output so we can evolve heuristics later without touching call sites.
    "command failed; retry without sandbox?".to_string()
}

fn allow_on_request_sandbox_startup_retry(
    approval_policy: AskForApproval,
    already_approved: bool,
    sandbox_startup_failure: bool,
) -> bool {
    matches!(approval_policy, AskForApproval::OnRequest)
        && already_approved
        && sandbox_startup_failure
}

fn is_sandbox_startup_failure(sandbox: SandboxType, output: &ExecToolCallOutput) -> bool {
    if sandbox != SandboxType::LinuxSeccomp {
        return false;
    }
    if !output.stdout.text.trim().is_empty() {
        return false;
    }

    let text =
        format!("{}\n{}", output.stderr.text, output.aggregated_output.text).to_ascii_lowercase();

    const STARTUP_FAILURE_MARKERS: &[&str] = &[
        "bubblewrap is unavailable",
        "failed to exec bundled bubblewrap",
        "failed to exec system bubblewrap",
        "loopback: failed rtm_newaddr",
        "loopback: failed rtm_newlink",
        "no permissions to create a new namespace",
        "setting up uid map: permission denied",
        "creating new namespace failed",
        "cannot create user namespace",
    ];

    STARTUP_FAILURE_MARKERS
        .iter()
        .any(|marker| text.contains(marker))
}

#[cfg(test)]
mod tests {
    use super::*;
    use codex_protocol::exec_output::StreamOutput;

    fn denied_output(text: &str) -> ExecToolCallOutput {
        output_with_streams("", text)
    }

    fn output_with_streams(stdout: &str, stderr: &str) -> ExecToolCallOutput {
        ExecToolCallOutput {
            exit_code: 1,
            stdout: StreamOutput::new(stdout.to_string()),
            stderr: StreamOutput::new(stderr.to_string()),
            aggregated_output: StreamOutput::new(format!("{stdout}\n{stderr}")),
            ..Default::default()
        }
    }

    #[test]
    fn linux_sandbox_startup_failure_is_classified_narrowly() {
        assert!(is_sandbox_startup_failure(
            SandboxType::LinuxSeccomp,
            &denied_output("bwrap: loopback: Failed RTM_NEWADDR: Operation not permitted"),
        ));
        assert!(is_sandbox_startup_failure(
            SandboxType::LinuxSeccomp,
            &denied_output("bubblewrap is unavailable: no system bwrap was found"),
        ));
        assert!(!is_sandbox_startup_failure(
            SandboxType::LinuxSeccomp,
            &denied_output("sandbox fails to start"),
        ));
        assert!(!is_sandbox_startup_failure(
            SandboxType::LinuxSeccomp,
            &output_with_streams(
                "command ran before failing",
                "bwrap: loopback: Failed RTM_NEWADDR: Operation not permitted",
            ),
        ));
        assert!(!is_sandbox_startup_failure(
            SandboxType::LinuxSeccomp,
            &denied_output("bash: ./script.sh: Permission denied"),
        ));
        assert!(!is_sandbox_startup_failure(
            SandboxType::None,
            &denied_output("bwrap: loopback: Failed RTM_NEWADDR: Operation not permitted"),
        ));
    }

    #[test]
    fn on_request_sandbox_startup_retry_requires_prior_approval() {
        assert!(allow_on_request_sandbox_startup_retry(
            AskForApproval::OnRequest,
            /*already_approved*/ true,
            /*sandbox_startup_failure*/ true,
        ));
        assert!(!allow_on_request_sandbox_startup_retry(
            AskForApproval::OnRequest,
            /*already_approved*/ false,
            /*sandbox_startup_failure*/ true,
        ));
        assert!(!allow_on_request_sandbox_startup_retry(
            AskForApproval::Never,
            /*already_approved*/ true,
            /*sandbox_startup_failure*/ true,
        ));
        assert!(!allow_on_request_sandbox_startup_retry(
            AskForApproval::OnRequest,
            /*already_approved*/ true,
            /*sandbox_startup_failure*/ false,
        ));
    }
}
