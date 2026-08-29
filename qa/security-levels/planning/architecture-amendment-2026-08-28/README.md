# Accepted architecture amendment validation

Date: 2026-08-28. Change class: routine planning/process under the active P0 plan.
User authorized adopting the assessed review findings; runtime implementation,
platform certification and release publication are not part of this change.

The pre-amendment snapshot was committed first as `87857faedec1053aefde7925697e308d905700d7`.
This record belongs to the subsequent refinement commit. `validated-files.json`
identifies the exact 80 changed plan/process/checker files checked;
the raw outside report and historical assessment are preserved separately.

## Final checks

| Check | Command / method | Result |
| --- | --- | --- |
| Plan lifecycle | `python3 docs/plans/check.py` | PASS; 1 active of 2 |
| Sprint lifecycle / graph / allocation | `python3 docs/sprints/check.py` | PASS; 75 current = 68 security + 7 unrelated Autoreview; 72 archived |
| Plan checker tests | `python3 -m unittest discover -s docs/plans/tests` | PASS; 3 tests |
| Sprint checker tests | `python3 -m unittest discover -s docs/sprints/tests` | PASS; 19 tests, including parallel limits/scopes, draft cycles/order and archived dependency gates |
| Python formatting | `uvx --from black==25.1.0 black docs/sprints/check.py docs/sprints/tests/test_check.py` | Applied before final tests |
| Portable skills | `python3 scripts/check_portable_skills.py` | PASS; 25 files match |
| Documentation | `uv run --with-requirements requirements-docs.txt --no-project mkdocs build --strict --site-dir /tmp/corbanu-security-plan-validation.4jriEu/site` | PASS; site path is this run's disposable output, not policy |
| Whitespace | `git diff --check` | PASS |
| Raw review integrity | SHA-256 of preserved OPUS_REVIEW.md | Unchanged: `c3b28a1d71229729d91c55149458d89c7b2beb95a385b5f4c362e69fbbdf6aa3` |
| Index and semantic dependencies | Parsed checker JSON and compared each security index order/dependency row; recursively checked six semantic prerequisite assertions | PASS; `graph-check.json` |

The strict build initially found an evidence link outside the documentation tree;
the reference was made an explicit repository evidence path and the final build
passed. Existing excluded-archive links remain informational, not new warnings.

Graph depth is an unweighted 35-stage chain, not a calendar estimate. No sprint
was activated/completed: PF-15-S01 alone remains ready; five new drafts are
unallocated. Archived records, seven Autoreview drafts and Corbanu runtime code
are unchanged.

The earlier OpenClaw review's two existing log files are included explicitly
despite the repository's general log ignore rule; their contents remain historical
evidence and are not new test runs. No core suite, live OS/container/browser,
true-TUI, live-repository, financial or human qualification was performed here.
Those remain mandatory in the owning sprints and final release gates.
