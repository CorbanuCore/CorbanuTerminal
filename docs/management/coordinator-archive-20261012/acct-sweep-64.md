# Worker brief: accounting follow-ups after acct-failure-63 (incremental expiry sweep; other DB writers)

You are an Opus 5.5 High worker. Work autonomously; do not ask questions.

Read first: `/Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/acct-failure-63.md` and `.result.md` (same
directory). The fix landed on `integrate/management-workstreams-20260911` at `79aef0daf3` (commits `f4d6c5888a`,
`a956468dad`, `7bef7a8dfd`, `79aef0daf3`).

## Do
1. **Day-90 expiry sweep (deadline ~2026-12-21).** When ledger records start expiring (90-day retention), the hourly
   path falls back to the full sweep under the write lock (~9–17 s on the live-sized DB), which starves every other
   writer of the shared state DB. Make expiry incremental: bounded batches in short write transactions (e.g. delete /
   re-checkpoint N expired rows per transaction, yielding the lock between batches), with validation preserved
   (corrupted data still fails visibly) and idempotent progress across crashes/restarts. Test it with a synthetic
   ledger that crosses the 90-day boundary (use the existing fixture style; clock injected, no sleeping), including a
   concurrent writer that must never wait more than ~1 s, and a crash mid-sweep.
2. **Other state-DB writers.** Writes outside accounting still give up after SQLite's 5 s busy timeout. Find the
   state-DB writers that surface "database is locked" to users (e.g. `failed to check provider request throttle
   state: ... database is locked`, which Travis hit) and give them the same treatment: short transactions plus a
   bounded busy-retry on SQLITE_BUSY only, with logging. Keep the change surgical; add a test that holds the lock
   for longer than 5 s and shows the writer now succeeds.
3. **Optional, only if low-risk:** optimise dependencies in the dev profile (e.g. `[profile.dev.package."*"]
   opt-level` for sqlite/serde) if it does not change the debug-only accounting checks; measure before/after.
4. Fresh worktree `/Volumes/CorbanuDrive/Corbanu/worktrees/acct-sweep-64` off the integration tip. `just fmt`, the
   `codex-state` and `codex-core` accounting suites, clippy for touched crates.
5. Independent review with `~/.local/bin/corbanu exec -m claude-opus-5-5-plan -c model_provider="claude-plan" -c
   model_reasoning_effort="high"`; fix findings.
6. Push fast-forward to `integrate/management-workstreams-20260911` (rebase onto the tip first; never force). Don't
   work in `worktrees/management-workstreams-20260911`.
7. Rebuild and install locally and on RTX exactly as acct-failure-63 did (SOP in its brief), keeping the previous
   binary as `corbanu.prev-79aef0daf`.

## Never
Never print secrets. `rm -f` is blocked by policy. No force-push. Don't edit the live state DB by hand.

## Final report (your last message)
What changed (commits), tests before/after, review verdict, build/install receipts, and anything still open.
