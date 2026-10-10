**Verdict: approve with fixes.** I found nothing blocking. The Windows sandbox log no longer has a write path that skips redaction, and the exec-policy change is safe. The fixes below close gaps in what the patterns catch and in the documentation.

The sandbox here is read-only, so I couldn't build or run the tests. I checked the pattern results by running the 16 patterns from `redact.rs`, ported to Python's `re` on bytes, against the shapes listed below.

## Findings

**1. Windows sandbox log: complete, but multi-word values leak. Should-fix.**
- Everything that writes to the sandbox log now goes through `log_writer`, which redacts:
  - `log_note`, `log_start`, `log_success`, `log_failure` and `debug_log`;
  - the setup helper's writer (`setup_main/win.rs:408,458`);
  - the command runner, whose log dir is `.sandbox` (`elevated_impl.rs:135`).
- `setup_error.json` and `sandbox_users.json` hold no command text.
- The line-buffering writer (`windows-sandbox-rs/src/logging.rs:87-108`) behaves correctly: it splits at the last newline, redacts several lines in one pass, and writes a partial line on flush or drop. Rotation is handled inside `RollingFileAppender`.
- Nit on the writer: if the inner write fails, the drained lines are lost but `pending` already contains `buf`, so a retry duplicates data. Clear `pending` on error.
- **The gap:** `preview` (`logging.rs:29-33`) joins argv with spaces before redacting. A credential argument that itself contains spaces only loses its first word:
  - `["tool","--password=correct horse battery"]` becomes `--password=REDACTED horse battery`;
  - `-H X-Api-Key: part1 part2` keeps `part2`.
  - The `CreateProcess` debug line has the same problem with Windows-quoted `"--password=a b"`.
  - Commands wrapped in `cmd /c`, `pwsh -c` or `bash -lc` keep their quotes, which limits the impact.
  - **Fix:** redact each argv element before joining, with a whole-element rule: an element that matches `name[=:]value` gets everything after the separator redacted.

**2. Other persisted command logs: covered, but the exception list is incomplete. Should-fix (docs and small code changes).**
These are covered or clean:
- tracing sinks all use `RedactingMakeWriter`, and `log_db.rs:235` redacts;
- analytics records counts only (`analytics/src/reducer.rs:1873+`);
- the security-audit journal is secret-free by design;
- Seatbelt, bwrap and the Linux sandbox write no logs of their own.

Not covered and not documented:
- **Claude pane audit** (`tui/src/claude_panes/execution.rs:1224-1270`): `turn-NNNN.audit.json` stores a preview of each tool call (`tool_events[].preview`). For Bash it's the description, or the summarized command when there is none. It's only checked against known vault secrets, not the credential patterns. **Fix:** run each preview through `codex_log_guard::redact_credentials`, and list `panes/*/turn-*.jsonl` as a transcript exception.
- **Telegram approvals** (`telegram/src/approvals.rs:86-87`): the full command is sent to Telegram's servers, where it stays in chat history. **Fix:** redact the command or document it as an approval-channel exception.
- **`CODEX_TUI_RECORD_SESSION` recorder** (`tui/src/session_log.rs:85`): opt-in, but it records every command. Add it to the documented exceptions.
- **`/feedback` attaches the raw sandbox log** (`app-server/src/request_processors/feedback_processor.rs:367`, `tui/src/chatwidget.rs:1085`). **Fix:** pass it through `redact_credentials_bytes` when attaching. That also covers writes from older builds after the scrub marker exists.
- Nit: very old builds wrote `CODEX_HOME/sandbox_commands.rust.log`, which the scrub doesn't touch.

**3. Redaction patterns: common credential shapes still missed. Should-fix.**

Handled correctly:
- `-H 'Authorization: token x'`
- `--header=Authorization:Bearer x`
- `set "DB_PASSWORD=x"` and `export GH_TOKEN="a b"`
- `wget --http-password=`, `--ftp-password` and `--proxy-password`
- `git -c http.extraHeader="Authorization: Basic …"`
- `AKIA…` access key IDs and `AWS_SECRET_ACCESS_KEY=`
- Debug argv with `", "` separators
- PowerShell `@{Authorization="Bearer x"}`

Missed:
- **Compact JSON argv:** `["tool","--password","pw1"]` isn't redacted, because `option_separator` (`redact.rs:48`) requires whitespace after the comma. Change it to `(?:=|\\*["']?[ \t]*,[ \t]*\\*["']?|\\*["']?[ \t]+)`.
- **Env names without an underscore** before the suffix: `PGPASSWORD=x` and `MYSQL_PWD=x` (`redact.rs:22`, the pattern at about line 92 requires `_`). Also `_authToken=` (npm) and `OPENAI_KEY`/`STRIPE_SK`. Allow `[a-z0-9]*` directly before `password|passwd|token|secret`, add `pwd`, and add `npm_…` keys.
- **Attached short options:** `mysql -pSECRET`, `sshpass -p x`, `docker login -p x`. Add a rule tied to the command name (`(?:mysql|mysqldump|mariadb)\b[^\n]*?\s-p(?P<v>\S+)`, plus `sshpass -p`).
- Lower priority: `openssl -passin/-passout pass:x`, `curl --oauth2-bearer x`, space-separated `aws configure set aws_secret_access_key x`, and multi-word `bearer a b` inside quotes (only the first word is redacted).

False positives (nit; harmless but they cost diagnostics):
- Bare `token`, `secret` and `password` in `CREDENTIAL_NAMES` (`redact.rs:16`) hit ordinary messages. `"Failed to refresh token: 401 Unauthorized"` (`login/src/auth/manager.rs:2039`) loses the status code, and `"invalid token: expected …"` is damaged the same way.
- `docker run -u 1000:1000` and `glob_pat=*.rs` are also redacted.
- **Fix:** for bare names, require `=`, a quote, or a value of at least 8 characters.

UTF-8 and performance:
- Spans always start after an ASCII separator and end before an ASCII delimiter or at end of text, so no character is cut. Truncation happens after redaction and uses `take_bytes_at_char_boundary`.
- The regex crate is linear-time, so there's no catastrophic backtracking. The 16-pattern case-insensitive `RegexSet` plus captures runs on every event. That's likely microseconds per event, but add a small benchmark over a typical 200-byte line to catch regressions.

**4. Exec policy: correct. One misleading message. Should-fix.**
- The filter is applied at every place a proposal is created (`core/src/exec_policy.rs:431` and `:461`, covering requested, prompt-derived and allow-derived proposals).
- An amendment the client sends anyway skips the file write but still updates the in-memory policy (`:487-525`). No error reaches the client, and the warning doesn't include the command.
- I found no other way to persist one: network amendments store only the host, and `AcceptForSession` lives in memory.
- **The issue:** after "saving", `handlers.rs:518-530` still calls `record_execpolicy_amendment_message`, which tells the model "Approved command prefix saved" (`session/mod.rs:2598`). The TUI wording also implies the rule persists. **Fix:** have `append_amendment_and_update` report whether it persisted, and show "applies for this session only" when it didn't.
- Test gap: the auto-derived proposal path (no `prefix_rule`) is untested.

**5. Clippy enforcement: works for the main cases. Nit.**
- `disallowed_methods` sees the trait method's DefId even through the blanket impl, so `SubscriberInitExt::try_init/init` is caught.
- Builder calls (`fmt().init()`) are caught by the `SubscriberBuilder::{init,try_init}` entries.
- The `tracing::dispatcher` re-export points to the same function as `tracing_core::dispatcher::set_global_default`, so both are caught.
- Bazel uses the same config (`.bazelrc:112` points `clippy.toml` at `//codex-rs:clippy.toml`).
- The `#[traced_test]` allowances are all in test-only files.

Gaps:
- `log::set_logger`, `log::set_boxed_logger` and `env_logger::{init, Builder::init, Builder::try_init}` aren't covered. Disallow them, with an `#[allow]` in `execpolicy-legacy/src/main.rs:63`.
- `subscriber::set_default` and `dispatcher::set_default` are only used in tests today, so they're fine as is.
- The lint enforces `guard()`, which only caps noisy library targets. It doesn't force redacting writers; that's still a review convention (all current sinks comply). State this in the README.
- `#[cfg(windows)]` code is only linted by a Windows clippy job, so confirm one exists.

**6. Old-log scrub: sound, with two edge cases. Should-fix (minor).**
- Masking is in place and keeps the file length; it reads only up to the starting length and detects truncation. Two scrubs running at once (the TUI plus the in-process app-server) do identical, idempotent work.
- The real user has read/write on `.sandbox` (`setup_main/win.rs:650-657`).
- **Edge case 1:** `?` at `state/src/log_scrub.rs:244` stops the loop on the first file error. One unreadable file means the remaining logs are never scrubbed and the scrub retries on every start. **Fix:** continue past per-file errors and skip writing the marker only if any file failed.
- **Edge case 2:** the sandbox group also has write access to `.sandbox`, so sandboxed code could pre-create `.sandbox-logs.scrub-v1` and suppress the scrub. Low impact, since that code can already read the logs. Document it.

**7. Tests: they would fail without the fix. Nit-level gaps.**
- The new patterns (`password`, `-u`, env assignments), the `preview` straddle case, the split-write test, the scrub test, and the `--password=` checks in both sandbox suites all depend on the change. They're deterministic.
- Gaps:
  - several lines in one `write` call;
  - a partial line followed by more writes;
  - the auto-derived amendment path;
  - compact JSON argv;
  - a direct argv element containing spaces.
