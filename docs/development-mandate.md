# Corbanu Terminal development mandate

Corbanu contributors need the same development workflow whether they enter
through the documentation site or through an agent-loaded skill. This page
makes that workflow visible to humans without changing which file agents load.

!!! warning "Intentional duplicate"

    The section between the **BEGIN DUPLICATED SKILL MIRROR** and
    **END DUPLICATED SKILL MIRROR** comments intentionally duplicates
    `.codex/skills/corbanu-terminal-development/SKILL.md`.

    The skill file is the canonical agent-loading source. This page is its
    rendered documentation mirror: YAML fields are presented as labels and the
    source title is demoted one heading level for this page. Any change to the
    mirrored content must update both files in the same commit. If the copies
    diverge, follow the skill file and treat this page as defective.

<!-- BEGIN DUPLICATED SKILL MIRROR -->

**Skill name:** `corbanu-terminal-development`

**Trigger scope:** Develop, plan, test, document, benchmark, or release the
Corbanu Terminal Codex fork. Use for CorbanuTerminal product behavior, user
workflows, behavior-changing fixes, active plans and worktrees, true-TUI QA,
TensorCash or Isometric Game qualification, Hermes/Kilo comparisons,
shipped-feature documentation, or release readiness.

## Corbanu Terminal Development

Route work through the repository's canonical policy; do not restate it.

1. Locate the Corbanu Terminal root and read its `AGENTS.md` completely.
2. Classify the change using the root policy.
3. Read the cited product-spec heading and requirement excerpt.
4. For a product initiative, read `plans/README.md`, inspect
   `plans/active/`, and create or update a plan from
   `plans/PLAN_TEMPLATE.md`.
5. Read the nearest nested `AGENTS.md` for every implementation path.
6. Implement only the classified scope in the recorded worktree.
7. Collect the evidence required by the root policy and the applicable record.
8. For a release, read `benchmarks/README.md` and the versioned record under
   `qa/release/<version>/` before deciding readiness.

Report the change class, product citation, plan when required, worktree,
final-tree tests, interactive evidence when required, live-repository evidence,
human sign-off, documentation, benchmark state, and release blockers.

<!-- END DUPLICATED SKILL MIRROR -->

## Governing sources

This mirror does not replace the repository policy or its evidence records.
Use these sources together:

- [Repository development policy](https://github.com/CorbanuCore/CorbanuTerminal/blob/main/AGENTS.md)
- [Canonical development skill](https://github.com/CorbanuCore/CorbanuTerminal/blob/main/.codex/skills/corbanu-terminal-development/SKILL.md)
- [Product specification](corbanu-product-spec.md)
- [Plan process](https://github.com/CorbanuCore/CorbanuTerminal/blob/main/plans/README.md)
- [Benchmark tracker](https://github.com/CorbanuCore/CorbanuTerminal/blob/main/benchmarks/README.md)
