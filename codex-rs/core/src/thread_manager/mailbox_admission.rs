//! Orders unloaded-mail admission before publication without holding the thread map.
//! Lock order is recipient gate, then thread map. Never wait for a gate with the map held.

use codex_protocol::ThreadId;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::PoisonError;
use std::sync::Weak;
use tokio::sync::Mutex as AsyncMutex;
use tokio::sync::OwnedMutexGuard;

#[derive(Default)]
pub(crate) struct MailboxAdmission {
    gates: Mutex<HashMap<ThreadId, Weak<AsyncMutex<()>>>>,
    #[cfg(test)]
    pub(crate) pause: Mutex<Option<Arc<AdmissionPause>>>,
    #[cfg(test)]
    pub(crate) publishing: tokio::sync::Notify,
}

impl MailboxAdmission {
    pub(crate) async fn lock(&self, recipient: ThreadId) -> OwnedMutexGuard<()> {
        let gate = {
            let mut gates = self.gates.lock().unwrap_or_else(PoisonError::into_inner);
            // Only in-flight admissions/publications retain gates, including queued waiters.
            gates.retain(|_, gate| gate.strong_count() > 0);
            let entry = gates.entry(recipient).or_default();
            if let Some(gate) = entry.upgrade() {
                gate
            } else {
                let gate = Arc::new(AsyncMutex::new(()));
                *entry = Arc::downgrade(&gate);
                gate
            }
        };
        gate.lock_owned().await
    }

    #[cfg(test)]
    pub(crate) async fn pause_admission(&self, stage: AdmissionStage) {
        let pause = self.pause.lock().unwrap().clone();
        if let Some(pause) = pause
            && pause.stage == stage
        {
            pause.entered.notify_one();
            pause.release.notified().await;
        }
    }
}

#[cfg(test)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum AdmissionStage {
    BeforeWrite,
    BeforeRetention,
}

#[cfg(test)]
pub(crate) struct AdmissionPause {
    pub(crate) stage: AdmissionStage,
    pub(crate) entered: tokio::sync::Notify,
    pub(crate) release: tokio::sync::Notify,
}
