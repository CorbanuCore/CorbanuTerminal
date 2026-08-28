# Corbanu Terminal development skill

Corbanu contributors need the same development workflow whether they enter
through the documentation site or through an agent-loaded skill. This page
renders the `corbanu-terminal-development` skill for human readers without
changing which file agents load.

!!! warning "Canonical skill mirror — intentional duplicate"

    The section between the **BEGIN DUPLICATED SKILL MIRROR** and
    **END DUPLICATED SKILL MIRROR** comments intentionally duplicates
    `.codex/skills/corbanu-terminal-development/SKILL.md`. The same agent
    source is mirrored at
    `.agents/skills/corbanu-terminal-development/SKILL.md` for portability.

    The `.codex` file is the canonical editing source. The fenced block below
    reproduces it verbatim, including YAML metadata, while
    `scripts/check_portable_skills.py` enforces the `.agents` copy. Any
    change must update all three representations in the same commit. If the
    documentation copy diverges, follow `.codex` and treat this page as
    defective.

<!-- BEGIN DUPLICATED SKILL MIRROR -->

## Exact `SKILL.md` source

```markdown
---
name: corbanu-terminal-development
description: Develop, plan, test, document, benchmark, or release the Corbanu Terminal Codex fork. Use for CorbanuTerminal product behavior, user workflows, behavior-changing fixes, active plans and worktrees, true-TUI QA, TensorCash or Isometric Game qualification, Hermes/Kilo comparisons, shipped-feature documentation, or release readiness.
---

# Corbanu Terminal Development

Route work through the repository's canonical policy; do not restate it.

1. Locate the Corbanu Terminal root and read `docs/development-policy.md` completely.
2. Classify the change using the root policy.
3. Read the cited product-spec heading and requirement excerpt.
4. For a product initiative, read `docs/plans/index.md`, inspect
   `docs/plans/active/`, and create or update a plan from
   `docs/plans/PLAN_TEMPLATE.md`.
5. Before product-initiative implementation, read `docs/sprints/index.md` and
   select or create one sprint from `docs/sprints/SPRINT_TEMPLATE.md`. Confirm it
   links exactly one plan feature, is `ready` or `in_progress`, and records the
   active plan's exact worktree coordinates. Run `python3 docs/sprints/check.py`.
6. Read `docs/rust-development-policy.md` and the relevant `docs/crate-notes/` file for every implementation path.
7. Implement only the selected sprint's remaining checklist in its recorded
   worktree. Update its `Done` and `Remaining` ledgers as evidence changes.
8. Collect the evidence required by the root policy, plan, and sprint. Archive a
   completed sprint so it leaves the current documentation view.
9. For a release, read `benchmarks/README.md` and the versioned record under
   `qa/release/<version>/` before deciding readiness.

Report the change class, product citation, plan, sprint id and status, worktree,
final-tree tests, interactive evidence when required, live-repository evidence,
human sign-off, documentation, benchmark state, and release blockers.
```

<!-- END DUPLICATED SKILL MIRROR -->

## Governing sources

This mirror does not replace the repository policy or its evidence records.
Use these sources together:

- [Repository development policy](https://github.com/CorbanuCore/CorbanuTerminal/blob/main/docs/development-policy.md)
- [Canonical development skill](https://github.com/CorbanuCore/CorbanuTerminal/blob/main/.codex/skills/corbanu-terminal-development/SKILL.md)
- [Portable agent mirror](https://github.com/CorbanuCore/CorbanuTerminal/blob/main/.agents/skills/corbanu-terminal-development/SKILL.md)
- [Product specification](corbanu-product-spec.md)
- [Plan process](plans/index.md)
- [Sprint process](sprints/index.md)
- [Benchmark tracker](https://github.com/CorbanuCore/CorbanuTerminal/blob/main/benchmarks/README.md)
