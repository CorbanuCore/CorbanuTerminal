use super::TurnInput as PendingTurnInput;
use super::session::Session;
use super::turn_context::TurnContext;
use codex_analytics::ImagePreparationMetadata;
use codex_features::Feature;
use codex_history::CodexHarnessMetadata;
use codex_history::ResponseItemEnvelope;
use codex_protocol::error::CodexErr;
use codex_protocol::error::Result as CodexResult;
use codex_protocol::models::AudioReference;
use codex_protocol::models::ContentItem;
use codex_protocol::models::ResponseItem;
use codex_protocol::openai_models::ModelInfo;

/// Uploaded files need the admission checks on typed turn input before entering history.
pub(crate) fn validate_injected_response_item(item: &ResponseItem) -> CodexResult<()> {
    if let ResponseItem::Message { content, .. } = item
        && content.iter().any(|part| {
            matches!(
                part,
                ContentItem::InputAudio {
                    audio: AudioReference::File { .. }
                }
            )
        })
    {
        return Err(CodexErr::InvalidRequest(
            "Uploaded audio files must be submitted as turn input.".to_string(),
        ));
    }
    Ok(())
}

enum InjectionSource<'a> {
    RunningTurn,
    Extension,
    Context(&'a TurnContext),
}

impl crate::codex_thread::CodexThread {
    /// Admits extension work separately from outputs produced by the running turn.
    pub async fn inject_extension_if_running(
        &self,
        input: Vec<ResponseItem>,
    ) -> Result<(), Vec<ResponseItem>> {
        self.session.inject_extension_if_running(input).await
    }
}

impl Session {
    pub async fn inject_if_running(
        &self,
        input: Vec<ResponseItem>,
    ) -> Result<(), Vec<ResponseItem>> {
        self.inject_running_items(input, InjectionSource::RunningTurn)
            .await
    }

    pub(crate) async fn inject_extension_if_running(
        &self,
        input: Vec<ResponseItem>,
    ) -> Result<(), Vec<ResponseItem>> {
        self.inject_running_items(input, InjectionSource::Extension)
            .await
    }

    /// Returns the input if there is no active turn to inject into.
    #[expect(
        clippy::await_holding_invalid_type,
        reason = "active turn checks and turn state updates must remain atomic"
    )]
<<<<<<< HEAD
    async fn inject_running_items(
||||||| 413492cd6c
    pub async fn inject_if_running(
=======
    pub(crate) async fn inject_if_running<T: Into<ResponseItemEnvelope>>(
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        &self,
<<<<<<< HEAD
        input: Vec<ResponseItem>,
        source: InjectionSource<'_>,
    ) -> Result<(), Vec<ResponseItem>> {
||||||| 413492cd6c
        input: Vec<ResponseItem>,
    ) -> Result<(), Vec<ResponseItem>> {
=======
        input: Vec<T>,
    ) -> Result<(), Vec<T>> {
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        let mut active = self.active_turn.lock().await;
        match active.as_mut().filter(|turn| turn.finishing.is_none()) {
            Some(active_turn) => {
                let state = self.state.lock().await;
                if active_turn.task.as_ref().is_some_and(|task| {
                    let needs_admission = match source {
                        InjectionSource::RunningTurn => false,
                        InjectionSource::Extension => true,
                        // The RPC supplies a fresh context. Only the captured task context
                        // proves these items belong to work already running in this turn.
                        InjectionSource::Context(context) => {
                            !std::ptr::eq(context, task.turn_context.as_ref())
                        }
                    };
                    needs_admission
                        && (task.kind != crate::state::TaskKind::Regular
                            || !Self::authorization_matches(
                                &state.session_configuration,
                                &task.turn_context,
                            ))
                }) {
                    self.input_queue
                        .defer_input_for_turn_state(
                            active_turn.turn_state.as_ref(),
                            input.into_iter().map(TurnInput::ResponseItem).collect(),
                            /*final_output_json_schema*/ None,
                        )
                        .await;
                    return Ok(());
                }
                self.input_queue
                    .extend_pending_input_and_accept_mailbox_delivery_for_turn_state(
                        active_turn.turn_state.as_ref(),
                        input
                            .into_iter()
                            .map(Into::into)
                            .map(PendingTurnInput::ResponseItem)
                            .collect(),
                    )
                    .await;
                Ok(())
            }
            None => Err(input),
        }
    }

    /// Injects hook context into the running turn atomically.
    #[expect(
        clippy::await_holding_invalid_type,
        reason = "active turn provenance and turn state updates must remain atomic"
    )]
    pub(crate) async fn inject_hook_context_if_running(
        &self,
        input: Vec<ResponseItem>,
<<<<<<< HEAD
    ) -> Result<(), TryStartTurnIfIdleError> {
        if input.is_empty() {
            return Ok(());
        }
        // #507: no automatic idle turn while the kill switch is on; turning
        // it off starts nothing either (nothing here watches for that).
        if self.services.model_client().kill_switch_on() {
            return Err(TryStartTurnIfIdleError::new(
                TryStartTurnIfIdleRejectionReason::KillSwitch,
                input,
            ));
        }
        if self.input_queue.has_trigger_turn_mailbox_items().await {
            return Err(TryStartTurnIfIdleError::new(
                TryStartTurnIfIdleRejectionReason::PendingTriggerTurn,
                input,
            ));
        }
        if self.collaboration_mode().await.mode == ModeKind::Plan {
            return Err(TryStartTurnIfIdleError::new(
                TryStartTurnIfIdleRejectionReason::PlanMode,
                input,
            ));
        }

        let turn_state = {
            let mut active_turn = self.active_turn.lock().await;
            if active_turn.is_some() {
                return Err(TryStartTurnIfIdleError::new(
                    TryStartTurnIfIdleRejectionReason::Busy,
                    input,
                ));
            }
            let active_turn = active_turn.get_or_insert_with(ActiveTurn::default);
            Arc::clone(&active_turn.turn_state)
||||||| 413492cd6c
    ) -> Result<(), TryStartTurnIfIdleError> {
        if input.is_empty() {
            return Ok(());
        }
        if self.input_queue.has_trigger_turn_mailbox_items().await {
            return Err(TryStartTurnIfIdleError::new(
                TryStartTurnIfIdleRejectionReason::PendingTriggerTurn,
                input,
            ));
        }
        if self.collaboration_mode().await.mode == ModeKind::Plan {
            return Err(TryStartTurnIfIdleError::new(
                TryStartTurnIfIdleRejectionReason::PlanMode,
                input,
            ));
        }

        let turn_state = {
            let mut active_turn = self.active_turn.lock().await;
            if active_turn.is_some() {
                return Err(TryStartTurnIfIdleError::new(
                    TryStartTurnIfIdleRejectionReason::Busy,
                    input,
                ));
            }
            let active_turn = active_turn.get_or_insert_with(ActiveTurn::default);
            Arc::clone(&active_turn.turn_state)
=======
    ) -> Result<(), Vec<ResponseItem>> {
        let mut active = self.active_turn.lock().await;
        let Some(active_turn) = active.as_mut() else {
            return Err(input);
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        };
        if active_turn.task.is_none() {
            return Err(input);
        }
        self.input_queue
            .extend_pending_input_and_accept_mailbox_delivery_for_turn_state(
                active_turn.turn_state.as_ref(),
                input
                    .into_iter()
                    .map(ResponseItemEnvelope::new)
                    .map(PendingTurnInput::ResponseItem)
                    .collect(),
            )
            .await;
        Ok(())
    }

    /// Preserves trusted client provenance while items wait for an active turn.
    #[expect(
        clippy::await_holding_invalid_type,
        reason = "active turn checks and turn state updates must remain atomic"
    )]
    pub(crate) async fn inject_client_response_items(
        &self,
        items: Vec<ResponseItem>,
        turn_context: &TurnContext,
    ) {
        let items = items
            .into_iter()
            .map(|item| self.annotate_client_response_item(item))
            .collect::<Vec<_>>();
        let mut active = self.active_turn.lock().await;
        if let Some(active_turn) = active.as_mut().filter(|turn| turn.finishing.is_none()) {
            self.input_queue
                .extend_pending_input_and_accept_mailbox_delivery_for_turn_state(
                    active_turn.turn_state.as_ref(),
                    items
                        .into_iter()
                        .map(PendingTurnInput::ResponseItem)
                        .collect(),
                )
                .await;
            return;
        }
        let finishing = active.as_ref().and_then(|turn| turn.finishing.clone());
        drop(active);
        if let Some(finishing) = finishing {
            finishing.cancelled().await;
        }
        self.record_annotated_conversation_items(turn_context, turn_context.model_info(), items)
            .await;
    }

    pub(crate) fn annotate_client_response_item(&self, item: ResponseItem) -> ResponseItemEnvelope {
        let metadata = (self.enabled(Feature::RetainClientDeveloperMessages)
            && matches!(&item, ResponseItem::Message { role, .. } if role == "developer"))
        .then_some(CodexHarnessMetadata {
            client_authored: true,
            ..Default::default()
        });

        ResponseItemEnvelope { item, metadata }
    }

    pub(crate) async fn record_annotated_conversation_items(
        &self,
        turn_context: &TurnContext,
        model_info: &ModelInfo,
        items: Vec<ResponseItemEnvelope>,
    ) {
        if items.iter().all(|item| item.metadata.is_none()) {
            let items = items
                .into_iter()
                .map(ResponseItemEnvelope::into_item)
                .collect::<Vec<_>>();
            self.record_conversation_items(turn_context, model_info, &items)
                .await;
            return;
        }

        let (annotated_items, image_preparations) = self
            .prepare_annotated_conversation_items_for_history(turn_context, model_info, items)
            .await;
        self.record_prepared_conversation_items(
            turn_context,
            model_info,
            annotated_items,
            image_preparations,
        )
        .await;
    }

    pub(crate) async fn prepare_annotated_conversation_items_for_history(
        &self,
        turn_context: &TurnContext,
        model_info: &ModelInfo,
        items: Vec<ResponseItemEnvelope>,
    ) -> (Vec<ResponseItemEnvelope>, Vec<ImagePreparationMetadata>) {
        let mut annotated_items = Vec::with_capacity(items.len());
        let mut image_preparations = Vec::new();
        for envelope in items {
            let (prepared_items, prepared_images) = self
                .prepare_conversation_items_for_history(
                    turn_context,
                    model_info,
                    std::slice::from_ref(&envelope.item),
                )
                .await;
            image_preparations.extend(prepared_images);

            let mut metadata = envelope.metadata;
            annotated_items.extend(prepared_items.into_owned().into_iter().map(|item| {
                ResponseItemEnvelope {
                    item,
                    metadata: metadata.take(),
                }
            }));
        }
        (annotated_items, image_preparations)
    }

    /// Injects items into active work, or records them without starting a turn.
    pub(crate) async fn inject_no_new_turn(
        &self,
        items: Vec<ResponseItem>,
        current_turn_context: Option<&TurnContext>,
    ) {
        let source = current_turn_context
            .map(InjectionSource::Context)
            .unwrap_or(InjectionSource::RunningTurn);
        let Err(items) = self.inject_running_items(items, source).await else {
            return;
        };
        let finishing = self
            .active_turn
            .lock()
            .await
            .as_ref()
            .and_then(|turn| turn.finishing.clone());
        if let Some(finishing) = finishing {
            finishing.cancelled().await;
        }
        let default_turn_context;
        let turn_context = match current_turn_context {
            Some(turn_context) => turn_context,
            None => {
                default_turn_context = self.new_default_turn().await;
                default_turn_context.as_ref()
            }
        };
        self.record_conversation_items(turn_context, turn_context.model_info(), &items)
            .await;
    }
}

#[cfg(test)]
#[path = "inject_tests.rs"]
mod tests;
