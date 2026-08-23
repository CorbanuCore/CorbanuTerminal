---
title: "<product initiative>"
status: draft
change_class: product-initiative
owner: "<name or accountable role>"
target_release: "<version or TBD>"
created: YYYY-MM-DD
updated: YYYY-MM-DD
product_spec:
  file: docs/corbanu-product-spec.md
  heading: "<exact heading text>"
  requirement_excerpt: "<short exact excerpt>"
implementation_worktrees:
  - path: "<path>"
    branch: "<branch>"
    base_commit: "<commit>"
---

# <Plan title>

Policy: repository-root `AGENTS.md`
Plan lifecycle: `plans/README.md`

## User pain

<What is painful or impossible for the user today?>

## Product intent and ideal flow

<Describe the best end-to-end experience in plain English: entry, success,
failure, recovery, and return use.>

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

## Ownership and implementation worktrees

| Owner | Worktree | Branch | Base commit | Scope |
| --- | --- | --- | --- | --- |
|  |  |  |  |  |

## Useful code references

| Path or symbol | Why it matters |
| --- | --- |
|  |  |

## Acceptance flows

| Flow | Starting state | User action | Expected visible result | Pass criterion |
| --- | --- | --- | --- | --- |
| Primary success |  |  |  |  |
| Failure |  |  |  |  |
| Recovery/resume |  |  |  |  |

## Implementation sequence

1. `<first implementation step>`
2. `<second implementation step>`
3. `<third implementation step>`

## Automated evidence

| Check | Final-tree command | Result | Artifact |
| --- | --- | --- | --- |
| Focused |  | pending |  |
| Integration |  | pending |  |
| Snapshot, if applicable |  | pending |  |

## True-TUI evidence

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

## Release linkage

- Release record: `qa/release/<version>/`
- Benchmark tracker row:
- Remaining blocker:

## Completion

- [ ] Product linkage, scope, and worktrees are current.
- [ ] Required final-tree automated evidence passes.
- [ ] Required true-TUI and live-repository evidence passes.
- [ ] Human acceptance passes.
- [ ] Finished documentation matches the candidate.
- [ ] Release and benchmark records are linked.
- [ ] No hard release gate remains pending.
