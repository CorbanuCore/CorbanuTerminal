//! D642: replay a quiet burst Enter through the ordinary question key path.
//! Redraws only flush text. A unique token prevents a delayed key from reaching
//! another question or a panel restored after navigation or a modal interruption.

use super::*;
use crate::app_event::AppEvent;
use crate::bottom_pane::paste_burst::BURST_ENTER_LOOKAHEAD;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

static NEXT_BURST_ENTER: AtomicU64 = AtomicU64::new(1);

impl AsyncQuestions {
    pub(super) fn schedule_burst_enter(&mut self) {
        if !self.composer.has_pending_burst_enter() {
            self.pending_burst_enter = None;
            return;
        }
        if self.pending_burst_enter.is_some() {
            return;
        }
        let token = NEXT_BURST_ENTER.fetch_add(1, Ordering::Relaxed);
        self.pending_burst_enter = Some(token);
        let tx = self.app_event_tx.clone();
        // Compute the deadline before spawning so scheduler latency cannot
        // restart the lookahead window.
        let deadline = tokio::time::Instant::now() + BURST_ENTER_LOOKAHEAD;
        tokio::spawn(async move {
            tokio::time::sleep_until(deadline).await;
            tx.send(AppEvent::AsyncQuestionBurstEnter { token });
        });
    }

    pub(crate) fn cancel_pending_burst_enter(&mut self) {
        self.pending_burst_enter = None;
        self.submission = None;
        self.composer.cancel_pending_burst_submission();
    }

    pub(crate) fn prepare_burst_enter_key(&mut self, token: u64) -> bool {
        if !self.expanded || self.pending_burst_enter != Some(token) {
            return false;
        }
        self.pending_burst_enter = None;
        self.composer
            .prepare_burst_enter_key(tokio::time::Instant::now().into_std())
    }
}
