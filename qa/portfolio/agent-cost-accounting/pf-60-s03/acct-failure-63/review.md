**Verdict: APPROVE WITH NITS.** One major issue needs a follow-up ticket; it doesn't need to block this fix.

I only read code. I didn't edit anything or run any tests.

**Answers to your questions**

- **Q1, ledger semantics:** For an intact ledger, `day_quotes_on_connection` gives the same result as `latest_quote_on_connection`. It picks rules the same way: the recorded rules when the current evidence was recorded, version 1 when the field is missing, otherwise `PRICING_RULES`. It also handles unbound attempts, bindings with no snapshot, snapshot reads and ordering the same way. The refactored `latest_quote_on_connection` still checks every stored version exactly as before. The only change is that it reads the attempt, binding and snapshot once instead of once per version, and they can't change inside one transaction. `check_day_on_connection` is only called from the write path, so the full sweep still verifies everything.
- **Q2, hourly validation:** When the fast path's `!due(as_of)` check passes, the full sweep would only rewrite the same compact days and run `gc_snapshots`. Snapshots only become unreferenced through raw removal or owner deletion, and both of those run gc. The fast path also requires the retention state to be `Active`, so it never skips setting `admission_active`. Clock problems all fail safe: a clock that goes backward, a write that crosses an hour boundary, or `AsOf::At` in tests all end in the full sweep. Validating on a separate read snapshot leaves one gap, which the code documents: rows committed between that snapshot and the write lock aren't checked by this hour's sweep.
- **Q3, retry:** A contended call never commits anything. In WAL mode, `BUSY` comes from `BEGIN IMMEDIATE` or a read, not from `COMMIT`. So a retry can't record anything twice. `dispatched_at` is read once outside the retry. Cancelling while it waits drops the future: an admit then has no attempt and sends nothing, and observe's `Completion` drop closes the sampling. Both fail closed.
- **Q4, logging:** I found no secrets, prompt content or endpoint URLs in any `failure(...)` cause. The causes are static strings, sqlx error messages, URL parse errors and HTTP status codes. Provider usage errors are deliberately replaced with a fixed message.
- **Q5, `auto_vacuum`:** Existing databases behave the same. The setting lives in the database header and was never applied after the first table existed anyway. One edge case is in finding 3.

**Findings**

1. **Major — the 16-second lock hold comes back once the ledger is 90 days old.** `state/src/runtime/accounting_retention_atomic.rs:215-231` and `:180-201`.
   - The read-snapshot fast path only applies when `!retention_due(as_of)`. `DETAIL_MS` is 90 days.
   - Once a ledger has 90+ days of continuous use, almost every hour has some attempts that expired since the last sweep. So `validate_hour` returns false and the sweep reads and prices every attempt again under `BEGIN IMMEDIATE`.
   - The 120 s retry hides this from accounting, but other writers to the shared state DB in other processes still hit the 5 s busy timeout.
   - **Fix:** build the plan on the read snapshot, then under the lock re-check and apply only the change. Re-read and re-quote just the attempts being removed (`raw_removals`), confirm nothing else changed, and apply. That makes the work under the lock proportional to what is expiring, not to the whole ledger. At minimum, add a test that a sweep with something due finishes under a lock-time limit, and file the follow-up.

2. **Minor — a write that crosses the hour still runs the full sweep under the lock.** `state/src/runtime/accounting_store.rs:392-412` and `accounting_native.rs:87`.
   - `admit` and `observe` call `maintain_for_write_on_connection(.., None)`.
   - If `open` runs at hh:59:59.9 and the write at hh+1:00:00.1, the checkpoint is earlier than `hour_start`. The write then runs the old full sweep under the lock.
   - **Fix:** run the same read-snapshot pre-validation in `write()` before `BEGIN IMMEDIATE` and pass `validated_at_ms` through. Alternatively, pass `open`'s validated hour into `write`.

3. **Minor — the process that creates a new DB keeps the old locking behaviour for its whole lifetime.** `state/src/sqlite.rs:341`.
   - The options are fixed when the pool is built. sqlx replaces idle and expired connections over time, and every replacement in the creating process still runs `PRAGMA auto_vacuum=INCREMENTAL`.
   - That process is usually a long-running TUI on first launch, so new connections there can still wait on another process's write lock.
   - **Fix:** set `auto_vacuum` once on a single connection right after creating the DB, before migrations. Leave it out of the pool's per-connection options.

4. **Minor — a retry redoes the whole-ledger validation every time.** `core/src/accounting.rs:521` together with `accounting_store.rs:447`.
   - `store_call` retries all of `AccountingStore::open`. So every `BUSY` at the write step re-runs the snapshot validation, about 16 s on the real ledger.
   - At the top of a contended hour, several processes each pay roughly 16 s plus a 5 s busy wait per retry, inside a 120 s budget. That leaves only about five tries.
   - **Fix:** retry only the `BEGIN IMMEDIATE` stage inside `maintain_for_write` with the already-computed `validated_at_ms`, or cache the validated hour across retries.

5. **Minor — the process-wide `WRITES` lock is held while retrying.** `core/src/accounting.rs:582` and its use in `admit_with_tier` and `observe_patch`.
   - `open` and `admit` each get their own 120 s budget, so the permit can be held for 240 s or more.
   - Other sessions and subagents in the same process queue behind it with no time limit of their own, so the waits add up one after another.
   - Holding the permit is acceptable, since they would contend for the DB anyway. Either put one overall deadline across both `store_call`s, or document the worst case.

6. **Nit — `SQLITE_LOCKED` is retried as if it were contention.** `state/src/runtime/accounting_store.rs:252`.
   - Without shared cache, code 6 means a conflict inside the same connection. That won't clear up, so the retry just stalls for 120 s before failing.
   - **Fix:** retry only primary code 5 (`BUSY`, including `BUSY_RECOVERY` and `BUSY_SNAPSHOT`) plus `PoolTimedOut`.

7. **Nit — the extension path logs a misleading message.** `core/src/accounting_extensions.rs:181`.
   - `failure()` logs the text "…request stopped without a repair send", but in this path the request has already been sent. The function just returns false.
   - **Fix:** use a plain `tracing::warn!` with the step and cause.

8. **Nit — one stop path has no real cause.** `core/src/accounting.rs`, the `self.previous.lock()` poison handlers near lines 897 and 940.
   - These return `anyhow!(FAILURE)`, so the caller logs FAILURE as its own cause.
   - **Fix:** use `"previous-attempt lock poisoned"` as the message.

9. **Nit — `day_quotes_on_connection` is looser than `recorded_rules` on corrupt data.** `state/src/runtime/accounting_estimates.rs:214`.
   - A stored JSON `null` or non-integer `pricing_rules` maps to version 1 or a decode error, where `recorded_rules` rejects it.
   - It doesn't matter for intact ledgers, but tighten it to `json_type(...)` = `'integer'` or absent if you want the two paths to fail the same way.

10. **Minor (tests) — they don't fully prove the claims.**
    - **Core contention test** (`core/src/accounting_tests.rs:824`): it isn't flaky, since it only checks lower bounds, but the 5.6 s margin proves a retry only if the spawned task reaches `BEGIN IMMEDIATE` within 0.6 s. On a loaded CI machine it can pass without ever retrying. Assert retries happened (`traced_test` plus `logs_contain("retrying")`), or hold the lock for something like 7 s. Each run also takes about 11 s.
    - **State tests:** nothing checks the public `AccountingStore::open` path end to end (lock held only briefly while another connection holds a read). Nothing checks that `validate_hour` returns false when retention is due. Nothing checks that the fast path is refused when something becomes due between validation and the lock.
    - **`day_quotes` test:** add a bound attempt with no snapshot (`Some(None)`) and two attempts sharing one snapshot.
    - **`contention_is_named_and_nothing_else_is`:** it spends a deterministic 5 s waiting for the busy timeout. That's fine.

The full diff I reviewed is at `/Volumes/CorbanuDrive/Corbanu/.codex-work/acct63/review.diff`.