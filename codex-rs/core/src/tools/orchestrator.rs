/*
Module: orchestrator

Central place for approvals + sandbox selection + retry semantics. Drives a
simple sequence for any ToolRuntime: approval → select sandbox → attempt →
retry with an escalated sandbox strategy on denial (no re‑approval thanks to
caching).
*/
use super::approvals::ApprovalResolutionSource;
use super::approvals::ApprovalReviewer;
use super::approvals::DeclineRecord;
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
        // Issue #513: who approved this run, if anyone was asked.
        let mut approved_by = None;

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
        // #623: under Aggressive, a command that creates a link to the
        // project's `.codex` or `.agents` (or Corbanu's own files) is refused
        // before anyone is asked, so nothing is created; it says why. Shell
        // and exec_command requests carry their command here; a new runtime
        // whose `approval_action` has none is not checked.
        if !matches!(requirement, ExecApprovalRequirement::Forbidden { .. })
            && let Some((_, Some((command, cwd)))) = grant_operation.as_ref()
            && let Some(refusal) = crate::security::protected_links::refusal(
                &tool_ctx.session,
                turn_ctx,
                command,
                std::path::Path::new(cwd),
            )
            .await
        {
            return Err(ToolError::Rejected(refusal));
        }
        // #739 (option B, security sign-off): under Permissive with protected
        // launch armed, a request to run outside the sandbox is refused before
        // anyone is asked, since the launch contract would refuse the run. A
        // forbidden request keeps its own refusal; where denied reads already
        // keep the run in the sandbox, nothing is refused.
        let permissive_launch =
            crate::security::protected_additional::permissive_launch_keeps_sandbox(
                &tool_ctx.session,
            );
        if permissive_launch
            && !matches!(requirement, ExecApprovalRequirement::Forbidden { .. })
            && unsandboxed_execution_allowed(&file_system_sandbox_policy)
            && tool
                .sandbox_permissions(req)
                .requires_escalated_permissions()
        {
            // The person sees it too, not only the model (an exec_command
            // also gets a "Refused: <this reason>" cell since #873).
            warn(
                tool_ctx,
                crate::security::protected_additional::PROTECTED_LAUNCH_NO_ESCALATION,
            )
            .await;
            return Err(ToolError::Rejected(
                crate::security::protected_additional::PROTECTED_LAUNCH_NO_ESCALATION.to_string(),
            ));
        }
        // Registered just before the user (not an automatic reviewer) is
        // asked. A cached approval or a permission hook may still answer
        // without them; then no review is shown and nothing is confirmed.
        let mut grant_offer = None;
        // #747 (D3): under Moderate a run never leaves the sandbox.
        let moderate_keeps_sandbox =
            crate::security::protected_additional::moderate_keeps_sandbox(&tool_ctx.session);
        // Issue #513: under Moderate, the persistence file a person's approval
        // of this request would let it write, for the approval prompt.
        let mut persistence_note = None;
        // PF-30-S03: a protected action after untrusted content needs fresh,
        // exact human approval, whatever the requirement above would allow.
        let post_taint = post_taint_action(tool, req, tool_ctx, &requirement).await;
        // Issue #245: additional permissions never reopen a protected path.
        // Refused before anyone is asked where no approval can lift the
        // rules for this run; checked again below once the rules are known.
        // Issue #513: nor a runtime entry of the Corbanu home (`tmp` holds
        // the sandbox launchers) for writing, even where an approval lifts
        // the other rules. Other write protections wait for the second
        // check: an Aggressive grant confirmed in the approval, or under
        // Moderate a person's fresh approval of this command, may lift them;
        // under Moderate they count here already where no person will decide
        // this run (the automatic reviewer, or nobody, is asked).
        // A forbidden request keeps its own refusal; an external sandbox
        // takes no protected-path rules and ignores additional permissions.
        if let Some(state) = tool_ctx
            .session
            .services
            .model_client()
            .post_taint_state()
            .filter(crate::security::tainted_action::PostTaintState::protected_paths_apply)
            && !matches!(requirement, ExecApprovalRequirement::Forbidden { .. })
            && !matches!(
                permission_profile,
                codex_protocol::models::PermissionProfile::External { .. }
            )
            && let Some(additional) = tool.additional_permissions(req)
        {
            use crate::security::protected_additional;
            let human_route = post_taint.is_some()
                || moderate_escalation_before_taint(tool, req, tool_ctx, &requirement);
            let reads = !(state.moderate_bound() && human_route);
            let person_may_decide = human_route
                || (matches!(requirement, ExecApprovalRequirement::NeedsApproval { .. })
                    && !strict_auto_review
                    && ApprovalReviewer::for_turn(turn_ctx) == ApprovalReviewer::User);
            let writes = protected_additional::ReadOnlyScope::before_approval(
                state.level,
                person_may_decide,
            );
            #[allow(deprecated)]
            let turn_cwd = turn_ctx.cwd.clone();
            let cwds = resolution_folders(tool, req, &turn_cwd, &materialized_workspace_roots);
            let codex_home = tool_ctx.turn.config.codex_home.to_path_buf();
            let sqlite_home = tool_ctx.turn.config.sqlite.home().to_path_buf();
            let (roots, profile, additional) = (
                materialized_workspace_roots.clone(),
                permissions.clone(),
                additional.clone(),
            );
            // Issue #524: building the denials reads folders; off the async
            // thread. A failed check refuses (fail closed).
            // Where a person decides, say which persistence file approving
            // opens (the second check refuses it for any other approver).
            let note_persistence = state.level == codex_security_policy::SecurityLevel::Moderate
                && !human_route
                && person_may_decide;
            let checked = tokio::task::spawn_blocking(move || {
                let denials = crate::security::protected_surface::ReadDenials::for_turn(
                    &codex_home,
                    &sqlite_home,
                    &turn_cwd,
                    &roots,
                    &profile,
                );
                let protected = denials
                    .apply(&profile)
                    .map(|protected| protected.file_system_sandbox_policy());
                let check = protected_additional::Check {
                    denials: &denials,
                    protected: protected.as_ref(),
                    reads,
                    writes,
                };
                let cwds: Vec<&std::path::Path> = cwds.iter().map(AsRef::as_ref).collect();
                let liftable = note_persistence
                    .then(|| {
                        protected_additional::opened_read_only_path(
                            &denials,
                            protected.as_ref(),
                            &additional,
                            &cwds,
                            protected_additional::ReadOnlyScope::Kept,
                        )
                    })
                    .flatten();
                (
                    protected_additional::opened(check, &additional, &cwds),
                    liftable,
                )
            })
            .await
            .unwrap_or((
                Some(crate::security::protected_additional::Opened::Unchecked),
                None,
            ));
            let (opened, liftable) = checked;
            refuse_opened(opened, state.level, tool_ctx).await?;
            persistence_note = liftable
                .map(|path| protected_additional::persistence_note(&path.display().to_string()));
        }
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
            let protected_read_note =
                protected_read_note(grant_operation.as_ref(), tool_ctx, turn_ctx).await;
            // #841: what approving cannot unlock under Moderate.
            let kept_read_note =
                moderate_kept_read_note(tool, req, tool_ctx, turn_ctx, action).await;
            // #747: and that it stays in the sandbox under Moderate.
            let stays_sandboxed = moderate_sandbox_note(tool, req, moderate_keeps_sandbox);
            // #826: that what needs a user namespace fails on Linux.
            let user_namespaces_note =
                user_namespaces_note(tool, req, tool_ctx, turn_ctx).map(|(note, _)| note);
            let approval_ctx = ApprovalCtx {
                session: &tool_ctx.session,
                turn: &tool_ctx.turn,
                call_id: &tool_ctx.call_id,
                retry_reason: Some(
                    [
                        Some(action.approval_reason()),
                        protected_read_note.clone(),
                        kept_read_note.map(str::to_string),
                        stays_sandboxed.map(str::to_string),
                        user_namespaces_note,
                    ]
                    .into_iter()
                    .flatten()
                    .collect::<Vec<_>>()
                    .join(" "),
                ),
                network_approval_context: None,
                fresh_human_authority: true,
            };
            let asked = Instant::now();
            // #442: no grant for an action whose reads stay denied anyway.
            if !action.reads_stay_denied() && protected_read_note.is_none() {
                grant_offer = aggressive_grant_offer(grant_operation.as_ref(), tool_ctx);
            }
            let mut pending = crate::security::inspection::PendingProtectedAction::new(
                tool_ctx.session.thread_id(),
                action.kind,
            );
            let decision = resolve_tool_apporval(
                tool,
                req,
                tool_ctx.call_id.as_str(),
                approval_ctx,
                tool_ctx,
                ApprovalReviewer::User,
                &otel,
                DeclineRecord::ByCaller,
            )
            .await;
            pending.answered();
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
            approved_by = Some(decision?);
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
            && moderate_escalation_before_taint(tool, req, tool_ctx, &requirement);
        let mut escalation_approved_by_human = false;
        match &requirement {
            _ if post_taint.is_some() => {}
            _ if ask_human_to_escalate => {
                // #747: and it stays inside the sandbox.
                let note = crate::security::protected_additional::moderate_escalation_note();
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
                // #826: and that what needs a user namespace fails on Linux.
                let note = match user_namespaces_note(tool, req, tool_ctx, turn_ctx) {
                    Some((user_namespaces, _)) => format!("{note} {user_namespaces}"),
                    None => note,
                };
                approval_ctx.retry_reason = Some(match why {
                    Some(why) => format!("{why} {note}"),
                    None => note,
                });
                approved_by = Some(
                    resolve_tool_apporval(
                        tool,
                        req,
                        tool_ctx.call_id.as_str(),
                        approval_ctx,
                        tool_ctx,
                        ApprovalReviewer::User,
                        &otel,
                        DeclineRecord::Here,
                    )
                    .await?,
                );
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
                    approved_by = Some(
                        resolve_tool_apporval(
                            tool,
                            req,
                            tool_ctx.call_id.as_str(),
                            approval_ctx,
                            tool_ctx,
                            ApprovalReviewer::Guardian,
                            &otel,
                            DeclineRecord::Here,
                        )
                        .await?,
                    );
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
                let reviewer = if strict_auto_review {
                    ApprovalReviewer::Guardian
                } else {
                    ApprovalReviewer::for_turn(turn_ctx)
                };
                let protected_read_note = if reviewer == ApprovalReviewer::User {
                    protected_read_note(grant_operation.as_ref(), tool_ctx, turn_ctx).await
                } else {
                    None
                };
                // #747: a request to run outside the sandbox stays in it
                // under Moderate; the person is told before approving.
                let stays_sandboxed =
                    moderate_sandbox_note(tool, req, moderate_keeps_sandbox).map(str::to_string);
                // #826: that what needs a user namespace fails on Linux.
                let (user_namespaces_note, why) =
                    match user_namespaces_note(tool, req, tool_ctx, turn_ctx)
                        .filter(|_| reviewer == ApprovalReviewer::User)
                    {
                        // Keep why the command runs (the model's justification)
                        // ahead of the note when there is no other reason.
                        Some((note, justification)) => {
                            (Some(note), justification.filter(|_| reason.is_none()))
                        }
                        None => (None, None),
                    };
                let approval_ctx = ApprovalCtx {
                    session: &tool_ctx.session,
                    turn: &tool_ctx.turn,
                    call_id: &tool_ctx.call_id,
                    retry_reason: [
                        reason.as_ref(),
                        why.as_ref(),
                        protected_read_note.as_ref(),
                        persistence_note
                            .as_ref()
                            .filter(|_| reviewer == ApprovalReviewer::User),
                        stays_sandboxed.as_ref(),
                        user_namespaces_note.as_ref(),
                    ]
                    .into_iter()
                    .flatten()
                    .cloned()
                    .reduce(|joined, part| format!("{joined} {part}")),
                    network_approval_context: None,
                    fresh_human_authority: false,
                };
                if reviewer == ApprovalReviewer::User && protected_read_note.is_none() {
                    grant_offer = aggressive_grant_offer(grant_operation.as_ref(), tool_ctx);
                }
                approved_by = Some(
                    resolve_tool_apporval(
                        tool,
                        req,
                        tool_ctx.call_id.as_str(),
                        approval_ctx,
                        tool_ctx,
                        reviewer,
                        &otel,
                        DeclineRecord::Here,
                    )
                    .await?,
                );
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
        // #826: Core's level, where the protected-path rules apply.
        let protected_level = denied.as_ref().map(|(_, _, _, level)| *level);
        let (denied_exec_server_permissions, permissions) = match denied {
            Some((exec_server, materialized, denials, level)) => {
                // Issue #245: the rules apply to this run, so its additional
                // permissions may not reopen what they deny.
                // Issue #513: nor make a read-only path writable: the
                // runtime entries always; under Aggressive every one a grant
                // for this run did not lift; under Moderate every one unless
                // a person approved this command when asked about it now (not
                // the automatic reviewer, a hook, the approval cache or a
                // grant: Travis, 2026-10-11).
                if let Some(additional) = tool.additional_permissions(req) {
                    #[allow(deprecated)]
                    let turn_cwd = turn_ctx.cwd.clone();
                    let cwds =
                        resolution_folders(tool, req, &turn_cwd, &materialized_workspace_roots);
                    let protected = materialized.file_system_sandbox_policy();
                    let check = crate::security::protected_additional::Check {
                        denials: &denials,
                        protected: Some(&protected),
                        reads: true,
                        writes:
                            crate::security::protected_additional::ReadOnlyScope::after_approval(
                                level,
                                approved_by == Some(ApprovalResolutionSource::User),
                            ),
                    };
                    let cwds: Vec<&std::path::Path> = cwds.iter().map(AsRef::as_ref).collect();
                    let opened =
                        crate::security::protected_additional::opened(check, additional, &cwds);
                    refuse_opened(opened, level, tool_ctx).await?;
                }
                (Some(exec_server), materialized)
            }
            None => (None, permissions),
        };
        // #826 (Travis, 2026-10-15): under Moderate and Aggressive (also
        // where only the launch enforces Aggressive) agent commands may not
        // create user namespaces, in which they could open what the Linux
        // sandbox hides and defeat nested-launch detection. No approval or
        // grant lifts it. Permissive is unchanged.
        let (denied_exec_server_permissions, permissions) =
            if crate::security::user_namespaces::rule_applies(
                protected_level,
                turn_ctx.config.permissions.launch_enforces_aggressive(),
            ) {
                (
                    Some(deny_user_namespaces(
                        denied_exec_server_permissions
                            .as_ref()
                            .unwrap_or(permission_profile),
                    )),
                    deny_user_namespaces(&permissions),
                )
            } else {
                (denied_exec_server_permissions, permissions)
            };
        let permission_profile = denied_exec_server_permissions
            .as_ref()
            .unwrap_or(permission_profile);
        let file_system_sandbox_policy = permissions.file_system_sandbox_policy();

        // Issue #540: where the protected-path rules apply, a run leaves the
        // sandbox only where a person approves running it outside: a request
        // to (`require_escalated`, which needs a person's approval under
        // these levels), or the retry after a sandbox denial of a protected
        // action (asked afresh, checked below). Never through an exec-policy
        // allow rule: in a run whose rules a person lifted, only the sandbox
        // keeps the Corbanu home's runtime entries read-only. (Elsewhere the
        // rules' read denials already keep such runs sandboxed.)
        // #747 (D3): under Moderate nothing leaves the sandbox at all (read
        // again: a level confirmed while the person was asked counts).
        // #739: nor under Permissive with protected launch armed, whose launch
        // contract refuses every run outside the sandbox.
        let permissive_launch = permissive_launch
            || crate::security::protected_additional::permissive_launch_keeps_sandbox(
                &tool_ctx.session,
            );
        let always_sandboxed = permissive_launch
            || moderate_keeps_sandbox
            || crate::security::protected_additional::moderate_keeps_sandbox(&tool_ctx.session);
        let keeps_sandbox = always_sandboxed
            || (crate::security::protected_additional::protected_level(&tool_ctx.session)
                .is_some()
                && !tool
                    .sandbox_permissions(req)
                    .requires_escalated_permissions());
        let policy_allows_no_sandbox =
            !always_sandboxed && unsandboxed_execution_allowed(&file_system_sandbox_policy);

        // 2) First attempt under the selected sandbox.
        let unkept_override = sandbox_override_for_first_attempt(
            tool.sandbox_permissions(req),
            &requirement,
            &file_system_sandbox_policy,
        );
        let sandbox_override = if keeps_sandbox {
            SandboxOverride::NoOverride
        } else {
            unkept_override
        };
        // #739: a remembered approval or allow rule that would skip the
        // sandbox is not applied silently under protected launch: the run
        // stays in it, and the person is told why (once per session).
        if permissive_launch
            && matches!(unkept_override, SandboxOverride::BypassSandboxFirstAttempt)
            && !tool
                .sandbox_permissions(req)
                .requires_escalated_permissions()
            && crate::security::protected_additional::first_rule_note(tool_ctx.session.thread_id())
        {
            warn(
                tool_ctx,
                crate::security::protected_additional::PROTECTED_LAUNCH_RULE_STAYS_SANDBOXED,
            )
            .await;
        }
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

        // Issue #535: a writable root this run adds (or the session has)
        // that is a link into another root's read-only path is left out of
        // the sandbox; explain it before the run.
        if initial_sandbox != SandboxType::None
            && let Ok(policy_cwd) = sandbox_policy_cwd.to_abs_path()
        {
            let effective =
                codex_sandboxing::policy_transforms::effective_file_system_sandbox_policy(
                    &file_system_sandbox_policy,
                    tool.additional_permissions(req),
                );
            for message in crate::tools::writable_root_notices::excluded_writable_link_warnings(
                tool_ctx.session.thread_id(),
                &effective,
                policy_cwd.as_path(),
            ) {
                tool_ctx
                    .session
                    .send_event(
                        &tool_ctx.turn,
                        codex_protocol::protocol::EventMsg::Warning(
                            codex_protocol::protocol::WarningEvent { message },
                        ),
                    )
                    .await;
            }
        }
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
                let unsandboxed_allowed = retry_may_leave_sandbox(
                    policy_allows_no_sandbox,
                    keeps_sandbox,
                    post_taint.is_some(),
                    network_approval_context.is_some(),
                );
                // Under `Never` or `OnRequest`, do not retry without sandbox;
                // surface a concise sandbox denial that preserves the original
                // output. Exception: if the platform sandbox itself failed to
                // start after an explicit OnRequest approval, the approval is
                // enough to retry unsandboxed when doing so would not drop
                // denied-read enforcement, nor (#540) leave a kept sandbox
                // outside a protected action's fresh approval
                // (`retry_may_leave_sandbox`).
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
                    // #739: where only protected launch keeps this run in the
                    // sandbox, say why no retry outside it is offered.
                    if permissive_launch
                        && unsandboxed_execution_allowed(&file_system_sandbox_policy)
                    {
                        warn(
                            tool_ctx,
                            crate::security::protected_additional::PROTECTED_LAUNCH_NO_RETRY,
                        )
                        .await;
                    }
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
                    let retry_approved_by = resolve_tool_apporval(
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
                        DeclineRecord::Here,
                    )
                    .await?;
                    if let Some(action) = &post_taint {
                        post_taint_recheck(action, tool_ctx)?;
                    }
                    // Issue #540: leaving the sandbox here takes a person.
                    if keeps_sandbox
                        && unsandboxed_allowed
                        && retry_approved_by != ApprovalResolutionSource::User
                    {
                        return Err(ToolError::Rejected(
                            "running outside the sandbox needs a person's approval here"
                                .to_string(),
                        ));
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

/// Issue #540: whether the retry after a sandbox denial may run without a
/// sandbox. A run that keeps its sandbox (`keeps_sandbox`: the protected-path
/// rules apply and it did not ask to run outside) leaves it only for a
/// protected action's retry, which a person approves afresh, and never for
/// a network approval: that person allowed a host, not an unsandboxed run.
fn retry_may_leave_sandbox(
    policy_allows: bool,
    keeps_sandbox: bool,
    protected_action: bool,
    network_retry: bool,
) -> bool {
    policy_allows && (!keeps_sandbox || (protected_action && !network_retry))
}

fn sandbox_outcome_from_tool_error(err: &ToolError) -> Option<&'static str> {
    match err {
        ToolError::Codex(err) => match err.details() {
            CodexErrorDetails::Sandbox(SandboxErr::Denied { .. }) => Some("denied"),
            CodexErrorDetails::Sandbox(SandboxErr::Timeout { .. }) => Some("timed_out"),
            CodexErrorDetails::Sandbox(SandboxErr::Signal(_)) => Some("signal"),
            _ => None,
        },
        ToolError::Rejected(_) | ToolError::Declined(_) => None,
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

/// #739: tells the person `message` (a warning in the transcript).
async fn warn(tool_ctx: &ToolCtx, message: &str) {
    tool_ctx
        .session
        .send_event(
            &tool_ctx.turn,
            codex_protocol::protocol::EventMsg::Warning(codex_protocol::protocol::WarningEvent {
                message: message.to_string(),
            }),
        )
        .await;
}

/// #747 (D3): under Moderate, what a person is told before approving a
/// request to run outside the sandbox: it stays in it. `None` elsewhere. (An
/// approved patch outside the writable folders is applied inside the sandbox
/// with that one extra write, under every level.)
fn moderate_sandbox_note<Rq, Out, T>(
    tool: &T,
    req: &Rq,
    moderate_keeps_sandbox: bool,
) -> Option<&'static str>
where
    T: ToolRuntime<Rq, Out>,
{
    (moderate_keeps_sandbox
        && tool
            .sandbox_permissions(req)
            .requires_escalated_permissions())
    .then_some(crate::security::protected_additional::MODERATE_STAYS_SANDBOXED)
}

/// Issue #239: under Moderate before untrusted content, whether this command
/// asks to run outside the sandbox (a human approval then lifts the
/// protected-path rules for it, as after untrusted content).
/// #840 (D6): not where a saved "don't ask again" rule allows it: the rule
/// skips the prompt and the command runs inside the sandbox with the rules
/// kept, as any command that does not ask to leave it.
fn moderate_escalation_before_taint<Rq, Out, T>(
    tool: &T,
    req: &Rq,
    tool_ctx: &ToolCtx,
    requirement: &ExecApprovalRequirement,
) -> bool
where
    T: ToolRuntime<Rq, Out>,
{
    !matches!(
        requirement,
        ExecApprovalRequirement::Skip {
            bypass_sandbox: true,
            ..
        }
    ) && tool
        .sandbox_permissions(req)
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

/// #442: under Aggressive, a command that names a protected path (credential
/// files, the Corbanu home) is never granted: no grant is offered for it, and
/// its approval prompt carries this explanation. Looked up only where a
/// person is about to be asked.
async fn protected_read_note(
    operation: Option<&(String, GrantShown)>,
    tool_ctx: &ToolCtx,
    turn_ctx: &crate::session::turn_context::TurnContext,
) -> Option<String> {
    let (_, Some((command, cwd))) = operation? else {
        return None;
    };
    crate::security::protected_reads::protected_note_in_command(
        &tool_ctx.session,
        turn_ctx,
        command,
        std::path::Path::new(cwd),
    )
    .await
    .map(|found| found.explanation())
}

/// #826 follow-up: for a person asked to approve a command that runs a
/// program needing a user namespace (`unshare`, `bwrap`, rootless Podman,
/// …) where agent commands cannot create one, the note that it fails and
/// that no approval lifts the rule, with the model's justification for the
/// command (a prompt's reason replaces the justification).
fn user_namespaces_note<Rq, Out, T>(
    tool: &T,
    req: &Rq,
    tool_ctx: &ToolCtx,
    turn_ctx: &crate::session::turn_context::TurnContext,
) -> Option<(String, Option<String>)>
where
    T: ToolRuntime<Rq, Out>,
{
    if !cfg!(target_os = "linux") {
        return None;
    }
    let ctx = ApprovalCtx {
        session: &tool_ctx.session,
        turn: &tool_ctx.turn,
        call_id: &tool_ctx.call_id,
        retry_reason: None,
        network_approval_context: None,
        fresh_human_authority: false,
    };
    let (command, justification) = match tool.approval_action(req, &ctx).ok()? {
        ApprovalAction::Shell {
            command,
            justification,
            ..
        }
        | ApprovalAction::ExecCommand {
            command,
            justification,
            ..
        } => (command, justification),
        _ => return None,
    };
    crate::security::user_namespaces::approval_note(&tool_ctx.session, turn_ctx, &command)
        .map(|note| (note, justification))
}

/// #841: under Moderate, for a protected action after untrusted content that
/// reads files the secretless launch contract keeps unreadable at every
/// level (Corbanu's sign-in, vault, config, history): approving cannot
/// unlock them, and the person is told so before answering.
async fn moderate_kept_read_note<Rq, Out, T>(
    tool: &T,
    req: &Rq,
    tool_ctx: &ToolCtx,
    turn_ctx: &crate::session::turn_context::TurnContext,
    action: &crate::security::tainted_action::PostTaintAction,
) -> Option<&'static str>
where
    T: ToolRuntime<Rq, Out>,
{
    use crate::security::tainted_action::ProtectedActionKind;
    if !action.state.moderate_bound()
        || !matches!(
            action.kind,
            ProtectedActionKind::SecurityPolicy
                | ProtectedActionKind::Vault
                | ProtectedActionKind::Credentials
        )
    {
        return None;
    }
    let ctx = ApprovalCtx {
        session: &tool_ctx.session,
        turn: &tool_ctx.turn,
        call_id: &tool_ctx.call_id,
        retry_reason: None,
        network_approval_context: None,
        fresh_human_authority: true,
    };
    let (command, cwd) = match tool.approval_action(req, &ctx).ok()? {
        ApprovalAction::Shell { command, cwd, .. }
        | ApprovalAction::ExecCommand { command, cwd, .. } => (command, cwd),
        _ => return None,
    };
    let cwd = cwd.to_abs_path().ok()?;
    crate::security::protected_reads::moderate_kept_read_in_command(
        &tool_ctx.session,
        turn_ctx,
        &command,
        cwd.as_path(),
    )
    .await
    .then_some(crate::security::protected_reads::MODERATE_KEPT_READ_NOTE)
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
/// command's request to run outside the sandbox. The Corbanu home's runtime
/// entries stay read-only even then (#540). `human_approved_at` is the
/// taint generation that approval was given under; new taint since voids it.
/// Under Aggressive an approval never does; a matching human grant for this
/// exact command (`security::aggressive`) lifts the write protections only:
/// reads of credential files and the Corbanu home stay denied (#442).
/// Lifting only leaves these rules out: every denial of the profile itself
/// stays. An external sandbox cannot take the
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
    crate::security::protected_surface::ReadDenials,
    codex_security_policy::SecurityLevel,
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
        // Issue #540: except the Corbanu home's runtime entries, which stay
        // read-only in the command's sandbox (under Moderate every command
        // keeps one, #747).
        let mut denials = ReadDenials::for_turn(
            tool_ctx.turn.config.codex_home.as_path(),
            tool_ctx.turn.config.sqlite.home(),
            &cwd,
            workspace_roots,
            materialized,
        )
        .runtime_only();
        // #766: the approval lifts no rule a nested `corbanu exec` relies on.
        if !tool_ctx
            .turn
            .config
            .permissions
            .launch_enforces_aggressive()
        {
            denials.mark_moderate(tool_ctx.turn.config.codex_home.as_path());
        } else {
            ReadDenials::keep_moderate_marker_readable(tool_ctx.turn.config.codex_home.as_path());
        }
        tracing::info!(
            target: "codex_core::security::tainted_action",
            taint_generation = state.taint_generation,
            read_only = denials.read_only_paths().count(),
            call_id = %tool_ctx.call_id,
            "protected-path rules lifted by the human approval of this command, but for the runtime entries"
        );
        let (exec_server, materialized) =
            (denials.apply(exec_server)?, denials.apply(materialized)?);
        return Some((exec_server, materialized, denials, state.level));
    }
    // #442: a grant lifts the write protections only. Reads of credential
    // files and the Corbanu home are never granted under Aggressive.
    let mut granted = false;
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
                    "write protections lifted by the grant the human confirmed for this command; protected reads stay denied"
                );
                granted = true;
            }
            Err(reason) => tracing::warn!(
                target: "codex_core::security::tainted_action",
                call_id = %tool_ctx.call_id,
                reason,
                "the confirmed grant does not apply; the rules stay"
            ),
        }
    }
    if !granted
        && let Some(operation) = grant_operation
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
            "write protections lifted by an Aggressive grant for this command; protected reads stay denied"
        );
        granted = true;
    }
    let mut denials = ReadDenials::for_turn(
        tool_ctx.turn.config.codex_home.as_path(),
        tool_ctx.turn.config.sqlite.home(),
        &cwd,
        workspace_roots,
        materialized,
    );
    if granted {
        denials.lift_write_rules();
    }
    // #506, #758: no grant lifts these either.
    if state.level == codex_security_policy::SecurityLevel::Aggressive {
        denials.deny_home_listing();
        denials.deny_keychain_services();
    }
    // #766: `corbanu exec` started by this command runs with Moderate; not
    // where the launch enforces Aggressive, whose own nested-launch rules
    // apply (its level file may be unreadable under Core's Moderate rules).
    if state.level == codex_security_policy::SecurityLevel::Moderate
        && !tool_ctx
            .turn
            .config
            .permissions
            .launch_enforces_aggressive()
    {
        denials.mark_moderate(tool_ctx.turn.config.codex_home.as_path());
    } else {
        ReadDenials::keep_moderate_marker_readable(tool_ctx.turn.config.codex_home.as_path());
    }
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
    Some((exec_server, materialized, denials, state.level))
}

/// #826: `profile` with user namespaces denied to its commands. A profile
/// without a restricted filesystem policy (no sandbox, or an external one)
/// cannot carry the rule and is returned as it is.
fn deny_user_namespaces(
    profile: &codex_protocol::models::PermissionProfile,
) -> codex_protocol::models::PermissionProfile {
    use codex_protocol::permissions::FileSystemSandboxEntry;
    use codex_protocol::permissions::FileSystemSandboxKind;
    let (mut file_system, network) = profile.to_runtime_permissions();
    if file_system.kind != FileSystemSandboxKind::Restricted || file_system.denies_user_namespaces()
    {
        return profile.clone();
    }
    file_system
        .entries
        .push(FileSystemSandboxEntry::deny_user_namespaces());
    codex_protocol::models::PermissionProfile::from_runtime_permissions_with_enforcement(
        profile.enforcement(),
        &file_system,
        network,
    )
}

/// Issue #245: every folder the sandbox may resolve a relative or
/// project-root additional permission against: the run's own folder, the
/// turn's folder and the workspace roots.
fn resolution_folders<Rq, Out, T>(
    tool: &T,
    req: &Rq,
    turn_cwd: &codex_utils_absolute_path::AbsolutePathBuf,
    workspace_roots: &[codex_utils_absolute_path::AbsolutePathBuf],
) -> Vec<codex_utils_absolute_path::AbsolutePathBuf>
where
    T: ToolRuntime<Rq, Out>,
{
    let mut folders: Vec<_> = tool
        .sandbox_cwd(req)
        .and_then(|cwd| cwd.to_abs_path().ok())
        .into_iter()
        .collect();
    folders.push(turn_cwd.clone());
    folders.extend(workspace_roots.iter().cloned());
    folders.dedup();
    folders
}

/// Issues #245 and #513: refuses a run whose additional permissions would
/// open a protected path or reopen a read-only one, and explains why to the
/// person and the model.
async fn refuse_opened(
    opened: Option<crate::security::protected_additional::Opened>,
    level: codex_security_policy::SecurityLevel,
    tool_ctx: &ToolCtx,
) -> Result<(), ToolError> {
    let Some(opened) = opened else {
        return Ok(());
    };
    if opened == crate::security::protected_additional::Opened::Unchecked {
        tracing::warn!(
            target: "codex_core::security::tainted_action",
            call_id = %tool_ctx.call_id,
            "additional permissions could not be checked against the protected paths; refused"
        );
    } else {
        tracing::warn!(
            target: "codex_core::security::tainted_action",
            call_id = %tool_ctx.call_id,
            "additional permissions would open a protected path; refused"
        );
    }
    let refusal = opened.refusal(level);
    crate::security::protected_reads::notify(&tool_ctx.session, &tool_ctx.turn, refusal.clone())
        .await;
    Err(ToolError::Rejected(refusal))
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
    crate::security::inspection::record_protected_action(
        Some(tool_ctx.session.thread_id()),
        action.kind,
        outcome,
    );
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

    /// Issue #540: a run that keeps its sandbox leaves it on retry only for
    /// a protected action a person approves afresh, never for a network
    /// approval; other runs are unchanged.
    #[test]
    fn issue_540_retry_leaves_the_sandbox_only_where_a_person_approves_it() {
        // (policy allows, keeps sandbox, protected action, network) -> leaves
        for (policy, keeps, protected, network, leaves) in [
            (true, false, false, false, true),
            (true, false, false, true, true),
            (false, false, true, false, false),
            (true, true, false, false, false),
            (true, true, true, false, true),
            (true, true, true, true, false),
            (true, true, false, true, false),
            (false, true, true, false, false),
        ] {
            assert_eq!(
                retry_may_leave_sandbox(policy, keeps, protected, network),
                leaves,
                "{policy} {keeps} {protected} {network}"
            );
        }
    }

    /// #826: the rule joins a sandboxed profile once and changes nothing
    /// else; a profile without Corbanu's sandbox cannot carry it.
    #[test]
    fn issue_826_user_namespace_rule_joins_sandboxed_profiles_only() {
        use codex_protocol::models::PermissionProfile;
        use codex_protocol::permissions::NetworkSandboxPolicy;
        let workspace = PermissionProfile::workspace_write_with(
            &[],
            NetworkSandboxPolicy::Restricted,
            /*exclude_tmpdir_env_var*/ true,
            /*exclude_slash_tmp*/ true,
        );
        let denied = deny_user_namespaces(&workspace);
        let (policy, network) = denied.to_runtime_permissions();
        let (before, before_network) = workspace.to_runtime_permissions();
        assert!(policy.denies_user_namespaces());
        assert!(!before.denies_user_namespaces());
        assert_eq!(network, before_network);
        assert_eq!(denied.enforcement(), workspace.enforcement());
        assert_eq!(policy.entries.len(), before.entries.len() + 1);
        assert_eq!(deny_user_namespaces(&denied), denied);
        for profile in [
            PermissionProfile::Disabled,
            PermissionProfile::External {
                network: NetworkSandboxPolicy::Restricted,
            },
        ] {
            assert_eq!(deny_user_namespaces(&profile), profile);
        }
    }

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
        // #563: the helper's refusal when no bubblewrap is found.
        #[cfg(target_os = "linux")]
        assert!(is_sandbox_startup_failure(
            SandboxType::LinuxSeccomp,
            &denied_output(codex_sandboxing::BWRAP_UNAVAILABLE_MESSAGE),
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
