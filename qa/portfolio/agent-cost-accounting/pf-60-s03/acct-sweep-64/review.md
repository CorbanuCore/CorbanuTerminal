No blockers. I'd approve with nits, after the two Minors marked "before merge" below are fixed.

The batched expiry gives the same result as the full sweep, a crash mid-way loses nothing, and validation still happens before anything is removed. The open points are robustness and test runtime.

I made no edits. I ran 9 targeted tests on the clean tree and all passed. Run alone, `expiry_at_the_detail_horizon_never_holds_up_another_writer` took 25.0 s; the other writer's longest wait was 50 ms during a 3.19 s expiry.

## 1. Ledger semantics: correct
I compared `expire_raw`, `remove_raw`, `reduce` and `read_validated_input_on_connection` line by line:
- **Same records, same checks.**
  - The batch selects due records with the same predicates as `retention_due_on_connection`.
  - It checks for a tombstone collision and reads the latest quote, which verifies every estimate version as the full sweep does.
  - If a contribution exists, `validate_reference` runs; a missing or stale contribution is folded anyway, as `reduce` does.
- **Day folding matches.** A raw attempt is folded only if its day hasn't reached the 365-day expiry. That uses the same limit as compact-day expiry, so a raw-only day can't be brought back, in any batch order.
- **Compact days that already exist** are decoded and added to, so a record folded in an earlier batch is never lost.
- **No double-apply.** The fold, the removal and the tombstone are one transaction. Selection happens under the write lock, so two processes running batches are serialised and a repeated batch finds nothing left to do.
- **Snapshot cleanup.** Every reference a batch removes goes into `unbound` and gets cleaned up.
- **Reads while the checkpoint lags.** Day totals are unchanged by folding. Inspection still reports the folded day as `DetailUnavailable`, through `compact && window.is_none()` and `lower <= read_at - 90d`, as long as the reader's `read_at` is at or after the batch's time.

## 2. Validation contract: sound
- Whole-ledger validation runs on the read snapshot before any batch.
- Records committed after that snapshot are verified individually when a batch removes them.
- `checkpoint >= hour_start` is only true if a validation ran this hour: only `validated_this_hour` writes and full sweeps move the checkpoint.
- The relaxed rule is sound. Anything due before the hour began must be gone before the checkpoint moves. Later leftovers show up as `CheckpointLag` in inspection or `NeedsMaintenance` in `read_retained`, as branch 1 already allowed.

## Findings

### Minor
1. **Inter-batch pause can line up with SQLite's retry timing** (`accounting_store.rs:237,511`). The pause equals the batch's hold time, clamped to 10–100 ms.
   - After about 228 ms of waiting, SQLite's default busy handler retries every 100 ms.
   - Release-build batches likely hold for 10–25 ms, so the lock cycle can be 20, 25 or 50 ms, all of which divide 100 ms evenly.
   - A waiter that has fallen into steady 100 ms retries can then keep landing inside the lock. Only timing noise breaks the pattern. Your measurements were debug builds, where holds are longer.
   - **Fix:** add random jitter to the pause, or make the minimum pause ≥110 ms whenever more batches follow. That costs about 3.5 s for 1,000 due records, only on the first day ~1,000 records are due at once (about 2026-12-21); after that it's a few records an hour.
2. **Retries reuse the original `now_ms`** (`provider_requests.rs:422-470`). After waiting up to 60 s:
   - the cooldown check can block on a cooldown that has already ended, and overstate `remaining_ms`;
   - a 429 result sets `cooldown_until = stale now + retry_after`, so the provider's retry-after is cut short by the time spent waiting;
   - the lease can end up to 60 s early, which is harmless against a 10-minute lease.

   **Fix:** shift the time by the elapsed retry time inside each wrapper, or sample the clock inside the retry closure.
3. **Misleading warning on interrupt** (`core/src/session/turn.rs:2661`, `Drop` at `:2990`). When a turn is interrupted before any lease was written (the common case: waiting on the lock), the provisional guard's release matches 0 rows. That logs the warning "lease guard drop did not match active lease owner".
   - **Fix:** mark the guard as provisional and log at debug/trace for 0 rows, or use a separate release path.
   - The placeholder `lease_until_ms: now_ms` is unused and misleading; at least comment it.
   - The release ordering itself is correct: the acquire holds `BEGIN IMMEDIATE` until it commits, so a release on another connection always runs after any committed lease.
4. **Expired compact days and tombstones are deleted without checks** (`accounting_retention_incremental.rs:65-85,100-108`). The full sweep checks their keys (`day_key`), decodes compact-day payloads and range-checks tombstones; batches don't.
   - Rows present at the snapshot were checked. Rows written later by late import or lifecycle delete that expire within the same hour are removed silently.
   - **Fix:** for each selected row, run `day_key` and `CompactValues::decode` and check the tombstone's replay range (`dispatch` within 0..=as_of) before deleting. It's cheap and bounded at 32 rows.
5. **The 2,000-attempt test is slow** (`accounting_store_tests.rs`, `expiry_at_the_detail_horizon_…`). It takes 25 s alone in debug; nextest kills tests at 60 s (`slow-timeout 30s × 2`), so under parallel CI load it is at risk. Fix before merge.
   - Most of the time is building the fixture with 2,000 `admit` calls.
   - **Fix:** insert the fixture rows with SQL in one transaction, or cut to about 600 attempts while keeping a measured "before" failure. The timing check (<1 s against ~50 ms measured) has plenty of margin.
6. **Test gaps:**
   - no test for the hour-turn loop in `prepare_write`; `AsOf::At` can never reach it, so consider an injectable clock;
   - no test for the relaxed rule (checkpoint moves with due-after-hour-start leftovers, and inspection then reports `CheckpointLag`);
   - no test of two processes running `expire_due` concurrently;
   - no core test for `or_cancel` plus the provisional guard. The commit relies on existing suites; add one test where an interrupt lands while the lock is held, asserting `TurnAborted` and no lease left behind.
7. **Small full-table scans under the lock per batch.** Each batch scans every attempt with `json_extract` about four times: `retention_due` twice, the due `SELECT`, and the `due_for_write` check. That's cheap at 7k rows and bounded by the 90-day window, but it scales with ledger size × number of batches.
   - A future migration adding an expression index on `json_extract(payload,'$.dispatched_at_ms')` would remove it.
   - The `unbound` set already avoids the full snapshot cleanup, which was the ~1 s cost.

### Nit
- **Price-snapshot cleanup** (`accounting_retention_incremental.rs:86-99`). Batches clean up only what they unbind, so a snapshot that was already orphaned is never removed. No current code path seems to create orphans, since bindings are `ON CONFLICT DO NOTHING`. Say this in the comment rather than "exactly as the full sweep."
- **Hard-to-read condition** (`accounting_retention_atomic.rs:256-260`). A side-effecting `expire_batch_on_connection` sits inside `!(a && b && c)` within an `if let` chain. Split it into explicit steps.
- **Docs that overstate:**
  - The incremental module doc says "the checkpoint is left alone… complete only once nothing is due", and the commit message says the checkpoint moves "only once nothing due is left". Both are true of batches but not of the write transaction, which moves it with leftovers due after the hour began.
  - `maintain_for_write`'s doc says the write "only advances the checkpoint", but it can also run one batch, or the full sweep.
  - The "9-17 s" and "10-17 s" figures disagree across comments.
- **Residual hour-turn window.** If the UTC hour turns between `prepare_write` returning and the write's `BEGIN IMMEDIATE`, the write still runs the full sweep under the lock. It's rare; note it in the `prepare_write` doc.
- **Extra read transaction per write.** `expire_due` opens a second read transaction (and schema validation) on every write. It could share `validate_hour`'s snapshot.
- **Naming:** `EXPIRY_BATCH` could be `pub(super)`; `interrupted` reads better as `provisional_lease`.
- **`busy_retry` is otherwise correct.**
  - It retries only codes where `code & 0xff == 5`, i.e. SQLITE_BUSY and its extended codes.
  - Every wrapped write is one transaction (or one statement) that rolls back whole, so repeating it is safe, including the 429 counter and the lease.
  - Switching the read-then-write transactions to `BEGIN IMMEDIATE` is right, since a deferred transaction can hit `BUSY_SNAPSHOT`, which the busy handler doesn't wait out.
  - Not retrying `PoolTimedOut` is deliberate; say so in a comment.

## Lock behaviour (question 3)
In normal operation, no path holds the write lock for work proportional to the ledger, apart from the cheap scans in Minor 7. Explicit `maintain`, delete and import still run the full sweep, by design. The rare fallbacks are a missing validation or the hour-turn window. `prepare_write` always finishes: `AsOf::At` takes one pass, and `AsOf::Now` repeats only when the UTC hour changes, revalidating on the read snapshot outside the lock. The pause between batches works in practice; Minor 1 covers the timing risk.

**Verdict: approve with nits.** Fix Minor 1 (jitter or a ≥110 ms pause) and Minor 5 (test runtime) before merge; the rest can follow.