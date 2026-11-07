use super::Submission;
use crate::WithTurnExtensionData;
use crate::realtime_conversation::handle_audio as handle_realtime_conversation_audio;
use crate::realtime_conversation::handle_close as handle_realtime_conversation_close;
use crate::realtime_conversation::handle_speech as handle_realtime_conversation_speech;
use crate::realtime_conversation::handle_start as handle_realtime_conversation_start;
use crate::realtime_conversation::handle_text as handle_realtime_conversation_text;
use async_channel::Receiver;
use codex_features::Feature;
use codex_otel::set_parent_from_w3c_trace_context;
use tracing::Instrument;
use tracing::debug_span;
use tracing::info_span;

use crate::session::session::Session;
use crate::session::thread_settings;
use crate::session::turn_input;

use crate::config::Config;
use crate::context::ContextualUserFragment;
use crate::context::GuardianApprovedAction;
use crate::review_prompts::resolve_review_request;
use crate::session::spawn_review_thread;
use crate::tasks::CompactTask;
use crate::tasks::UserShellCommandMode;
use crate::tasks::UserShellCommandTask;
use crate::tasks::execute_user_shell_command;
use crate::thread_manager::AgentTreeShutdownFailure;
use crate::thread_manager::thread_store_error_kind;
use codex_protocol::approvals::ExecApprovalKind;
use codex_protocol::error::CodexErr;
use codex_protocol::protocol::CodexErrorInfo;
use codex_protocol::protocol::ErrorEvent;
use codex_protocol::protocol::Event;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::GuardianAssessmentEvent;
use codex_protocol::protocol::GuardianAssessmentStatus;
use codex_protocol::protocol::Op;
use codex_protocol::protocol::RealtimeConversationListVoicesResponseEvent;
use codex_protocol::protocol::RealtimeVoicesList;
use codex_protocol::protocol::ReviewDecision;
use codex_protocol::protocol::ReviewRequest;
use codex_protocol::protocol::ThreadMemoryMode;
use codex_protocol::protocol::TurnAbortReason;
use codex_protocol::protocol::WarningEvent;
use codex_protocol::request_permissions::RequestPermissionsResponse;
use codex_protocol::request_user_input::RequestUserInputResponse;
<<<<<<< HEAD
use codex_protocol::user_input::UserInput;
||||||| 413492cd6c
=======
use codex_thread_store::PersistContext;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a

use crate::context_manager::is_user_turn_boundary;
use codex_protocol::dynamic_tools::DynamicToolResponse;
use codex_protocol::mcp::RequestId as ProtocolRequestId;
use codex_protocol::protocol::AdditionalContextEntry;
use codex_rmcp_client::ElicitationAction;
use codex_rmcp_client::ElicitationResponse;
use serde_json::Value;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::sync::Arc;
#[cfg(test)]
use std::sync::Mutex;
#[cfg(test)]
use std::sync::OnceLock;
use tracing::debug;
use tracing::info;
use tracing::warn;

#[cfg(test)]
struct ActiveTurnNotSteerableDeferHook {
    reached: tokio::sync::oneshot::Sender<()>,
    release: tokio::sync::oneshot::Receiver<()>,
}

#[cfg(test)]
static ACTIVE_TURN_NOT_STEERABLE_DEFER_HOOK: OnceLock<
    Mutex<Option<ActiveTurnNotSteerableDeferHook>>,
> = OnceLock::new();

#[cfg(test)]
pub(crate) fn set_active_turn_not_steerable_defer_hook_for_test(
    reached: tokio::sync::oneshot::Sender<()>,
    release: tokio::sync::oneshot::Receiver<()>,
) {
    let hook = ACTIVE_TURN_NOT_STEERABLE_DEFER_HOOK.get_or_init(|| Mutex::new(None));
    *hook.lock().expect("defer hook lock") =
        Some(ActiveTurnNotSteerableDeferHook { reached, release });
}

#[cfg(test)]
async fn active_turn_not_steerable_defer_hook_for_test() {
    let hook = ACTIVE_TURN_NOT_STEERABLE_DEFER_HOOK
        .get_or_init(|| Mutex::new(None))
        .lock()
        .expect("defer hook lock")
        .take();
    if let Some(hook) = hook {
        let _ = hook.reached.send(());
        let _ = hook.release.await;
    }
}

pub async fn interrupt(sess: &Arc<Session>) {
    sess.interrupt_task().await;
}

pub async fn clean_background_terminals(sess: &Arc<Session>) {
    sess.close_unified_exec_processes().await;
}

pub async fn realtime_conversation_list_voices(sess: &Session, sub_id: String) {
    sess.send_event_raw(Event {
        id: sub_id,
        msg: EventMsg::RealtimeConversationListVoicesResponse(
            RealtimeConversationListVoicesResponseEvent {
                voices: RealtimeVoicesList::builtin(),
            },
        ),
    })
    .await;
}

<<<<<<< HEAD
pub async fn user_input_or_turn(
    sess: &Arc<Session>,
    sub_id: String,
    op: Op,
    client_user_message_id: Option<String>,
    parent_turn_id: Option<String>,
) {
    user_input_or_turn_inner(sess, sub_id, op, client_user_message_id, parent_turn_id).await;
}

pub async fn update_thread_settings(
    sess: &Arc<Session>,
    sub_id: String,
    thread_settings: ThreadSettingsOverrides,
) {
    let updates = thread_settings_update(sess, thread_settings).await;
    match sess.update_settings(updates).await {
        Ok(()) => {
            sess.send_event_raw_without_materializing_rollout(Event {
                id: sub_id,
                msg: thread_settings_applied_event(sess).await,
            })
            .await;
        }
        Err(err) => {
            sess.send_event_raw(Event {
                id: sub_id,
                msg: EventMsg::Error(ErrorEvent {
                    message: format!("invalid thread settings override: {err}"),
                    codex_error_info: Some(CodexErrorInfo::BadRequest),
                }),
            })
            .await;
        }
    }
}

async fn thread_settings_update(
    sess: &Session,
    thread_settings: ThreadSettingsOverrides,
) -> SessionSettingsUpdate {
    let ThreadSettingsOverrides {
        environments,
        profile_workspace_roots,
        approval_policy,
        approvals_reviewer,
        sandbox_policy,
        permission_profile,
        active_permission_profile,
        windows_sandbox_level,
        model,
        model_provider,
        effort,
        summary,
        service_tier,
        collaboration_mode,
        personality,
    } = thread_settings;
    let collaboration_mode = match collaboration_mode {
        Some(collaboration_mode) => collaboration_mode,
        None => {
            let state = sess.state.lock().await;
            // Model and reasoning effort live in CollaborationMode settings today, so
            // partial thread-settings updates refresh those fields on the active mode.
            state
                .session_configuration
                .collaboration_mode
                .with_updates(model, effort, /*developer_instructions*/ None)
        }
    };
    SessionSettingsUpdate {
        environments,
        profile_workspace_roots,
        approval_policy,
        approvals_reviewer,
        sandbox_policy,
        permission_profile,
        active_permission_profile,
        windows_sandbox_level,
        collaboration_mode: Some(collaboration_mode),
        model_provider,
        reasoning_summary: summary,
        service_tier,
        personality,
        ..Default::default()
    }
}

pub(super) async fn thread_settings_applied_event(sess: &Session) -> EventMsg {
    let snapshot = {
        let state = sess.state.lock().await;
        state.session_configuration.thread_config_snapshot()
    };
    EventMsg::ThreadSettingsApplied(ThreadSettingsAppliedEvent {
        thread_settings: snapshot.into_thread_settings_snapshot(),
    })
}

pub(super) async fn user_input_or_turn_inner(
    sess: &Arc<Session>,
    sub_id: String,
    op: Op,
    client_user_message_id: Option<String>,
    parent_turn_id: Option<String>,
) {
    let Op::UserInput {
        items,
        final_output_json_schema,
        responsesapi_client_metadata,
        additional_context,
        thread_settings,
    } = op
    else {
        unreachable!();
    };
    if !items.is_empty() {
        sess.services
            .agent_control
            .note_native_operator_input(sess.thread_id);
    }
    let emit_thread_settings_applied = thread_settings != ThreadSettingsOverrides::default();
    let mut updates = if emit_thread_settings_applied {
        thread_settings_update(sess, thread_settings).await
    } else {
        SessionSettingsUpdate::default()
    };
    updates.final_output_json_schema = Some(final_output_json_schema);

    let Ok(current_context) = sess.new_turn_with_sub_id(sub_id.clone(), updates).await else {
        // new_turn_with_sub_id already emits the error event.
        return;
    };
    if emit_thread_settings_applied {
        sess.send_event_raw_without_materializing_rollout(Event {
            id: sub_id.clone(),
            msg: thread_settings_applied_event(sess).await,
        })
        .await;
    }
    sess.maybe_emit_model_warnings_for_turn(current_context.as_ref())
        .await;
    match sess
        .steer_input(
            items.clone(),
            additional_context.clone(),
            /*expected_turn_id*/ None,
            client_user_message_id.clone(),
            responsesapi_client_metadata.clone(),
        )
        .await
    {
        Ok(_) => {
            current_context.session_telemetry.user_prompt(&items);
        }
        Err(SteerInputError::NoActiveTurn(items)) => {
            if let Some(id) = parent_turn_id {
                current_context.turn_metadata_state.set_parent_turn_id(id);
            }
            spawn_regular_turn_for_no_active_input(
                sess,
                &current_context,
                items,
                additional_context,
                client_user_message_id,
                responsesapi_client_metadata,
                /*record_user_prompt*/ true,
            )
            .await;
        }
        Err(
            err @ (SteerInputError::AuthorizationChanged(_)
            | SteerInputError::ActiveTurnNotSteerable { .. }),
        ) => {
            let items = match err {
                SteerInputError::AuthorizationChanged(items) => items,
                _ => items,
            };
            #[cfg(test)]
            active_turn_not_steerable_defer_hook_for_test().await;

            match sess
                .defer_user_input_until_active_turn_finished(
                    items.clone(),
                    additional_context.clone(),
                    client_user_message_id.clone(),
                    current_context.final_output_json_schema.clone(),
                )
                .await
            {
                Ok(_) => {
                    current_context.session_telemetry.user_prompt(&items);
                }
                Err(SteerInputError::NoActiveTurn(items)) => {
                    let Ok(current_context) = sess
                        .new_turn_with_sub_id(
                            sub_id.clone(),
                            SessionSettingsUpdate {
                                final_output_json_schema: Some(
                                    current_context.final_output_json_schema.clone(),
                                ),
                                ..Default::default()
                            },
                        )
                        .await
                    else {
                        return;
                    };
                    if let Some(id) = parent_turn_id {
                        current_context.turn_metadata_state.set_parent_turn_id(id);
                    }
                    spawn_regular_turn_for_no_active_input(
                        sess,
                        &current_context,
                        items,
                        additional_context,
                        client_user_message_id,
                        responsesapi_client_metadata,
                        /*record_user_prompt*/ true,
                    )
                    .await;
                }
                Err(err) => {
                    sess.send_event_raw(Event {
                        id: sub_id,
                        msg: EventMsg::Error(err.to_error_event()),
                    })
                    .await;
                }
            }
        }
        Err(err) => {
            sess.send_event_raw(Event {
                id: sub_id,
                msg: EventMsg::Error(err.to_error_event()),
            })
            .await;
        }
    }
}

async fn spawn_regular_turn_for_no_active_input(
    sess: &Arc<Session>,
    current_context: &Arc<crate::session::turn_context::TurnContext>,
    items: Vec<UserInput>,
    additional_context: BTreeMap<String, AdditionalContextEntry>,
    client_user_message_id: Option<String>,
    responsesapi_client_metadata: Option<HashMap<String, String>>,
    record_user_prompt: bool,
) {
    if let Some(responsesapi_client_metadata) = responsesapi_client_metadata {
        current_context
            .turn_metadata_state
            .set_responsesapi_client_metadata(responsesapi_client_metadata);
    }
    if record_user_prompt {
        current_context.session_telemetry.user_prompt(&items);
    }
    sess.refresh_mcp_if_dirty().await;
    let additional_context_input = {
        let mut state = sess.state.lock().await;
        state.additional_context.merge(additional_context)
    };
    let mut task_input = additional_context_input
        .into_iter()
        .map(ResponseItem::from)
        .map(TurnInput::ResponseItem)
        .collect::<Vec<_>>();
    // Queue-only collaboration mail must join the next admitted turn before its first model
    // request. Leaving it in the session mailbox until after the user's request has already been
    // sampled creates an unnecessary continuation and can loop when that continuation has no new
    // model output. The submission loop serializes this idle-to-active boundary, so draining here
    // also preserves mailbox order relative to the newly admitted user input.
    let (mailbox_input, mailbox_parent_turn_id) =
        sess.input_queue.drain_ready_mailbox_input_items().await;
    task_input.extend(mailbox_input);
    if let Some(parent_turn_id) = mailbox_parent_turn_id {
        current_context
            .turn_metadata_state
            .set_parent_turn_id(parent_turn_id);
    }
    if !items.is_empty() {
        task_input.push(TurnInput::UserInput {
            content: items,
            client_id: client_user_message_id,
        });
    }
    sess.spawn_task(
        Arc::clone(current_context),
        task_input,
        crate::tasks::RegularTask::new(),
    )
    .await;
}

||||||| 413492cd6c
pub async fn user_input_or_turn(
    sess: &Arc<Session>,
    sub_id: String,
    op: Op,
    client_user_message_id: Option<String>,
    parent_turn_id: Option<String>,
) {
    user_input_or_turn_inner(sess, sub_id, op, client_user_message_id, parent_turn_id).await;
}

pub async fn update_thread_settings(
    sess: &Arc<Session>,
    sub_id: String,
    thread_settings: ThreadSettingsOverrides,
) {
    let updates = thread_settings_update(sess, thread_settings).await;
    match sess.update_settings(updates).await {
        Ok(()) => {
            sess.send_event_raw_without_materializing_rollout(Event {
                id: sub_id,
                msg: thread_settings_applied_event(sess).await,
            })
            .await;
        }
        Err(err) => {
            sess.send_event_raw(Event {
                id: sub_id,
                msg: EventMsg::Error(ErrorEvent {
                    message: format!("invalid thread settings override: {err}"),
                    codex_error_info: Some(CodexErrorInfo::BadRequest),
                }),
            })
            .await;
        }
    }
}

async fn thread_settings_update(
    sess: &Session,
    thread_settings: ThreadSettingsOverrides,
) -> SessionSettingsUpdate {
    let ThreadSettingsOverrides {
        environments,
        profile_workspace_roots,
        approval_policy,
        approvals_reviewer,
        sandbox_policy,
        permission_profile,
        active_permission_profile,
        windows_sandbox_level,
        model,
        effort,
        summary,
        service_tier,
        collaboration_mode,
        personality,
    } = thread_settings;
    let collaboration_mode = match collaboration_mode {
        Some(collaboration_mode) => collaboration_mode,
        None => {
            let state = sess.state.lock().await;
            // Model and reasoning effort live in CollaborationMode settings today, so
            // partial thread-settings updates refresh those fields on the active mode.
            state
                .session_configuration
                .collaboration_mode
                .with_updates(model, effort, /*developer_instructions*/ None)
        }
    };
    SessionSettingsUpdate {
        environments,
        profile_workspace_roots,
        approval_policy,
        approvals_reviewer,
        sandbox_policy,
        permission_profile,
        active_permission_profile,
        windows_sandbox_level,
        collaboration_mode: Some(collaboration_mode),
        reasoning_summary: summary,
        service_tier,
        personality,
        ..Default::default()
    }
}

pub(super) async fn thread_settings_applied_event(sess: &Session) -> EventMsg {
    let snapshot = {
        let state = sess.state.lock().await;
        state.session_configuration.thread_config_snapshot()
    };
    EventMsg::ThreadSettingsApplied(ThreadSettingsAppliedEvent {
        thread_settings: snapshot.into_thread_settings_snapshot(),
    })
}

pub(super) async fn user_input_or_turn_inner(
    sess: &Arc<Session>,
    sub_id: String,
    op: Op,
    client_user_message_id: Option<String>,
    parent_turn_id: Option<String>,
) {
    let Op::UserInput {
        items,
        final_output_json_schema,
        responsesapi_client_metadata,
        additional_context,
        thread_settings,
    } = op
    else {
        unreachable!();
    };
    let emit_thread_settings_applied = thread_settings != ThreadSettingsOverrides::default();
    let mut updates = if emit_thread_settings_applied {
        thread_settings_update(sess, thread_settings).await
    } else {
        SessionSettingsUpdate::default()
    };
    updates.final_output_json_schema = Some(final_output_json_schema);

    let Ok(current_context) = sess.new_turn_with_sub_id(sub_id.clone(), updates).await else {
        // new_turn_with_sub_id already emits the error event.
        return;
    };
    if emit_thread_settings_applied {
        sess.send_event_raw_without_materializing_rollout(Event {
            id: sub_id.clone(),
            msg: thread_settings_applied_event(sess).await,
        })
        .await;
    }
    sess.maybe_emit_model_warnings_for_turn(current_context.as_ref())
        .await;
    match sess
        .steer_input(
            items.clone(),
            additional_context.clone(),
            /*expected_turn_id*/ None,
            client_user_message_id.clone(),
            responsesapi_client_metadata.clone(),
        )
        .await
    {
        Ok(_) => {
            current_context.session_telemetry.user_prompt(&items);
        }
        Err(SteerInputError::NoActiveTurn(items)) => {
            if let Some(id) = parent_turn_id {
                current_context.turn_metadata_state.set_parent_turn_id(id);
            }
            if let Some(responsesapi_client_metadata) = responsesapi_client_metadata {
                current_context
                    .turn_metadata_state
                    .set_responsesapi_client_metadata(responsesapi_client_metadata);
            }
            current_context.session_telemetry.user_prompt(&items);
            let additional_context_input = {
                let mut state = sess.state.lock().await;
                state.additional_context.merge(additional_context)
            };
            let mut task_input = additional_context_input
                .into_iter()
                .map(ResponseItem::from)
                .map(TurnInput::ResponseItem)
                .collect::<Vec<_>>();
            if !items.is_empty() {
                task_input.push(TurnInput::UserInput {
                    content: items,
                    client_id: client_user_message_id,
                });
            }
            sess.spawn_task(
                Arc::clone(&current_context),
                task_input,
                crate::tasks::RegularTask::new(),
            )
            .await;
        }
        Err(err) => {
            sess.send_event_raw(Event {
                id: sub_id,
                msg: EventMsg::Error(err.to_error_event()),
            })
            .await;
        }
    }
}

=======
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
/// Queues an inter-agent message, then lets the shared pending-work scheduler
/// decide whether an idle session should start a regular turn.
pub async fn inter_agent_communication(
    sess: &Arc<Session>,
    sub_id: String,
    communication: crate::PendingAgentMessage,
    start_options: codex_protocol::turn_input::TurnStartOptions,
) {
    let trigger_turn = communication.trigger_turn;
    sess.input_queue
<<<<<<< HEAD
        .enqueue_mailbox_communication_for_session(
            sess,
            communication,
            parent_turn_id.filter(|_| trigger_turn),
        )
||||||| 413492cd6c
        .enqueue_mailbox_communication(communication, parent_turn_id.filter(|_| trigger_turn))
=======
        .enqueue_mailbox_communication(communication, start_options)
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        .await;
    crate::agent_communication::emit_agent_communication_receive(&sub_id);
    if trigger_turn
        || sess.features.enabled(Feature::WakeOnAgentMessages)
        || sess.has_outstanding_durable_sleep()
    {
        sess.maybe_start_turn_for_pending_work_with_sub_id(sub_id)
            .await;
    }
}

pub async fn run_user_shell_command(
    sess: &Arc<Session>,
    sub_id: String,
    command: String,
    timeout_ms: Option<u64>,
) {
    if let Some((turn_context, cancellation_token)) =
        sess.active_turn_context_and_cancellation_token().await
    {
        let session = Arc::clone(sess);
        tokio::spawn(async move {
            execute_user_shell_command(
                session,
                turn_context,
                command,
                timeout_ms,
                cancellation_token,
                UserShellCommandMode::ActiveTurnAuxiliary,
            )
            .await;
        });
        return;
    }

    let turn_context = sess
        .new_turn_with_default_settings(sub_id, Default::default())
        .await;
    sess.spawn_task(
        turn_context,
        Vec::new(),
        UserShellCommandTask::new(command, timeout_ms),
    )
    .await;
}

pub async fn resolve_elicitation(
    sess: &Arc<Session>,
    server_name: String,
    request_id: ProtocolRequestId,
    decision: codex_protocol::approvals::ElicitationAction,
    content: Option<Value>,
    meta: Option<Value>,
) {
    let action = match decision {
        codex_protocol::approvals::ElicitationAction::Accept => ElicitationAction::Accept,
        codex_protocol::approvals::ElicitationAction::Decline => ElicitationAction::Decline,
        codex_protocol::approvals::ElicitationAction::Cancel => ElicitationAction::Cancel,
    };
    let content = match action {
        // Preserve the legacy fallback for clients that only send an action.
        ElicitationAction::Accept => Some(content.unwrap_or_else(|| serde_json::json!({}))),
        ElicitationAction::Decline | ElicitationAction::Cancel => None,
        _ => None,
    };
    let response = ElicitationResponse {
        action,
        content,
        meta,
    };
    let request_id = match request_id {
        ProtocolRequestId::String(value) => {
            rmcp::model::NumberOrString::String(std::sync::Arc::from(value))
        }
        ProtocolRequestId::Integer(value) => rmcp::model::NumberOrString::Number(value),
    };
    if let Err(err) = sess
        .resolve_elicitation(server_name, request_id, response)
        .await
    {
        warn!(
            error = %err,
            "failed to resolve elicitation request in session"
        );
    }
}

<<<<<<< HEAD
/// Propagate a user's exec approval decision to the session.
/// Also optionally applies an execpolicy amendment.
pub async fn exec_approval(
    sess: &Arc<Session>,
    approval_id: String,
    turn_id: Option<String>,
    decision: ReviewDecision,
) {
    let event_turn_id = turn_id.unwrap_or_else(|| approval_id.clone());
    let mut stale_amendment = false;
    if let ReviewDecision::ApprovedExecpolicyAmendment {
        proposed_execpolicy_amendment,
    } = &decision
    {
        // #945: only an answer to an open prompt may save a rule. One with no
        // open prompt (its turn ended) is dropped by `notify_approval` anyway.
        let persisted = match sess.pending_approval_shown_under(&approval_id).await {
            Some(shown_under) => Some(
                sess.persist_execpolicy_amendment(proposed_execpolicy_amendment, shown_under)
                    .await,
            ),
            None => {
                tracing::warn!(
                    "not saving a \"don't ask again\" rule for {approval_id}: no prompt is open"
                );
                None
            }
        };
        match persisted {
            None => {}
            // #945: the prompt was shown before a security transition. The
            // rule is not saved; D50 (#1001): nor does the answer approve the
            // command, which is asked about again under the new level.
            Some(Ok(None)) => {
                let message = "The security level changed while this approval was open, so \
                               \"don't ask again\" was not saved and the answer was not used."
                    .to_string();
                sess.send_event_raw(Event {
                    id: event_turn_id.clone(),
                    msg: EventMsg::Warning(WarningEvent { message }),
                })
                .await;
                stale_amendment = true;
            }
            Some(Ok(Some(crate::exec_policy::AmendmentScope::Saved))) => {
                sess.record_execpolicy_amendment_message(
                    &event_turn_id,
                    proposed_execpolicy_amendment,
||||||| 413492cd6c
/// Propagate a user's exec approval decision to the session.
/// Also optionally applies an execpolicy amendment.
pub async fn exec_approval(
    sess: &Arc<Session>,
    approval_id: String,
    turn_id: Option<String>,
    decision: ReviewDecision,
) {
    let event_turn_id = turn_id.unwrap_or_else(|| approval_id.clone());
    if let ReviewDecision::ApprovedExecpolicyAmendment {
        proposed_execpolicy_amendment,
    } = &decision
    {
        match sess
            .persist_execpolicy_amendment(proposed_execpolicy_amendment)
            .await
        {
            Ok(()) => {
                sess.record_execpolicy_amendment_message(
                    &event_turn_id,
                    proposed_execpolicy_amendment,
=======
impl Session {
    /// Claim the prompt before applying a decision that can outlive the requesting execution.
    pub(crate) async fn resolve_exec_approval(
        self: &Arc<Self>,
        approval_id: String,
        turn_id: Option<String>,
        decision: ReviewDecision,
    ) {
        let Some((pending, active_turn_id)) = self
            .take_pending_approval(&approval_id, turn_id.as_deref())
            .await
        else {
            return;
        };
        let decision = pending.validate(decision);
        if decision == ReviewDecision::Abort {
            let turn_id = if pending.kind == ExecApprovalKind::HumanCommand {
                Some(&pending.turn_id)
            } else {
                active_turn_id.as_ref()
            };
            if let Some(turn_id) = turn_id {
                self.abort_turn_if_active(
                    turn_id,
                    TurnAbortReason::Interrupted,
                    /*error*/ None,
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
                )
                .await;
            }
<<<<<<< HEAD
            Some(Ok(Some(crate::exec_policy::AmendmentScope::Session))) => {
                let message = "This command contains a credential, so the approval applies to \
                               this session only and was not saved to your rules."
                    .to_string();
                let warning = EventMsg::Warning(WarningEvent { message });
                sess.send_event_raw(Event {
                    id: event_turn_id.clone(),
                    msg: warning,
                })
                .await;
            }
            Some(Err(err)) => {
                let message = format!("Failed to apply execpolicy amendment: {err}");
                tracing::warn!("{message}");
                let warning = EventMsg::Warning(WarningEvent { message });
                sess.send_event_raw(Event {
                    id: event_turn_id.clone(),
                    msg: warning,
                })
                .await;
            }
||||||| 413492cd6c
            Err(err) => {
                let message = format!("Failed to apply execpolicy amendment: {err}");
                tracing::warn!("{message}");
                let warning = EventMsg::Warning(WarningEvent { message });
                sess.send_event_raw(Event {
                    id: event_turn_id.clone(),
                    msg: warning,
                })
                .await;
            }
=======
        } else if let ReviewDecision::ApprovedExecpolicyAmendment {
            proposed_execpolicy_amendment,
        } = &decision
            && let Err(err) = self
                .persist_execpolicy_amendment(proposed_execpolicy_amendment)
                .await
        {
            let message = format!("Failed to apply execpolicy amendment: {err}");
            tracing::warn!("{message}");
            self.send_event_raw(Event {
                id: turn_id.unwrap_or(approval_id),
                msg: EventMsg::Warning(WarningEvent { message }),
            })
            .await;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        }
<<<<<<< HEAD
    }
    let decision = if stale_amendment {
        ReviewDecision::Approved
    } else {
        decision
    };
    match decision {
        ReviewDecision::Abort => {
            sess.interrupt_task().await;
        }
        other => sess.notify_approval(&approval_id, other).await,
||||||| 413492cd6c
    }
    match decision {
        ReviewDecision::Abort => {
            sess.interrupt_task().await;
        }
        other => sess.notify_approval(&approval_id, other).await,
=======
        let _ = pending.sender.send(decision);
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
    }
}

pub async fn patch_approval(sess: &Arc<Session>, id: String, decision: ReviewDecision) {
    match decision {
        ReviewDecision::Abort => {
            sess.interrupt_task().await;
        }
        other => sess.notify_approval(&id, other).await,
    }
}

pub async fn request_user_input_response(
    sess: &Arc<Session>,
    id: String,
    response: RequestUserInputResponse,
) {
    sess.notify_user_input_response(&id, response).await;
}

pub async fn request_permissions_response(
    sess: &Arc<Session>,
    id: String,
    response: RequestPermissionsResponse,
) {
    sess.notify_request_permissions_response(&id, response)
        .await;
}

pub async fn dynamic_tool_response(sess: &Arc<Session>, id: String, response: DynamicToolResponse) {
    sess.notify_dynamic_tool_response(&id, response).await;
}

pub fn refresh_mcp_servers(sess: &Session) {
    sess.services.mcp_runtime.reconnect_on_next_refresh();
    sess.request_mcp_runtime_refresh();
}

pub async fn reload_user_config(sess: &Arc<Session>) {
    Box::pin(sess.reload_user_config_layer()).await;
}

pub async fn compact(sess: &Arc<Session>, sub_id: String) {
    // Stop the old turn before the compact task picks up the next turn's environments.
    sess.abort_all_tasks(TurnAbortReason::Replaced).await;
    let turn_context = sess
        .new_turn_with_default_settings(sub_id, Default::default())
        .await;

    sess.spawn_task(turn_context, Vec::new(), CompactTask).await;
}

pub(super) async fn persist_thread_memory_mode_update(
    sess: &Arc<Session>,
    mode: ThreadMemoryMode,
) -> anyhow::Result<()> {
    let live_thread = sess.live_thread_for_persistence("update thread memory mode")?;
    live_thread.persist(PersistContext::Standard).await?;
    live_thread.flush().await?;
    live_thread
        .update_memory_mode(mode, /*include_archived*/ false)
        .await?;
    live_thread.flush().await?;
    Ok(())
}

/// Persists thread-level memory mode metadata for the active session.
///
/// This does not involve the model and only affects whether the thread is
/// eligible for future memory generation.
pub async fn set_thread_memory_mode(sess: &Arc<Session>, sub_id: String, mode: ThreadMemoryMode) {
    if let Err(err) = persist_thread_memory_mode_update(sess, mode).await {
        warn!("Failed to persist thread memory mode update to rollout: {err}");
        let event = Event {
            id: sub_id,
            msg: EventMsg::Error(ErrorEvent {
                misalignment: None,
                message: err.to_string(),
                codex_error_info: Some(CodexErrorInfo::Other),
            }),
        };
        sess.send_event_raw(event).await;
    }
}

pub(super) async fn shutdown_session_runtime(sess: &Arc<Session>) {
    let startup_prewarm = {
        let mut state = sess.state.lock().await;
        // Stop admission and take the current warmup together so resume cannot replace it.
        state.shutting_down = true;
        state.take_session_startup_prewarm()
    };
    if let Some(startup_prewarm) = startup_prewarm {
        startup_prewarm.abort().await;
    }
    let _ = sess.conversation.shutdown().await;
    sess.abort_all_tasks(TurnAbortReason::Interrupted).await;
    super::async_compaction::shutdown(sess).await;
    let shell_snapshot_prewarm = sess.state.lock().await.shell_snapshot_prewarm.take();
    if let Some(shell_snapshot_prewarm) = shell_snapshot_prewarm {
        shell_snapshot_prewarm.abort();
        let _ = shell_snapshot_prewarm.await;
    }
    sess.hooks().shutdown().await;
    sess.async_hook_results.close();
    while sess.async_hook_results.try_recv().is_ok() {}
    sess.services
        .unified_exec_manager
        .terminate_all_processes()
        .await;
    if let Err(err) = sess.services.code_mode_service.shutdown().await {
        sess.services.local_agent_runtime.record_shutdown_failure(
            AgentTreeShutdownFailure::operation_failed(
                "session_shutdown",
                "code_mode",
                Some(sess.thread_id),
                "code_mode_error",
            ),
        );
        warn!("failed to shutdown code mode session: {err}");
    }
    sess.stop_mcp_prewarm_worker().await;
    {
        let _refresh = sess.mcp_refresh.acquire().await;
        sess.mcp_refresh.close();
        sess.services.mcp_runtime.shutdown().await;
    }
<<<<<<< HEAD
    sess.guardian_review_session.shutdown().await;
    // PF-23-S03: run end revokes this session's brokered credentials.
    crate::security::transition::RevocationSink::revoke(sess.as_ref());
||||||| 413492cd6c
    sess.guardian_review_session.shutdown().await;
=======

    sess.drain_code_mode_messages().await;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a

    crate::hook_runtime::run_session_end_hooks(sess).await;
    emit_thread_stop_lifecycle(sess).await;
}

async fn emit_thread_stop_lifecycle(sess: &Session) {
    for contributor in sess.services.extensions.thread_lifecycle_contributors() {
        contributor
            .on_thread_stop(codex_extension_api::ThreadStopInput {
                session_store: &sess.services.session_extension_data,
                thread_store: &sess.services.thread_extension_data,
            })
            .await;
    }
}

pub async fn shutdown(sess: &Arc<Session>, sub_id: String) -> bool {
    shutdown_session_runtime(sess).await;
    info!("Shutting down Codex instance");
    let history = sess.clone_history().await;
    let turn_count = history
        .raw_items()
        .filter(|item| is_user_turn_boundary(item))
        .count();
    sess.services.session_telemetry.counter(
        "codex.conversation.turn.count",
        i64::try_from(turn_count).unwrap_or(0),
        &[],
    );

    // Gracefully flush and shutdown thread persistence on session end so tests
    // that inspect durable state do not race with the background writer.
    if let Some(live_thread) = sess.live_thread()
        && let Err(e) = live_thread.shutdown().await
    {
        sess.services.local_agent_runtime.record_shutdown_failure(
            AgentTreeShutdownFailure::operation_failed(
                "session_shutdown",
                "close_persistence",
                Some(sess.thread_id),
                thread_store_error_kind(&e),
            ),
        );
        warn!("failed to shutdown thread persistence: {e}");
        let event = Event {
            id: sub_id.clone(),
            msg: EventMsg::Error(ErrorEvent {
                misalignment: None,
                message: "Failed to shutdown thread persistence".to_string(),
                codex_error_info: Some(CodexErrorInfo::Other),
            }),
        };
        sess.send_event_raw(event).await;
    }
    // #872: nothing writes this thread's items after its shutdown.
    crate::home_access::note_thread_ended(sess.thread_id());

    let event = Event {
        id: sub_id,
        msg: EventMsg::ShutdownComplete,
    };
    sess.services
        .rollout_thread_trace
        .record_protocol_event(&event.msg);
    sess.services
        .rollout_thread_trace
        .record_ended(codex_rollout_trace::RolloutStatus::Completed);
    sess.deliver_event_raw(event).await;
    true
}

pub async fn review(
    sess: &Arc<Session>,
    config: &Arc<Config>,
    sub_id: String,
    review_request: ReviewRequest,
) {
    let turn_context = sess
        .new_turn_with_default_settings(sub_id.clone(), Default::default())
        .await;
    sess.maybe_emit_model_warnings_for_turn(turn_context.as_ref())
        .await;
    #[allow(deprecated)]
    match resolve_review_request(review_request, &turn_context.cwd) {
        Ok(resolved) => {
            spawn_review_thread(
                Arc::clone(sess),
                Arc::clone(config),
                turn_context.clone(),
                sub_id,
                resolved,
            )
            .await;
        }
        Err(err) => {
            let event = Event {
                id: sub_id,
                msg: EventMsg::Error(ErrorEvent {
                    misalignment: None,
                    message: err.to_string(),
                    codex_error_info: Some(CodexErrorInfo::Other),
                }),
            };
            sess.send_event(&turn_context, event.msg).await;
        }
    }
}

pub(super) async fn submission_loop(
    sess: Arc<Session>,
    config: Arc<Config>,
    rx_sub: Receiver<Submission>,
) {
    // Session shutdown and tree shutdown both use the existing teardown handler.
    let mut shutdown_received = false;
<<<<<<< HEAD
    loop {
        let sub = tokio::select! {
            sub = rx_sub.recv() => match sub {
                Ok(sub) => sub,
                Err(_) => break,
            },
            (input, final_output_json_schema) = sess.input_queue.next_interrupted_deferred_input() => {
                let (op, client_user_message_id) = match input {
                    TurnInput::UserInput { content, client_id } => (Op::UserInput {
                        items: content,
                        final_output_json_schema,
                        responsesapi_client_metadata: None,
                        additional_context: Default::default(),
                        thread_settings: Default::default(),
                    }, client_id),
                    TurnInput::ResponseItem(item) => {
                        if let Err(items) = sess.inject_extension_if_running(vec![item]).await {
                            let context = sess.new_default_turn().await;
                            sess.record_conversation_items(&context, &items).await;
                        }
                        continue;
                    }
                    TurnInput::InterAgentCommunication(communication) => (
                        Op::InterAgentCommunication { communication }, None,
                    ),
                };
                Submission {
                    id: uuid::Uuid::new_v4().to_string(),
                    op,
                    parent_turn_id: None,
                    client_user_message_id,
                    trace: None,
                }
            }
        };
        debug!(?sub, "Submission");
||||||| 413492cd6c
    while let Ok(sub) = rx_sub.recv().await {
        debug!(?sub, "Submission");
=======
    let mut mailbox = sess.input_queue.mailbox_updates();
    if let Some(updates) = &mut mailbox {
        updates.mark_changed();
    }
    loop {
        let sub = tokio::select! {
            biased;
            _ = sess.services.local_agent_runtime.shutdown.cancelled() => {
                shutdown_received = shutdown(&sess, super::new_submission_id()).await;
                break;
            }
            sub = rx_sub.recv() => match sub {
                Ok(sub) => sub,
                Err(_) => break,
            },
            update = async {
                match &mut mailbox {
                    Some(updates) => updates.changed().await.map(|()| *updates.borrow_and_update()),
                    None => std::future::pending().await,
                }
            } => {
                match update {
                    Ok(true) => {
                        sess.input_queue.notify_mailbox();
                        if sess.features.enabled(Feature::WakeOnAgentMessages)
                            || sess.has_outstanding_durable_sleep()
                        {
                            sess.maybe_start_turn_for_pending_work().await;
                        }
                    }
                    Ok(false) => {}
                    Err(_) => mailbox = None,
                }
                continue;
            }
        };
        debug!(
            thread_id = %sess.thread_id(),
            submission_id = %sub.id,
            operation = sub.op.kind(),
            parent_turn_id = ?sub.parent_turn_id,
            root_turn_id = ?sub.root_turn_id,
            op = ?sub.op,
            "Submission"
        );
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        let dispatch_span = submission_dispatch_span(&sub);
        let should_exit = async {
            match sub.op {
                Op::Interrupt => {
                    interrupt(&sess).await;
                    false
                }
                Op::InterruptIfNoPendingInput { turn_id, reply } => {
                    sess.interrupt_turn_if_no_pending_input(&turn_id, reply)
                        .await;
                    false
                }
                Op::CleanBackgroundTerminals => {
                    clean_background_terminals(&sess).await;
                    false
                }
                Op::RealtimeConversationStart(params) => {
                    if let Err(err) =
                        handle_realtime_conversation_start(&sess, sub.id.clone(), params).await
                    {
                        sess.send_event_raw(Event {
                            id: sub.id.clone(),
                            msg: EventMsg::Error(ErrorEvent {
                                misalignment: None,
                                message: err.to_string(),
                                codex_error_info: Some(CodexErrorInfo::Other),
                            }),
                        })
                        .await;
                    }
                    false
                }
                Op::RealtimeConversationAttach { params, reply } => {
                    let result =
                        crate::realtime_conversation::handle_attach(&sess, sub.id, params).await;
                    let _ = reply.send(result);
                    false
                }
                Op::RealtimeConversationDetach {
                    realtime_session_id,
                    reply,
                } => {
                    crate::realtime_conversation::handle_detach(
                        &sess,
                        Some(&realtime_session_id),
                        &sub.id,
                    )
                    .await;
                    let _ = reply.send(Ok(()));
                    false
                }
                Op::RealtimeConversationAudio(params) => {
                    handle_realtime_conversation_audio(&sess, sub.id.clone(), params).await;
                    false
                }
                Op::RealtimeConversationText(params) => {
                    handle_realtime_conversation_text(&sess, sub.id.clone(), params).await;
                    false
                }
                Op::RealtimeConversationSpeech(params) => {
                    handle_realtime_conversation_speech(&sess, sub.id.clone(), params).await;
                    false
                }
                Op::RealtimeConversationClose => {
                    handle_realtime_conversation_close(&sess, sub.id.clone()).await;
                    false
                }
                Op::RealtimeConversationListVoices => {
                    realtime_conversation_list_voices(&sess, sub.id.clone()).await;
                    false
                }
                Op::TurnInput {
                    request,
                    mode,
                    reply,
                } => {
                    let request = WithTurnExtensionData {
                        request: *request,
                        turn_extension_init: sub.turn_extension_init,
                    };
                    let result = turn_input::handle(&sess, request, mode, sub.id.clone()).await;
                    let _ = reply.send(result);
                    false
                }
                Op::RecoverTurn {
                    thread_settings,
                    start_options,
                    reply,
                } => {
                    let result = turn_input::handle_recovery(
                        &sess,
                        WithTurnExtensionData {
                            request: thread_settings,
                            turn_extension_init: sub.turn_extension_init,
                        },
                        start_options,
                        sub.id.clone(),
                    )
                    .await;
                    let _ = reply.send(result);
                    false
                }
                Op::SuspendTurnAndShutdown { reply } => {
                    let result =
                        super::turn_suspension::suspend_turn_and_shutdown(&sess, sub.id.clone())
                            .await;
                    // Exit only after history is durable and its writer has closed; an error
                    // must leave responsibility for the thread with the current worker.
                    let should_exit = matches!(
                        &result,
                        Ok(codex_protocol::turn_input::SuspendTurnOutcome::Suspended { .. })
                    );
                    let _ = reply.send(result);
                    should_exit
                }
                Op::ThreadSettings {
                    thread_settings,
                    reply,
                } => {
                    let _settings_guard = thread_settings::acquire_persistence_lock(&sess).await;
                    let thread_settings = WithTurnExtensionData {
                        request: thread_settings,
                        turn_extension_init: sub.turn_extension_init,
                    };
                    match thread_settings::update(&sess, thread_settings).await {
                        Ok(snapshot) => {
                            // Reply first: the caller may hold a lock its event consumer needs.
                            if let Some(reply) = reply {
                                let _ = reply.send(Ok(()));
                            }
                            thread_settings::emit_applied(&sess, sub.id.clone(), snapshot).await;
                        }
                        Err(error) => {
                            let message = format!("invalid thread settings override: {error}");
                            if let Some(reply) = reply {
                                let _ = reply.send(Err(CodexErr::InvalidRequest(message)));
                            } else {
                                sess.send_event_raw(Event {
                                    id: sub.id.clone(),
                                    msg: EventMsg::Error(ErrorEvent {
                                        misalignment: None,
                                        message,
                                        codex_error_info: Some(CodexErrorInfo::BadRequest),
                                    }),
                                })
                                .await;
                            }
                        }
                    }
                    false
                }
                Op::ReplaceDynamicTools { tools, reply } => {
                    let _ = reply.send(sess.replace_dynamic_tools(tools).await);
                    false
                }
                Op::TurnSettings {
                    turn_id,
                    update,
                    reply,
                } => {
                    let outcome = sess.apply_turn_settings(&turn_id, update).await;
                    let _ = reply.send(outcome);
                    false
                }
                Op::InterAgentCommunication {
                    communication,
                    start_options,
                } => {
                    let communication =
                        crate::PendingAgentMessage::new(communication, sub.message_delivery);
                    inter_agent_communication(&sess, sub.id.clone(), communication, start_options)
                        .await;
                    false
                }
                Op::WakePendingWork => {
                    sess.maybe_start_turn_for_pending_work_with_sub_id(sub.id.clone())
                        .await;
                    false
                }
                Op::ExecApproval {
                    id: approval_id,
                    turn_id,
                    decision,
                } => {
                    sess.resolve_exec_approval(approval_id, turn_id, decision)
                        .await;
                    false
                }
                Op::PatchApproval { id, decision } => {
                    patch_approval(&sess, id, decision).await;
                    false
                }
                Op::UserInputAnswer { id, response } => {
                    request_user_input_response(&sess, id, response).await;
                    false
                }
                Op::RequestPermissionsResponse { id, response } => {
                    request_permissions_response(&sess, id, response).await;
                    false
                }
                Op::DynamicToolResponse { id, response } => {
                    dynamic_tool_response(&sess, id, response).await;
                    false
                }
                Op::RefreshMcpServers => {
                    refresh_mcp_servers(&sess);
                    false
                }
                Op::ReloadUserConfig => {
                    reload_user_config(&sess).await;
                    false
                }
                Op::Compact => {
                    compact(&sess, sub.id.clone()).await;
                    false
                }
                Op::SetThreadMemoryMode { mode } => {
                    set_thread_memory_mode(&sess, sub.id.clone(), mode).await;
                    false
                }
                Op::RunUserShellCommand {
                    command,
                    timeout_ms,
                } => {
                    run_user_shell_command(&sess, sub.id.clone(), command, timeout_ms).await;
                    false
                }
                Op::ResolveElicitation {
                    server_name,
                    request_id,
                    decision,
                    content,
                    meta,
                } => {
                    resolve_elicitation(&sess, server_name, request_id, decision, content, meta)
                        .await;
                    false
                }
                Op::Shutdown => shutdown(&sess, sub.id.clone()).await,
                Op::Review { review_request } => {
                    review(&sess, &config, sub.id.clone(), review_request).await;
                    false
                }
                Op::ApproveGuardianDeniedAction { event } => {
                    approve_guardian_denied_action(&sess, event).await;
                    false
                }
                _ => false, // Ignore unknown ops; enum is non_exhaustive to allow extensions.
            }
        }
        .instrument(dispatch_span)
        .await;
        drop(sub.residency_guard);
        if should_exit {
            shutdown_received = true;
            break;
        }
    }
    // If the submission loop exits because the channel closed without an
    // explicit shutdown op, still run session teardown.
    if !shutdown_received {
        shutdown_session_runtime(&sess).await;
        if let Some(live_thread) = sess.live_thread()
            && let Err(err) = live_thread.shutdown().await
        {
            sess.services.local_agent_runtime.record_shutdown_failure(
                AgentTreeShutdownFailure::operation_failed(
                    "session_shutdown",
                    "close_persistence_after_channel_closed",
                    Some(sess.thread_id),
                    thread_store_error_kind(&err),
                ),
            );
            warn!("failed to shutdown thread persistence after submission channel closed: {err}");
        }
<<<<<<< HEAD
        crate::home_access::note_thread_ended(sess.thread_id());
||||||| 413492cd6c
=======
        sess.services
            .rollout_thread_trace
            .record_ended(codex_rollout_trace::RolloutStatus::Aborted);
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
    }
    debug!("Agent loop exited");
}

async fn approve_guardian_denied_action(sess: &Arc<Session>, event: GuardianAssessmentEvent) {
    if event.status != GuardianAssessmentStatus::Denied {
        warn!(
            review_id = event.id.as_str(),
            "ignoring approval for non-denied Guardian assessment"
        );
        return;
    }

    let approved_action = serde_json::json!({
        "action": &event.action,
        "outcome": "allowed",
    });
    let approved_action_json = match serde_json::to_string_pretty(&approved_action) {
        Ok(approved_action_json) => approved_action_json,
        Err(error) => {
            warn!(%error, review_id = event.id.as_str(), "failed to serialize approved Guardian action");
            return;
        }
    };
    let items = vec![ContextualUserFragment::into(GuardianApprovedAction::new(
        approved_action_json,
    ))];

    sess.inject_no_new_turn(items, /*current_turn_context*/ None)
        .await;
}

pub(super) fn submission_dispatch_span(sub: &Submission) -> tracing::Span {
    let op_name = sub.op.kind();
    let span_name = format!("op.dispatch.{op_name}");
    let dispatch_span = match &sub.op {
        Op::RealtimeConversationAudio(_) => {
            debug_span!(
                "submission_dispatch",
                otel.name = span_name.as_str(),
                submission.id = sub.id.as_str(),
                codex.op = op_name
            )
        }
        _ => info_span!(
            "submission_dispatch",
            otel.name = span_name.as_str(),
            submission.id = sub.id.as_str(),
            codex.op = op_name
        ),
    };
    if let Some(trace) = sub.trace.as_ref()
        && !set_parent_from_w3c_trace_context(&dispatch_span, trace)
    {
        warn!(
            submission.id = sub.id.as_str(),
            "ignoring invalid submission trace carrier"
        );
    }
    dispatch_span
}
