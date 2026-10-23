You are an independent, read-only code reviewer (do not edit files, do not run builds, tests or the TUI).

Repository worktree: /Volumes/CorbanuDrive/Corbanu/worktrees/fix-494-20261011 (branch fix/494-exec-shutdown, PR #497).
Review `git -C /Volumes/CorbanuDrive/Corbanu/worktrees/fix-494-20261011 diff -w origin/main...HEAD` (codex-rs/exec/src/lib.rs, exec/tests/suite/plugin_sync_shutdown.rs). Issue #494 (`gh issue view 494`): early `?` returns in run_exec_session after InProcessAppServerClient::start skipped client.shutdown(), so the app-server shutdown cleanup (since #492: cancel and wait for the plugin sync git) never ran and git could outlive exec.

Fix: everything after the client starts runs in an `async { ... Ok::<bool, anyhow::Error>(error_seen) }` block; client.shutdown() always runs after it; then `session?`, print_final_output, exit(1) on error_seen.

Check with file:line evidence and rank findings (blocking / should-fix / nit):
1. Is shutdown now reached on every exit after client start (any `return`, `std::process::exit`, panics, or other paths left — inside the block, in helpers like resolve_resume_thread_id/handle_server_request, or after it)? Any behavior change on success or error output (ordering of error printing vs shutdown; print_final_output only on success as before; exit codes)?
2. Borrowing/async semantics: the block borrows client (&mut via next_event), event_processor, request_ids; the ctrl_c task; could the shutdown hang longer on error paths (bounded by #492's 3 s)?
3. Test `exec_stops_the_plugin_sync_git_when_turn_start_fails`: is it deterministic (required MCP sh-script gate waits for the fake git fetch before answering initialize; turn/start rejects a > MAX_USER_INPUT_TEXT_CHARS prompt from stdin); portability (Linux/macOS CI, sh/sed); could it hang or leak; does it fail without the fix (PR body says it does)?
4. The PR body's TUI quit timing claim (`gh pr view 497`) — anything missing from that check?
Report findings concisely with a verdict APPROVE / REQUEST CHANGES.
