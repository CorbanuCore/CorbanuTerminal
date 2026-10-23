//! #616: a panic on a runtime worker must not leave `exec` waiting forever.
//!
//! Core finishes a turn whose task panicked with an error, but a panic in any
//! other task the turn depends on can still leave it without an end. Exec
//! records every panic in the process; once one has happened during the turn
//! and the turn then makes no progress (no event) for [`PANIC_STALL_TIMEOUT`],
//! exec stops the turn and exits non-zero instead of hanging. A panic that was
//! caught and handled does count, so a turn that is also silent for the whole
//! timeout after it ends; that trade favours never hanging.

use std::sync::Once;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::time::Duration;

use tokio::sync::Notify;

/// How long a turn may go without any event after a panic before exec gives
/// up on it. Long enough for a slow model response, short enough that a hung
/// automation run is noticed.
pub(crate) const PANIC_STALL_TIMEOUT: Duration = Duration::from_secs(300);

pub(crate) const PANIC_STALL_MESSAGE: &str = "an internal error (a panic) stopped this run from \
     making progress; exiting. The details are in the log.";

/// Panics observed in this process.
pub(crate) struct PanicSignal {
    count: AtomicU64,
    notify: Notify,
}

impl PanicSignal {
    pub(crate) const fn new() -> Self {
        Self {
            count: AtomicU64::new(0),
            notify: Notify::const_new(),
        }
    }

    pub(crate) fn record(&self) {
        self.count.fetch_add(1, Ordering::SeqCst);
        self.notify.notify_waiters();
    }

    fn count(&self) -> u64 {
        self.count.load(Ordering::SeqCst)
    }

    async fn wait_for_panic_after(&self, baseline: u64) {
        loop {
            let notified = self.notify.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            if self.count() > baseline {
                return;
            }
            notified.await;
        }
    }
}

pub(crate) static PROCESS_PANICS: PanicSignal = PanicSignal::new();

/// Records every panic in [`PROCESS_PANICS`], keeping the existing hook's
/// report. A panic inside a Vault scoped-credential callback is recovered
/// there and neither formatted nor recorded.
pub(crate) fn install_panic_hook() {
    static INSTALL: Once = Once::new();
    INSTALL.call_once(|| {
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            if codex_vault::scoped_credential_callback_active() {
                return;
            }
            PROCESS_PANICS.record();
            previous(info);
        }));
    });
}

/// Watches a turn for panics that happen after it started.
///
/// Build a fresh [`PanicStallWatch::stalled`] future after every event: it
/// completes once a panic newer than the watch has happened and then no event
/// arrived for the whole timeout. Panics before the watch (for example one
/// caught during telemetry setup) never arm it.
pub(crate) struct PanicStallWatch {
    signal: &'static PanicSignal,
    baseline: u64,
    timeout: Duration,
}

impl PanicStallWatch {
    pub(crate) fn new(signal: &'static PanicSignal, timeout: Duration) -> Self {
        Self {
            signal,
            baseline: signal.count(),
            timeout,
        }
    }

    /// Completes `timeout` after a panic newer than this watch was noticed.
    /// Never completes without one.
    pub(crate) async fn stalled(&self) {
        self.signal.wait_for_panic_after(self.baseline).await;
        tokio::time::sleep(self.timeout).await;
    }
}

#[cfg(test)]
#[path = "panic_watch_tests.rs"]
mod tests;
