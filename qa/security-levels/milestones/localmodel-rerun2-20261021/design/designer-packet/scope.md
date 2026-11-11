# Scope of the second local-model re-run

Three signed-off (or recorded) code-blind runs had cases or items that were never exercised because the product's
hosted model declined the step before any tool ran. This re-run executes only those, with the owner's local model as
the product's model (D74, D75; MOD-09 items 5 and 7 because of D72/#1091/#1095), each group in a fresh virtual
machine, profile and copy of the test project, with a fresh AI tester. Results are recorded as a follow-up to the
records; no other case is re-run.

| Case | Cases in force (packet folder) | Why it is re-run |
| --- | --- | --- |
| AGG-18 (blocker) | `agg/`: frozen + part 1 + part 3 amendment (`.husky/pre-commit`) | Expected results 1-3 passed earlier; for `cat .env` (result 4) the product's model declined twice, so the product's own denial and its wording were never shown |
| AGG-R07 (blocker) | `agg/`: frozen (new case, part 2) | Steps 1-5 were product denials with the full wording; step 6 ("List the files in P") was declined by the model, whose text did not meet the wording |
| AGG-R10 (blocker) | `agg/`: part 2 + part 5 amendment (`rg`) | The model declined every step, including the three `cat` reads, and the `rg` search never ran |
| AGG-20 (blocker) | `agg/`: frozen | The child agent's model answered every request itself and ran no tool; no full-access role start reached the product ("the weaker role is refused at start" never exercised). The owner also recorded it as a named limit |
| ACC-04 (advisory) | `acc/`: frozen + amendments 1 and 2 | The product's model declined to start a sub-agent inside `/side` |
| MOD-09 items 5 and 7 (blocker) | `mod/`: frozen + v2 + v3 + v5 | Last run: item 5's image-tool read was refused silently (#1087); item 7's approved read stayed blocked by protected launch, with misleading wording (#1077). D72 accepts the item-7 behaviour as intended; #1091 and #1095 changed the wording. Only items 5 and 7 are re-run (owner brief) |

## What the amendment must do

Amend a case only where a decision in `decisions.md` changes what the case expects or how it is run, or where the case
cannot be run as written because of a decision or the test machine (for example the product's model, how the tester
launches it, how much of a case is run, or which earlier steps a case's starting state needs). Do not tighten or loosen
anything else. In particular the expected results of AGG-18, AGG-R07, AGG-R10, AGG-20 and ACC-04 stay as in force;
MOD-09's expected results stay as in force except where D72 changes item 7.
