#380 is fixed and closed. PR #387 merged into main (merge commit `051f974722`) after all CI checks passed (38 pass, 8 skipped, none pending). The review also found a second leak, AWS session tokens, which is fixed too.

**What changed**
- **New `codex-log-guard` crate.** It wraps every logging setup and drops detailed (TRACE) output from the HTTP, websocket, TLS and AWS libraries before any `RUST_LOG` filter is applied. This holds even when a user asks for `trace` or `tungstenite=trace`. Two libraries are capped harder because they print credentials at less detailed levels: the OTLP exporter and the MCP OAuth module lose DEBUG too, and the AWS container-credentials module loses WARN.
- **Where it applies.** TUI, exec, app-server, mcp-server, the exec-server, `login`, cloud-tasks, the execve wrapper and code-mode-host. The Windows sandbox writes its own command log, so it isn't a sink for library output.
- **Second layer.** The TUI log, the `/feedback` buffer, the logs database and stderr also blank out anything shaped like a credential: auth and cookie headers, bearer tokens, `sk-` keys, JWTs, and key/token URL parameters.
- **AWS leak.** The review found that the AWS signing library logs every signed request's session token at TRACE. The logs database and `/feedback` buffer keep that level by default, so it leaked with no `RUST_LOG` set. AWS library targets are now capped as well.

**Gate evidence**
- **Tests:** about 430 focused tests pass, including the new regression tests. The core and exec websocket tests fail with the guard removed and pass with it. The new AWS test shows the token appearing without the guard and not with it.
  - The new sentinel-key tests run a websocket turn and an HTTP turn under a trace `RUST_LOG` against local mock servers. They cover the log file, feedback buffer and logs database (core), and stderr plus the whole home through the real `codex-exec` binary (exec).
  - The new HTTP tests are canaries rather than regression tests: those libraries don't log the header today.
  - One TUI tmux test fails on this machine only: `tmux_first_run_anthropic_account_selects_claude_login_after_success`. The long worktree path cuts off the status line it waits for, so it's unrelated to this change.
- **Linux clippy on RTX** (`-D warnings`, all touched crates): no warnings on all three commits.
- **Review (Opus 5.5 High, read-only):**
  - Round 1 asked for changes: the AWS leak plus gaps in the pattern-based redaction.
  - Round 2 approved with fixes: a regression I introduced in round 1, the container-credentials token, and smaller items.
  - Everything is fixed or recorded in `qa/security-levels/pf24-followups/issue-380/review/disposition.md`.
- **Functional check with the real OpenAI key:** debug build of the final code, disposable home, `CORBANU_TEST_NO_NATIVE_KEYRING=1`, `RUST_LOG=trace`.
  - One exec turn and one TUI turn in tmux, both on gpt-5.4 over the websocket. The handshake was logged without the request line.
  - A `grep -F` scan of every file in the home, with the key read from the vault helper and never printed, found 0 matches.
  - Same result on the two earlier commits. The homes were deleted.
- **Video:** none recorded; this assignment's gate didn't ask for one.
- **Cleanup:** the build target, local worktree and RTX checkout/target are removed. The local branch `fix/380-trace-header-leak` is still there: `git branch -d` refused because local main is behind; the branch is fully merged on origin.

**Still open**
- Nothing stops a future subscriber from skipping the guard. A test that scans the source wouldn't work under Bazel, so this is a follow-up.
- Custom provider header names aren't matched by name in the second layer. Their values are already hidden in our own logs (#196), and library dumps are now capped.
- The Windows sandbox command log could record credentials a user types into a command (e.g. `curl -H`). That's outside #380 and should get its own issue.