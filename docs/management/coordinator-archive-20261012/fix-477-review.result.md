**Verdict: APPROVE.** Nothing blocks this PR. The main path is correct, and the exec test should fail without the fix. Three should-fix gaps remain where the claim "covers every exec/TUI/app-server exit" doesn't hold yet. I only read code; I didn't build or run anything.

## Should-fix

**S1. The HTTP and backup fallbacks can't be cancelled and can still write to the home after shutdown returns.**
- `startup_sync.rs:138-160` checks `cancel` only once, right after git fails. It doesn't check again before the HTTP sync (`:165`), between HTTP and the backup archive, or inside them.
- If git fails quickly but not because of a cancel (say git is missing, or offline DNS fails), the HTTP sync is still running at quit. It blocks for up to 30 s per request (`:344-375`).
- When that happens, `shutdown_curated_repo_sync` gives up after 3 s. The processor then keeps going through the thread-processor drain (up to 10 s) and `shutdown_threads` (up to 10 s), and the sync can download, extract, activate the repo and write the sha during that time.
- If the process exits in the middle, the `plugins-clone-*` folder from the HTTP path (`:364`) is never cleaned up.
- It also makes quitting the TUI up to 3 s slower in this offline case, which changes what users see.
- **Fix:** check `cancel` before the HTTP sync, before the backup archive, and before `prepare_curated_repo_parent_and_temp_dir`. Ideally also race each `block_on` against the flag.

**S2. In the in-process server, the cancel can be skipped completely.**
- `in_process.rs:~561-566` runs `connection_closed` first. That can wait up to 30 s for in-flight RPCs (`message_processor.rs:726`).
- Only after that does it reach `drain_background_tasks`, where the cancel flag is set.
- The whole processor task is aborted after 5 s (`in_process.rs:729`). So if the user quits the TUI with any RPC still in flight, the flag is never set and git outlives the process.
- The stdio app-server's forced shutdown has the same gap: `lib.rs:1171` skips the drain entirely.
- **Fix:** split the call into a synchronous `cancel_curated_repo_sync()` that only sets the flag, and call it first in the cleanup on all paths, including forced. Keep the bounded wait in the drain.

**S3. Some exec exits skip `client.shutdown()` entirely.**
- After the client starts (`exec/src/lib.rs:908`), several `?` returns exit early: `:945`, `:949`, `:962`, `:966`, `:979`, `:982`, `:1062`, `:1082`. A failed `thread/start` or `turn/start`, for example, never runs the drain.
- This is older code, but it contradicts "covering in-process exec". Either track it as a follow-up or wrap these paths so shutdown always runs.

## Nits

**Cancellation details**
- After a successful git sync, `refresh_curated_plugin_cache` and the effective-plugins callback still run even when cancelled (`manager.rs:2617-2635`). Skip them once `cancel` is set.
- If `cancel` is already set, `run_git_command_with_timeout` (`startup_sync.rs:742-757`) still spawns git and then kills it. Check the flag before spawning.
- `git_head_sha` uses `.output()` (`:667-674`), so it has no process group, timeout or cancel. It's a quick local `rev-parse`, so the risk is low.
- The lock wait in `lock_curated_plugins_startup_sync` (`:209-222`) blocks forever and ignores `cancel`. No child process exists at that point, so only the 3 s wait is wasted.

**Process groups**
- The `unsafe` block is sound. The child hasn't been reaped and nothing in the codebase sets SIGCHLD to SIG_IGN, so its pid and group id can't be reused before `kill(-pid)`. The rest of the pid-handling is fine:
  - The `try_from` is fine.
  - The extra `child.kill()` is harmless.
  - macOS supports `process_group` through posix_spawn.
  - Non-unix builds still compile.
- Behaviour change: git no longer gets terminal SIGINT, SIGTSTP or hangup SIGHUP. exec only installs its `ctrl_c` handler after the session is configured (`lib.rs:1018`). A Ctrl-C before that now leaves git running, where before it killed it.
- If git tries to prompt on `/dev/tty`, it now gets stopped by SIGTTIN until the 30 s timeout. I'd set `GIT_TERMINAL_PROMPT=0` in `git_command`.
- Windows still kills only `git.exe`, so helpers survive. That's pre-existing; a job object would fix it later.

**Concurrency**
- Handling a poisoned Mutex with `into_inner` is fine, and the guard is dropped before any `.await`.
- There's a small race between `spawn` and storing the handle (`manager.rs:2645`). If shutdown runs in that window, or an account change calls `maybe_start_curated_repo_sync_for_config` after shutdown, the new sync is never cancelled. A `shutting_down` flag on the manager would close it.
- The global `CURATED_REPO_SYNC_STARTED` flag is acceptable: a cancel resets it to false, and only the manager that won the swap holds a handle.

**Tests**
- Both tests check liveness with `kill(pid, 0)`, which is still true for a zombie. The orphaned helper relies on init or a subreaper to reap it. That's fine on CI VMs, launchd and the Bazel linux-sandbox, but it would hang in a container whose PID 1 doesn't reap.
- In the unit test, `echo $$ > git.pid` isn't atomic. The exec test uses `.tmp` + `mv`; do the same here.
- In the exec test's failure path, the cleanup `SIGKILL` could in theory hit a reused pid.
- `AfterFetchStarts` blocks wiremock's thread for up to 20 s. Any POST to `/responses` gets the same body, which is fine.
- PATH lookup works on both platforms: `which::which` on macOS and `Command` lookup on Linux both see the test's PATH.
- Without the fix, the exec test fails deterministically: git keeps writing for about 30 s and the clone folder remains.
- There's no unit test for `shutdown_curated_repo_sync` itself (the timeout path and the no-sync path).

**Other**
- The cancel error repeats itself ("…cancelled at shutdown: …: …cancelled at shutdown"), and every quick normal exit logs it at `warn!`. Log it at debug when cancelled.
- Changing `sync_openai_plugins_repo` to take `cancel` breaks any external callers of this public function. I found no other callers in the workspace.
- `libc` under `[target.'cfg(unix)']` is fine. It's already in `Cargo.lock` for `codex-core-plugins` as a dev-dependency, so Bazel and the `MODULE.bazel.lock` check aren't affected.
- Policy: the commit message doesn't cite a product-spec heading and excerpt for this bounded fix. Make sure the PR body has one. The TUI quit path changed (up to 3 s extra), so it also needs a true-TUI PTY check.