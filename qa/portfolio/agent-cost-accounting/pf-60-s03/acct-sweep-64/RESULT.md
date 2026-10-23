Both items are done, pushed and installed on this Mac and on the RTX. Processes that are already running keep the old binary, so restart the TUI and the workers to pick this up.

## What changed
Pushed to `integrate/management-workstreams-20260911` as a fast-forward, `79aef0daf3..4c748fe5e4`:

- **`d56cdee121`: the turn's throttle check no longer fails with "database is locked".**
  - The four provider-throttle database writes now retry for up to 60 s when another process holds the lock. They retry only on that "locked" error and log each retry.
  - After a wait, cooldowns and leases count from when the write actually lands.
  - Because the check can now wait longer, an interrupt stops it immediately. If the interrupt lands just as a lease was saved, that lease is released.
  - I tried the same retry on thread-metadata writes, but it kept interrupted turns from ending, so I took it out. Those writes only ever logged a warning anyway.
- **`4c748fe5e4`: day-90 expiry is now incremental.**
  - The hourly whole-ledger check now runs before anything is deleted, even when records are due, without taking the write lock. Corrupted data still makes the write fail visibly.
  - Expired records are then removed in batches of at most 32 (about 100 ms), each in its own short write. Each record is checked and applied exactly as the full sweep would.
  - Between batches the lock is left free for about as long as the batch held it, plus a little random delay, so other writers get in.
  - A crash keeps every finished batch, and the next write carries on from there.
  - The full sweep under the lock is now only a rare fallback (see Still open).

## Measured on a copy of the live database, clock moved to day 90
Another process wrote every 10 ms during each run.

| Records expiring | Before | After |
|---|---|---|
| 1 | the other writer failed after 5.34 s; the sweep held the lock about 9 s | longest wait 42 ms, no failures |
| 1,014 | the other writer failed after 5.40 s | longest wait 78–110 ms, no failures |

The first write of the hour still takes about 7–12 s, but almost all of that is the whole-ledger check, which no longer blocks anyone.

## Tests
- **`codex-state`:** 351 unit tests pass (344 before; 7 new) and all integration tests pass.
- **`codex-core` accounting:** 77/77 unit tests (one at a time, as before) and 88/88 integration tests pass. The 16 turn tests pass.
- **Clippy and format:** clippy shows no new warnings and `just fmt` is clean. I reverted the unrelated Python files it reformats, as last time.
- **Fail before, pass after:**
  - **Other writer during expiry:** before, it never got in mid-expiry and only managed 2 writes; after, it gets in between batches and never waits 1 s.
  - **Lock held 6.5 s during a throttle check:** before, it failed with "database is locked" after 5.36 s; after, it succeeds.
- **Other new tests:**
  - Batched expiry leaves exactly the same rows as the full sweep at every boundary from day 90 to past day 365. That includes an interrupted batch and restarts between batches.
  - A crash mid-sweep resumes to the same result.
  - A corrupted record fails its batch and changes nothing.
  - Records left over within the hour show as a short lag and are cleaned up by later writes.
- **Two existing tests broke along the way and are now fixed:**
  - The two core cancellation tests timed out until I made the throttle check give way to an interrupt.
  - Two `codex-state` tests were updated for the new behaviour. A rejected write can now leave behind expiry that was due anyway, and the hourly check now also runs when records are due.

## Review
The independent review said **approve with nits**, with no blockers, and it confirmed the batched result matches the full sweep. I fixed:
- the random delay between batches (Minor 1, a must-fix);
- the slow test, which went from 25 s to about 4 s (Minor 5, a must-fix);
- the stale clock after a retry;
- a misleading warning on interrupt;
- checks on expired compact days and tombstones before deleting them;
- a test for the within-hour leftovers;
- several wording nits.

## Build and install
- **Mac:** commit `4c748fe5e4`, SHA-256 `13d5387d…`. Signed with Developer ID (`com.corbanu.corbanu`, timestamped), verified, and moved into place. `INSTALL.json` is updated and the previous binary is kept as `corbanu.prev-79aef0daf`.
- **Mac smoke test:** 3 concurrent `corbanu exec` runs all exited 0.
- **RTX:** `4c748fe5e`, `BUILD_OK`, SHA-256 `55cfd1c0…`. I also kept the previous binary there as `~/corbanu-rtx/bin/corbanu.prev-79aef0daf`.

## Still open
- **Speeding up dependencies in dev builds:** I measured it but didn't change it. Optimising SQLite, serde and sqlx makes the first write of each hour about 17–22% faster (7.0 → 5.75 s and 12.0 → 9.4 s). SQLite alone gives 2–11%. The catch is that it forces a near-full rebuild of every dev and CI target. Say if you want it.
- **Thread deletion and explicit maintenance still run the full sweep under the lock**, about 9 s per deleted conversation on the live ledger. This is not new.
- **A rare remaining fallback:** if the hour turns while a write is waiting for the lock, that write still runs the full sweep.
- **Thread-metadata writes** still give up after 5 s, but they only log a warning.
- **Review items not done:**
  - no core test for an interrupt landing during a lease write;
  - no test for two processes expiring at the same time;
  - a few small full-table scans per batch, which a future index would remove.
- **The diff is about 1,230 lines**, roughly 580 of them tests.