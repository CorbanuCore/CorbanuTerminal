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
- A plan may have at most one `in_progress` sprint, and executable dependencies
  must already be completed and archived.
- A sprint never grants authority beyond its active plan.

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
| [P0 `/security` levels](../plans/active/p0-security-levels.md) | Active | [14 current sprints](current/p0-security-levels/index.md) | PF-13-S02 is ready; PF-15 through PF-22 and PF-13-S01 are completed and archived |
| [Persistent linked sessions](../plans/active/persistent-linked-sessions.md) | Active | [1 current sprint](current/persistent-linked-sessions/index.md) | PF-27-S01 is in progress |
| [Arbitrary-model Autoreview](../plans/proposed/arbitrary-model-autoreview.md) | Proposed | [7 draft sprints](current/arbitrary-model-autoreview/index.md) | None until plan activation and sprint worktree allocation |
| [Prompt-injection firewall and brokered authority](../plans/proposed/prompt-injection-firewall.md) | Proposed | 0 | The superseded 72-sprint decomposition is retained in the excluded archive |

## Machine check

```bash
python3 docs/sprints/check.py
```

The checker validates lifecycle placement, one-feature linkage, plan backlinks,
required checkbox ledgers, line limits, status authorization, exact plan/worktree
agreement, dependency completion, one-in-progress-per-plan, and archive
completion.
