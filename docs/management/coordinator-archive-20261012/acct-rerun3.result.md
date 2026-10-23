Every item in this re-run passes, but this is not a clean bill for PF-60-S03: a separate code-blind reviewer called the record "supported with corrections", and S03's own acceptance blockers are still open. I relaunched from the dead run's state. All test runs were already finished, and its README had been written outside the worktree, so I moved it in. Then I ran the reviewer, applied its corrections, and merged the record as docs-only **PR #328**.

The build is `origin/main` `ccc38bfc03`, which is the #322 merge, so **#322 and #318 are both included**. I used real GLM 5.2 calls on `zai`: a default and a developer macOS build, plus a developer Linux build on the RTX box for the faked-clock cases. Every displayed total matched my own recomputation from provider-reported usage. The reviewer found no number mismatch either.

| Item | Verdict | Evidence |
| --- | --- | --- |
| 1. #289 R1/R1b/13b (default build) | **PASS** (2 sub-items NOT VERIFIABLE) | `/usage requests`, the past day 2026-10-07 and the future day 2026-10-09 each print only "Per-request cost history is not part of this build.", with no page and no next step. `/usage` says "Sign in with ChatGPT to view OpenAI account usage." and `/cost` is unrecognized. The signed-in `/usage` menu and the usage error text #318 changed can't be seen in a signed-out test home. |
| 2. #289 R2 (developer build) | **PASS** | The week and month buckets that reach today are titled "Cost so far — this conversation" ($0.015922, counted in the range total, no next step). The week after today reads "Not started yet". |
| 2. #289 R3 | **PASS** | The range header and the day pages both say "daily totals kept since 2025-10-09". |
| 3. #308, delete with the clock behind | **PASS** (delete succeeded, state consistent) | With the clock about 57 s and 6 days behind the checkpoint, both deletes exit 0 and remove the session. `resume` then says "No saved session found" and the other sessions are untouched. Today's page says "Deleted conversations or subagents sent 4 request attempts on this day" (2 + 2), and the faked day 2026-10-02 is empty. The "fail and leave it intact" branch never happened, so it wasn't exercised. |
| 4. Regression 8a | **PASS** | The no-usage conversation says "check the bill from zainousage". |
| 4. Regression 9 | **PASS** | Deleting a conversation and its subagent on macOS leads to the page reporting "6 request attempts", which matches the 6 requests it made. |
| 4. Regression 15 | **PASS** | A turn 58 s behind real time completed with all 7 requests recorded: 50,427 tokens, exact USD 0.01776348, matching my recomputation. |

I commented on #287 about a known leftover that still reproduces here (residual 2). While the clock is behind, the warning says requests are recorded "at the ledger's time", but they are actually recorded at the behind process time. It's low severity, and #287 stays closed. No new defect was found. The scratch build directories and test homes on the Mac and the RTX box, plus the worktree, are removed.

**Recommendation for Travis:** On `ccc38bfc03` every defect this re-run targeted is fixed: #289 R1, R1b, R2, R3, criterion 13b, and #308. If those were the only blockers, the functional evidence supports accepting PF-60-S03. It is not unconditional, because this re-run does not clear the blockers recorded in `acct-acceptance-75/acceptance.md`:
- **Scope-zero finding (P2):** still open, with no fix or accepted waiver, so shipping `/usage requests` stays on hold.
- **Round-66 test timeout:** the intermittent failure is still unexplained.

Those need your fix-or-waive decision. Smaller leftovers still present or not re-tested:
- #287 residual 2 (reproduced here); residuals 1 and 3 were not re-tested.
- #288 item 3: custom providers labelled "Pay per use".
- #289: a 401-rejected attempt is counted as "billed", and "Partial — excluded" appears on past buckets (#318's follow-up).
- The signed-in `/usage` menu and the usage error text are unseen.
- Criterion 15 is still unverified on macOS.
- #290 is open but outside S03.