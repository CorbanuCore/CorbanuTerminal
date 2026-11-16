use anyhow::Result;
use app_test_support::MockResponsesConfig;
use app_test_support::TestAppServer;
use app_test_support::create_final_assistant_message_sse_response;
use app_test_support::write_models_cache_with_models;
use codex_app_server_protocol::ClientRequest;
use codex_app_server_protocol::FileChangeApprovalDecision;
use codex_app_server_protocol::FileChangeRequestApprovalResponse;
use codex_app_server_protocol::JSONRPCMessage;
use codex_app_server_protocol::PermissionGrantScope;
use codex_app_server_protocol::ServerRequest;
use codex_app_server_protocol::ThreadStartParams;
use codex_app_server_protocol::TurnCompletedNotification;
use codex_app_server_protocol::TurnStartParams;
use codex_app_server_protocol::TurnStartResponse;
use codex_app_server_protocol::TurnStatus;
use codex_app_server_protocol::UserInput;
use codex_exec_server::ReadFileOptions;
use codex_features::Feature;
use codex_models_manager::model_info::model_info_from_slug;
use codex_protocol::openai_models::ApplyPatchToolType;
use core_test_support::responses;
use pretty_assertions::assert_eq;
use serde_json::json;

/// A new code cell in the next turn must ask again even after an earlier cell used its grant.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn new_turn_code_cell_does_not_inherit_the_previous_cells_grant() -> Result<()> {
    let home = tempfile::tempdir()?;
    let server = responses::start_mock_server().await;
    MockResponsesConfig::new(&server.uri())
        .with_model("gpt-5.4")
        .with_approval_policy("on-request")
        .with_root_config("approvals_reviewer = \"user\"\ndefault_permissions = \"grant-fixture\"")
        .with_extra_config(
            "[permissions.grant-fixture.filesystem]\n\":root\" = \"read\"\n\
             [permissions.grant-fixture.filesystem.\":project_roots\"]\n\
             initial_write = \"write\"\n",
        )
        .enable_feature(Feature::RequestPermissionsTool)
        .enable_feature(Feature::CodeModeOnly)
        .write(home.path())?;
    let mut model = model_info_from_slug("gpt-5.4");
    model.apply_patch_tool_type = Some(ApplyPatchToolType::Freeform);
    write_models_cache_with_models(home.path(), vec![model]).await?;
    let mut app = TestAppServer::builder()
        .with_codex_home(home.path())
        .build_initialized()
        .await?;
    let cwd = app.auto_env()?.selection().cwd.clone();
    let fs = app.auto_env()?.environment().get_filesystem();
    let thread = app.start_thread(ThreadStartParams::default()).await?.thread;
    let mut approvals = Vec::new();
    for (index, file) in ["approved.txt", "unapproved.txt"].into_iter().enumerate() {
        let request = if index == 0 {
            r#"await tools.request_permissions({permissions: {file_system: {write: ["."]}}});"#
        } else {
            ""
        };
        let cell = format!(
            r#"// @exec: {{"yield_time_ms": 60000}}
{request}
text(await tools.apply_patch("*** Begin Patch\n*** Add File: {file}\n+cell write\n*** End Patch"));"#,
        );
        let cell_id = format!("cell-{index}");
        let mock = responses::mount_sse_sequence(
            &server,
            vec![
                responses::sse(vec![
                    responses::ev_response_created(&cell_id),
                    responses::ev_custom_tool_call(&cell_id, "exec", &cell),
                    responses::ev_completed(&cell_id),
                ]),
                create_final_assistant_message_sse_response("done")?,
            ],
        )
        .await;
        let TurnStartResponse { turn } = app
            .request(|request_id| ClientRequest::TurnStart {
                request_id,
                params: TurnStartParams {
                    thread_id: thread.id.clone(),
                    input: vec![UserInput::Text {
                        text: "run the cell".to_string(),
                        text_elements: Vec::new(),
                    }],
                    ..Default::default()
                },
            })
            .await?;
        loop {
            match tokio::time::timeout(std::time::Duration::from_secs(10), app.read_next_message())
                .await??
            {
                JSONRPCMessage::Request(request) => match ServerRequest::try_from(request)? {
                    ServerRequest::PermissionsRequestApproval { request_id, params } => {
                        assert_eq!(index, 0, "the second cell did not request a new grant");
                        assert_eq!((&params.thread_id, &params.turn_id), (&thread.id, &turn.id));
                        approvals.push("turn grant");
                        app.send_response(
                            request_id,
                            json!({
                                "permissions": params.permissions,
                                "scope": PermissionGrantScope::Turn,
                                "strictAutoReview": false,
                            }),
                        )
                        .await?;
                    }
                    ServerRequest::FileChangeRequestApproval { request_id, params } => {
                        assert_eq!(index, 1, "the first cell already holds its own grant");
                        assert_eq!((&params.thread_id, &params.turn_id), (&thread.id, &turn.id));
                        approvals.push("new cell write");
                        app.send_response(
                            request_id,
                            serde_json::to_value(FileChangeRequestApprovalResponse {
                                decision: FileChangeApprovalDecision::Decline,
                            })?,
                        )
                        .await?;
                    }
                    unexpected => anyhow::bail!("unexpected approval: {unexpected:?}"),
                },
                JSONRPCMessage::Notification(notification)
                    if notification.method == "turn/completed" =>
                {
                    let completed: TurnCompletedNotification =
                        serde_json::from_value(notification.params.unwrap())?;
                    assert_eq!(
                        (&completed.turn.id, completed.turn.status),
                        (&turn.id, TurnStatus::Completed)
                    );
                    break;
                }
                _ => {}
            }
        }
        assert_eq!(mock.requests().len(), 2);
    }
    assert_eq!(approvals, vec!["turn grant", "new cell write"]);
    assert_eq!(
        fs.read_file(
            &cwd.join("approved.txt")?,
            ReadFileOptions::default(),
            /*sandbox*/ None
        )
        .await?,
        b"cell write\n"
    );
    assert_eq!(
        fs.read_file(
            &cwd.join("unapproved.txt")?,
            ReadFileOptions::default(),
            /*sandbox*/ None
        )
        .await
        .expect_err("the new cell's declined write must not exist")
        .kind(),
        std::io::ErrorKind::NotFound,
    );
    Ok(())
}
