The shared-DB contention fix is merged: PR #538 (`3c70eda13a`), and the merge auto-closed #359 and #324; I added the evidence to both as comments. With 10 sessions on a copy of the 578 MB DB, all 150 requests were recorded and none failed, and no wait over a second was on a request's path.

**What held the lock**
- **Ledger writes:** each held the lock 0.3–0.9 s, up to 11 s, 4–6 times per request. The owner's day and the due expiries were found by parsing all 61k attempt payloads under the lock, and dev builds ran SQLite's C code unoptimized.
- **Hourly validation:** every process's first request each hour spent about 75 s validating the whole ledger.
- **Deletes (#324):** a full sweep per owner, run twice because of the #308 preflight.
- **Metadata and throttle (#359, #351):** these then waited out the 5–10 s busy timeouts behind the above.

**Fix**
- **Indexes:** two lookup indexes on the ledger's attempts table make every ledger write hold the lock for milliseconds. They are created at runtime, not by a ledger migration, so older builds sharing the home keep working. On a copy of the live DB, creating them held the lock 154–240 ms and grew the file by 4 MB; main's binary then ran and recorded normally on that DB.
- **Ledger writer:** each process has one. A request saves its record to a pending folder in the home, waits at most 500 ms (1 s cap) and only while nothing is ahead, then goes on. Records are never dropped for contention or disk errors, and a later process writes anything a dead one left behind.
- **Validation:** one process at a time runs the hourly validation (now 28 s, after raising optimisation for SQLite and the state crates in dev builds).
- **Metadata (#359):** waits at most 500 ms; while the DB is known busy it is held and written with the next update.
- **Throttle:** waits about 1 s instead of 10 s.
- **Deletes (#324):** one bounded maintenance pass per delete.
- **Instrumentation:** every write transaction now logs its wait and hold with the call site.
- **Ledger location:** it stays in the state DB. A separate file would break atomic deletes and the owner-row check, mean moving 540 MB, and leave older builds writing the old location.

**10-session load test** (copy of the live DB, mock provider, home already warmed up)

| | Before | After | After, with another process holding the lock 20 s |
| --- | --- | --- | --- |
| Tool response → next request, p99 / max | 40.9 s / 49.9 s | 216 ms / 239 ms | 571 ms / 573 ms |
| Lock wait p99 / max | 5.17 s / 5.46 s | 101 ms / 178 ms | 5.4 s / 5.4 s |
| Lock holds over 250 ms | 469 | 0 | 0 |
| Requests recorded | 147 / 150 | 150 / 150 | 150 / 150 |
| busy/locked/failed log lines | 144 | 0 | 29 |

All waits over a second in the held-lock run are the background ledger writer retrying, not requests. That run's 29 log lines are throttle retries and fallbacks.

There is also a CI regression test that runs 10 runtimes against a 3 s held lock. It asserts request-wait p99 under 1.5 s, max under 2.5 s, no failures and every record written.

**Gate**
- **Tests:**
  - **Passed:** codex-state 432/432, thread-store 162/162, exec 148/148, app-server delete/accounting 11/11.
  - **codex-core:** focused tests passed 411/413 with developer-accounting and 407/408 without. The only failures are `remote_compact_trim_estimate…` and `token_budget_auto_compact_fallback…`, which #299 records as also failing on main; neither touches the state DB.
- **Clippy `-D warnings`:** clean on macOS and on glitch, with and without developer-accounting.
- **Review:** Opus 5.5 High read-only review, three rounds. Round 1 requested changes (one blocker, eight should-fix), all fixed; rounds 2 and 3 approved, and I fixed round 2's four remaining items too.
- **tmux check:** 8 TUI sessions on GLM 5.3 Flash, disposable home, developer accounting on, `CORBANU_TEST_NO_NATIVE_KEYRING=1`, key fetched inline per session. All 8 finished with no busy, locked or accounting lines in any pane or the log DB. The ledger has 32 attempts and 32 observations across the 8 conversations, nothing left pending.
- **CI:** all 40 checks passed before merging with `--match-head-commit`. The Cargo.toml change triggered the full v8-canary run, which took about 3 hours.
- **No demo videos:** this brief's gate didn't list them.

Evidence, the load-test harness and the tmux script are in `qa/reliability/shared-state-db-contention-20261011/`.

**Cleanup:** the worktree, the 145 GB target dir, all DB copies and binaries, and the glitch checkout and target are removed. Only about 6 MB of logs and review notes remain in `.codex-work/db-contention-20261011/`. The local branch wasn't deleted because the local main hasn't pulled the merge; nothing was force-deleted.

**Open**
- **#549 (new):** ten processes launched at the same moment on a brand-new home race to create the goals/memories DBs, and the losers run without a state DB. Main does this too; homes already in use aren't affected.
- **Hourly validation cost:** still about 28 s per hour in a debug build on this ledger. Requests don't wait for it, but an `exec` that happens to be validating can take up to 120 s to exit, once an hour. Making it incremental would mean changing the ledger design.
- **Lock held through shutdown:** a thread's held metadata is retried once at shutdown. If another process holds the lock through that retry, the update is logged and left to the rollout, which stays the record of truth. This happened for 10 threads in the 20 s held-lock test.