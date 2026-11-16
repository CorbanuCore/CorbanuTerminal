//! D621 (#2310): Aggressive's approval policy lives only in the launch
//! overrides, which the embedded server never loads. The resumes that
//! otherwise keep or restore a thread's saved settings must still send it.

use super::disconnect::serve_reconnect_requests;
use super::session_lifecycle_requests::RecordedRequests;
use super::session_lifecycle_requests::recorded_params;
use super::session_lifecycle_requests::start_recording_app_server;
use super::*;
use crate::app::reconnect::ReconnectPresentation;
use crate::app::reconnect::reconnect;
use pretty_assertions::assert_eq;
use serde_json::json;
use tokio::net::TcpListener;

const TEST_STACK_SIZE_BYTES: usize = 8 * 1024 * 1024;

fn run(test: impl std::future::Future<Output = Result<()>>) -> Result<()> {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(/*val*/ 1)
        .thread_stack_size(TEST_STACK_SIZE_BYTES)
        .enable_all()
        .build()?
        .block_on(test)
}

/// The approval policy each recorded `thread/resume` carried.
fn resume_approvals(requests: &RecordedRequests) -> Vec<Option<String>> {
    recorded_params(requests, "thread/resume")
        .iter()
        .map(|params| params["approvalPolicy"].as_str().map(str::to_string))
        .collect()
}

fn expected(aggressive: bool) -> Option<String> {
    aggressive.then(|| "untrusted".to_string())
}

#[test]
fn live_attach_and_snapshot_refresh_send_the_aggressive_approval() -> Result<()> {
    run(async {
        for aggressive in [false, true] {
            let mut app = make_test_app().await;
            let (mut app_server, requests, _proxy) = start_recording_app_server(
                &app.config,
                /*blocked_thread_list*/ None,
                /*failed_thread_name*/ None,
            )
            .await?;
            let thread_id = app_server
                .start_thread(&app.config)
                .await?
                .session
                .thread_id;
            if aggressive {
                crate::security::aggressive::apply_launch_overrides(&mut app.harness_overrides);
            }
            app.agent_navigation.upsert(
                thread_id,
                Some("Scout".to_string()),
                Some("worker".to_string()),
                /*is_closed*/ false,
            );
            requests.lock().expect("request recorder lock").clear();

            assert!(
                app.attach_live_thread_for_selection(&mut app_server, thread_id)
                    .await?
            );
            assert_eq!(
                resume_approvals(&requests),
                vec![expected(aggressive)],
                "live attach, aggressive={aggressive}"
            );

            requests.lock().expect("request recorder lock").clear();
            let mut snapshot = ThreadEventSnapshot {
                session: None,
                delegated_turns: Vec::new(),
                turns: Vec::new(),
                events: Vec::new(),
                active_reasoning_item: None,
                input_state: None,
            };
            assert!(
                app.refresh_snapshot_session_if_needed(
                    &mut app_server,
                    thread_id,
                    /*is_replay_only*/ false,
                    &mut snapshot,
                )
                .await
            );
            assert_eq!(
                resume_approvals(&requests),
                vec![expected(aggressive)],
                "thread routing refresh, aggressive={aggressive}"
            );
        }
        Ok(())
    })
}

#[test]
fn saved_pane_restore_sends_the_aggressive_approval() -> Result<()> {
    run(async {
        for aggressive in [false, true] {
            let mut app = make_test_app().await;
            let (mut app_server, requests, _proxy) = start_recording_app_server(
                &app.config,
                /*blocked_thread_list*/ None,
                /*failed_thread_name*/ None,
            )
            .await?;
            if aggressive {
                crate::security::aggressive::apply_launch_overrides(&mut app.harness_overrides);
            }
            app.primary_thread_id = Some(ThreadId::new());
            let pane = ThreadId::new();
            app.upsert_agent_picker_thread(
                pane,
                Some("Operator pane".to_string()),
                /*agent_role*/ None,
                /*is_closed*/ false,
            );
            assert_eq!(
                app.restorable_operator_owned_codex_user_pane_ids(),
                vec![pane]
            );
            requests.lock().expect("request recorder lock").clear();

            // The thread has no rollout, so the restore fails after the request.
            app.restore_codex_user_panes_from_saved_state(&mut app_server)
                .await;

            assert_eq!(
                resume_approvals(&requests),
                vec![expected(aggressive)],
                "saved pane restore, aggressive={aggressive}"
            );
        }
        Ok(())
    })
}

#[tokio::test]
async fn reconnect_sends_the_aggressive_approval() -> Result<()> {
    for aggressive in [false, true] {
        let (mut app, _events, _ops) = make_test_app_with_channels().await;
        app.config.model = Some("gpt-test".into());
        if aggressive {
            crate::security::aggressive::apply_launch_overrides(&mut app.harness_overrides);
        }
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let endpoint = crate::RemoteAppServerEndpoint::WebSocket {
            websocket_url: format!("ws://{}", listener.local_addr()?),
            auth_token: None,
        };
        let resumes = Arc::new(std::sync::Mutex::new(Vec::<serde_json::Value>::new()));
        let recorded = Arc::clone(&resumes);
        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await?;
            serve_reconnect_requests(
                tokio_tungstenite::accept_async(stream).await?,
                move |request| {
                    let response = match request.method.as_str() {
                        "getAuthStatus" => json!({"result": {}}),
                        "thread/resume" => {
                            recorded
                                .lock()
                                .expect("resume recorder lock")
                                .push(request.params.clone().unwrap_or_default());
                            json!({"error": {"code": -32600, "message": "thread no longer exists"}})
                        }
                        _ => {
                            json!({"error": {"code": -32600, "message": "thread no longer exists"}})
                        }
                    };
                    std::future::ready(Some(response))
                },
            )
            .await
        });
        // A local daemon takes the client's thread settings, as the embedded
        // server does.
        let result = reconnect(
            crate::backend_id::BackendId::new(),
            AppServerTarget::LocalDaemon {
                endpoint,
                allow_embedded_fallback: false,
            },
            app.config.clone(),
            app.local_settings.clone(),
            app.launch_approval_resume_permissions(),
            Some(ThreadId::new()),
            /*remote_cwd*/ None,
            crate::dynamic_tools_mcp::ThreadToolTransport::Dynamic,
            ReconnectPresentation::Conversation,
        )
        .await;
        drop(result);
        server.abort();
        let approvals = resumes
            .lock()
            .expect("resume recorder lock")
            .iter()
            .map(|params| params["approvalPolicy"].as_str().map(str::to_string))
            .collect::<Vec<_>>();
        assert_eq!(
            approvals,
            vec![expected(aggressive)],
            "reconnect, aggressive={aggressive}"
        );
    }
    Ok(())
}

/// Requests built without the launch overrides: `AppServerSession` starts,
/// forks and plain resumes, and the agent tools' own resume, fork and start.
#[tokio::test]
async fn agent_tools_and_session_requests_send_the_aggressive_approval() -> Result<()> {
    use crate::security::level::ChosenLevel;

    for aggressive in [false, true] {
        let _levels = aggressive.then(|| {
            crate::security::level::test_levels::set(
                ChosenLevel::Aggressive,
                ChosenLevel::Aggressive,
            )
        });
        let app = make_test_app().await;
        let (mut app_server, requests, _proxy) = start_recording_app_server(
            &app.config,
            /*blocked_thread_list*/ None,
            /*failed_thread_name*/ None,
        )
        .await?;
        let caller = app_server
            .start_thread(&app.config)
            .await?
            .session
            .thread_id;
        let target = app_server
            .start_thread(&app.config)
            .await?
            .session
            .thread_id;
        let starts = recorded_params(&requests, "thread/start");
        assert_eq!(starts.len(), 2);
        if aggressive {
            for params in &starts {
                assert_eq!(params["approvalPolicy"], "untrusted", "{params}");
            }
        }
        requests.lock().expect("request recorder lock").clear();

        // `AppServerSession` resume and fork with no permission choice.
        let _ = app_server
            .resume_thread(
                &app.local_settings,
                app.config.clone(),
                target,
                crate::app_server_session::ResumeModelSettings::RestoreFromThread,
            )
            .await;
        let _ = app_server
            .fork_thread(&app.local_settings, app.config.clone(), target)
            .await;
        assert_eq!(
            resume_approvals(&requests),
            vec![expected(aggressive)],
            "session resume, aggressive={aggressive}"
        );
        assert_eq!(
            recorded_params(&requests, "thread/fork")
                .iter()
                .filter(|params| params["approvalPolicy"] == "untrusted")
                .count(),
            usize::from(aggressive),
            "session fork, aggressive={aggressive}"
        );
        requests.lock().expect("request recorder lock").clear();

        // The agent tools build their own requests.
        for (tool, arguments) in [
            ("fork_thread", json!({"threadId": target})),
            (
                "send_message_to_thread",
                json!({"threadId": target, "prompt": "hello"}),
            ),
            ("create_thread", json!({"prompt": "hello"})),
        ] {
            let (_status_sender, status_updates) =
                tokio::sync::broadcast::channel(/*capacity*/ 1);
            let _ = tokio::time::timeout(
                Duration::from_secs(/*secs*/ 20),
                crate::dynamic_tools::execute(
                    app_server.request_handle(),
                    codex_app_server_protocol::DynamicToolCallParams {
                        thread_id: caller.to_string(),
                        turn_id: "source-turn".to_string(),
                        call_id: format!("d621-{tool}"),
                        namespace: Some(crate::dynamic_tools::NAMESPACE.to_string()),
                        tool: tool.to_string(),
                        arguments,
                    },
                    codex_app_server_protocol::ThreadStartParams::default(),
                    app.config.features.get().clone(),
                    status_updates,
                    /*app_event_tx*/ None,
                ),
            )
            .await;
        }
        let sent = ["thread/fork", "thread/resume", "thread/start"]
            .into_iter()
            .flat_map(|method| {
                recorded_params(&requests, method)
                    .into_iter()
                    .map(move |params| {
                        (
                            method,
                            params["approvalPolicy"].as_str().map(str::to_string),
                        )
                    })
            })
            .collect::<Vec<_>>();
        // fork_thread's fork, send_message_to_thread's resume, and
        // create_thread's source resume and start.
        assert_eq!(
            sent.iter().map(|(method, _)| *method).collect::<Vec<_>>(),
            vec![
                "thread/fork",
                "thread/resume",
                "thread/resume",
                "thread/start"
            ],
            "{sent:?}"
        );
        if aggressive {
            assert!(
                sent.iter()
                    .all(|(_, policy)| policy.as_deref() == Some("untrusted")),
                "{sent:?}"
            );
        } else {
            // Outside Aggressive the tools leave the policy to the thread
            // (create_thread copies the source thread's).
            assert_eq!(sent[0].1, None, "{sent:?}");
            assert_eq!(sent[1].1, None, "{sent:?}");
            assert_eq!(sent[2].1, None, "{sent:?}");
        }
    }
    Ok(())
}
