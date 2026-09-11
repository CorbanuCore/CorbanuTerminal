# Main publication evidence — September 10

Scope: publish the curated 16 proposals and 50 draft sprints for Alex's scrum,
with navigation and a receiving-main decision page. No runtime, root policy,
active allocation, deployment, feature enablement or Task Node write change.

Receiving base: `3cec54d9917b776bedaffb586247d7cd7c633df6`.
Original planning base: `6f8f4ce46446e951437e1ce7e7c4a526e5b9e546`.
The older [review record](../2026-09-09/review-evidence.md) is historical; its
hashes and passing global checker do not describe this receiving baseline.

## Inherited baseline

[baseline.json](baseline.json) was captured by running main's unchanged sprint
checker against an archive of the exact receiving base's plan/sprint trees.
It contains seven existing errors, 60 current and 121 archived records, the two
reserved security sprints and the existing ID inventory. No error is suppressed
in the repository checker, and no existing ID is renumbered.

[check.py](check.py) verifies the new portfolio, exact inherited error set,
50-record increase, unchanged archives/reservations, non-colliding new IDs and
publication Markdown links. It deliberately describes a fixed publication
baseline; it is not a permanent lifecycle gate or a waiver for dispatch/release.

## Validation and independent review

Completed 2026-09-10. Full Fable review judged the patch correct with one P3
navigation issue. Accepted in scope: removed a GitHub/MkDocs-dependent fragment
from the Campaign Tracker link and put the exact heading in its label.
Focused closeout returned **helper exit 0, findings []**; see
[review-result.json](review-result.json). No additional review was run after
that clean result.

| Check | Observed result |
| --- | --- |
| `python3 docs/plans/check.py` | PASS: 2/2 active, zero free slots |
| `python3 docs/sprints/check.py` | FAIL: exactly the seven inherited errors, 110 current / 121 archived |
| `python3 qa/portfolio/2026-09-09/validate.py` | PASS: 16 plans, 50 drafts, maximum 73 sprint lines; exact citations, backlinks, navigation, draft/unallocated fields and file links |
| `python3 qa/portfolio/2026-09-10-main-integration/check.py` | PASS: no new errors, unchanged reserved sprints/archives, no new ID collisions; global checker failure remains explicit |
| `python3 -m unittest discover -s docs/plans/tests` | PASS: 4 tests |
| `python3 -m unittest discover -s docs/sprints/tests` | PASS: 19 tests |
| `ruby -ryaml -e 'YAML.parse_file("mkdocs.yml")'` | PASS: YAML syntax |
| `git diff --cached --check` | PASS |

Focused checks and both unit suites were rerun after the link correction.
The [validation receipt](validation.json) records final publication-check output.
MkDocs was unavailable; no full rendered-site build is claimed. This publication
does not execute future sprint commands, test product runtime, or qualify human
acceptance, a deployment or a release.

## Frozen review scope and execution

- [Review inputs](review-inputs.json): 80 staged planning/navigation/QA files.
  Scope at first review: 7,587 added lines (990 QA/evidence), zero removed lines,
  zero production runtime lines. The correction changed one row plus appended
  its focused review instruction. All 80 input hashes matched at clean closeout.
- Receiving work used a clean worktree based on main, separate from the original
  dirty recovery checkout. Existing policy, product spec, runtime, active plans,
  sprint allocations, CI and release records remained unchanged.
- Requested route: `claude-plan / claude-fable-5-1-plan`, reasoning `high`,
  no fallback. Trace metadata recorded that alias and effort; provider response
  identity was `claude-fable-5-1` in both passes.
- Engine: established autoreview helper with the existing Corbanu review wrapper,
  read-only inspection, project-config isolation, web search disabled and no
  nested reviewer. Both passes ran in task-owned tmux, not the user's server.
- Binary: Corbanu `0.1.38`, SHA-256
  `950d42af0ca423ecf24bdde0101363008406776e1f9753136e66fca088ffd7e7`.
  Its source-build commit was not established; this is not a build of the main
  candidate. External exec-based review in tmux is not a true-TUI acceptance test.
- Host: Darwin arm64; tmux 3.7c, 160×48. Task-private directory:
  `/Volumes/CorbanuDrive/Corbanu/.codex-work/planning-main.X0b9gR`;
  socket `tmux.sock`, session `planning-review`, pane `0.0`.
  Literal launch text was captured before Enter was sent separately.
  After exit 0, the owned session was closed; the socket reported no server.
- Private scripts `run-review-1.sh` and `run-review-2.sh` invoke
  `python3 /Users/Neo/.codex/skills/autoreview/scripts/autoreview` with
  `--mode local --engine codex --model claude-fable-5-1-plan --thinking high`,
  `--no-web-search --stream-engine-output` and this directory's
  `review-scope.md` as `--prompt-file`. The established
  `initiative-control.oGQGyA/fable-engine` wrapper supplies the Corbanu route
  and `RUST_LOG=trace`; credentials and raw logs stay outside this repository.

This README, the copied result, input manifest and final validation JSON are
post-review receipts; no reviewed plan/sprint/checker changed after closeout.
The original September 9 manifests remain historical, unchanged evidence.

## Handoff limits

Main's seven inherited ledger errors still require a separately owned repair
before new dispatch. The approved three-initiative/sequential-sprint policy
transition, private dashboard implementation/deployment, live Task Node
write-back and feature enablement are not part of this merge. Draft publication
does not assign a machine, start an agent, replace human acceptance or authorize
financial/commercial action.
