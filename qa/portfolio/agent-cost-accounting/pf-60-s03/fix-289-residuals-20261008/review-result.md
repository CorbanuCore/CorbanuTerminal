**Verdict: REQUEST CHANGES**

The fixes for R1/R1b, R3 and criterion 13b are correct, and the in-progress check is sound. I'm asking for changes because of one Major: a bucket that hasn't started yet still hits the same dead end R2 describes. It is one bucket later than the case R2 names, and the fix is small. Nothing was built or run; everything below comes from reading the code.

**Answers to your six checks**

1. **`in_progress_end` is sound, but finding 1 still stands.**
   - The store only keeps coverage (`effective` is `Some`) when every day is `Ready` or `DetailUnavailable`, or is a `CheckpointLag` day after the coverage end (`accounting_store.rs:946-963`).
   - `in_progress_end` (`tokens.rs:1795-1817`) also rejects `DetailUnavailable`, `NeedsRefresh` and any other day state (`_ => false`).
   - Its rule for `CheckpointLag` days (`day*DAY >= end`) is exactly the store's `to <= day*DAY`, so a lagging day inside the covered part stays partial.
   - Days cut off at the start by the requested range or by retention fail `start == bucket.start_ms`. A bucket cut off by a requested end at or before the read time fails `requested.end_ms > read_at`.
   - Counting the bucket in the range total is safe. Inspection refuses any recorded request sent after the checkpoint (`ensure!(dispatch <= checkpoint, "future dispatch")`, `accounting_lifecycle.rs` near line 463). So "recorded through the checkpoint" is everything recorded.
   - Only one bucket can be in progress: buckets entirely in the future have no coverage.
2. **R3 is fixed.** The range header and the day pages now get the floor from the same `aggregate_day_floor(checkpoint)`, read in the same snapshot (`accounting_store.rs:904-907`).
   - Staging: day pages return `CheckpointLag` and show no floor; the range says "unknown" (see finding 6).
   - No ledger: the range code is never reached, because `inspect` returns `Absent` first (`accounting_store.rs:348-356`).
3. **Default build: no path leads to the cost page.**
   - The menu item is behind the feature flag (`usage.rs:76-92`).
   - `/usage requests` answers with the one line (`slash_dispatch.rs:806-816`).
   - `/cost` is hidden (`slash_command.rs:319`).
   - The only remaining pointer to a dead end is finding 3.
   - No clippy or dead-code risk: `cfg!` keeps both branches compiled, and `TryRecvError` is re-exported at `tests.rs:181`.
4. **Developer build is unchanged** apart from the intended range wording and the floor's source.
5. **The new tests fail without the fix:**
   - the TUI in-progress test has no in-progress line without it;
   - the state test gets `None` instead of 36, and won't compile against the old field name;
   - the default-build slash test would see the cost view open.

   Missing cases are listed in finding 8.
6. **Wording** is mostly plain. Findings 5 and 6 are small wording issues.

**Findings**

1. **Major: buckets that haven't started yet still show "Bucket unavailable" with a next step that can't help.** (`tokens.rs:1963`, the diagnostics branch near line 1995, and the total at 1925-1949)
   - The parser only refuses a future *start*; a future end is allowed (`tokens.rs:2266-2272`).
   - Example: `/cost 2026-10-01 2026-11-01 week` on 10-08 returns weeks 10-12, 10-19 and 10-26.
     - Each starts after now, so it has no coverage (store test `accounting_store_tests.rs:1208-1219`).
     - Each is titled "Bucket unavailable / Partial bucket — excluded from totals".
     - Each shows "Next step: send a turn… then select Refresh", which can't change a future bucket.
     - The range total is withheld, including the correct in-progress week.
   - **Fix (either):**
     - Refuse a future end at parse time ("future end is unavailable"), or clamp the end to the read time.
     - Or treat `bucket.start_ms > read_at` as "Not started", with no diagnostics, no next step, and not blocking the total.
   - Add a test for a range with a future end.

2. **Minor (existing behaviour, same kind of problem): a bucket that ends after the last recorded request is still "Partial — excluded", often with no next step.** (`tokens.rs:1963`, store at `accounting_store.rs:950-953`)
   - The checkpoint only moves forward when a request is recorded (`tokens.rs:806`).
   - So before any turn today, yesterday's bucket in `/cost 2026-10-01 2026-10-08 day` shows "Partial — excluded". Its day is `Ready`, so it gets no next step, and the range total is withheld.
   - Hour buckets that start after the checkpoint get the same treatment, also with no next step.
   - **Fix:** given the "future dispatch" rule, extend "so far / nothing recorded after X" to these buckets. Otherwise record this as an open #289 follow-up.

3. **Minor: the default build's `/usage` usage error still advertises `/usage requests [YYYY-MM-DD]`.** (`slash_dispatch.rs:821`)
   - Following it gives "not part of this build".
   - **Fix:** put the `or /usage requests…` part behind the feature flag, and add a default-build assertion.

4. **Nit: comments now out of date.**
   - `slash_command.rs:313-318` still says "`/usage requests` remains the only way to the (empty) view".
   - `tokens.rs:786` (the `cost_command` doc) and its `"/usage requests"` branch, plus the doc at `tokens.rs:1617`, describe a path the default build no longer has.
   - **Fix:** update the comments to say the view is developer-only.

5. **Nit: in-progress pages also say "Snapshot is not current; newer activity is unverified".** (`tokens.rs:1871-1879`, `882`)
   - A `Ready` day in progress always has a read time later than the checkpoint, so this line always appears next to "totals so far, recorded through X". The test fixture produces it too.
   - **Fix:** leave it out when the bucket is in progress, or reword it to something like "nothing recorded after X". Assert the result in the test.

6. **Nit: "daily totals kept since unknown" reads badly.** (`tokens.rs:1844`)
   - **Fix:** use "daily totals: not known yet", or leave the clause out in the Staging case.

7. **Nit: the 365-day retention length is now defined in two places.**
   - `aggregate_day_floor` (`accounting.rs:209-220`) defines its own `DAY_MS` and `REPLAY_MS`.
   - Expiry uses separate `REPLAY_MS` constants (`accounting_lifecycle.rs:9,226`; `accounting_retention_atomic.rs:327,381`).
   - The floor and the expiry could drift apart. **Fix:** define the constant once and use it everywhere.

8. **Minor: missing tests.** None of these cases has a test yet:
   - A bucket reaching today that contains a `NeedsRefresh`, a `DetailUnavailable`, or a `CheckpointLag` day before the coverage end. Each must stay partial and be withheld.
   - A bucket reaching today that is cut off by a requested end at or before the read time. It must stay partial.
   - A range with one in-progress bucket and one whole bucket, where the total must equal their sum.
   - Hour grouping.
   - One test that feeds real store output into `range_pages`. The current TUI test uses a hand-built bucket.