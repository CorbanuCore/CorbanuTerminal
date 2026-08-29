---
title: "<product initiative>"
status: draft
change_class: product-initiative
priority: "<P0, P1, or P2>"
owner: "<name or accountable role>"
max_active_sprints: 1
integration_owner: "<name or accountable role>"
activation_authority: "<person or role>"
activation_basis: "<existing product decision authorizing this outcome>"
target_release: "<version or TBD>"
deadline: "<YYYY-MM-DD, continuous, or TBD>"
created: YYYY-MM-DD
updated: YYYY-MM-DD
product_spec:
  file: docs/corbanu-product-spec.md
  heading: "<exact heading text>"
  requirement_excerpt: "<short exact excerpt>"
implementation_worktrees:
  - path: "<exact path>"
    branch: "<branch>"
    base_commit: "<40-character commit>"
---

# <Plan title>

Policy: repository-root `AGENTS.md`

Plan lifecycle: `docs/plans/index.md`

## Activation record

| Field | Value |
| --- | --- |
| Status | draft |
| Active-plan slot | pending |
| Product authority |  |
| Authoritative decision |  |
| Target release |  |
| Deadline |  |

## User pain

<What is painful or impossible for the user today?>

## Product intent and ideal flow

<Describe the best end-to-end experience in plain English: entry, success,
failure, recovery, resume, and return use.>

## Product linkage

| Field | Value |
| --- | --- |
| Exact product-spec heading |  |
| Requirement excerpt |  |
| Product outcome advanced |  |
| North-star criterion advanced |  |

## Scope

### In

- `<in-scope item>`

### Out

- `<out-of-scope item>`

## Invariants

- `<boundary that must remain true in every implementation stage>`

## Ownership and implementation worktrees

| Owner | Worktree | Branch | Base commit | Scope |
| --- | --- | --- | --- | --- |
|  |  |  |  |  |

## Useful code references

| Path or symbol | Why it matters |
| --- | --- |
|  |  |

## Upstream-touch record

Complete the fields required by the [upstream integration contract](upstream-integration.md).
Use repository-relative literal paths; unresolved fields block affected readiness.

| Baseline field | Value / evidence |
| --- | --- |
| Canonical upstream URL and verified full SHA | pending |
| Fork base and candidate SHA | pending |
| Proposed upstream update SHA, if applicable | pending |

| Feature / sprint | Upstream files and native interface | Product-owned boundary / reason | Integration owner | Contract tests / artifact | Upgrade disposition |
| --- | --- | --- | --- | --- | --- |
|  |  |  |  | pending | pending |

## Sprint execution map

Every implementation sprint must link exactly one feature in this plan. Link
current sprint records under `docs/sprints/current/`; replace completed links
with release evidence after the sprint is archived.

| Feature ID | Current sprint records | Completion evidence |
| --- | --- | --- |
|  |  | pending |

### Dependency graph and lane allocation

Use `depends_on` for hard prerequisites and `execution_order` only for display.
Follow the concurrency contract in `docs/sprints/index.md`; record actual
allocations before activating a sprint, not hypothetical worktree paths.

| Lane | Sprint(s) | Owner | Write scope | Shared-interface prerequisite | Integration checkpoint |
| --- | --- | --- | --- | --- | --- |
|  |  |  |  |  |  |

### Requirement traceability

| Product requirement / adopted design | Feature and sprint | State (covered, pending, deferred, excluded) | Acceptance evidence |
| --- | --- | --- | --- |
|  |  |  |  |

## Acceptance flows

| Flow | Starting state | User action | Expected visible result | Pass criterion |
| --- | --- | --- | --- | --- |
| Primary success |  |  |  |  |
| Failure/cancel |  |  |  |  |
| Recovery/resume |  |  |  |  |

## Implementation sequence

Keep each implementation stage reviewable under the repository change-size
guidance. A stage may land behind a non-user-accessible boundary, but the
initiative remains unfinished until every acceptance and release gate passes.

1. `<first implementation stage>`
2. `<second implementation stage>`
3. `<third implementation stage>`

## Automated evidence

Run fix and formatting tools before the final affected tests.

| Check | Final-tree command | Result | Artifact |
| --- | --- | --- | --- |
| Focused |  | pending |  |
| Integration |  | pending |  |
| Snapshot, if applicable |  | pending |  |
| Adversarial or security, if applicable |  | pending |  |

## True-TUI evidence

Launch through the repository TUI workflow. Send prompt text and Enter as
separate key actions. Corbanu `exec` is not acceptable proof.

| Flow | Candidate binary | Test repo/worktree | Keys/actions | Visible checkpoints | Result | Artifact |
| --- | --- | --- | --- | --- | --- | --- |
| Primary |  |  |  |  | pending |  |
| Failure/cancel |  |  |  |  | pending |  |
| Recovery/resume |  |  |  |  | pending |  |

## Live-repository applicability

| Repository | Applicable to this initiative? | Resolved checkout/test worktree | Base commit | Reason or result |
| --- | --- | --- | --- | --- |
| TensorCash | yes/no |  |  |  |
| Isometric Game | yes/no |  |  |  |

## Human acceptance

| Tester | Date | Candidate version/commit | Flow | Result | Evidence |
| --- | --- | --- | --- | --- | --- |
|  |  |  |  | pending |  |

## Documentation

| Finished-feature doc | Product-spec citation present | Verified candidate |
| --- | --- | --- |
|  |  |  |

## Dependencies, decisions, and blockers

| Item | Type | Owner | Needed by | State / decision |
| --- | --- | --- | --- | --- |
|  |  |  |  |  |

## Release linkage

- Release record: `qa/release/<version>/`
- Benchmark tracker row:
- Remaining blocker:

## Completion

- [ ] Product linkage, scope, invariants, and worktrees are current.
- [ ] Every implementation unit is represented by a valid single-feature sprint.
- [ ] Required final-tree automated evidence passes.
- [ ] Required true-TUI and live-repository evidence passes.
- [ ] Human acceptance passes.
- [ ] Finished documentation matches the candidate.
- [ ] Release and benchmark records are linked.
- [ ] No hard release gate remains pending.
