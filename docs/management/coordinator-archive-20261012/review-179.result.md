VERDICT: APPROVE

The commit's fix is correct, and I found no other place that logs exec or spawn environment values. I'm approving the commit as scoped to #179. The review did turn up one separate, older credential leak (finding 1) that is serious enough to fix before release. I didn't run the tests because the sandbox is read-only. Everything below comes from reading the code.

**1. The fix and the test are correct**
- `core/src/spawn.rs:63-67` now logs only the sorted env names. The old `{env:?}` printed the `HashMap` as `"NAME": "value"`, so the test's `!log_text.contains(SENTINEL_VALUE)` check and the byte scan over the DB files would both fail on the old code.
- The test uses the same file layer as the TUI (`tui/src/lib.rs:1330-1338`), the real `log_db::start` and `default_filter()`. It sets the sentinel through `shell_environment_policy.set` and checks that the child actually received it. It also checks that `spawn_child_async` reached both the log file and the DB rows. That proves the scoped subscriber captured the spawn, even though `set_default` only applies to the current thread.
- The byte scan covers the whole state directory, including WAL/SHM files.
- The app-server change compiles against the type: `env` there is `HashMap<String, Option<String>>` and `.keys()` works on it.

**2. Search coverage**
- **No problems found in:** the user shell, the unified exec and shell runtimes (`unix_escalation` logs only the program, args and working directory), the PTY utilities, `exec.rs`, exec-server (every `#[instrument]` uses `skip_all`, and the process handlers have no param logging), the Linux and Windows sandboxes (Windows logs only a command preview), hooks (`skip_all`), MCP stdio launch (logs only program name and stderr), the codex-api and `core/src/client.rs` instrument spans (all `skip_all`), the `debug!(?sub)` Submission log (no `Op` variant carries env or headers), and the app-server notification trace (strum `Display` prints only the variant name).
- No other `env`, `headers` or `bearer_token` value is Debug-formatted into a log anywhere I searched.

**Findings**

1. **High (existing bug, not caused by this commit; fix before release)**: `app-server/src/message_processor.rs:750`. The `info!` log of `"<- response: {:?}"` prints the full JSON-RPC result a stdio or websocket client sends back. One of those results is the reply to `account/chatgptAuthTokens/refresh`, which contains `accessToken` (`app-server-protocol/src/protocol/v2/account.rs:294`). The standalone app-server writes logs to the log DB at TRACE and to the feedback buffer at TRACE whatever `RUST_LOG` says (`app-server/src/lib.rs:672-684`). So that access token is written to disk and included in feedback uploads. **Fix:** log only `id`, plus at most the top-level result keys. Add a sentinel test like this commit's. Lines 630, 637 and 757 (notification and error logs) should be cut down to method or id too.

2. **Medium (scope gap)**: The leak was worse than the commit message says. The log DB filter (`state/src/log_db.rs:53-55`) and the feedback buffer (`feedback/src/lib.rs:211-217`) both record TRACE no matter what `RUST_LOG` is set to. Every `!` command and shell tool call wrote env values into the logs DB, not just runs at `RUST_LOG=trace`. The feedback buffer is uploaded off the machine by `/feedback`.
   - **Fix:** correct the release note. Add a one-time cleanup that deletes `logs` rows whose message contains `spawn_child_async`, then runs `VACUUM` so deleted pages don't keep the values. Tell users to rotate their keys.
   - **Test:** add `CodexFeedback::new().logger_layer()` to the test subscriber and assert the sentinel is absent from `snapshot().bytes`. Then the comment at `user_shell_cmd.rs:581` saying it mirrors the TUI subscriber will be accurate.

3. **Low**: `core/tests/suite/user_shell_cmd.rs:632`. The positive check `log_text.contains(SENTINEL_NAME)` already passes because the name is in the command args (`printenv CORBANU_SENTINEL_API_KEY`). It doesn't prove `env_names` was logged. **Fix:** assert that a line contains both `env_names=` and `"CORBANU_SENTINEL_API_KEY"`.

4. **Low**: The `command_exec_processor.rs:103-113` change has no test. **Fix:** add an app-server test that runs `command/exec` with a sentinel env value under a TRACE log DB and asserts the value never appears.

5. **Low**: `qa/demos/specs/shell-env-not-logged.toml:41`. The final wait only matches `codex-tui\.log:0`, so the demo would pass even if the logs DB count were non-zero. **Fix:** also wait for a zero count on each `pfterminal_logs_*` path, or use `grep -c ... | awk` to assert they sum to 0.

6. **Low (existing, follow-up)**: Other places where secrets can reach the always-TRACE sinks:
   - `mcp-server/src/message_processor.rs:341,423`: `info!` logs full `tools/call` arguments, including `config` overrides that can hold provider tokens. Log only the tool name.
   - `http-client/src/client.rs:124,~256` (`debug!`) and `codex-api/src/endpoint/responses_websocket.rs:602` (`info!`) log full response headers, which can include `set-cookie`. Leave out `set-cookie` and auth-related headers.
   - `login/src/auth/manager.rs:1943`: `error!` logs the raw response body when a token refresh fails. Log only the error code from `classify_refresh_token_failure`.

7. **Info (separate data-at-rest issue, not logging)**: `core/src/shell_snapshot.rs:373-394` saves `export -p` output to disk with only `PWD` and `OLDPWD` excluded, so shell-exported API keys sit in snapshot files. Track it separately.

8. **No issue**: the `scripts/demo_video.py` `tmux_literal` change is correct. tmux treats `\;` as a literal `;`, and the unit test covers it.