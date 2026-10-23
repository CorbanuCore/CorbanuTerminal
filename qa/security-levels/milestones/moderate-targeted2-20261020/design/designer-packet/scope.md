# Scope of the second targeted re-run

34 cases were designed (MOD-01..33 in `design-cases.md`, MOD-34 in `amendments-v2.md`). The first run executed all 34
under `amendments-v2.md`; the first targeted re-run executed 17 of them under `amendments-v3.md` (on top of v2). This
re-run executes only the two cases below, each in a fresh profile and fresh copy of the test project, with a fresh AI
tester per case.

| Case | Why it is re-run (decision in `decisions.md`) | In force before this amendment |
| --- | --- | --- |
| MOD-15 | D38: the refusal text now names the level and says how to proceed (#949); the candidate includes #900 | frozen + v2 + v3 (incl. R-2) |
| MOD-28 | D40: #942 fixed by #962; judge the revoke if a session approval exists, otherwise accept as "no session approval exists to revoke" | frozen + v2 + v3 |

Not re-run: all other cases. Their results stand or are dispositioned by the owner (D39, D41, D43); you do not need to
write anything about them.

## What the amendment must do

Amend a case only where a decision in `decisions.md` changes what the case expects, or where the case cannot be run as
written because of a decision. Fixes that make the product meet the case's existing expectations need no amendment.
Do not tighten or loosen anything else.
