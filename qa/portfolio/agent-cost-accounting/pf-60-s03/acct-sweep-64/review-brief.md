# Independent code review: incremental retention expiry + busy retry (acct-sweep-64)

You are a reviewer. Do NOT edit any files, commit, or push. Read-only review (rg/sed/git only; you may run
`cargo test -p codex-state --lib <filter>` if you need to, but do not modify the tree).

Repo worktree: /Volumes/CorbanuDrive/Corbanu/worktrees/acct-sweep-64 (branch acct-sweep-64), base 79aef0daf3,
2 commits (804cc6ed21, 42786b24ba). Full diff: /Volumes/CorbanuDrive/Corbanu/.codex-work/acct64/review.diff.
Read the commit messages (`git log -2`) for intent and measurements.

Context: the state SQLite DB is shared by several Corbanu processes (TUI, exec workers). Accounting keeps a raw
per-request ledger with 90-day detail retention and 365-day aggregate/replay retention; the full retention sweep
(`maintain_on_connection`) validated and priced every attempt under BEGIN IMMEDIATE (9-17 s on the live ledger),
starving other writers past SQLite's 5 s busy timeout. The previous round (acct-failure-63) moved the hourly
whole-ledger validation to a read snapshot but kept the full sweep under the lock whenever anything was due.
This round: (1) expiry is applied in bounded batches (`state/src/runtime/accounting_retention_incremental.rs`,
wiring in `accounting_retention_atomic.rs` and `accounting_store.rs`); (2) a bounded SQLITE_BUSY retry
(`state/src/runtime/busy_retry.rs`) for the provider throttle writes, plus `or_cancel` on the turn's throttle
calls in `core/src/session/turn.rs`.

Review for:
1. Ledger semantics: does `expire_batch_on_connection` apply exactly what `maintain_on_connection` +
   `reduce()` would (compact-day folding incl. pre-existing compact days, day-365 expiry not resurrecting a
   raw-only day, tombstones, snapshot GC, contribution validation)? Any record it could miss or double-apply?
   Is it safe that batches commit before the checkpoint moves (reads/inspection at the old checkpoint,
   `read_retained_on_connection`, `/cost` inspection)? Any crash/restart ordering hole?
2. Validation contract: corruption must still fail visibly. Is anything now applied without having been
   validated (validate_hour on snapshot, `validated_this_hour` via checkpoint >= hour_start, per-record checks)?
   Is the relaxed write-path rule (leftover due-after-hour-start records left as checkpoint lag; full sweep
   only if something due before the hour began is left) sound?
3. Lock behaviour: can any path still hold the write lock for O(ledger) work in normal operation? Is the
   inter-batch yield (sleep = batch hold, clamped 10-100 ms) enough for SQLite's busy-handler waiters? The
   `prepare_write` hour-turn loop: termination, AsOf::At in tests, AsOf::Now.
4. busy_retry: only SQLITE_BUSY retried? Are the wrapped writes really safe to repeat (lease acquire, 429
   counter in record_result Failed path)? BEGIN IMMEDIATE change. The core or_cancel + provisional lease guard
   (release by owner on interrupt): correctness, leaks, misleading logs.
5. Tests: do they prove the claims (parity with the full sweep, crash mid-sweep, concurrent writer < 1 s,
   corrupt record, > 5 s lock)? Flakiness risks (timing asserts, 2,000-attempt fixture runtime)?
6. Anything else: naming, comments that no longer match the code, dead code, clippy-style issues.

Report findings as Blocker / Major / Minor / Nit with file:line and a concrete suggestion, then a verdict
(approve / approve with nits / request changes). Write the final report as your last message.
