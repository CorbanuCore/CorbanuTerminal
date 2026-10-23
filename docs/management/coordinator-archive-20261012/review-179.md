You are an independent security reviewer. Review commit e6e7ca4d09 (`git show e6e7ca4d09`) in the worktree
/Volumes/CorbanuDrive/Corbanu/worktrees/sec-179-env-logging-20261005 (base origin/main 6bb61c4ac9). Do not edit files.

Context: GitHub issue #179: a `!` user shell command logged its full environment (incl. API keys) at trace level
into codex-tui.log and the logs SQLite DB, via `trace!` in codex-rs/core/src/spawn.rs spawn_child_async.
The fix logs env names only, also changes app-server command/exec debug logging, adds a trace-level regression test
(core/tests/suite/user_shell_cmd.rs, user_shell_cmd_env_values_never_reach_trace_logs), a demo spec and a tmux ';'
escape in scripts/demo_video.py.

Check:
1. Correctness of the fix and the test (does the test actually exercise the same subscriber/log DB path as the TUI,
   and would it fail on the old code?).
2. Completeness: search codex-rs (use rg, skip target dirs) for any OTHER place that logs an exec/spawn environment
   map, env overrides, MCP server env/headers, model provider http headers / bearer tokens, auth tokens, or request
   headers at any level (trace/debug/info/warn), including Debug-formatting of structs that contain such fields
   (`{x:?}`, `?x`, `#[instrument]` without skip). Cover user shell, unified exec, shell tool exec, sandbox spawn
   (seatbelt/landlock/windows), exec-server, MCP stdio launch, hooks, app-server request logging.
3. Anything else risky in the diff.

Output: a verdict line `VERDICT: APPROVE` or `VERDICT: CHANGES REQUESTED`, then numbered findings with file:line,
severity, and a concrete suggested fix. Never print secret values.
