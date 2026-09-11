# Main planning integration and kickoff — 2026-09-11

Change class: product-initiative planning/allocation plus bounded governance
implementation of Travis's explicit three-stream operating model. No Rust,
release pipeline, production Task Node, remote credential or runtime change.
Product citation: **Internal delivery control — TO BUILD**, “Use sequential
sprints per initiative”. [Source and handoff](../../docs/plans/main-workstreams-2026-09-11.md).

Receiving base: `295aed26e53b17f919f7199ae1c9748b1b1250ba`.
Integration checkout: `/Volumes/CorbanuDrive/Corbanu/worktrees/management-workstreams-20260911`,
branch `integrate/management-workstreams-20260911`. Recovery source left intact.

Reviewable boundaries: (1) small governance-code change and existing regression
tests preserved; (2) workstream allocations, PF-80 collision repair and mechanical
lifecycle links; (3) already reviewed beta/task planning and dated QA imported.
The aggregate exceeds 800 text lines largely because of reviewed-document imports
and lifecycle moves. The smallest coherent main landing includes the plan/sprint
backlinks and ID migration together. Runtime port is a separate worker stage,
not bundled into this merge. No broad recovery implementation is imported.

Before review: plan checker PASS, 3/3; sprint checker PASS, 115 current / 121
archived, exactly PF-35-S01/PF-60-S01/PF-80-S01 reserved. Plan tests PASS (5),
sprint tests PASS (22, including all three newer main identity regressions),
portable skills PASS (25 matching files), `git diff --check` PASS.
No interactive product change: new TUI/GUI/live-repository qualification is not
applicable to this planning merge. Existing release evidence is not relabeled.

Fable review, main commit/push and native agent dispatch receipts: pending at
this checkpoint. Allocated worker branches start at the base above and must
fast-forward to the merged planning commit before the two requested agents run.
No agent launch or remote source cutover is claimed by this preflight record.

Initial Fable 5.1 high review found one P3 stale “active” label in the moved
provider-auth sprint index. Corrected it and marked the touched older portfolio/
scrum allocation summaries as historical with current-handoff links. This is
the same in-scope lifecycle-prose issue; no runtime or owner boundary expanded.
Corrected-tree Fable review remains pending. Portable-skill unit tests also pass
(3); all 30 governance/portable tests pass. Link audit found no new missing
targets; 17 occurrences of five broken provider-QA links predate the move.

Corrected-tree Fable 5.1 (`claude-fable-5-1-plan`, high) through the Corbanu tmux
Autoreview harness: **no actionable findings; patch correct**. Review stopped
after that clean pass. Final scope: 61 files, principally reviewed-document
imports and mechanical link/lifecycle changes; no runtime/Rust/CI edit. Main
push and native subagent IDs are recorded in the subsequent kickoff receipt.
