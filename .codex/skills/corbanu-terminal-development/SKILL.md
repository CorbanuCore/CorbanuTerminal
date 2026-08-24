---
name: corbanu-terminal-development
description: Develop, plan, test, document, benchmark, or release the Corbanu Terminal Codex fork. Use for CorbanuTerminal product behavior, user workflows, behavior-changing fixes, active plans and worktrees, true-TUI QA, TensorCash or Isometric Game qualification, Hermes/Kilo comparisons, shipped-feature documentation, or release readiness.
---

# Corbanu Terminal Development

Route work through the repository's canonical policy; do not restate it.

1. Locate the Corbanu Terminal root and read its `AGENTS.md` completely.
2. Classify the change using the root policy.
3. Read the cited product-spec heading and requirement excerpt.
4. For a product initiative, read `docs/plans/index.md`, inspect
   `docs/plans/active/`, and create or update a plan from
   `docs/plans/PLAN_TEMPLATE.md`.
5. Before product-initiative implementation, read `docs/sprints/index.md` and
   select or create one sprint from `docs/sprints/SPRINT_TEMPLATE.md`. Confirm it
   links exactly one plan feature, is `ready` or `in_progress`, and records the
   active plan's exact worktree coordinates. Run `python3 docs/sprints/check.py`.
6. Read the nearest nested `AGENTS.md` for every implementation path.
7. Implement only the selected sprint's remaining checklist in its recorded
   worktree. Update its `Done` and `Remaining` ledgers as evidence changes.
8. Collect the evidence required by the root policy, plan, and sprint. Archive a
   completed sprint so it leaves the current documentation view.
9. For a release, read `benchmarks/README.md` and the versioned record under
   `qa/release/<version>/` before deciding readiness.

Report the change class, product citation, plan, sprint id and status, worktree,
final-tree tests, interactive evidence when required, live-repository evidence,
human sign-off, documentation, benchmark state, and release blockers.
