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
- Executable dependencies must already be completed and archived.
- A sprint never grants authority beyond its active plan.
- New or amended implementation records link their plan's upstream-touch rows
  and resolve the [upstream integration contract](../plans/upstream-integration.md)
  before readiness; adapter evidence is part of completion.

## Bounded concurrency

Plans default to `max_active_sprints: 1`. An active plan may opt into two or
three slots and name an `integration_owner`. Both `in_progress` and `blocked`
consume slots; `ready` does not authorize code changes or reserve a slot.
The two-plan portfolio limit is unchanged. Do not split an initiative into
overlapping plans to evade either limit.

Before opting in, record the dependency graph and lane allocation in the plan.
Every executable sprint in that plan records `lane` and `write_scope` as well
as its owner and exact worktree coordinates. `write_scope` is a comma-separated
list of literal repository-relative files or directory prefixes, not globs;
`.`/`..`, absolute paths, and backslashes are invalid. Include implementation
and test paths; the sprint's own record and evidence directory are implicit.
Shared plan/navigation metadata is updated serially by the integration owner.

Concurrently active sprints must have distinct lanes, worktrees, and branches,
and disjoint write scopes, including across plans. Separate worktrees alone
do not make edits to the same module independent. Land shared interfaces and
registration points first, then give each lane its own files. Shared-file
changes require a serialized integration sprint or dependency, not an overlap
exception. An integration owner cannot waive a collision or missing evidence.

`depends_on` is the hard prerequisite graph, including completed interface
contracts; cycles and self-dependencies are invalid even in drafts.
`execution_order` is a unique display priority within a plan, not a dependency
or an obligation to wait for every lower number. Split harness construction
from final qualification instead of treating unfinished prerequisites as soft.

Each worker follows one sprint. Platform tests, independent review, and
isolated live-repository flows may run concurrently within a qualification
sprint against the same recorded candidate. No new sprint is activated by a
plan edit alone. Rebase/integrate through the named owner, then re-run affected
tests and true-PTY proof on the final integrated candidate. Interactive sprint
completion requires its own applicable true-PTY proof; final release workflows
repeat that proof across the integrated product.

## Execution loop

1. Select a dependency-complete sprint linked from the active plan.
2. Resolve its owner, lane, write scope, and exact worktree coordinates; set it to `ready`.
3. Check slot availability and collisions; set it to `in_progress` and run the checker before code changes.
4. Execute only `Remaining` items; move verified work to `Done` with `[x]`.
5. Run formatting before final affected tests and true-TUI QA.
6. Complete every verification and exit-evidence checkbox.
7. Set status to `completed`, move the file to `archive/<plan-slug>/`, and remove
   it from current MkDocs navigation.
8. Replace the plan's current-sprint link with release or completion evidence.

## Current sprint portfolio

| Plan | Plan status | Current sprints | Execution authority |
| --- | --- | ---: | --- |
| [P0 `/security` levels](../plans/active/p0-security-levels.md) | Active | [Current sprints and dependency graph](current/p0-security-levels/index.md) | PF-13-S05 and PF-30-S01 in progress; PF-27-S01/PF-26-S01 completed; PF-29 allocated pending readiness |
| [Corbanu API balance and keys](../plans/active/corbanu-api-balance.md) | Active | None; [PF-33-S02 archived](archive/corbanu-api-balance/pf-33-s02-customer-response-boundary.md) | PF-31/PF-32/PF-33 complete; PF-34 is next |
| [Arbitrary-model Autoreview](../plans/proposed/arbitrary-model-autoreview.md) | Proposed | [7 draft sprints](current/arbitrary-model-autoreview/index.md) | None until plan activation and sprint worktree allocation |
| [Prompt-injection firewall and brokered authority](../plans/proposed/prompt-injection-firewall.md) | Proposed | 0 | The superseded 72-sprint decomposition is retained in the excluded archive |

## Machine check

```bash
python3 docs/sprints/check.py
```

The checker validates lifecycle placement, one-feature linkage, plan backlinks,
required checkbox ledgers, line limits, status authorization, exact plan/worktree
agreement, dependency completion and cycles, bounded active slots, lane/path/
branch collisions, and archive completion.

The checker does not validate upstream ancestry or semantic adapter compatibility.
The integration owner reviews that evidence under the upstream contract; a
passing structural check is not an upstream-qualification pass.
