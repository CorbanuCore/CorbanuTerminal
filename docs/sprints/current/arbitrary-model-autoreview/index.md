# Arbitrary-model Autoreview execution sprints

These are the current mechanical records for the proposed
[Arbitrary-model Autoreview](../../../plans/proposed/arbitrary-model-autoreview.md)
plan. All seven sprints implement the single `PF-13` feature.

The plan and every sprint remain `draft`. They do not authorize implementation.
Before a sprint can move to `ready`, the plan must be activated, an exact
worktree/branch/base commit must be assigned, and its dependencies must be
completed and archived.

| Order | Sprint | Outcome | Status | Depends on |
| ---: | --- | --- | --- | --- |
| 1 | [PF-13-S01](pf-13-s01-request-and-skill-contract.md) | Explicit skill and typed request | draft | — |
| 2 | [PF-13-S02](pf-13-s02-review-packet-and-secret-gate.md) | Complete packet and fail-closed scan | draft | `PF-13-S01` |
| 3 | [PF-13-S03](pf-13-s03-core-provider-readiness.md) | Shared Core auth readiness | draft | `PF-13-S01` |
| 4 | [PF-13-S04](pf-13-s04-exact-runtime-dispatch.md) | Deterministic cross-provider dispatch | draft | `PF-13-S03` |
| 5 | [PF-13-S05](pf-13-s05-full-history-invariant.md) | Fail-closed full-history inheritance | draft | `PF-13-S04` |
| 6 | [PF-13-S06](pf-13-s06-isolated-review-runner.md) | Isolated bounded reviewer and report | draft | `PF-13-S02, PF-13-S04, PF-13-S05` |
| 7 | [PF-13-S07](pf-13-s07-tui-qualification-and-docs.md) | Qualified TUI flow and finished docs | draft | `PF-13-S06` |

Work only from one sprint's **Remaining** checklist. Completed records move to
`docs/sprints/archive/arbitrary-model-autoreview/` and leave MkDocs navigation.

## Machine check

```bash
python3 docs/sprints/check.py --json
```
