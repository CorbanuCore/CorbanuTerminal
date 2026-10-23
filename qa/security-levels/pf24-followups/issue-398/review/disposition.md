# Review disposition (Opus 5.5 High, round 1: approve with fixes, nothing blocking)

| # | Finding | Disposition |
| --- | --- | --- |
| 1 | Argument values holding spaces lose only their first word | Fixed: `codex_log_guard::redact_command` redacts each argument whole (`--password=a b`, `--token` + next argument, `Header: a b`) before joining; quoted `"--password=a b"` / `"NAME=a b"` and `"Header: a b"` shapes added for Windows command lines. Tests in log-guard and the sandbox logging test. |
| 1 | Writer keeps `buf` in `pending` after a failed write | Fixed: `pending` is cleared on error. |
| 2 | Claude pane audit tool previews | Fixed: redacted; turn transcripts listed as transcript exceptions. |
| 2 | Telegram approval messages | Fixed: the command is redacted before sending. |
| 2 | `CODEX_TUI_RECORD_SESSION` | Documented as an opt-in transcript exception. |
| 2 | `/feedback` attaches the raw sandbox log | Fixed: the attachment is redacted at upload. |
| 2 | Legacy `sandbox_commands.rust.log` | Fixed: scrubbed with the daily logs. |
| 3 | Compact JSON argv, `PGPASSWORD`/`MYSQL_PWD`/`_authToken`, `mysql -p`, `sshpass -p`, `docker login -p`, `-passin pass:`, `--oauth2-bearer`, `aws configure set`, npm/Stripe keys, multi-word quoted values | Fixed, with tests. `OPENAI_KEY`/`STRIPE_SK` names not matched (a generic `_KEY` suffix hides cache keys and paths); their values are caught when they have a known key format. |
| 3 | False positives on bare `token`/`password`/`secret`, `-u 1000:1000`, `glob_pat` | Fixed: bare names after `:` need a value of 8+ characters; `-u` values must contain a non-digit; `pat` dropped. Tests in `keeps_redacted_values_and_ordinary_text`. |
| 3 | Benchmark | Not added: the regex crate is linear-time; patterns run only through `RegexSet` first. Noted. |
| 4 | "Approved command prefix saved" when not saved | Fixed: `append_amendment_and_update` returns `AmendmentScope`; a session-only rule sends a warning instead of the saved message. Auto-derived proposal path now tested. |
| 5 | `log::set_logger`, `env_logger` | Fixed: disallowed; `execpolicy-legacy`'s dev CLI has an `expect` with a reason. README states that redacting writers remain a review convention. Windows clippy: Bazel clippy runs a Windows target. |
| 6 | One failing file stopped the scrub | Fixed: failures are skipped, the marker is written only when every file succeeded. Marker pre-creation by sandboxed code documented. |
| 7 | Test gaps | Fixed: several lines per write and partial-then-more writes, auto-derived amendment, compact JSON argv, argv elements with spaces. |
