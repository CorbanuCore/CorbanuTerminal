use super::*;
use crate::session::tests::make_session_and_context_with_rx;
use crate::session::tests::test_environment_id;
use codex_models_manager::model_info::model_info_from_slug;
use codex_protocol::approvals::NetworkPolicyAmendment;
use pretty_assertions::assert_eq;

#[test]
fn approval_resolution_rejects_denied_network_policy_amendment() {
    let resolution = ApprovalResolution {
        decision: ReviewDecision::NetworkPolicyAmendment {
            network_policy_amendment: NetworkPolicyAmendment {
                host: "denied.example.com".to_string(),
                action: NetworkPolicyRuleAction::Deny,
            },
        },
        source: ApprovalResolutionSource::User,
        cancellation_reason: None,
    };

    assert!(matches!(
        resolution.into_tool_result(&model_info_from_slug("acting-model")),
        Err(ToolError::Declined(rejection)) if rejection == "rejected by user"
    ));
}

#[test]
fn approval_resolution_rejects_mcp_policy_amendment() {
    let resolution = ApprovalResolution {
        decision: ReviewDecision::ApprovedMcpPolicyAmendment,
        source: ApprovalResolutionSource::User,
        cancellation_reason: None,
    };

    assert!(matches!(
        resolution.into_tool_result(&model_info_from_slug("acting-model")),
        Err(ToolError::Rejected(rejection)) if rejection == "Error while requesting approval"
    ));
}

#[test]
fn approval_resolution_aborts_turn_when_approval_is_aborted() {
    let resolution = ApprovalResolution {
        decision: ReviewDecision::Abort,
        source: ApprovalResolutionSource::User,
        cancellation_reason: None,
    };

    assert!(matches!(
        resolution.into_tool_result(&model_info_from_slug("acting-model")),
        Err(ToolError::Codex(error))
            if matches!(
                error.details(),
                codex_protocol::error::CodexErrorDetails::TurnAborted
            )
    ));
}

#[test]
fn approval_resolution_uses_acting_model_timeout_instructions() {
    let mut model = model_info_from_slug("acting-model");
    for timeout_instructions in ["Catalog timeout instructions.", ""] {
        model.model_messages = Some(
            serde_json::from_value(serde_json::json!({
                "auto_review": {
                    "timeout_instructions": timeout_instructions,
                },
            }))
            .expect("model messages should deserialize"),
        );
        let resolution = ApprovalResolution {
            decision: ReviewDecision::TimedOut,
            source: ApprovalResolutionSource::Guardian,
            cancellation_reason: None,
        };

        assert!(matches!(
            resolution.into_tool_result(&model),
            Err(ToolError::Declined(rejection)) if rejection == timeout_instructions
        ));
    }
}

#[cfg(unix)]
#[test_case::test_case(ApprovalsReviewer::User, codex_extension_api::ApprovalDecision::AskUser; "manual prompt")]
#[test_case::test_case(ApprovalsReviewer::AutoReview, codex_extension_api::ApprovalDecision::Allow; "cached allow")]
#[tokio::test]
async fn non_utf8_cwd_preserves_approval_routing(
    reviewer: ApprovalsReviewer,
    decision: codex_extension_api::ApprovalDecision,
) -> anyhow::Result<()> {
    use anyhow::Context;
    use codex_extension_api::ApprovalDecision;
    use std::os::unix::ffi::OsStringExt;

    struct Contributor {
        cwd: codex_utils_path_uri::LegacyAppPathString,
        decision: ApprovalDecision,
    }

    impl codex_extension_api::ApprovalReviewContributor for Contributor {
        fn decide<'a>(
            &'a self,
            input: &'a codex_extension_api::ApprovalDecisionInput<'_>,
        ) -> codex_extension_api::ExtensionFuture<'a, Option<ApprovalDecision>> {
            Box::pin(async move {
                assert_eq!(input.action["cwd"], serde_json::json!(self.cwd));
                Some(self.decision.clone())
            })
        }
    }

    let cwd = PathUri::from_abs_path(&AbsolutePathBuf::try_from(PathBuf::from(
        std::ffi::OsString::from_vec(b"/tmp/non-utf8-\xe9".to_vec()),
    ))?);
    let (mut session, turn, events) = make_session_and_context_with_rx().await;
    let mut extensions = codex_extension_api::ExtensionRegistryBuilder::new();
    extensions.approval_review_contributor(Arc::new(Contributor {
        cwd: codex_utils_path_uri::LegacyAppPathString::from_path_uri(&cwd, PathConvention::Posix)?,
        decision,
    }));
    Arc::get_mut(&mut session)
        .context("session is uniquely owned")?
        .services
        .extensions = Arc::new(extensions.build());
    *session.active_turn.lock().await = Some(crate::state::ActiveTurn::default());
    let mut review_context = GuardianReviewContext::from(&turn);
    review_context.approval_policy = AskForApproval::OnRequest;
    review_context.approvals_reviewer = reviewer;
    let context = ApprovalContext {
        exec_requirement: None,
        review_context,
        cancellation_token: None,
        call_id: "non-utf8-cwd".to_string(),
        tool_name: ToolName::plain("exec_command"),
        strict_auto_review: false,
        approval_reason: None,
        retry_reason: None,
        network_approval_context: None,
    };
    let action = ApprovalAction::ExecCommand {
        id: context.call_id.clone(),
        environment_id: codex_exec_server::LOCAL_ENVIRONMENT_ID.to_string(),
        command: vec!["npm".to_string(), "install".to_string()],
        hook_command: "npm install".to_string(),
        cwd: cwd.clone(),
        sandbox_permissions: if reviewer == ApprovalsReviewer::User {
            SandboxPermissions::RequireEscalated
        } else {
            SandboxPermissions::UseDefault
        },
        additional_permissions: None,
        justification: None,
        tty: false,
        proposed_execpolicy_amendment: None,
    };
    let prompt = session.request_user_approval(&action, &context);
    let approval =
        session.request_reviewer_approval(&action, &context, prompt, /*human_only*/ false);
    tokio::pin!(approval);
    let expected = if reviewer == ApprovalsReviewer::User {
        tokio::select! {
            resolution = &mut approval => panic!("expected a user prompt, got {resolution:?}"),
            event = events.recv() => {
                let codex_protocol::protocol::EventMsg::ExecApprovalRequest(request) =
                    event.context("receive user prompt")?.msg
                else {
                    panic!("expected a command approval prompt");
                };
                assert_eq!(request.cwd, codex_utils_path_uri::LegacyAppPathString::from(cwd));
                assert_eq!(request.command, vec!["npm", "install"]);
                session.notify_approval(&request.call_id, ReviewDecision::Approved).await;
            }
        }
        ApprovalResolution {
            decision: ReviewDecision::Approved,
            source: ApprovalResolutionSource::User,
            cancellation_reason: None,
        }
    } else {
        ApprovalResolution {
            decision: ReviewDecision::Approved,
            source: ApprovalResolutionSource::Guardian,
            cancellation_reason: None,
        }
    };
    assert_eq!(approval.await, expected);
    assert!(events.try_recv().is_err());
    Ok(())
}

#[tokio::test]
async fn explicit_mcp_reviewer_override_takes_precedence_over_action_context() {
    let (session, turn, events) = make_session_and_context_with_rx().await;
    let action = ApprovalAction::McpToolCall {
        id: "mcp-override".to_string(),
        environment_id: test_environment_id(&turn).to_owned(),
        server: "example".to_string(),
        tool_name: "dangerous".to_string(),
        arguments: None,
        connector_id: None,
        connector_name: None,
        connector_description: None,
        connected_account_email: None,
        tool_title: None,
        tool_description: None,
        annotations: None,
        hook_tool_name: HookToolName::new("mcp__example__dangerous"),
        approval_policy: AskForApproval::OnRequest,
        reviewer: ApprovalsReviewer::User,
        approval_mode: AppToolApproval::Prompt,
        allow_session_remember: false,
        allow_persistent_approval: false,
    };
    let mut review_context = GuardianReviewContext::from(&turn);
    review_context.approval_policy = AskForApproval::OnRequest;
    review_context.approvals_reviewer = ApprovalsReviewer::AutoReview;
    let context = ApprovalContext {
        exec_requirement: None,
        review_context,
        cancellation_token: None,
        call_id: "mcp-override".to_string(),
        tool_name: ToolName::plain("dangerous"),
        strict_auto_review: false,
        approval_reason: None,
        retry_reason: None,
        network_approval_context: None,
    };

    let prompt = session.request_user_approval(&action, &context);
    tokio::select! {
        resolution = session.request_reviewer_approval(
            &action,
            &context,
            prompt,
            /*human_only*/ false,
        ) => {
            panic!("expected a user approval request, got {resolution:?}");
        }
        event = events.recv() => {
            let codex_protocol::protocol::EventMsg::ElicitationRequest(request) =
                event.expect("receive user approval request").msg
            else {
                panic!("expected an MCP user approval request");
            };
            assert_eq!(request.server_name, "example");
            assert_eq!(
                request.id,
                codex_protocol::mcp::RequestId::String(
                    "mcp_tool_call_approval_mcp-override".to_string()
                )
            );
        }
    }
}

#[test]
fn network_hook_cwd_overrides_only_local_commands() {
    let local_cwd = AbsolutePathBuf::try_from(std::env::temp_dir()).unwrap();
    for (environment_id, uri) in [
        (codex_exec_server::LOCAL_ENVIRONMENT_ID, None),
        (
            codex_exec_server::LOCAL_ENVIRONMENT_ID,
            Some("file:///local/repo"),
        ),
        ("remote", None),
        ("remote", Some("file:///remote/repo")),
        ("remote", Some("file:///C:/remote/repo")),
    ] {
        let expected_cwd = uri.map_or_else(
            || PathUri::from_abs_path(&local_cwd),
            |uri| PathUri::parse(uri).unwrap(),
        );
        let action = ApprovalAction::NetworkAccess {
            id: "network-1".into(),
            turn_id: "turn-1".into(),
            environment_id: environment_id.into(),
            target: "https://example.com:443".into(),
            host: "example.com".into(),
            protocol: NetworkApprovalProtocol::Https,
            port: 443,
            trigger: uri.map(|_| GuardianNetworkAccessTrigger {
                call_id: "exec-1".into(),
                tool_name: "exec_command".into(),
                command: vec!["curl".into(), "https://example.com".into()],
                cwd: expected_cwd.clone(),
                sandbox_permissions: SandboxPermissions::UseDefault,
                additional_permissions: None,
                justification: None,
                tty: None,
            }),
            hook_command: "curl https://example.com".into(),
            hook_run_id: "hook-1".into(),
            command: vec!["network-access".into(), "https://example.com:443".into()],
            cwd: local_cwd.clone(),
        };
        assert_eq!(
            action.permission_request_payload(),
            PermissionRequestPayload {
                tool_name: HookToolName::bash(),
                tool_input: serde_json::json!({
                    "command": "curl https://example.com",
                    "description": "network-access https://example.com:443",
                }),
                tool_cwd: (environment_id == codex_exec_server::LOCAL_ENVIRONMENT_ID)
                    .then_some(expected_cwd),
            }
        );
    }
}

/// #722 (D246): a permission hook may allow a local network request, but not
/// a remote one (one not in the turn counts as remote unless it is the local
/// id); other actions are unaffected.
#[tokio::test]
async fn issue_722_hook_may_not_allow_remote_network_requests() {
    let (_session, turn, _events) = make_session_and_context_with_rx().await;
    let environments = crate::environment_selection::TurnEnvironmentSnapshot::default();
    let cwd = AbsolutePathBuf::try_from(std::env::temp_dir()).unwrap();
    let network = |environment_id: &str| ApprovalAction::NetworkAccess {
        id: "network-1".into(),
        turn_id: "turn-1".into(),
        environment_id: environment_id.into(),
        target: "https://example.com:443".into(),
        host: "example.com".into(),
        protocol: NetworkApprovalProtocol::Https,
        port: 443,
        trigger: None,
        hook_command: "curl https://example.com".into(),
        hook_run_id: "hook-1".into(),
        command: vec!["network-access".into(), "https://example.com:443".into()],
        cwd: cwd.clone(),
    };

    assert!(
        network(codex_exec_server::LOCAL_ENVIRONMENT_ID).permission_hook_may_allow(&environments)
    );
    assert!(!network("remote").permission_hook_may_allow(&environments));
    let (command, _) = exec_approval(&turn, "call-1", ApprovalsReviewer::User);
    assert!(command.permission_hook_may_allow(&environments));
}

/// #437: what the inspector lists for each refused approval.
#[test]
fn sec_437_refused_approvals_are_described_for_the_inspector() {
    for (decision, source, expected) in [
        (
            ReviewDecision::Abort,
            ApprovalResolutionSource::User,
            Some("declined by you"),
        ),
        (
            ReviewDecision::denied("no"),
            ApprovalResolutionSource::Hook,
            Some("declined by a permission hook"),
        ),
        (
            ReviewDecision::denied("no"),
            ApprovalResolutionSource::Guardian,
            Some("declined by the automatic reviewer"),
        ),
        (
            ReviewDecision::TimedOut,
            ApprovalResolutionSource::Guardian,
            Some("not answered in time"),
        ),
        (
            ReviewDecision::Approved,
            ApprovalResolutionSource::User,
            None,
        ),
    ] {
        assert_eq!(decline_outcome(&decision, source), expected);
    }
    assert_eq!(approval_kind("exec_command"), "command approval");
    assert_eq!(approval_kind("apply_patch"), "file edit approval");
    assert_eq!(approval_kind("mcp__x__y"), "tool approval");
}

/// Issue #513: an answer from the approval cache ("don't ask again" given
/// earlier) is not a person asked about this request now; a fresh request
/// skips the cache and asks.
#[tokio::test]
async fn issue_513_a_remembered_approval_is_not_a_person_asked_now() {
    use crate::tools::sandboxing::asking_person;
    use crate::tools::sandboxing::with_cached_approval;
    let (session, _turn) = crate::session::tests::make_session_and_context().await;
    let services = &session.services;
    let ask = |decision: ReviewDecision| move || async move { decision };
    assert_eq!(
        asking_person(|| with_cached_approval(
            services,
            "shell",
            vec!["issue-513"],
            /*fresh_human_authority*/ false,
            ask(ReviewDecision::ApprovedForSession),
        ))
        .await,
        (ReviewDecision::ApprovedForSession, true)
    );
    assert_eq!(
        asking_person(|| with_cached_approval(
            services,
            "shell",
            vec!["issue-513"],
            /*fresh_human_authority*/ false,
            ask(ReviewDecision::denied("the cache answers first")),
        ))
        .await,
        (ReviewDecision::ApprovedForSession, false)
    );
    assert_eq!(
        asking_person(|| with_cached_approval(
            services,
            "shell",
            vec!["issue-513"],
            /*fresh_human_authority*/ true,
            ask(ReviewDecision::Approved),
        ))
        .await,
        (ReviewDecision::Approved, true)
    );
    // Nothing marked: not asked (fail closed).
    assert_eq!(
        asking_person(|| async { ReviewDecision::Approved }).await,
        (ReviewDecision::Approved, false)
    );
}

/// A command approval context for the person (no automatic reviewer).
fn exec_approval(
    turn: &Arc<crate::session::turn_context::TurnContext>,
    call_id: &str,
    reviewer: ApprovalsReviewer,
) -> (ApprovalAction, ApprovalContext) {
    let mut review_context = GuardianReviewContext::from(turn);
    review_context.approval_policy = AskForApproval::OnRequest;
    review_context.approvals_reviewer = reviewer;
    let context = ApprovalContext {
        exec_requirement: None,
        review_context,
        cancellation_token: None,
        call_id: call_id.to_string(),
        tool_name: ToolName::plain("exec_command"),
        strict_auto_review: false,
        approval_reason: None,
        retry_reason: None,
        network_approval_context: None,
    };
    let action = ApprovalAction::ExecCommand {
        id: call_id.to_string(),
        environment_id: codex_exec_server::LOCAL_ENVIRONMENT_ID.to_string(),
        command: vec!["npm".to_string(), "publish".to_string()],
        hook_command: "npm publish".to_string(),
        #[allow(deprecated)]
        cwd: PathUri::from_abs_path(&turn.cwd),
        sandbox_permissions: SandboxPermissions::UseDefault,
        additional_permissions: None,
        justification: None,
        tty: false,
        proposed_execpolicy_amendment: Some(ExecPolicyAmendment::new(vec![
            "npm".to_string(),
            "publish".to_string(),
        ])),
    };
    (action, context)
}

/// Issue #513 and PF-30-S03 on upstream's approval flow: a "don't ask again"
/// answer is remembered, and a later approval it answers is
/// `Remembered`, not a person asked now. With fresh human authority the
/// person is asked again (no cache, no "don't ask again" rule offered), the
/// automatic reviewer is skipped, and "for session" counts for this call
/// only.
#[tokio::test]
async fn sec_513_fresh_human_authority_and_remembered_answers() -> anyhow::Result<()> {
    use anyhow::Context;
    let (session, turn, events) = make_session_and_context_with_rx().await;
    *session.active_turn.lock().await = Some(crate::state::ActiveTurn::default());

    // A person answers "for session": asked now.
    let (action, context) = exec_approval(&turn, "sec-513-first", ApprovalsReviewer::User);
    let prompt = session.request_user_approval(&action, &context);
    let approval =
        session.request_reviewer_approval(&action, &context, prompt, /*human_only*/ false);
    tokio::pin!(approval);
    tokio::select! {
        resolution = &mut approval => panic!("expected a prompt, got {resolution:?}"),
        event = events.recv() => {
            let codex_protocol::protocol::EventMsg::ExecApprovalRequest(request) =
                event.context("receive prompt")?.msg
            else {
                panic!("expected a command approval prompt");
            };
            assert!(request.proposed_execpolicy_amendment.is_some());
            session
                .notify_approval(&request.call_id, ReviewDecision::ApprovedForSession)
                .await;
        }
    }
    let first = approval.await;
    assert_eq!(first.source, ApprovalResolutionSource::User);
    assert_eq!(first.decision, ReviewDecision::ApprovedForSession);

    // The same command again: answered from the cache, nobody asked.
    let (action, context) = exec_approval(&turn, "sec-513-again", ApprovalsReviewer::User);
    let prompt = session.request_user_approval(&action, &context);
    let again = session
        .request_reviewer_approval(&action, &context, prompt, /*human_only*/ false)
        .await;
    assert_eq!(again.source, ApprovalResolutionSource::Remembered);
    assert_eq!(again.decision, ReviewDecision::ApprovedForSession);
    assert!(
        events.try_recv().is_err(),
        "no prompt for a remembered answer"
    );

    // Fresh human authority, with the automatic reviewer configured: the
    // person is asked anyway, without the rule offer, and "for session"
    // becomes this call only.
    let (action, context) = exec_approval(&turn, "sec-513-fresh", ApprovalsReviewer::AutoReview);
    let approval = session.request_approval_with_authority(
        &action,
        &context,
        session.request_user_approval_with_authority(
            &action, &context, /*fresh_human_authority*/ true,
        ),
        /*fresh_human_authority*/ true,
    );
    tokio::pin!(approval);
    tokio::select! {
        result = &mut approval => panic!("expected a prompt, got {:?}", result.result),
        event = events.recv() => {
            let codex_protocol::protocol::EventMsg::ExecApprovalRequest(request) =
                event.context("receive fresh prompt")?.msg
            else {
                panic!("expected a command approval prompt");
            };
            assert_eq!(request.proposed_execpolicy_amendment, None);
            session
                .notify_approval(&request.call_id, ReviewDecision::ApprovedForSession)
                .await;
        }
    }
    let fresh = approval.await;
    assert_eq!(fresh.source, Some(ApprovalResolutionSource::User));
    assert_eq!(fresh.decision, Some(ReviewDecision::Approved));
    assert!(matches!(fresh.result, Ok(ReviewDecision::Approved)));
    Ok(())
}

/// PF-30-S03: granted permissions preapprove a patch, but not a post-taint
/// protected one: with fresh human authority the person is asked.
#[tokio::test]
async fn pf_30_s03_fresh_authority_ignores_patch_preapproval() -> anyhow::Result<()> {
    use anyhow::Context;
    let (session, turn, events) = make_session_and_context_with_rx().await;
    *session.active_turn.lock().await = Some(crate::state::ActiveTurn::default());
    #[allow(deprecated)]
    let cwd = PathUri::from_abs_path(&turn.cwd);
    let action = ApprovalAction::ApplyPatch {
        id: "pf30-patch".to_string(),
        environment_id: codex_exec_server::LOCAL_ENVIRONMENT_ID.to_string(),
        cwd: cwd.clone(),
        files: vec![cwd],
        patch: "*** Begin Patch\n*** End Patch\n".to_string(),
        changes: Arc::new(HashMap::new()),
        permissions_preapproved: true,
    };
    let (_, mut context) = exec_approval(&turn, "pf30-patch", ApprovalsReviewer::User);
    context.tool_name = ToolName::plain("apply_patch");

    let preapproved = session
        .request_user_approval_with_authority(
            &action, &context, /*fresh_human_authority*/ false,
        )
        .await;
    assert_eq!(preapproved.decision, ReviewDecision::Approved);
    assert!(
        events.try_recv().is_err(),
        "a preapproved patch asks nobody"
    );

    let fresh = session.request_user_approval_with_authority(
        &action, &context, /*fresh_human_authority*/ true,
    );
    tokio::pin!(fresh);
    tokio::select! {
        resolution = &mut fresh => panic!("expected a prompt, got {resolution:?}"),
        event = events.recv() => {
            let codex_protocol::protocol::EventMsg::ApplyPatchApprovalRequest(request) =
                event.context("receive patch prompt")?.msg
            else {
                panic!("expected a patch approval prompt");
            };
            session
                .notify_approval(&request.call_id, ReviewDecision::ApprovedForSession)
                .await;
        }
    }
    assert_eq!(fresh.await.decision, ReviewDecision::Approved);
    Ok(())
}

/// Issue #709: "approve for session" of a patch in one environment is not an
/// approval of the same path in another; each environment asks on its own
/// (ported from the runtime's `approval_keys` to `ApprovalAction::cache_keys`).
#[tokio::test]
async fn issue_709_session_approval_does_not_carry_across_environments() {
    let (session, turn) = crate::session::tests::make_session_and_context().await;
    #[allow(deprecated)]
    let path = PathUri::from_abs_path(&turn.cwd.join("issue-709-approval-scope.txt"));
    let keys_for = |environment_id: &str| {
        ApprovalAction::ApplyPatch {
            id: "issue-709".to_string(),
            environment_id: environment_id.to_string(),
            cwd: path.clone(),
            files: vec![path.clone()],
            patch: String::new(),
            changes: Arc::new(HashMap::new()),
            permissions_preapproved: false,
        }
        .cache_keys()
    };
    let ask = |keys, decision: ReviewDecision| {
        let services = &session.services;
        asking_person(move || {
            with_cached_approval(
                services,
                "apply_patch",
                keys,
                /*fresh_human_authority*/ false,
                move || async move { decision },
            )
        })
    };
    let denied = || ReviewDecision::denied("issue 709");

    // A person approves the local patch for the session.
    assert_eq!(
        ask(keys_for("local"), ReviewDecision::ApprovedForSession).await,
        (ReviewDecision::ApprovedForSession, true)
    );
    // The same path in the remote environment still asks the person.
    assert_eq!(ask(keys_for("remote"), denied()).await, (denied(), true));
    // The local approval is still remembered for the local environment.
    assert_eq!(
        ask(keys_for("local"), denied()).await,
        (ReviewDecision::ApprovedForSession, false)
    );
    // A remote session approval covers the remote environment only.
    assert_eq!(
        ask(keys_for("remote"), ReviewDecision::ApprovedForSession).await,
        (ReviewDecision::ApprovedForSession, true)
    );
    assert_eq!(
        ask(keys_for("remote"), denied()).await,
        (ReviewDecision::ApprovedForSession, false)
    );
    assert_eq!(
        ask(keys_for("other-remote"), denied()).await,
        (denied(), true)
    );
}
