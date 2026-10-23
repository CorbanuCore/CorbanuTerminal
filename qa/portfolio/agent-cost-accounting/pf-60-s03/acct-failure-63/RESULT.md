The accounting failure is fixed, and the fix is pushed, built and installed on this Mac and on the RTX. The root cause was SQLite lock contention between the Corbanu processes that share one state database. Under the same contention that killed the worker, the old binary failed with the same two errors and the new one waited and finished.

Processes that are already running keep the old binary. That includes the TUI started Oct 2, `ci-bazel-test2`, and this worker. The old TUI still holds the database lock for about 17 s on its first accounting write of each hour, so **restart the TUI and the workers** to get the fix everywhere.

## Root cause
- **Long lock holds:** accounting writes held the state DB's write lock far longer than SQLite's 5 s busy timeout, and a timeout was reported as the fatal accounting error. I measured on a copy of the live DB (6,971 recorded requests, the installed debug build type):
  - The first accounting write of each UTC hour checked the whole ledger while holding the lock: **16–17 s**.
  - Every write rechecked every request the conversation had made that day: **1.4 s per write** for the TUI conversation's 811 requests today, several writes per request.
  - Every new pooled connection ran `PRAGMA auto_vacuum`, which needs the write lock, so even reads failed after 5.2 s. That is the 5.2 s "slow statement" in the brief.
- **The 21:04 death:** the worker's last recorded request was at 20:59:17. The TUI made no accounting write between 20:59:47 and 21:04:37, so its 21:04:37 write was the first of hour 21 and ran the full check. The three 5.2 s timeouts at 21:04:24/29/34 fall inside that check. Writes from `exec` don't go to the logs DB and those rows had rotated out, so this rests on the state DB timestamps plus the reproductions below.
- **Reproduced three ways:**
  - The old code on the DB copy: a second writer failed with `(code: 5) database is locked` during the hourly check.
  - The old installed binary on a scratch home, with a cheap local model and another process holding the lock: `EXIT:1` with the same two `Native Anthropic accounting failed` lines as the worker log.
  - The new binary in the same setup: logged "state DB busy; retrying", then "succeeded after contention", `EXIT:0`, with the request and its usage recorded.

## Fix (on `integrate/management-workstreams-20260911`, 21162588cd..79aef0daf3, fast-forward)
- **`f4d6c5888a`:** new connections to an existing database no longer take the write lock. The vacuum setting is applied once, when the database is created.
- **`a956468dad`:** the hourly whole-ledger check now runs on a read snapshot, which blocks no writer, at most once per process per hour. Hours where old records expire still do the full sweep under the lock. The per-write day check reads the day in four bulk queries and gives the same totals. Corrupted data still fails visibly.
- **`7bef7a8dfd`:** contended accounting writes retry with backoff for up to 120 s per operation, then still fail closed. Every path that ends in the accounting failure now logs which step failed and the full error chain, without secrets, prompt content or endpoints. `exec` prints these on stderr.
- **`79aef0daf3`:** fixes a race in the websocket accounting tests' wait helper, which the faster writes exposed.

On the DB copy afterwards: another writer waited 0.2 ms during the hourly check (it used to fail), request writes take 0.13 s (was 1.4 s), and the explicit full sweep takes 9 s (was 17 s).

## Tests
- **Fail before, pass after:**
  - New connection while another process writes: failed with "database is locked" after 5.21 s.
  - Accounting waits out a lock held past the busy timeout: admission failed after 5 s.
- **New, passing:** day totals match the full check; the hourly snapshot check; contention detection; failures log their step and cause.
- **`codex-state`:** 344/344 unit tests plus all integration tests pass.
- **`codex-core` accounting:** 77/77 unit tests and 88/88 integration tests pass. Clippy shows no new warnings and `just fmt` is clean (I reverted the unrelated files it reformats).
- **Run caveats:**
  - Three core accounting unit tests fail when run in parallel. The base commit fails the same three; with `--test-threads=1` all pass.
  - The integration test binary overflowed its stack without `RUST_MIN_STACK=32MB`; I didn't check whether that also happens on the base commit.

## Review
The independent review said **approve with nits**. I fixed all the minor items and nits: write-path validation across an hour boundary, a single-connection vacuum setup, no re-validation on retry, one shared retry deadline, retrying only on SQLite's busy code, clearer log messages, stricter checking of stored pricing versions, and stronger tests.

Its one major finding stays open: once the ledger is 90 days old, around 2026-12-21, old records expire every hour and the full sweep under the lock comes back. Accounting would survive it through the retries, but other writers to the database would not. It needs an incremental sweep before then.

## Build and install
- **Local:** commit 79aef0daf3, SHA-256 `e35723a1…`. Signed with Developer ID (`com.corbanu.corbanu`, timestamped) and verified, then moved into place. `INSTALL.json` is updated and the previous binary is kept as `corbanu.prev-704650ccd`.
- **RTX:** `79aef0daf`, `BUILD_OK`, SHA-256 `e79796dc…`.

## Functional check (installed build, real home, live TUI running)
- **Round A (22:49–22:51):** 3 concurrent `corbanu exec` sessions, all exit 0, no accounting errors, 15 requests recorded with usage. Each session's fourth command was blocked by the product's repeated-call guard, not by accounting.
- **Round B (22:59:05–23:01:21), across the UTC hour:** 3 sessions × 5 requests, all recorded with usage, all exit 0. All three wrote their first accounting record of hour 23 within 0.6 s of each other at 23:00:25, and all succeeded.
- **`/cost`:** on a resumed round-B conversation it shows "5 requests, 86,176 tokens" ($0.105979 at API prices), plus the day's other conversations.

## Still open
- **Running processes:** restart the TUI and the workers, as above.
- **The day-90 sweep** described under Review.
- **Other database writers:** writes outside accounting still give up after 5 s. With the long lock holds gone they are much less exposed.
- **Optional speed-up:** dev builds compile SQLite and serde unoptimised. Optimising just the dependencies would speed every database operation without affecting the debug-only accounting check.
- **Local branch ref:** I never touched the management worktree, so its local `integrate/…` branch is now behind origin.