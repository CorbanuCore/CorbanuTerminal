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

Each of at most three active initiatives has `parallel_sprint_limit: 1` and a
named `integration_owner`. Across all plans, at most **three** sprints may be
`in_progress` or `blocked`; blocked work keeps its reservation until explicitly
returned to draft with a recorded handoff. This is Travis's September 10/11
operating-model decision, not additional within-plan parallelism.

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

### Manager-owned continuation

Travis's September 11 request to fix stalled orchestration authorizes the named
integration owner to perform local receiving-branch integration, reconcile
evidence, and revise exact worker allocations within already-approved active
features. This is not permission to change a product contract or a hard gate.
Do not ask the human to do routine allocation, branch preparation or bookkeeping.

Before a worker finishes, prepare the next bounded assignment and human test
plan. On return, inspect its literal diff and independent review, integrate
reviewed changes on the recorded local receiving branch, and run the combined
tree's affected tests. Update Done/Remaining without equating a worker return,
a passing test or a local merge with human acceptance. Preserve feature-OFF
boundaries. Shared registration and policy edits remain manager-owned.

Then either dispatch remaining work in the same sprint, or complete/archive
the sprint and activate exactly one dependency-complete successor once all
required evidence and decisions actually exist. A changed literal file
allocation within approved feature scope is manager work: record it in plan
and sprint, check overlap and coordinates, run both checkers, then dispatch.
Preparing a successor while its predecessor is unfinished is allowed; starting
its implementation is not. Do not split or waive dependencies merely to stay busy.

Each reserved lane must have a running assignment, an executable next action,
or a concrete blocker with owner, exact question, recommendation, affected scope
and evidence. Manager-owned missing work stays in the manager queue, not marked
as waiting on the human. Deliver newly required decisions to the human; a note
hidden in a private packet does not count as asking. Continue independent work
inside the approved sprint while waiting, without manufacturing busywork.

Existing review limits remain binding; one fresh independent review of each new
material candidate plus scoped corrections is the normal closeout, not repeated
reviews of unchanged clean code. Do not reset exhausted budgets or resume paused
PF13/security work. Main merges/pushes, releases, deployments, live Task Node
actions, new product contracts and paid-service commitments still need their
separate authority. Current assignments/evidence are in the
[manager handoff](../plans/workstream-manager-handoff-2026-09-11.md).

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
| [PF-13 / security — workstream 1](../plans/active/p0-security-levels.md) | Active | [53 current sprints](current/p0-security-levels/index.md), 28 completed archives | PF-35-S01 retains its reservation; unusable PF-27-S04 allocation returned to draft with preserved recovery pin; PF13 task untouched |
| [Accounting — workstream 2](../plans/active/portfolio-agent-cost-accounting.md) | Active | 4 sequential PF-60 sprints | S01 contract/fixtures selected for independent Astra High allocation after main merge |
| [Task Node — workstream 3](../plans/active/initiative-delivery-control.md) | Active | PF-80-S01, two PF-79 beta drafts, PF-81-S01 visual QA draft | Scoped main port and progress qualification first; manager selects one sequential follow-up; no follow-up launched |
| [Unified provider onboarding and management](../plans/proposed/unified-provider-auth.md) | Deferred | [0 current sprints](current/unified-provider-auth/index.md) | Completed PF-55/provider evidence remains unchanged; no completion claim added |
| [Arbitrary-model Autoreview](../plans/proposed/arbitrary-model-autoreview.md) | Proposed | [7 draft sprints](current/arbitrary-model-autoreview/index.md) | None until plan activation and sprint worktree allocation |
| [Prompt-injection firewall and brokered authority](../plans/proposed/prompt-injection-firewall.md) | Proposed | 0 | Historical 72-sprint decomposition remains cancelled; every record maps into the active P0 plan's current work |

## Machine check

September 10 planning publication and call follow-up add [16 proposals and 52 draft sprints](../plans/portfolio-2026-09-09.md)
(PF-60–PF-75), with no execution authority or allocations. The receiving `main`
baseline had 60 current and 121 archived records; the initial 50 plus two new PF-70 drafts yield 112 current
and 121 archived records. The seven inherited sprint-link/ID errors are resolved;
see the [identity reconciliation](identity-reconciliation-2026-09-10.md).
September 11 adds PF-80-S01 (renamed operations PF-76) and PF-79-S01/S02:
115 current, 121 archived. The subsequent PF-81-S01 visual-QA planning amendment
adds one dependent draft: 116 current, 121 archived, still only three reserved.
The three-initiative policy is now explicit; all
identity/dependency regression checks remain, with no error exceptions.
Each proposal's sprint execution map links every draft directly.

```bash
python3 docs/sprints/check.py
```

The checker validates lifecycle placement, one-feature linkage, plan backlinks,
required checkbox ledgers, line limits, status authorization, exact plan/worktree
agreement, dependency completion/order/cycles, bounded parallel allocations and
archive completion.
