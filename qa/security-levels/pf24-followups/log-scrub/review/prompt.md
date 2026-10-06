You are an independent senior security and Rust reviewer (Opus 5.5, high effort). Review the last commit on this branch (`git show HEAD`, base origin/main). Do not edit files; read the code and report.

Context: older builds wrote credentials to the logs DB (`pfterminal_logs_2.sqlite`, table `logs`, column `feedback_log_body`) and to `codex-tui.log` when `log_dir` is configured: provider bearer tokens/header/query values in the `Configuring session` Debug line (#196), env values of spawned commands (#179), request URLs with `?key=`, and `set-cookie` response headers. New builds no longer log them (#183, #201 and a parallel PR). This change cleans old data once:
- `codex_state::log_scrub`: regex detector for secret-shaped values (quoted credential-named map entries incl. one level of escaping, Rust Debug `*_token: Some("…")` fields, `SECRET_NAME=value`, Bearer/Basic, URL query values and userinfo, well-known key formats). Over-matching is accepted.
- Logs DB: in `run_logs_startup_maintenance` (runs in `StateRuntime::init` for every process), once per DB (`PRAGMA user_version` 0 -> 1): batch-update rows, `secure_delete`, then `VACUUM` and `wal_checkpoint(TRUNCATE)` so deleted rows/freed space no longer hold old values; marker only set if the checkpoint was not busy; pragma-modified connection is closed on drop.
- `codex-tui.log` in a configured `log_dir`: background thread masks values in place with same-length `*` (offsets never change, so concurrent appends are safe), then writes a `.codex-tui.log.scrub-v1` marker. The default `codex_home/log/codex-tui.log` is already deleted at startup by existing code.

Look for:
- Safety: data loss or DB corruption risk, startup latency (VACUUM on a large DB, busy timeouts with another corbanu process open, short-lived processes), interaction with concurrent log inserts/pruning, WAL/readers, the marker logic (could it be set while secrets remain, or never be set and re-run forever?), Windows.
- Whether `user_version` is free to use for this, and old binaries opening the DB afterwards.
- Detector correctness: missed leaked forms from #179/#196/set-cookie, catastrophic false positives, regex performance on 50+ MB.
- In-place file masking correctness (line splitting, long lines, non-UTF-8, file growing/shrinking during the pass, truncation by another process).
- Test validity (the fixture DB test).
