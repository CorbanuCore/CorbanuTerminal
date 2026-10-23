**Verdict: approve with fixes**

The staged marker logic, the AUTOINCREMENT-based batch ranges, the BLOB reads, idempotence and the file truncation guard all hold up. Most of the first-pass findings are properly fixed. I found one high-severity leak in the new `NAME=value` rule; the remaining findings are medium or lower. I couldn't build or run anything in this read-only sandbox, so the regex findings come from rebuilding the patterns in Python, not from the Rust crate.

1. **High** — `state/src/log_scrub.rs:55-58`: when a `NAME=` value is in quotes, only the first word is hidden. The value pattern `[^\s"'\\&;,]+` stops at the first space. In my Python check:
   - `MNEMONIC="abandon ability able about"` hid only `abandon`.
   - `DB_PASS='two words'` hid only `two`.
   - `TOKEN=\"a b\"` hid only `a`.
   
   `mnemonic` and `seed` were added specifically, so 11 of 12 seed words would stay in the log. **Fix:** use one pattern per form, so the capture names never repeat: `=\\?"{QUOTED_VALUE-contents}\\?"`, `='(?P<v>[^']*)'`, and the current unquoted form. Add test cases with multi-word quoted values in all three quoting styles.

2. **Medium** — `state/src/log_scrub.rs:36-37`: the `tokens` suffix rule also skips real secrets such as `refresh_tokens`, `access_tokens`, `auth_tokens` and `GITHUB_TOKENS`. I confirmed that `"refresh_tokens": "rt-secret"` is left unredacted. **Fix:** skip a `*tokens` name only when its value is all digits, or list the known counters (`input_tokens`, `output_tokens`, `max_tokens`, `cached_tokens`, `reasoning_tokens`, `total_tokens`).

3. **Medium** — `state/src/runtime/log_scrub.rs:22-29` (task lifetime): the spawned task holds `Arc<StateRuntime>`. That keeps every pool (state, goals, memories) alive after the owner drops its handle. It also means `close()` blocks on `logs_pool.close()` until an in-flight VACUUM finishes, so shutdown can wait as long as a full rewrite of the database. The task also runs in every process and test that calls `init`, and more than once per process if `init` is called repeatedly. **Fix:**
   - Move the work to a free function that takes only `Arc<SqlitePool>` for the logs pool.
   - Keep a `CancellationToken` or `AbortHandle` in `StateRuntime` and cancel it in `close()`.
   - Guard with a process-wide `OnceLock` keyed by the database path.

4. **Medium** — `state/src/runtime/log_scrub.rs:33-44` (fresh databases): a newly created logs database starts at `user_version = 0`. The background scrub then rewrites rows this build inserts during the race window and runs a needless VACUUM. That is wasted work on every fresh install and every test runtime. It is also a latent source of flaky tests anywhere a test inserts a log containing `?x=`, `session`, `key` and so on, then asserts the stored text. **Fix:** in `open_logs_db`, detect a newly created file (the `database_has_content` check already exists) and set `PRAGMA user_version = 2` before returning the pool.

5. **Medium/low** — `state/src/runtime/log_scrub.rs:39` and `:49-52` (lock contention):
   - VACUUM holds the write lock for the whole rewrite. A logs database of hundreds of MB can take longer than the 5 s busy timeout. When that happens, `log_db` insert batches are dropped silently (`log_db.rs:436`), and `delete_thread` (`threads.rs:1090`) can fail visibly for the user.
   - While a TRUNCATE checkpoint is pending, it blocks new writers for up to 5 s. This repeats on every start that stays at stage 1.
   - VACUUM also writes the whole database into the WAL, so the WAL stays database-sized until a later TRUNCATE succeeds.
   
   **Fix:** at minimum, document this. Better, wrap the update transaction and the VACUUM in the existing `busy_retry` helper. For the checkpoint, use a short dedicated connection with a small `busy_timeout`, so a busy result returns quickly instead of stalling writers.

6. **Low** — `state/src/runtime/log_scrub.rs:67-69`: `length(CAST(feedback_log_body AS BLOB))` loads every body (including overflow pages) just to size it. The second query then reads the same rows again. **Fix:** size batches with the existing `estimated_bytes` column.

7. **Low** — `state/src/runtime/log_scrub_tests.rs:196-199`: the busy test sleeps for 7 s and depends on the 5 s busy timeout. It can't tell the busy (`busy = 1`) result apart from the checkpoint failing with an error, because both leave stage 1. It also adds 7 s to every run. **Fix:** don't rely on the background task. Call `scrub_logged_secrets_once()` directly, with the background spawn turned off as in fix #4, assert that it returns `Ok` at stage 1, then release the reader and run it again.

8. **Low** — `state/src/runtime/log_scrub_tests.rs:119-123`: the "every secret present" check compares a count of (secret, file) pairs with `SECRETS.len()`. If a WAL is left behind, or one secret sits in two files, it fails or passes for the wrong reason. **Fix:** collect the matching secret names into a set and compare it with `SECRETS`.

9. **Low** — `state/src/log_scrub.rs:190` and `tui/src/lib.rs:1403-1414`: if `log_dir` doesn't exist (no file log configured), `File::create(marker)` fails. The warning then repeats on every start and is written into the logs database. **Fix:** skip writing the marker when the parent directory is missing, or treat `NotFound` on the marker as done.

10. **Low** — `state/src/log_scrub.rs:215-222`: there is still a check-then-write race (TOCTOU) between the length check and `write_all`. If another process truncates and rewrites the file in between, this pass writes the old (masked) bytes over the new content. This is rare, so documenting it is enough; alternatively, compare the inode or file ID before each write.

11. **Low** — `state/src/log_scrub.rs:40-41`: the quoted-value patterns allow only one level of escaping. Twice-escaped JSON such as `\"{\\\"token\\\": \\\"v\\\"}\"` matches nothing (confirmed). Userinfo with only a token and no `user:` part (`https://<token>@host`) is also missed. **Fix:** allow `\\*"` around names and values, and add a pattern for userinfo without a colon.

12. **Nit** — `state/src/lib.rs:26-27`: the doc comment `/// Preferred entrypoint: owns configuration and metrics.` now sits above `scrub_log_file_once` instead of `StateRuntime`. Move it back.