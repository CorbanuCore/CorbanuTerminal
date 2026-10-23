# Scrub old leaked values from the logs (#179, #183, #196 follow-up)

Builds before #183 and #201 wrote credentials to the logs database and, when
`log_dir` is configured, to `codex-tui.log`:
- provider bearer tokens and header and query values;
- environment values of spawned commands;
- request URLs with `?key=`;
- `set-cookie` headers.

Deleted rows also stay in the database file's free pages: a 51 MB personal
logs DB held 975 live rows and about 48 MB of free pages containing 499
`set-cookie` values.

## Fix

**Detector.** `codex_state`'s `log_scrub` finds credential-shaped values:
- quoted credential-named map entries, quoted at any escaping depth;
- Debug fields such as `*_token: Some("…")`;
- `NAME=value` assignments, including quoted values with several words;
- Bearer tokens, and Basic credentials after `authorization`;
- URL query values and userinfo;
- well-known key formats;
- every value in a provider's `http_headers` / `query_params`, and every value
  on `spawn_child_async` / `ExecOneOffCommand` lines.

Token counters, `*_id` and similar diagnostic names are skipped. Matching
deliberately over-matches.

**Logs DB.** The scrub runs once per database, in the background after
`StateRuntime::init`, so startup is not delayed:
1. Rows are redacted in batches of 1000 rows or 8 MiB.
2. `VACUUM` rewrites the file.
3. A `wal_checkpoint(TRUNCATE)` with a 100 ms timeout empties the WAL.

Progress is kept in `PRAGMA user_version`: 1 means rows scrubbed and the file
rewritten; 2 means the WAL is truncated too. A busy checkpoint is retried on a
later start. A new database starts at 2. Each database is scrubbed once per
process.

**Files.** `codex-tui.log` and `codex-login.log` in `log_dir` are masked in
place with `*` of the same length, so lines appended at the same time are kept.
A `.<name>.scrub-v1` marker is then written. The pass stops if another process
truncates the file.

**Not covered:**
- values split across a 1 MiB line piece;
- lines an older build still running writes after the pass;
- free pages in the state DB from its long-dropped `logs` table;
- daemon stderr logs.

## Gate evidence

- **Tests:** after `just fmt` and `just fix`, `just test -p codex-state` ran
  382 tests and all passed.
  - Detector cases cover every leaked form, idempotence, and the clean lines
    that must stay unchanged.
  - The file is masked in place once, and a missing file or directory is
    handled.
  - The fixture DB test plants leaked rows, a non-UTF-8 row and 200 deleted
    rows, and asserts that every secret is in the files beforehand. After
    startup, the rows read `REDACTED` and no secret byte is left in the DB or
    WAL while the runtime still has it open. A later start does not touch new
    rows.
  - A busy checkpoint stays at stage 1, and a later start finishes it.
  - A new DB starts done.
- **Real data (a copy of a 51 MB personal logs DB, `corbanu exec`):**
  - Stage 2 was reached, and the file shrank to 0.8 MB.
  - It held 499 `set-cookie` values before and 0 after.
  - The run took 1.2 s with a cold cache. The scrub runs in the background, so
    startup does not wait for it.
- **Worst case (55 MB of rows, every row leaking):**
  - The regexes take 0.2 s in a release build.
  - A short-lived `corbanu exec` exits before the debug-build scrub finishes;
    the next start continues.
- **tmux run on GLM 5.2:** the demo spec run on main
  (`tmux-run/before-main.txt`) leaves both planted values (`copies left: 4`).
  The candidate masks the file line, shows `REDACTED` in the DB, and reaches
  `user_version` 2 with `copies left: 0`.
- **Reviews (Opus 5.5 High):**
  - first pass: request changes;
  - second pass on the reworked code: approve with fixes.

  The findings were addressed; see `review/`.
- **Video:** `qa/demos/index/log-scrub.md`.
