# Review ledger: acc-tart1 (PF-60 follow-up live run)

Budget: 5 rounds in total for design and evidence checks (acc-tart1 brief; wave2-common REVIEW BUDGET rule: an
extension needs Travis's or the coordinator's authorization through `acc-tart1.request.md`; none self-authorized).

| # | Kind | Agent / session | Model | Isolation | Result |
| --- | --- | --- | --- | --- | --- |
| 1 | Code-blind design (fresh designer) from the intent packet | `01a156fc-532e-7ee0-a6c0-0096f4ca3b19` | claude-opus-5-5-plan, high | instruction-only, read-only sandbox, packet directory only | 24 cases ([design/designer-output.md](design/designer-output.md), [designer-session.md](design/designer-session.md)) |
| 2 | Independent design review against intent, owner decisions and harness facts | `01a15705-7322-7e62-8fba-775f047cca89` | claude-opus-5-5-plan, high | instruction-only, read-only sandbox, review packet only | **CHANGES**, 25 amendments (new case FUP-25), applied as written, not re-reviewed ([design/review-output.md](design/review-output.md), [review-session.md](design/review-session.md)) |

Budget at the freeze: 2 of 5 used; 3 remain for independent evidence checks.

| # | Kind | Agent / session | Model | Isolation | Result |
| --- | --- | --- | --- | --- | --- |
| 3 | Independent evidence check r1 (fresh session) | `01a157d1-376d-7cc2-aece-a2cdf576e998` | claude-opus-5-5-plan, high | instruction-only, read-only sandbox, record copy only (`git archive` of commit 0991967f0e) | **fail**. Confirmed: the freeze, packet identity, amendments, isolation (109 probes per group, 28 receipts), caps, key deletion and the totals. Corrections: FUP-01's sum omitted the cache-read cost; FUP-16's sum carried FUP-10's Request 1 slip; FUP-10's Request 1 derivation and the skipped intermediate audits; FUP-17's navigation facts and the pty.raw source; FUP-14's code-derived claims labelled; FUP-02 passed -> blocked; FUP-03's sum wording; the shakedown's real OpenAI WebSocket (probable ~$0.029 warm-up; D16 total now stated as ~$0.063, over $0.05); screen archives (Z, O only); limits (designer and reviewer isolation, lock release and UTM end state, now in common/host-end-check.txt; corrections to the hash-bound policy go to common/effective-policy-addendum.txt). All applied ([r1](review/evidence-check-r1.md)) |
| 4 | Independent evidence check r2 (fresh session) | `01a157e2-17e7-7000-8edc-68542fa9b504` | claude-opus-5-5-plan, high | same as r1 (`git archive` of commit 52dea79698) | **fail**. Every r1 correction confirmed; the freeze, isolation, the 18/2/5 totals and all 25 dispositions confirmed. Corrections: FUP-05 cited the wrong screen file (now `group-Z/fup05-r2-attempt-40.txt`); results.json's review-budget count was stale; the shakedown's WebSocket opened about 75 ms *after* the realtime refusal, not before. Minor: the audit after FUP-09 was partial too; designer-session.md says eight screens, but there are nine. All applied ([r2](review/evidence-check-r2.md)) |
| 5 | Independent evidence check r3 (fresh session), the last within budget | `01a157f5-2b8a-7e23-9a71-35d0f97d6438` | claude-opus-5-5-plan, high | same as r1 (`git archive` of commit 4a9c4d44bf) | **pass**. Every r1 and r2 correction confirmed, along with the freeze, isolation, the 18/2/5 totals and all 25 dispositions; no correction required. Four optional clarifications were applied afterwards and **not independently verified**: FUP-15's fork ID and time labelled as the ledger read; FUP-02's wording ("request-detail history", daily totals kept since 2025-10-21); the Moonshot link seen on K3 Request 1's page only; FUP-10's audit window 03:47:30–04:04:09Z ([r3](review/evidence-check-r3.md)) |

Budget: 5 of 5 used; no extension requested. The last independent verdict is **pass**.
