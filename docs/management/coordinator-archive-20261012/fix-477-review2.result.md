**Verdict: APPROVE.** Nothing blocks the PR. S1 is fixed except for a narrower leftover I list below, S2 is fully fixed, and the new locking can't deadlock. I only read code; I didn't build, test or run the TUI.

## S1 and S2

**S2: fixed.**
- The cancel is now the first thing every shutdown path does:
  - in-process: `in_process.rs:559` runs before `connection_closed` (:563)
  - stdio app-server: `lib.rs:1171` runs before the `forced()` branch, so forced shutdown is covered too
- `cancel_plugin_sync` (`message_processor.rs:520-524`) is synchronous and only sets the flag (`manager.rs:2673-2683`).
- The bounded wait still happens in `drain_background_tasks` (`message_processor.rs:706-713`), which calls the cancel again (`manager.rs:2689`). Calling it twice is harmless.

**S1: fixed for every point where a new step would start.**
- No fallback starts after a cancel:
  - before HTTP: `startup_sync.rs:162-164`
  - before the backup archive: `:182-183`
  - the existing check after git fails: `:146-147`
- No git is spawned once cancelled: `:756-758`.
- No cache refresh or callback runs after a cancel, and the flag is reset (`manager.rs:2627-2629`).
- Cancelled syncs log at debug (`:2653-2657`).
- `GIT_TERMINAL_PROMPT=0` is set (`:716-718`).
- The unit test now writes the pid file atomically (`startup_sync_tests.rs:894`).

## Deadlock check: no risk

- **Lock type:** `curated_repo_sync` is a `std::sync::Mutex`.
- **What `start` does under the lock** (`manager.rs:2593-2662`): it maps the callback (`Handle::try_current` doesn't block), clones values, and calls `Builder::spawn`. Nothing in that span locks the slot again.
- **Thread body:** it never touches the slot. The callbacks either run through `runtime.spawn` or, if there's no runtime, inline on the sync thread. Even if a callback eventually led back to `start`, it would be on a different thread and would only wait briefly. The thread that spawned it never joins while holding the guard.
- **Call sites:** `start` is reached only from `manager.rs:606`, which in turn is reached from `:2118` (`message_processor.rs:460`) and `account_processor.rs:240`. None of these already holds the slot lock.
- **Cancel and shutdown:** `cancel` holds the lock only for two atomic stores. In `shutdown`, the guard is a temporary in the `let … else`, so it's dropped before the `sleep().await` loop. Holding it across the await would also make the future `!Send`.
- **Poisoned lock:** `into_inner` is used at all three lock sites. Nothing in the locked spans panics in practice, because `Builder::spawn` returns an `Err` instead of panicking.
- **No race:** the flag is stored and read under the same lock. Either `start` stores its handle before `cancel` runs and then gets cancelled, or `start` sees the flag and returns before the `STARTED.swap`, so it doesn't leave `STARTED=true` behind. That closes the round-1 race between spawn and storing the handle. If the cancel lands just after the spawn, the thread catches it at `startup_sync.rs:138`.

## The shut-down flag can't block a running server

- The flag belongs to one `PluginsManager`.
- Each `MessageProcessor` builds its own `ThreadManager`, and that creates its own `PluginsManager` (`message_processor.rs:260`, `:311`; `thread_manager.rs:374`).
- Only the two shutdown tails and `drain_background_tasks` set the flag. The only other caller is a test teardown (`message_processor_tracing_tests.rs:166`).
- An account change in a live server therefore still starts a sync. After shutdown it's correctly ignored.

## Findings

**Blocking:** none.

**Should-fix (fine as a follow-up issue):**
- **S1 leftover: an HTTP or backup request that's already running isn't interrupted.**
  - The `block_on` calls at `startup_sync.rs:360`, `:368` and `:393` aren't raced against `cancel`, and there's no check between the sha fetch and the zipball download.
  - On a Mac without the developer tools, `git_binary` is `None`, so the sync goes straight to HTTP. On those machines this is the normal path, not an edge case.
  - What can still happen:
    - quitting takes up to 3 s longer while shutdown waits;
    - the sync can activate the repo and write the sha after the drain has given up;
    - a `plugins-clone-*` folder (`:429`) is left behind if the process exits mid-extract.
  - No child process outlives the app, so the core claim of #477 still holds.

**Nits:**
1. **Abort path in the in-process server.** `process_client_request` is awaited inline in the loop (`in_process.rs:497-505`). If a long request is still running at quit, the 5 s abort (`:730-731`) fires before the loop ever reaches `:559`. The sync thread holds an `Arc<PluginsManager>`, so dropping the manager doesn't cancel it either. Fix: a `Drop` on `MessageProcessor`, or a guard that calls `cancel_curated_repo_sync`.
2. **Misleading warning.** The "falling back to GitHub HTTP" `warn!` (`startup_sync.rs:156-160`) is logged before the cancel re-check at `:162`. If the cancel lands in that window, the warning describes a fallback that never happens. Move the check above the `warn!`.
3. **Still unchanged from round 1:**
   - `git_head_sha` (`:670-677`) uses `.output()`, so it can still spawn after a cancel.
   - The lock wait in `lock_curated_plugins_startup_sync` ignores `cancel`.
   - The global `CURATED_REPO_SYNC_STARTED` can stop a second manager in the same process from syncing while a cancelled sync is still finishing an HTTP request.
4. **No tests for the new behaviour:**
   - `start` after `cancel` should do nothing and store no handle;
   - a pre-cancelled `run_git_command_with_timeout` should spawn no process;
   - a cancelled HTTP or backup fallback should be skipped.
5. **Process:** the PR body still needs the product-spec heading and excerpt for this bounded fix. The changed TUI quit path also still needs its true-TUI PTY run. #494 tracks S3.