# Review ledger: milestone "Aggressive ships" re-run

Budget: default maximum 5 (root `AGENTS.md`), counted from 0 for this record.

| # | Kind | Agent / session | Model | Isolation | Result |
| --- | --- | --- | --- | --- | --- |
| 1 | Amendment review + new-case design, pass 1 | `01a12e74-b702-7402-b93d-63d03a499d02` | claude-opus-5-5-plan, high | instruction-only, read-only sandbox, packet directory only ([access record](design/designer-session.md)) | corrections to 11 amendment rows, 3 missing amendments, 18 new cases, 9 questions |
| 2 | Same session, confirmation pass | same | same | same | agreed with v2 except one AGG-22 clause (removed); revised R01, R03, R04, R05, R07, R08; frozen at `4cf4083227` |
| 3 | Independent evidence check r1 (fresh session) | `01a1302b-3ad0-7850-860b-a0dadecdee00` | claude-opus-5-5-plan, high | instruction-only, read-only sandbox, record copy only | **fail**: AGG-25, AGG-27, AGG-32 to failed, AGG-38 to blocked, AGG-01/33/R03 summaries corrected, disclosures and limits added, missing files ([r1](review/evidence-check-r1.md)); led to #626 |
| 4 | Independent evidence check r2 (fresh session) | `01a13038-a3e5-76f3-a5f4-bb5ec711ab1e` | claude-opus-5-5-plan, high | same | **fail**: AGG-01 to failed, AGG-20 and AGG-R14 to blocked, AGG-14 and six smaller disclosures, README limits ([r2](review/evidence-check-r2.md)) |
| 5 | Independent evidence check r3 (fresh session), last within budget | `01a13044-a44c-7fe2-9062-6bd8970aa2c3` | claude-opus-5-5-plan, high | same | **fail**, mechanical only: every disposition supported; isolation receipts' policy hash broken by an appended note, AGG-20 model field, README G26 ([review/evidence-check.md](review/evidence-check.md)); corrected afterwards, **not independently verified** ([review/r3-corrections.md](review/r3-corrections.md)) |

Execution sessions are test work, not reviews: 34 executor sessions (28 groups, 6 replays), all with the
candidate's `corbanu exec` on GLM 5.3 Flash, one at a time, 2026-10-12 05:06Z–12:49Z.

## Budget

All 5 reviews are used. A sixth evidence check, to verify the r3 corrections, needs a budget extension from the
integration owner or Travis; none was granted, so the record's last independent verdict is **fail** (mechanical).
