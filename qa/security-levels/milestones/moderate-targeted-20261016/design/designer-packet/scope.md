# Scope of the targeted re-run

34 cases were designed (MOD-01..33 in `design-cases.md`, MOD-34 in `amendments-v2.md`). The first run executed all
34 under `amendments-v2.md` (rules G-1..G-3, named limits, case amendments, coordinator answers). This re-run executes
only the cases below, each in a fresh profile and fresh copy of the test project, with a fresh AI tester per group.

## Re-run (17 cases) and the decision that applies

| Case | Why it is re-run (decision in `decisions.md`) |
| --- | --- |
| MOD-01 | A(12), B: #739 option B decided after the case was written |
| MOD-22 | A(12), B: as MOD-01 (its Permissive steps after the relaunch) |
| MOD-06 | A(4) edit-then-run asks only after untrusted content; A(7) #847 agent file tools; its git-commit part is NOT re-run (D: #837 pending, D5) |
| MOD-08 | A(3) #840 saved rules keep working (setup saves rules under Permissive; see B for protected launch) |
| MOD-09 | A(13) different test model |
| MOD-21 | A(13) different test model |
| MOD-10 | A(1) #838; C refusal wording |
| MOD-20 | A(5) #839 inspector |
| MOD-30 | A(4) right reason for Corbanu's files |
| MOD-15 | A(5) #839 Recent denials; C refusal wording |
| MOD-24 | A(5) #839 model-keys line / broker degraded |
| MOD-17 | A(6) #845 startup |
| MOD-26 | A(4) keep the `.env` prompt |
| MOD-29 | A(9) #846 wording |
| MOD-28 | A(11) revoke does not suspend saved rules; re-run with a session approval |
| MOD-32 | A(8) other windows on the same home switch to Moderate too |
| MOD-33 | A(10) #843 spurious PATH-aliases warning |

The tester groups (fixed by the integrator): MOD-01 / MOD-06 / MOD-08 / MOD-09 / MOD-10, MOD-20, MOD-30 (in that
order, one session, as in the first run: MOD-20 starts "after MOD-10") / MOD-15 / MOD-17 / MOD-21 / MOD-22 / MOD-24 /
MOD-26, MOD-29 (in that order) / MOD-28 / MOD-32 / MOD-33.

## Not re-run

- Passed in the first run, unchanged: MOD-02, 03, 04, 05, 07, 12, 13, 14, 16, 18, 19, 23, 25, 27, 31, 34.
- Pending owner decision D5 (#837), not re-run and recorded as "pending owner decision": MOD-11, and MOD-06's
  git-commit part.

## What the amendment must do

Amend a case only where a decision in `decisions.md` changes what the case expects, or where the case cannot be run as
written because of a decision (for example MOD-28's "session approval", the different model for MOD-09/21, MOD-06's
git part). Fixes that make the product meet the case's existing expectations need no amendment: those cases are re-run
as already in force (`design-cases.md` plus `amendments-v2.md`). Do not tighten or loosen anything else.
