# Scope of the local-model re-run

34 cases were designed (MOD-01..33 in `design-cases.md`, MOD-34 in `amendments-v2.md`). The first run executed all 34
under `amendments-v2.md`; the first targeted re-run executed 17 of them under `amendments-v3.md`; the second targeted
re-run executed 2 under `amendments-v4.md`. Moderate is signed off. This re-run executes only the two cases below, each
in a fresh virtual machine, profile and copy of the test project, with a fresh AI tester per case.

| Case | Why it is re-run (decision in `decisions.md`) | In force before this amendment |
| --- | --- | --- |
| MOD-09 | Owner request with D61 / D67: both earlier product models declined before any tool ran; the local model is meant to attempt the steps, so the product's own protections are reached | frozen + v2 + v3 (incl. R-1, which this re-run replaces); v4 does not touch it |
| MOD-21 | Same | frozen + v2 + v3 (incl. R-1); v4 does not touch it |

Not re-run: all other cases. Their results stand; you do not need to write anything about them.

## What the amendment must do

Amend a case only where a decision in `decisions.md` changes what the case expects or how it is run, or where the case
cannot be run as written because of a decision (for example the product's model and how the tester launches it). Do
not tighten or loosen anything else. In particular the expected results of MOD-09 and MOD-21 stay as in force.
