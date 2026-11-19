use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Mutex;
use tokio::sync::Notify;
// tokio's clock equals the system monotonic clock in production; using it lets
// tests drive the stopwatch with a paused, deterministic clock.
use tokio::time::Instant;
use tokio_util::sync::CancellationToken;

#[derive(Clone, Debug)]
pub struct Stopwatch {
    limit: Option<Duration>,
    inner: Arc<Mutex<StopwatchState>>,
    notify: Arc<Notify>,
}

#[derive(Debug)]
struct StopwatchState {
    elapsed: Duration,
    running_since: Option<Instant>,
    active_pauses: u32,
}

impl StopwatchState {
    /// Records one more pause. Returns true when the clock stopped.
    fn pause(&mut self) -> bool {
        self.active_pauses += 1;
        if self.active_pauses == 1
            && let Some(since) = self.running_since.take()
        {
            self.elapsed += since.elapsed();
            return true;
        }
        false
    }

    /// Lifts one pause. Returns true when the clock restarted.
    fn resume(&mut self) -> bool {
        if self.active_pauses == 0 {
            return false;
        }
        self.active_pauses -= 1;
        if self.active_pauses == 0 && self.running_since.is_none() {
            self.running_since = Some(Instant::now());
            return true;
        }
        false
    }
}

impl Stopwatch {
    pub fn new(limit: Duration) -> Self {
        Self {
            inner: Arc::new(Mutex::new(StopwatchState {
                elapsed: Duration::ZERO,
                running_since: Some(Instant::now()),
                active_pauses: 0,
            })),
            notify: Arc::new(Notify::new()),
            limit: Some(limit),
        }
    }

    pub fn unlimited() -> Self {
        Self {
            inner: Arc::new(Mutex::new(StopwatchState {
                elapsed: Duration::ZERO,
                running_since: Some(Instant::now()),
                active_pauses: 0,
            })),
            notify: Arc::new(Notify::new()),
            limit: None,
        }
    }

    pub fn cancellation_token(&self) -> CancellationToken {
        let token = CancellationToken::new();
        let Some(limit) = self.limit else {
            return token;
        };
        let cancel = token.clone();
        let inner = Arc::clone(&self.inner);
        let notify = Arc::clone(&self.notify);
        tokio::spawn(async move {
            wait_for_limit(inner, notify, limit, |_running| {}).await;
            cancel.cancel();
        });
        token
    }

    /// Runs `fut`, pausing the stopwatch while the future is pending. The clock
    /// resumes automatically when the future completes. Nested/overlapping
    /// calls are reference-counted so the stopwatch only resumes when every
    /// pause is lifted.
    pub async fn pause_for<F, T>(&self, fut: F) -> T
    where
        F: Future<Output = T>,
    {
        self.pause().await;
        let result = fut.await;
        self.resume().await;
        result
    }

    async fn pause(&self) {
        if self.inner.lock().await.pause() {
            self.notify.notify_waiters();
        }
    }

    async fn resume(&self) {
        if self.inner.lock().await.resume() {
            self.notify.notify_waiters();
        }
    }
}

/// Returns once the stopwatch has run for `limit`. `after_state_read` runs
/// right after each state read (tests use it to interleave a pause/resume).
async fn wait_for_limit(
    inner: Arc<Mutex<StopwatchState>>,
    notify: Arc<Notify>,
    limit: Duration,
    mut after_state_read: impl FnMut(bool),
) {
    loop {
        // Subscribe before reading the state: `notify_waiters` only wakes
        // `Notified` futures that already exist, so a pause/resume landing
        // between the read and the wait would otherwise be lost (a lost resume
        // stops the timeout from ever firing).
        let notified = notify.notified();
        tokio::pin!(notified);
        let (remaining, running) = {
            let guard = inner.lock().await;
            let elapsed = guard.elapsed
                + guard
                    .running_since
                    .map(|since| since.elapsed())
                    .unwrap_or_default();
            if elapsed >= limit {
                return;
            }
            (limit - elapsed, guard.running_since.is_some())
        };
        after_state_read(running);

        if !running {
            notified.await;
            continue;
        }

        // Either way, re-read the state: it alone decides whether the limit
        // has been reached.
        tokio::select! {
            _ = tokio::time::sleep(remaining) => {}
            _ = &mut notified => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Stopwatch;
    use super::wait_for_limit;
    use pretty_assertions::assert_eq;
    use std::sync::Arc;
    use tokio::time::Duration;
    use tokio::time::Instant;
    use tokio::time::sleep;
    use tokio::time::timeout;

    // All tests run on tokio's paused clock: time only advances when every
    // task is idle, so a slow or descheduled test thread cannot leak real
    // running time into the stopwatch (the cause of the old flake, where the
    // limit expired before the pauses had started).

    #[tokio::test(start_paused = true)]
    async fn cancellation_receiver_fires_after_limit() {
        let stopwatch = Stopwatch::new(Duration::from_millis(50));
        let token = stopwatch.cancellation_token();
        let start = Instant::now();
        token.cancelled().await;
        assert_eq!(start.elapsed(), Duration::from_millis(50));
    }

    #[tokio::test(start_paused = true)]
    async fn pause_prevents_timeout_until_resumed() {
        let stopwatch = Stopwatch::new(Duration::from_millis(50));
        let token = stopwatch.cancellation_token();
        let start = Instant::now();

        let pause_handle = tokio::spawn({
            let stopwatch = stopwatch.clone();
            async move {
                stopwatch
                    .pause_for(async {
                        sleep(Duration::from_millis(100)).await;
                    })
                    .await;
            }
        });

        assert!(
            timeout(Duration::from_millis(30), token.cancelled())
                .await
                .is_err()
        );

        pause_handle.await.expect("pause task should finish");

        token.cancelled().await;
        assert_eq!(start.elapsed(), Duration::from_millis(150));
    }

    #[tokio::test(start_paused = true)]
    async fn overlapping_pauses_only_resume_once() {
        let stopwatch = Stopwatch::new(Duration::from_millis(50));
        let token = stopwatch.cancellation_token();
        let start = Instant::now();

        // First pause.
        let pause1 = {
            let stopwatch = stopwatch.clone();
            tokio::spawn(async move {
                stopwatch
                    .pause_for(async {
                        sleep(Duration::from_millis(80)).await;
                    })
                    .await;
            })
        };

        // Overlapping pause that ends sooner.
        let pause2 = {
            let stopwatch = stopwatch.clone();
            tokio::spawn(async move {
                stopwatch
                    .pause_for(async {
                        sleep(Duration::from_millis(30)).await;
                    })
                    .await;
            })
        };

        // While both pauses are active, the cancellation should not fire.
        assert!(
            timeout(Duration::from_millis(40), token.cancelled())
                .await
                .is_err()
        );

        pause2.await.expect("short pause should complete");

        // Still paused because the long pause is active.
        assert!(
            timeout(Duration::from_millis(30), token.cancelled())
                .await
                .is_err()
        );

        pause1.await.expect("long pause should complete");

        // The clock restarts only when the long pause ends (80 ms), then runs
        // the full 50 ms limit.
        token.cancelled().await;
        assert_eq!(start.elapsed(), Duration::from_millis(130));
    }

    #[tokio::test(start_paused = true)]
    async fn unlimited_stopwatch_never_cancels() {
        let stopwatch = Stopwatch::unlimited();
        let token = stopwatch.cancellation_token();

        assert!(
            timeout(Duration::from_secs(3600), token.cancelled())
                .await
                .is_err()
        );
    }

    /// A resume landing between the watcher's state read and its wait must
    /// still restart the countdown; otherwise the limit never fires.
    #[tokio::test(start_paused = true)]
    async fn resume_racing_the_state_read_is_not_lost() {
        let stopwatch = Stopwatch::new(Duration::from_millis(50));
        stopwatch.pause().await;
        let start = Instant::now();

        let inner = Arc::clone(&stopwatch.inner);
        let notify = Arc::clone(&stopwatch.notify);
        let mut resumed = false;
        let watcher = wait_for_limit(
            Arc::clone(&stopwatch.inner),
            Arc::clone(&stopwatch.notify),
            Duration::from_millis(50),
            move |running| {
                if !running && !resumed {
                    resumed = true;
                    let mut state = inner.try_lock().expect("state lock is free");
                    if state.resume() {
                        notify.notify_waiters();
                    }
                }
            },
        );

        timeout(Duration::from_secs(1), watcher)
            .await
            .expect("a resume after the state read must wake the watcher");
        assert_eq!(start.elapsed(), Duration::from_millis(50));
    }

    /// A pause landing between the watcher's state read and its wait must
    /// stop the countdown; otherwise the limit fires while paused.
    #[tokio::test(start_paused = true)]
    async fn pause_racing_the_state_read_is_not_lost() {
        let stopwatch = Stopwatch::new(Duration::from_millis(50));

        let inner = Arc::clone(&stopwatch.inner);
        let notify = Arc::clone(&stopwatch.notify);
        let mut paused = false;
        let watcher = wait_for_limit(
            Arc::clone(&stopwatch.inner),
            Arc::clone(&stopwatch.notify),
            Duration::from_millis(50),
            move |running| {
                if running && !paused {
                    paused = true;
                    let mut state = inner.try_lock().expect("state lock is free");
                    if state.pause() {
                        notify.notify_waiters();
                    }
                }
            },
        );

        assert!(
            timeout(Duration::from_secs(1), watcher).await.is_err(),
            "the limit must not fire while the stopwatch is paused"
        );
    }
}
