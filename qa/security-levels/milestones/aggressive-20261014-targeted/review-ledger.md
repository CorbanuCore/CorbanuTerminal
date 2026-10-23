# Review ledger: milestone "Aggressive ships" targeted re-run

Budget: default maximum 5 (root `AGENTS.md`), counted from 0 for this record. Binding rule of 2026-10-14 (#447,
wave2-common): going over the budget needs a major defect AND Travis's explicit authorization through the coordinator;
no self-authorized extensions.

| # | Kind | Agent / session | Model | Isolation | Result |
| --- | --- | --- | --- | --- | --- |
| 1 | Amendment-4 review against the decision comments only | `01a13954-bc1d-7d31-97e1-699adfdceda2` | claude-opus-5-5-plan, high | instruction-only, read-only sandbox, review packet only ([design/review-packet/](design/review-packet/), commands in [design/amendment-review-events.jsonl.gz](design/amendment-review-events.jsonl.gz)) | **CHANGES** with a final table: decision citations, AGG-25 part B wording, AGG-R10 expectation, "web search" removed from the #662 rows, AGG-04 added expectation tightened; no missing amendment; final table applied verbatim and frozen at `40e36a39cf` |
| 2 | Independent evidence check r1 (fresh session) | `01a13a3a-a529-76a1-aed6-c4b50e57cb9e` | claude-opus-5-5-plan, high | instruction-only, read-only sandbox, record copy only (`git archive` of commit 5d165941bb) | **fail**: AGG-R07 to blocked (model-decline rule read strictly as for AGG-18/R10), AGG-R03 reason, limits (lane processes carry over, relay-window overlap, leftover admin scripts), host-check binary, freeze time, AGG-R10 extra cat .env, frozen-file citation; every other disposition supported ([r1](review/evidence-check-r1.md)) |
| 3 | Independent evidence check r2 (fresh session) | `01a13a46-c095-7bd2-8af4-e1391424b87c` | claude-opus-5-5-plan, high | same as r1 (`git archive` of commit 7e833f9d0c) | **fail**: confirmed r1's corrections and the four model-decline blocks; AGG-37's stale-banner result untested (replayed as Q02c), AGG-R14's undisclosed 11:27:52Z approval of the child's skill-file read, unlabelled VM end-state lines, AGG-R07 wording; noticed Grant once offers on child prompts (filed as #789) ([r2](review/evidence-check-r2.md)) |
| 4 | Independent evidence check r3 (fresh session) | `01a13a5d-2390-76e3-96cd-7b9f65cd8fa7` | claude-opus-5-5-plan, high | same as r1 (`git archive` of commit fd4b0e66ab) | **fail**: confirmed r1/r2 corrections; AGG-04 to failed (inspector Vault row, #792), AGG-37 to blocked (readable refusals untested in a valid attempt), Q02c empty-profile launch disclosure, AGG-R14 protected-path label, guest-state-start-2 label, Q02c relay wording ([r3](review/evidence-check-r3.md)) |
| 5 | Independent evidence check r4 (fresh session), last within budget | `01a13a65-4759-7432-bd30-a190f6b62b2e` | claude-opus-5-5-plan, high | same as r1 (`git archive` of commit 7d052905bc) | **fail**, disclosure points only: AGG-R14 Q11b setup deviation (Esc, agms-tui stop, `start r`), README description of Q02c's empty-profile launch; every disposition, count, the freeze and isolation confirmed. Both corrected afterwards, **not independently verified** ([r4](review/evidence-check-r4.md)) |

Budget: 5 of 5 used, no extension (binding rule of 2026-10-14: an extension needs a major defect and Travis's explicit authorization; neither applies).

Execution sessions are test work, not reviews: 17 executor sessions (14 groups, 3 replays), all with the candidate's
`corbanu exec` on GLM 5.3 Flash, one at a time, 2026-10-14 08:10Z to 11:35Z and 12:10Z to 12:19Z.
