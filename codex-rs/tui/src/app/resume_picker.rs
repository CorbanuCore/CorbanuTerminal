//! Existing-session picker orchestration and its borrowed app input stream.

use super::*;
use std::task::Context;
use std::task::Poll;

struct ModalTuiEvents<'a> {
    receiver: &'a mut mpsc::UnboundedReceiver<TuiEvent>,
    watchdog: &'a TuiInputDrainWatchdog,
}

impl Stream for ModalTuiEvents<'_> {
    type Item = TuiEvent;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let event = self.receiver.poll_recv(cx);
        if matches!(event, Poll::Ready(Some(_))) {
            self.watchdog.note_handled();
        }
        event
    }
}

/// Borrows the app's sole terminal-event receiver for a modal without consuming later app input.
/// Progress is recorded on delivery, including draw, paste, and overlay events handled by the modal.
pub(super) fn modal_tui_events<'a>(
    receiver: &'a mut mpsc::UnboundedReceiver<TuiEvent>,
    watchdog: &'a TuiInputDrainWatchdog,
) -> impl Stream<Item = TuiEvent> + Unpin + 'a {
    ModalTuiEvents { receiver, watchdog }
}

impl App {
    pub(super) async fn handle_resume_picker_event(
        &mut self,
        tui: &mut tui::Tui,
        app_server: &mut AppServerSession,
        tui_events: &mut (impl Stream<Item = TuiEvent> + Unpin),
    ) -> Result<AppRunControl> {
        let picker_app_server = match crate::start_app_server_for_picker(
            &self.config,
            &self.app_server_target,
            self.state_db.clone(),
            self.environment_manager.clone(),
        )
        .await
        {
            Ok(app_server) => app_server,
            Err(err) => {
                self.chat_widget
                    .add_error_message(format!("Failed to start TUI session picker: {err}"));
                self.chat_widget.maybe_send_next_queued_input();
                return Ok(AppRunControl::Continue);
            }
        };
        match crate::resume_picker::run_resume_picker_from_existing_session_with_app_server(
            tui,
            &self.config,
            /*show_all*/ false,
            /*include_non_interactive*/ false,
            picker_app_server,
            tui_events,
        )
        .await?
        {
            SessionSelection::Resume(target_session) => {
                match self
                    .resume_target_session(tui, app_server, target_session)
                    .await?
                {
                    AppRunControl::Continue => {}
                    AppRunControl::Exit(reason) => {
                        return Ok(AppRunControl::Exit(reason));
                    }
                }
            }
            SessionSelection::Exit
            | SessionSelection::StartFresh
            | SessionSelection::ResumePanesOnly { .. } => {
                self.refresh_in_memory_config_from_disk_best_effort("closing the session picker")
                    .await;
            }
            SessionSelection::Fork(_) => {}
        }

        self.chat_widget.maybe_send_next_queued_input();
        // Leaving alt-screen may blank the inline viewport; force a redraw either way.
        tui.frame_requester().schedule_frame();
        Ok(AppRunControl::Continue)
    }
}

#[cfg(test)]
#[path = "resume_picker_tests.rs"]
mod tests;
