# Corbanu Terminal plans

This file is the sole authority for active-plan work in progress and lifecycle.
Use the root [change classes](../AGENTS.md#change-classes) to decide whether work
needs a plan.

## Work-in-progress limit

Corbanu may have at most **two active product-initiative plans**.

A plan counts toward the limit only when it:

- is a Markdown file under `plans/active/`;
- declares `status: active`; and
- covers a product initiative as defined by the root policy.

Routine work, bounded fixes, and release records do not consume a plan slot.
They also cannot introduce new product scope. A product initiative may not be
reclassified to evade the limit.

Before activation, count the active plans. If two exist, finish, cancel, or
explicitly replace one through the product decision process. Never activate a
third.

## Activation requirements

Create the plan from [`PLAN_TEMPLATE.md`](PLAN_TEMPLATE.md). It is ready to
activate when it records:

- the exact product-spec heading and a short requirement excerpt;
- user pain and the ideal end-to-end flow;
- scope and non-goals;
- owner, target release, and exact implementation worktree paths, branches, and
  base commits;
- useful code references;
- measurable acceptance flows;
- applicable automated, TUI, live-repository, human, documentation, and
  benchmark evidence fields.

Set `status: active` only after those fields are complete. Draft notes and
legacy files elsewhere in the repository do not authorize implementation.

## Lifecycle

1. **Propose:** Complete the template without starting implementation.
2. **Activate:** Confirm product authority, the WIP limit, ownership, worktrees,
   scope, and acceptance criteria.
3. **Build:** Work only in declared scope and worktrees. Update the plan when
   either changes.
4. **Qualify:** Fill the evidence fields required by the root policy.
5. **Release:** Link the versioned record under `qa/release/<version>/`.
6. **Close:** Move the plan to
   `plans/completed/<release>-<slug>.md` after release, or to
   `plans/cancelled/<slug>.md` with the cancellation decision.

A release record may aggregate either or both active initiatives without
becoming another active plan.
