S03 is **not accepted**. The cost arithmetic and attribution are exact, but three criteria fail on real runs. The evidence is merged as PR #292 (docs only, merge commit `a682edc61e`, all checks green).

I built current `origin/main` (`63ea3d0cbd`) with developer accounting on macOS and on the RTX box. I ran real GLM 5.2 sessions against three public repos, including subagents and concurrent `corbanu exec` runs, then checked `/cost` and `/usage requests`. I used Z.AI's published prices ($1.40 input, $0.26 cached input, $4.40 output per million tokens). Every displayed total, per-request amount and parent/subagent split matched my own recomputation to the micro-dollar. That held across restart and across day, hour, week and month boundaries. The QA cost about $0.65 at list price.

**Verdicts**

| Area | Verdict |
|---|---|
| Each total explainable from its requests; parent + subagents; same totals after restart; concurrent runs and subagents | PASS |
| Date ranges and boundaries; empty vs unavailable; missing price; narrow screen, mixed providers; `/cost` absent from a default build (Linux) | PASS |
| Deleted history | **FAIL** (#286) |
| Requests where the provider reported no usage | **FAIL** (#288) |
| Accounting failure handling | **FAIL** (#287) |
| Billed cost, Corbanu balance, comparison with a real provider bill | NOT VERIFIABLE (no settlement data, no access to the Z.AI bill) |

- **Deleted history:** after a session is deleted, the day screen says "No other conversation recorded requests on this day" on days where that session and its subagents really spent money. It's the same false day-wide-zero problem the earlier scope finding was about.
- **No usage reported:** these requests get no "check your provider's bill" step, and the only such line names a different provider. My test of the "no price available" mislabel was confounded, because the provider I used had no price anyway.
- **Accounting failure:** with developer accounting on, if the clock is behind the ledger's last write (I tested 58 seconds and 6 days), the turn fails before anything is sent. The error says "Native Anthropic accounting failed" on a Z.AI request. It recovered once the clock caught up, and it is probably inherited from S02.

**Other issues filed:**
- Wording inconsistencies in `/cost` (#289).
- `corbanu exec --json` reports zeros when usage is unknown and leaves out subagent usage (#290). It's outside S03 code, so I recorded it as an observation, not a FAIL.

**Review:** one reviewer (Opus 5.5 high, read-only, given only the evidence, not the source) concluded "supported, with corrections". It recomputed the main numbers with no discrepancies. I applied its corrections: the exec JSON row dropped from FAIL, #287's scope narrowed after I tested recovery, and the gaps it found disclosed. Those gaps are retries, user cancel, the busy-host refusal and similar paths that weren't exercised.

**Process notes:**
- Built binaries never touched the real keychain; every run used a disposable home with `CORBANU_TEST_NO_NATIVE_KEYRING=1`. A key scan of the evidence found nothing.
- One rule deviation, flagged by the reviewer: my run script held the Z.AI key in a shell variable instead of passing it straight to the command. It was never printed or written to disk.
- One of my capture steps accidentally forked a conversation; no model call was made, and I recorded it.
- I didn't touch acct-s03a's files.
- After merging I removed my build directory, worktree, RTX scratch directory and temp files.

**For Travis:**
- **#286 needs a decision.** Either keep deleted sessions' spend as an anonymous "deleted conversations" line, or keep removing it and say so on every day and range page. I recommend keeping the anonymous line, since the money was spent.
- **#287** should be fixed before developer accounting is turned on more widely.
- Still not covered: billed cost and Corbanu balance, a real provider bill, Windows, and a macOS default build.

Everything is in `qa/portfolio/agent-cost-accounting/pf-60-s03/independent-acceptance-20261008/`; start with `README.md` and `REVIEW.md`.