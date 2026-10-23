**Verdict: APPROVE.** I found no blocking or should-fix issues. I didn't build or run anything. I also couldn't fetch the PR body or the issue because `gh` has no network here.

### 1. Is shutdown reached on every exit after the client starts?
Yes, for every normal exit.
- `run_exec_session` from `lib.rs:917` to `1198` has no `return` and no `process::exit`. Every `?` now ends the block instead of the function. That covers `resolve_resume_thread_id` (`:1613`, which uses `?` and `Ok` returns only), thread/start, thread/resume, turn/start, review/start and the `session_configured_*` conversions.
- The helpers called inside the block, such as `handle_server_request`, `request_shutdown` and `maybe_backfill_turn_completed_items`, have no exit or panic calls.
- All the existing `process::exit` calls (prompt/stdin at `:2063` onward, output schema at `:1966`, the git-repo check at `:903`) run before the client starts. That was true before this change too.
- After the block, `client.shutdown()` runs at `:1202`, then `session?` at `:1205`, then `print_final_output`, then exit(1) if `error_seen`. On success, the order and exit codes are unchanged. On error, `print_final_output` still doesn't run, and the same `Err` reaches `main`, so the exit code is unchanged.
- A panic inside the block still skips shutdown. That was already true, and it's out of scope for #494. (nit)
- **Behavior change (nit, acceptable):** on error paths the error message now prints after shutdown. That shutdown includes the plugin sync's 3 s cancel wait (`CURATED_REPO_SYNC_SHUTDOWN_TIMEOUT`, `message_processor.rs:91`). Any `warn!` lines from shutdown can now appear before the error line. Shutdown on error is now as slow as on success, which is the point of the fix.

### 2. Borrowing and async behavior
- The block isn't `async move`. It borrows `client` (including `&mut` for `next_event`), `event_processor` and `request_ids`, and moves out `initial_operation` and `default_cwd`. Every borrow ends at `.await` on `:1200`, before `shutdown(self)` takes ownership of `client`. Nothing is held across that point.
- The ctrl_c task (`:1023`) can outlive the block. Its send fails harmlessly once `interrupt_rx` is dropped.
- **Nit:** if turn/start or review/start fails after the ctrl_c handler is installed, Ctrl-C is swallowed while shutdown runs. Before this change, exec returned immediately in that case. The success path already behaved this way.
- How long shutdown can take on error paths:
  - **Usual case:** the same bound as success, i.e. the sync's 3 s cancel wait plus thread shutdown.
  - **Worst case:** if the app-server is wedged, the client's timeouts allow about 90 s. That's `IN_PROCESS_SHUTDOWN_TIMEOUT` = 45 s for the reply plus 45 s for the worker join (`app-server-client/src/lib.rs:113,937,947`). The in-process side's own limits are 35 s + 5 s (`in_process.rs:99-101`). Before this change, an error exit skipped all of this. That's acceptable, but worth knowing.

### 3. The new test
- **Deterministic: yes.**
  - The plugin sync starts when the app-server starts (`message_processor.rs:454-464`), not on thread/start. So the required gate server holding thread/start can't deadlock with the sync.
  - turn/start is rejected on the server by `validate_v2_input_limit` (`turn_processor.rs:560-566`). Exec doesn't check the prompt length itself.
  - The error string includes the `turn/start: …` prefix (`lib.rs:1321-1326`).
  - If the sync never starts, the gate exits after 20 s, thread/start fails, and the `turn/start` assertion fails with a clear message rather than hanging.
- **Fails without the fix:** I agree. The old code returned `Err`, `main` returned, and the process exited without cancelling the sync. The fake git and its `sleep 30` helper would outlive exec, and `assert_stopped` would catch at least the helper.
- **Leaks and hangs:** the fake git is bounded (600 × 0.05 s), the helper is a 30 s `sleep`, `assert_stopped` sends SIGKILL to survivors, and the gate's `read` loop ends at EOF on stdin. Nothing can hang indefinitely.
- **Portability:** the test is gated with `#![cfg(unix)]`. The scripts use only POSIX `sh`, `sed -n s///p`, `$((…))` and fractional `sleep`, which work with GNU and BSD/macOS tools. `test_codex_exec` creates no `config.toml`, so writing one doesn't overwrite anything.
- **Nits:**
  - `plugin_sync_shutdown.rs:229` writes the TOML path using Rust's `{:?}` formatting. A path containing non-ASCII characters or backslashes would produce invalid TOML. CI temp paths are fine.
  - On failure, `:245` and `:247` would dump stderr/`{output:?}`, which may include the 1 MiB prompt from the config summary.
  - The gate replies `{}` to any other method. Harmless today.

### 4. The PR body's TUI quit-timing claim
I couldn't check this: GitHub was unreachable, so I couldn't read the claim itself. This diff only touches `codex-rs/exec`, so TUI quit behavior is unchanged by construction. A TUI timing check is supporting evidence, not proof of this fix.

What a complete record should include:
- An exec run that fails early with the sync in flight (the scenario this PR fixes), showing no `git`/`plugins-clone-*` left behind and how long the error exit took.
- The candidate commit.

Repository policy also wants a reasoned not-applicable decision for the code-blind functional test design: this is exec-only, non-interactive behavior.