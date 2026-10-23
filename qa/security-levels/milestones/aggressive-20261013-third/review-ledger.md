# Review ledger: milestone "Aggressive ships" third run

Budget: default maximum 5 (root `AGENTS.md`), counted from 0 for this record.

| # | Kind | Agent / session | Model | Isolation | Result |
| --- | --- | --- | --- | --- | --- |
| 1 | Amendment review + new-case design, pass 1 | `01a135bd-4733-7751-9da5-0f3f7a9fd13a` | claude-opus-5-5-plan, high | instruction-only, read-only sandbox, packet directory only ([access record](design/designer-session.md)) | changes to 6 rows, 4 missing amendments, 8 new cases, 7 questions |
| 2 | Same session, confirmation pass | same | same | same | agreed with v2 except AGG-32 (B), AGG-27, AGG-22 (applied in v3); withdrew AGG-T05, revised T07/T08; final table frozen at `090bd128a6` after the owner's answers |
| 3 | Independent evidence check r1 (fresh session) | `01a13814-f755-7283-8b0b-923ebd09c91b` | claude-opus-5-5-plan, high | instruction-only, read-only sandbox, record copy only (`git archive` of commit 07f9ff0155) | **fail**: AGG-R09 to passed (G24 never scrolled to Recent denials; #738 closed), AGG-18 to blocked, AGG-14/24/R14/R07/26/R04/10 summaries, README lists and limits, missing review/ and R06 evidence ([r1](review/evidence-check-r1.md)) |
| 4 | Independent evidence check r2 (fresh session) | `01a13822-5daf-7ae2-bf9b-76803ea72cbf` | claude-opus-5-5-plan, high | same as r1 (`git archive` of commit 1f2a9adb54) | **fail**: AGG-R07 and AGG-R10 to blocked (same rule as AGG-18), AGG-38 deviation disclosed, AGG-28 relay result, AGG-R07 summary, README run-2 comparison and limits (token exposure, window overlap, in-attempt retries, screen times) ([r2](review/evidence-check-r2.md)) |
| 5 | Independent evidence check r3 (fresh session), last within budget | `01a13831-7438-72d2-b70d-2c0b821fcb4c` | claude-opus-5-5-plan, high | same as r1 (`git archive` of commit db7d18a711) | **fail**, four mechanical points: AGG-R08 disclosure missing, replay reasons, token-exposure list, VM stop evidence; every disposition otherwise supported ([r3](review/evidence-check-r3.md)) |
| 6 | Budget extension (integrator, under the September 12 delegation): independent evidence check r4, scoped to verifying r3's four corrections and that results.json/dispositions still match | see r4 row | same | same | authorized because r3 found only mechanical points and the record should end on a verified check; prior usage 5 of 5, allowance +1; result in row 7 |
| 7 | Independent evidence check r4 (fresh session), extension | `01a1383e-5e8b-7913-99f5-15191dab76f3` | claude-opus-5-5-plan, high | same as r1 (`git archive` of commit 56af2ed1fa) | **fail**, two README wording points (AGG-19 wait listed as an executor deviation; token limit named r3's transcript); confirmed r3's four corrections and the record's consistency. Both points corrected afterwards, **not independently verified** ([r4](review/evidence-check-r4.md)) |

Budget: 5 of 5 used plus one integrator-authorized extension (row 6); Travis may confirm or overrule the extension.

Execution sessions are test work, not reviews: 37 executor sessions (32 groups, 5 replays), all with the candidate's
`corbanu exec` on GLM 5.3 Flash, one at a time, 2026-10-13 17:18Z to 2026-10-14 01:37Z.
