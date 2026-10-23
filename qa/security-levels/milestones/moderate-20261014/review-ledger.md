# Review ledger: milestone "Moderate ships", first run

Budget: default maximum 5 (root `AGENTS.md`), counted from 0 for this record. Extensions only with explicit
authorization from Travis or the coordinator (root `AGENTS.md`, "Review budget extensions", 2026-10-14), requested
through `.codex-work/workers-20261002/qual-mod1.request.md`; the integrator never self-authorizes.

| # | Kind | Agent / session | Model | Isolation | Result |
| --- | --- | --- | --- | --- | --- |
| 1 | Code-blind design | `01a13ad5-b332-7610-a491-7202b0eaa190` | claude-opus-5-5-plan, high | instruction-only, read-only sandbox, packet directory only ([access record](design/designer-session.md)) | 33 cases, 12 questions |
| 2 | Design review against the owner decisions (fresh session) | `01a13add-0c6c-79e0-b31a-d0c595bd15ae` | claude-opus-5-5-plan, high | instruction-only, read-only sandbox, review packet only ([access record](design-review/reviewer-session.md)) | **CHANGES**: nine minimum changes ([output](design-review/review-output.md)); answered with amendment table v1 and Q&A v2 |
| 3 | Same session, confirmation pass | same | same | same | **CHANGES**: three wording fixes ([output](design-review/confirm-output.md)), applied as listed in table v2, not independently re-checked |
| 4 | Independent evidence check r1 (fresh session) | `01a13e72-f12d-7200-8e0a-5d27860a6674` | claude-opus-5-5-plan, high | read-only sandbox, a copy of this record at `9d95c79eb2` only ([prompt](review/evidence-check-r1-prompt.md)), 07:24Z-07:37Z on 2026-10-15 | **fail** ([output](review/evidence-check-r1.md)): MOD-15, 17, 22, 29 to failed; summary fixes (MOD-03, 08, 09, 10, 11, 26, 28); replay G18b note described as a clarification; harness and fixture deviations; named limits. All applied; issues #845 and #846 filed |
| 5 | Independent evidence check r2, last within budget (fresh session) | `01a13e83-b874-74f1-8884-a8e688752e8a` | claude-opus-5-5-plan, high | read-only sandbox, a copy of this record at `67339ea600` only ([prompt](review/evidence-check-r2-prompt.md)), 07:43Z-07:53Z on 2026-10-15 | **fail** ([output](review/evidence-check-r2.md)): all r1 corrections applied; MOD-24 to failed; disclosures for MOD-06 (file-tool failure, issue #847), MOD-09 (item 6), MOD-22 (undisclosed MOD-07 run in G18b). Applied, not independently re-checked |

Budget: 5 of 5 used (design 1, design review 2, evidence checks 2). The last verdict is fail; its corrections are applied
but not re-checked. No extension was requested (root `AGENTS.md`, review budget rule): the corrections were specific and
verifiable, and none changes a pass to a pass.

Execution sessions are test work, not reviews; they are listed in the README.
