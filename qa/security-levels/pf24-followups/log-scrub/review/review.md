**Verdict: request changes.** The approach is sound. VACUUM plus `wal_checkpoint(TRUNCATE)`, same-length in-place masking and a `user_version` marker are safe, and I found no corruption risk. But the detector misses the exact leak forms it is meant to clean, and the DB pass runs during startup and can repeat on every launch.

These findings come from reading the code only. I did not build or run anything because the sandbox is read-only.

## Findings

1. **High — `state/src/log_scrub.rs:40-47`: the detector misses the known #196 and #179 leak forms.**
   - The fixes in #196 and #183 redact values *whatever the key name is*, because the name can't tell you whether the value is secret. The scrub only catches keys whose names contain a secret word.
   - **#196 leaks that remain:**
     - `http_headers: Some({"X-Sentinel": "…"})`
     - custom proxy headers
     - `query_params: Some({"sig": "…"})`
   - **#179 leaks that remain** (`spawn_child_async: … {env:?}` and `ExecOneOffCommand params: {params:?}`):
     - `MNEMONIC`, `SEED_PHRASE`, `DB_PASS`, `SENTRY_DSN`. Seed phrases matter most in a trading terminal.
   - Pattern 3 also misses quoted assignments such as `API_KEY="sk…"`, `API_KEY='…'` and the Debug-escaped `API_KEY=\"…\"`. The value class rejects a leading quote, so in the escaped form it masks only the `\`.
   - **Fix:** add rules based on the line's context:
     - In `ModelProviderInfo {` text, redact every value inside `http_headers: Some({…})` and `query_params: Some({…})`.
     - On `spawn_child_async:` and `ExecOneOffCommand params` lines, redact every value of the env map (`"K": "v"` → `"K": "REDACTED"`).
     - Change pattern 3 to `=(?:\\?["'])?([^\s"'\\&;,]+)`.
     - Add `mnemonic|seed|pass|pwd|private|jwt|dsn` to `SECRET_NAME`.
     - Add test cases for each of these forms.

2. **Medium — `runtime/logs.rs:307,322-388`: a full regex scan and VACUUM run synchronously inside `StateRuntime::init`, for every process.**
   - There is no global size cap. Partitions are 10 MiB per thread over 10 days, so the DB can reach hundreds of MB. The first launch after the upgrade (TUI, exec, app-server, MCP) can block for seconds or minutes with no feedback.
   - VACUUM holds the write lock for its whole run. Other corbanu processes' log inserts wait out their 5 s `busy_timeout` and fail.
   - VACUUM needs up to about 2× the DB size in free disk (temp file plus WAL). On a full disk it fails on every launch after the scan has already run.
   - `LOG_SCRUB_BATCH_ROWS = 2000` with no byte limit can load very large bodies into memory, since trace rows can be MBs each.
   - **Fix:**
     - Run the pass in a background task after `init` returns, or only in long-lived processes.
     - Batch by summed `estimated_bytes` (for example ≤ 8 MiB) as well as by row count.
     - Skip rows cheaply in SQL first (`instr`/`LIKE` on the trigger substrings) before running the regexes.

3. **Medium — `runtime/logs.rs:374-385`: the marker logic can make the expensive pass repeat on every launch.**
   - If the TRUNCATE checkpoint returns busy, the function returns `Ok` without setting the marker. A user who always has another session open (reader or writer active during the 250 ms window) re-scans and re-VACUUMs on every start. The same happens for a VACUUM that hits `SQLITE_BUSY` or `SQLITE_FULL`.
   - After VACUUM commits, any later full checkpoint (PASSIVE, auto-checkpoint) already brings the main file up to date and truncates it.
   - **Fix:** use staged markers:
     - Set `user_version = 1` after the rows are scrubbed and VACUUM commits.
     - Set it to `2` after a non-busy TRUNCATE.
     - On later starts at version 1, retry only the checkpoint.
   - Log the busy result at debug level instead of returning silently.

4. **Medium/low — `log_scrub.rs:40-55, 69-89`: regex cost.**
   - Seven separate `captures_iter` passes run over every row and line.
   - Unicode `\b` (patterns 2-4, 6, 7) forces the lazy DFA to fall back to slower engines on non-ASCII text (model output, box-drawing characters). On a 50+ MB file or DB that is several seconds.
   - **Fix:** use `(?-u:\b)` or `(?-u)` with ASCII classes, and put a `RegexSet` (or a memchr keyword check) in front so only lines that might match go through the capture engines.

5. **Low — `log_scrub.rs:29,160`: lines over 1 MiB are scanned in chunks**, so a secret that straddles a chunk boundary is missed or only partly masked. **Fix:** overlap chunks by about 4 KiB and re-scan the overlap.

6. **Low — `log_scrub.rs:167-170`: truncation by another process.** If another process truncates or rewrites the file during the pass, `seek` + `write_all` past the new end makes a sparse file that brings back the old (masked) line. **Fix:** before each write, check that `writer.metadata()?.len() >= offset + line.len()` and stop the pass if it isn't. The Windows share mode is fine because Rust opens files with read, write and delete sharing.

7. **Low — markers can be set while secrets remain.**
   - An older binary still running (or a downgrade) can write new secret rows or lines after the pass has moved past them. They are then never scrubbed.
   - A missing file creates the marker (`log_scrub.rs:140-143`), so a later old binary writing to that `log_dir` leaves its output unscrubbed.
   - Rows whose TEXT is not valid UTF-8 fail to decode as `Option<String>`, so the pass errors and never marks. Selecting `CAST(feedback_log_body AS BLOB)` avoids this.
   - **Fix:** document these limits in the release record; use the BLOB select.

8. **Low — files and databases that are not scrubbed:**
   - `log_dir/codex-login.log`
   - app-server-daemon `*.stderr.log` (it captures the daemon's tracing output on stderr)
   - free pages in `state_5`/`pfterminal_state` left by the dropped `logs` table (migration `0023_drop_logs`). `spawn_child_async`'s env dump predates the fork.

   **Fix:** list these in the release note, or VACUUM the state DB once as well.

9. **Low — `tui/src/lib.rs:1350-1354`: an early failure's warning is lost.** The thread starts before the subscriber is installed at line 1412, so a quick failure such as permission denied is logged to nothing. **Fix:** start the thread after `try_init()`.

10. **Low — test validity, `runtime/logs.rs:~688-814`:**
    - Nothing asserts that `KEY`/`DELETED` are actually in the fixture file before `init`, so the "not in file" check could pass trivially.
    - `pool.close()` (lines 743 and 794) checkpoints and removes the WAL by itself, which hides whether the scrub's own TRUNCATE worked.
    - There is no test for the busy path (an open read transaction on a second connection means no marker, then a retry).
    - The form tests cover none of: secrets under non-secret key names, quoted env assignments, lines over `MAX_LINE_BYTES`, appends during the pass, non-UTF-8 bytes.

    **Fix:** add a check that the secrets are present before `init`, a busy-path test, and these form cases.

11. **Nit — over-matching is noisy:**
    - `basic\s+` masks ordinary words ("basic functionality").
    - Pattern 2 masks `session_id`, `thread_key` and `tokens` values, which removes diagnostic IDs from `/feedback` uploads.

    This was accepted as a tradeoff, but consider excluding `*_id`/`*_count` and dropping `basic` unless it follows `Authorization:`.

12. **Nit — `codex-rs/AGENTS.md` style and hygiene:**
    - `runtime/logs.rs` is 2,126 lines. Per AGENTS.md, put new code in a new module instead (for example `runtime/log_scrub.rs`).
    - `pub mod log_scrub` makes `secret_spans`, `scrub_text`, `REDACTED` and the marker helper public. Make the module private and re-export only `scrub_log_file_once`.
    - `MARK_LOG_SCRUB_DONE` repeats `LOG_SCRUB_VERSION`. Build the pragma with `format!` from the constant.
    - `PRAGMA secure_delete = ON` does nothing useful because VACUUM rewrites the file anyway. Remove it.
    - The new `regex` dependency changes `Cargo.lock`, but `MODULE.bazel.lock` wasn't refreshed. Run `just bazel-lock-update`.

## Your specific questions
- **Is `user_version` free to use?** Yes. sqlx uses `_sqlx_migrations`, nothing else in the repo reads it, VACUUM keeps it, and older binaries ignore it (see #7 for old binaries writing after the marker).
- **Row IDs:** VACUUM keeps the `id INTEGER PRIMARY KEY AUTOINCREMENT` values and `sqlite_sequence`, so IDs don't change.
- **Batch scan:** paging by `id > ?` is safe with concurrent pruning. An UPDATE on a deleted row does nothing.
- **Connection reset:** `close_on_drop` works on sqlx 0.9, so the changed pragmas don't leak back into the pool.
- **UTF-8 in `scrub_text`:** the `from_utf8_lossy` calls never actually lose data, because Unicode-mode classes keep span edges on character boundaries.