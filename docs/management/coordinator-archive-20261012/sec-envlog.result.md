Both PRs are merged into main, and issue #179 is closed. Merging #183 closed it automatically, and I added a comment with the PR and demo links.

## What shipped
- **#183, "Stop logging exec environment values (#179)"**: merged as e192305d5c, after all 35 checks passed and 9 were skipped.
  - **The leak:** the shell spawn function in `core/src/spawn.rs` wrote every environment variable, values included, into the trace log. Anything that went through it was affected: `!` commands and approved model shell calls. It now logs the variable names only.
  - **Worse than the issue says:** the logs database and the `/feedback` buffer always record trace-level lines, whatever `RUST_LOG` is set to. So keys were being written on every run, not only when someone turned trace on.
  - **Other fixes in the same PR:**
    - The app-server `command/exec` log now records only command, cwd and env names.
    - The app-server no longer logs what a client sends back; replies to the auth-token refresh request contain an access token. It logs the request id or method name instead.
    - The `mcp-server` `tools/call` log records only the tool name.
    - The demo recorder now escapes `;` for tmux. Without that, tmux silently dropped it from typed commands, and in one run this put the fake key on screen.
  - **Merge note:** main picked up the same spawn fix from the broker lane (1a4370992f) while #183 was open. I kept my version, which only builds the name list when trace logging is on, and re-ran the tests after resolving the conflict.
- **#184, "Say Corbanu in approval history and Full Access text"**: merged as af89d1e99d with all checks green. Something merged it before I did; it was the exact commit I pushed. It changes:
  - every approval history line ("You approved Corbanu to run …", plus the network, denied, canceled, timed-out and patch versions);
  - the auto-reviewer lines;
  - the Full Access text ("When Corbanu runs with full access…").

  No open PRs touched these files when I merged main into the branch, so I didn't collide with sec-tui.

## Gate evidence
- **New regression test** `user_shell_cmd_env_values_never_reach_trace_logs`: it sets up the same logging as the TUI and runs a `!` command with a fake key in the environment. It checks the key never appears in the log file, the feedback buffer or the database files. It fails on the old code; I checked that with the fix removed.
- **Focused `just test` runs:**

  | Area | Passed |
  | --- | --- |
  | core `user_shell_cmd` | 9/9 (re-run after merging main) |
  | feedback and mcp-server | 25/25 |
  | app-server `command_exec` and `message_processor` | 38/38 |
  | demo-script unit tests | 10/10 |
  | tui approval, history, guardian and permissions tests | 335/335 (re-run after merging main) |

- **Full tui suite:** it has failures unrelated to this work. The IPC socket tests fail locally because the socket path is too long, and `command_popup_default_items` has an out-of-date snapshot on main.
- **Demo videos:**
  - [#179 demo](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-179-shell-env-not-logged-76b500f104e2-2026-10-05.mp4), 32 s, trace on, fake `DEMO_SENTINEL_API_KEY`. The `!` command sees the key, the log shows only its name, and the final screen reads "total matches: 0" across `codex-tui.log` and the logs database. The recorder's own scan found neither the fake key nor the real Z.AI key.
  - **Approved model shell calls:** I re-recorded the existing approval demo on both builds. The build without the fix leaked the real Z.AI key into the log and database (the recorder caught it and blanked it out); the fixed build was clean. Details are in a comment on #183.
  - **#184:** [approval history line](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/corbanu-strings-approval-outside-write-98d00a3a45f0-2026-10-05.mp4) and [Full Access popup](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/corbanu-strings-permissions-full-access-98d00a3a45f0-2026-10-05.mp4).
- **Reviews:** an independent Opus 5.5 High review approved each PR. For #183 I fixed its main findings before merging: the app-server reply logging, a weak check in the test, missing feedback-buffer coverage, and a demo that only checked the log file. A GLM tmux functional run isn't part of the bug-fix gate, so I didn't do one.

## Still open
- **Keys already on disk (needs your call, Travis):** existing users' logs databases still hold keys written before this fix. Options:
  - a one-time cleanup that deletes the affected log lines and compacts the database;
  - telling users to rotate their keys;
  - both.

  I recommend both.
- **Smaller logging follow-ups:**
  - response headers are logged at debug and can include `set-cookie`;
  - the raw response body is logged when a token refresh fails;
  - shell snapshots save `export -p` output to disk, which can include exported keys.
- **Remaining "Codex" strings:** the `/permissions` hint in the session header, the Windows sandbox prompts, and "Codex can now safely edit files … in your computer" in `event_dispatch.rs` (also "in" should be "on").