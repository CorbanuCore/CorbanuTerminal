You are a reliability bug-fix worker. Read /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md
(binding: keychain rule, disk hygiene, per-fix gate) and acct-fix1.result.md in that folder (how #299 made accounting fail
open when the state DB is locked or the clock is behind).
Fix GitHub issue #351 (`gh issue view 351`): when the shared state DB (`pfterminal_state_5.sqlite`) is busy for more than
about a minute (another process holds a write transaction — e.g. several Corbanu sessions/workers sharing one home) or is
read-only, the provider-request throttle check ends the whole turn with
`Fatal error: failed to check provider request throttle state: error returned from database: (code: 5) database is locked`
(or code 8 read-only). Travis has hit exactly this error in real use. Required end state:
- A busy or read-only state DB never kills a turn through the throttle check. Decide, and document in code, what the safe
  behaviour is: the throttle exists to protect provider rate limits/spend, so fail-open must not let a runaway loop
  bypass a configured hard limit. Prefer: wait briefly (bounded, honouring interrupts), then proceed with an in-memory
  per-process throttle as a fallback for that request, log a warning, and show one plain user-facing note (once per
  session) that the shared state database was busy. If a configured limit is a hard spend cap, keep it enforced from
  in-memory state rather than silently dropping it. Read-only DB: same fallback with a clear one-time message.
- Check all other per-request state-DB touchpoints on the request path (thread metadata, rate-limit snapshots, etc.) for
  the same failure mode and fix them consistently (best effort, logged), without changing what is persisted on success.
- Regression tests: hold an exclusive SQLite transaction on the state DB for longer than the wait (use the existing test
  helpers / an injected busy handler rather than wall-clock sleeps where possible), and make the DB read-only; a turn with
  several requests completes in both cases; the throttle still limits within the process.
Gate: focused tests (codex-state, codex-core, app-server as touched) with and without `developer-accounting`; Linux
clippy -D warnings (the RTX box ssh ambient@100.99.88.49 may be offline — if so rely on PR CI's Linux clippy and say so);
one independent Opus 5.5 High review via installed `corbanu exec` (read-only); a functional check with the freshly built
debug binary in a disposable home (CORBANU_TEST_NO_NATIVE_KEYRING=1, GLM 5.2 on zai with
`ZAI_API_KEY="$(corbanu vault auth-helper provider/zai_api_key)"` on the consuming command) while another process holds
the DB lock for 2 minutes — the turn must complete. New worktree off origin/main under /Volumes/CorbanuDrive/Corbanu/worktrees/.
PR to main; merge only when ALL checks are green, never with checks pending (`gh pr merge N --merge --match-head-commit <sha>`;
never force-push; never rm -f). Close #351 with evidence. Remove your build target when done. Report as in sec-common.md.
