# Sprints

Sprints are Corbanu's mechanical execution records. A plan defines a feature
contract. A sprint turns one feature into an exact code-and-evidence checklist.

## Record hierarchy

| Record | Owns |
| --- | --- |
| Product specification | Authorized outcome and decision roles |
| Plan | Feature contract, scope, sequencing, acceptance, and worktrees |
| Sprint | One feature's bounded code tasks and final-tree evidence |
| Release record | Candidate-wide TUI, live-repository, human, and release evidence |
| Feature documentation | Finished behavior only |

## Directory contract

| Directory | Allowed status | Documentation visibility |
| --- | --- | --- |
| `current/<plan-slug>/` | `draft`, `ready`, `in_progress`, `blocked` | Visible; unfinished work only |
| `archive/<plan-slug>/` | `completed`, `cancelled` | Excluded from MkDocs navigation and build |

## Non-negotiable sprint shape

- One sprint links one plan file and one feature id.
- One sprint cannot implement multiple plan features.
- Dependencies may cross features; implementation scope may not.
- Tasks name exact existing or planned code boundaries.
- `Done` contains checked items only; `Remaining` contains unchecked items only.
- Tests, TUI applicability, and exit evidence are checklists, not promises in prose.
- A `draft` sprint may use `UNALLOCATED` worktree coordinates.
- A `ready` or `in_progress` sprint requires an active plan plus an exact
  worktree, branch, and 40-character base commit matching that plan.
- Parallel implementation must satisfy the allocation rules below; executable
  dependencies must already be completed and archived.
- A sprint never grants authority beyond its active plan.

## Bounded parallel implementation

The default is one active implementation sprint per plan. A plan may opt into
two or three with `parallel_sprint_limit` and a named `integration_owner` in its
front matter. Across all plans, at most **three** sprints may be `in_progress`
or `blocked`; blocked work keeps its reservation until explicitly returned to
draft with a recorded handoff. The two-active-plan limit is unchanged.

Before a parallel allocation starts:

- Every dependency is completed and archived. `execution_order` is a
  topological reading order, not a serial scheduling lock. No dependency on
  an unfinished interface, draft, fixture-only integration or cancelled record
  is executable; use a completed single-feature contract sprint as a freeze point.
- Each worker has a distinct named owner, `parallel_lane`, exact worktree and
  branch. Worktree coordinates must appear in its active plan. Do not invent
  allocations or share a checkout merely because two tasks look independent.
- Each active record declares `write_scope`: comma-separated repository-relative
  file paths or directory prefixes (trailing `/`), without globs, `..` or root
  reservations. Concurrent scopes must not overlap, even across plans. Include
  manifests, lockfiles, shared registries and tests, not just the main module.
- Each record declares an `integration_gate` naming the receiving owner, merge
  boundary and tests to rerun on the combined tree. The plan's integration owner
  serializes shared Core/protocol/schema/lockfile edits. If these overlap, stop
  concurrency and schedule the changes sequentially; do not omit ownership.
  Concurrent plans each name their integration owner, even with one worker each.
- A blocked or completed lane's handoff records its commit, contract versions,
  test evidence and outstanding integration. Update coordinates/base commits and
  rerun checks before reallocation. An early artifact or interface pass does not
  enable protected behavior or replace PF-13/PF-26 qualification.

The checker enforces counts, concrete allocation fields, distinct owners/lanes/
worktrees/branches, disjoint declared paths, dependency order and cycles. It does
not inspect remote checkouts or prove the declarations truthful: the integration
owner must compare actual diffs to scope and record final combined-tree evidence.

## Execution loop

1. Select the next dependency-complete sprint linked from the active plan.
2. Resolve its exact worktree coordinates and set it to `ready`.
3. Set it to `in_progress` before code changes.
4. Execute only `Remaining` items; move verified work to `Done` with `[x]`.
5. Run formatting before final affected tests and true-TUI QA.
6. Complete every verification and exit-evidence checkbox.
7. Set status to `completed`, move the file to `archive/<plan-slug>/`, and remove
   it from current MkDocs navigation.
8. Replace the plan's current-sprint link with release or completion evidence.

## Current sprint portfolio

| Plan | Plan status | Current sprints | Execution authority |
| --- | --- | ---: | --- |
| [P0 `/security` levels](../plans/active/p0-security-levels.md) | Active | [51 current sprints](current/p0-security-levels/index.md), 25 completed archives | PF-22-S02 is integrated and archived; PF-27-S04 retains serialized integration, PF-30-S01 returned to unallocated draft after its unrecoverable worktree loss, and PF-35-S01 continues external qualification |
| [Unified provider onboarding and management](../plans/active/unified-provider-auth.md) | Active | [8 current sprints](current/unified-provider-auth/index.md), 7 completed archives including the inherited Claude foundation | PF-48-S01 is implemented, primary-accepted, and archived at `7936d83859`; PF-49 remains draft and unallocated |
| [Arbitrary-model Autoreview](../plans/proposed/arbitrary-model-autoreview.md) | Proposed | [7 draft sprints](current/arbitrary-model-autoreview/index.md) | None until plan activation and sprint worktree allocation |
| [Prompt-injection firewall and brokered authority](../plans/proposed/prompt-injection-firewall.md) | Proposed | 0 | Historical 72-sprint decomposition remains cancelled; every record maps into the active P0 plan's current work |

## Machine check

```bash
python3 docs/sprints/check.py
```

The checker validates lifecycle placement, one-feature linkage, plan backlinks,
required checkbox ledgers, line limits, status authorization, exact plan/worktree
agreement, dependency completion/order/cycles, bounded parallel allocations and
archive completion.
