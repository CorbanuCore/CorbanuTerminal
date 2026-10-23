I fixed #351 in PR #358 and merged it (merge commit `c0e242c1a9`) after every check was green. A busy or read-only state database no longer ends a turn through the provider request throttle.

**What changed**
- **Wait, then carry on:** each throttle step waits about 10 s for the database. After that it decides from this process's own copy of the throttle state. After a failure, the database is left alone for 60 s so later requests don't wait again.
- **Limits still hold:** 429 cooldowns this process recorded or read still hold requests back, and sub-agents in the process still take the lease one at a time. The throttle has no configured spend cap; the only cap-like limits are those cooldowns and the lease. What's lost while the database is unwritable is coordination with other Corbanu processes. This is documented in the code.
- **No stale leases:** a database lease that can't be released is released by the next step that reaches the database, so it can't block other processes for the 10-minute expiry. Recording a result is now interruptible, and an interrupt keeps the 429 cooldown.
- **One note per session**, naming the state database and whether it is busy, read-only or can't be written, plus a warning in the log.
- **Normal operation is unchanged:** while the database answers, the same writes happen as before.
- **Other touchpoints:** I checked the rest of the request path for the same fatal error. Thread metadata, mailbox updates and memory marks already fail softly and are logged; rate-limit snapshots aren't stored in the state database; accounting was fixed in #299.

**Gate evidence**
- **Tests:** codex-state, 402 passed. codex-core focused suites plus all library tests passed with and without `developer-accounting` (207 and 212 focused; 2,893 and 2,898 library). New regression tests cover:
  - another connection holding the lock past the wait;
  - a read-only database;
  - the queued lease release;
  - an interrupted record keeping its 429;
  - a 3-request turn completing in core, both busy and read-only.
- **Live check:** freshly built debug `corbanu` with developer accounting, in a disposable home with `CORBANU_TEST_NO_NATIVE_KEYRING=1`, on GLM 5.2 via zai.
  - An exclusive lock was held from 16:15:20 to 16:17:20Z. The 3-request turn ran from 16:15:28 to 16:17:22Z and completed, with one busy note and no fatal error.
  - With the database file read-only (`chmod a-w`), the turn completed in 6 s with one read-only note.
- **Review:** Opus 5.5 High through the installed `corbanu exec`, read-only. Round 1 asked for changes (two Major findings); I fixed those and the round 2 and 3 findings, and the final verdict was approve.
- **Linux clippy:** the RTX box was offline, so I relied on PR CI's Linux clippy, which passed.
- **CI:** 27 passed, 9 skipped. One run of the macOS credential canary failed in an unrelated `codex-network-proxy` broker test, and a Windows argument-comment lint failure from my code is fixed. Every other check was green at merge.
- **Cleanup:** #351 has an evidence comment. My build target, env file, worktree and disposable home are removed. Logs and review rounds are in `.codex-work/workers-20261002/fix-351/`.

**Still open**
- **#359 (new):** while the database is held, each thread-metadata write waits SQLite's 5 s busy timeout. A long lock makes turns slow, about 15 s extra per request, but they finish. Developer accounting adds its own ~16 s per request (#299).
- **Slow test:** the new core test that holds the lock takes about 60 s because of those metadata waits. I gave it a nextest timeout override.
- **Coverage gap:** the lease path for sub-agents is tested in codex-state, not end to end in core. Setting up a sub-agent on a third-party provider with a large uncached request is heavy.
- **Known minor:** a concurrent step can briefly see a queued stale lease before it's released. The reviewer accepted this as is.