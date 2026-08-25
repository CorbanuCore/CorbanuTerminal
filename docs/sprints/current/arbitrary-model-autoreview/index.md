# Arbitrary-model Autoreview execution sprints

These are the current mechanical records for the proposed
[Arbitrary-model Autoreview](../../../plans/proposed/arbitrary-model-autoreview.md)
plan. All seven sprints implement the single `PF-14` feature.

The plan and every sprint remain `draft`. They do not authorize implementation.
Before a sprint can move to `ready`, the plan must be activated, an exact
worktree/branch/base commit must be assigned, and its dependencies must be
completed and archived.

| Order | Sprint | Outcome | Status | Depends on |
| ---: | --- | --- | --- | --- |
| 1 | [PF-14-S01](pf-14-s01-request-and-skill-contract.md) | Explicit skill and typed request | draft | — |
| 2 | [PF-14-S02](pf-14-s02-review-packet-and-secret-gate.md) | Complete packet and fail-closed scan | draft | `PF-14-S01` |
| 3 | [PF-14-S03](pf-14-s03-core-provider-readiness.md) | Shared Core auth readiness | draft | `PF-14-S01` |
| 4 | [PF-14-S04](pf-14-s04-exact-runtime-dispatch.md) | Deterministic cross-provider dispatch | draft | `PF-14-S03` |
| 5 | [PF-14-S05](pf-14-s05-full-history-invariant.md) | Fail-closed full-history inheritance | draft | `PF-14-S04` |
| 6 | [PF-14-S06](pf-14-s06-isolated-review-runner.md) | Isolated bounded reviewer and report | draft | `PF-14-S02, PF-14-S04, PF-14-S05` |
| 7 | [PF-14-S07](pf-14-s07-tui-qualification-and-docs.md) | Qualified TUI flow and finished docs | draft | `PF-14-S06` |

Work only from one sprint's **Remaining** checklist. Completed records move to
`docs/sprints/archive/arbitrary-model-autoreview/` and leave MkDocs navigation.

## Machine check

```bash
python3 docs/sprints/check.py --json
```
