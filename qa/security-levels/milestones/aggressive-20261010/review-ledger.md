# Review ledger: milestone "Aggressive ships"

Budget: default maximum 5 (root `AGENTS.md`), counted from 0 for this milestone record.

| # | Kind | Agent / session | Model | Isolation | Result |
| --- | --- | --- | --- | --- | --- |
| 1 | Independent code-blind design | `01a12807-f56f-7531-a8b9-b7e909b7e460` | claude-opus-5-5-plan, high | instruction-only, read-only sandbox, packet directory only ([access record](design/designer-session.md)) | 38 cases + 13 questions, frozen in `design.json` at commit `d566fb4e06` before any execution |
| 2 | Independent evidence check r1 | `01a12974-ae6e-7a20-8adb-6854d7e3ac71` | claude-opus-5-5-plan, high | instruction-only, read-only sandbox, record copy only (it ran `ls ../` once, listing names only) | **fail**: six dispositions or claims corrected ([r1](review/evidence-check-r1.md)) |
| 3 | Independent evidence check r2 (fresh session) | `01a12981-9146-7363-b414-20df7aa48b34` | claude-opus-5-5-plan, high | same (record copy only) | **fail**: AGG-22 corrected, wording and record fixes ([r2](review/evidence-check-r2.md)) |
| 4 | Independent evidence check r3 (fresh session) | `01a12993-7e0b-7f00-9d1c-24781baca7bb` | claude-opus-5-5-plan, high | same | **fail**: grant-reachability claim narrowed, AGG-21 raw verdict fixed, wording ([r3](review/evidence-check-r3.md)) |
| 5 | Independent evidence check r4 (fresh session) | `01a129a2-fb80-7d72-9f4b-e8e81a1522e3` | claude-opus-5-5-plan, high | same | **fail**: AGG-20 S → blocked (timing confound), notes ([r4](review/evidence-check-r4.md)) |
| 6 | Independent evidence check r5 (fresh session), **budget extension** | `01a129b1-b584-7322-8fbf-629092303562` | claude-opus-5-5-plan, high | same | **fail**: three reasons corrected from product logs, no result changed ([r5](review/evidence-check-r5.md)) |
| 7 | Independent evidence check r6 (fresh session), **second budget extension** | `01a129c0-ac80-7d52-a60c-7caa66d15b36` | claude-opus-5-5-plan, high | same | **pass** ([review/evidence-check.md](review/evidence-check.md)); non-blocking notes in [review/r6-followups.md](review/r6-followups.md) |

Execution sessions are test work, not reviews, and are costed separately: 30 executor sessions (21 original groups
and 9 replays), all with the candidate's `corbanu exec` on GLM 5.3 Flash, run one at a time from 2026-10-10 23:09Z
to 2026-10-11 05:29Z. Over the whole campaign (screenshot capture, smoke test, executors and the product under test) the mediator
forwarded 1,400 model requests, all HTTP 200: 1,389 to GLM 5.3 Flash and 11 to GLM 5.2.

## Budget extension (review 6)

Authorized by the integration owner (Codex /root security coordinator, qualification lane), under Travis's
September 12, 2026 delegation in root `AGENTS.md` ("Integrator review discretion").

- **Scope:** one more independent evidence check of this record. It verifies the r4 corrections and re-checks every
  disposition.
- **Purpose:** r4 found a real evidence defect (AGG-20) that r1–r3 missed; an unverified correction must not reach
  the human sign-off.
- **Prior usage:** 5 of 5 (design plus r1–r4).
- **Allowance:** +1, so 6 of 5.

This extension is the integrator's own call under that delegation, not a product-authority decision. It changes no
case result. Travis may confirm or overrule it at sign-off.

## Second budget extension (review 7)

Same authority and reasoning as the first.

- **Why:** r5 found three reasons contradicted by bound product logs. The corrections must be verified before the
  sign-off request.
- **Prior usage:** 6 of 5.
- **Allowance:** +1, so 7 of 5.
- **Stopping rule:** no further extension. Whatever r6 finds is reported to Travis as is.
