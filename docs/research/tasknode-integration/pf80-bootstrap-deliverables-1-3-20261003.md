# PF-80-S01 bootstrap deliverables 1–3: commits and QA records

Record for Task Node task `task_9d4fc2d8b21c3fd3ab2bc5ce6b18a188`, written
2026-10-03 by the Task Node integration worker. It maps bootstrap deliverables
1–3 to their commits and QA records. Every label below repeats what the cited
QA record states, and nothing more.

- Sprint: PF-80-S01 (`in_progress`), plan `docs/plans/active/initiative-delivery-control.md`.
- Product citation: **Internal delivery control — TO BUILD**, “Use sequential
  sprints per initiative.”
- Task map: [tasknode-task-map.md](../../../qa/initiative-control/management-bootstrap/tasknode-task-map.md), section 3a.

## Ancestry

Command: `git merge-base --is-ancestor <commit> 577526daab36f31055ba3cd678dc4cc503d22d62`,
where `577526daab…` was the pushed tip of `integrate/management-workstreams-20260911`
(`git ls-remote origin`) on 2026-10-03. Every commit below is an ancestor.

| Commit | Full hash | Subject |
| --- | --- | --- |
| `c77123a7e9` | `c77123a7e9d4540a5b3ff19a5a9dc0f12848c923` | Merge commit 'adc9eece3ffdca782d0b529789c0762d8bc6bc56' into integrate |
| `ca0474fe4` | `ca0474fe43ec868c9171c3b671dfe7161d97c2c5` | Execute prepared lifecycle proposals in owner transactions |
| `ee93206c0` | `ee93206c0f8b070bd9cc767defb3d364ba4fd06b` | Merge commit 'ca0474fe43ec868c9171c3b671dfe7161d97c2c5' into integrate |
| `2093475e8` | `2093475e80a94a48b0b6bb25578a4a41423e3f8b` | Add durable native owner bridge with real lifecycle qualification |
| `d3f742ac3` | `d3f742ac38058ebd3f89364d82b65ddcec8ae89f` | Merge commit '2093475e80a94a48b0b6bb25578a4a41423e3f8b' into integrate |

## Deliverables

| # | Deliverable (task map) | Commits | QA record | Verified label, as the record states it |
| --- | --- | --- | --- | --- |
| 1 | Supervised fresh-Fable receive/failure/reverification loop | `c77123a7e9` | [supervised-qualification-20260913.md](../../../qa/initiative-control/management-bootstrap/supervised-qualification-20260913.md) | Supervised real handoff/recovery, not unattended recurrence or complete product acceptance |
| 2 | Owner lifecycle transactions | `ca0474fe4`, received at `ee93206c0` | [full-lifecycle-rehearsal-20260913.md](../../../qa/initiative-control/management-bootstrap/full-lifecycle-rehearsal-20260913.md) | Operator-driven completion/archive/successor rehearsal; does not qualify an unattended controller |
| 3 | Native owner bridge | `2093475e8`, received at `d3f742ac3` | [native-owner-receiving-20260913.md](../../../qa/initiative-control/management-bootstrap/native-owner-receiving-20260913.md) | Supervised real worker → fresh manager → fresh integration-worker checkpoint |
| 1–3 | Audit of all five deliverables | — | [bootstrap-deliverables-audit-20260916.md](../../../qa/initiative-control/management-bootstrap/bootstrap-deliverables-audit-20260916.md) | Deliverables 1, 2 and 3 “satisfied, live”; 4 “partly open”; 5 “open” (as of 2026-09-16) |

## Test counts, as stated in each record

- `supervised-qualification-20260913.md`: first run “482 tests, one failure in
  272.299 seconds”; rerun “All 482 tests passed in 272.454 seconds”; “Plans 3/3,
  sprints 115 current/126 archived”.
- `full-lifecycle-rehearsal-20260913.md`: no unit-test count. It states “Four
  bad-status and three wrong-action checks reject; both positive controls pass”,
  and that the CLI ledger holds 79 successful operations and 11 expected denials,
  which “are operation counts, not79 distinct tests”.
- `native-owner-receiving-20260913.md`: “passed63 tests”; “Tests104 then73
  passed”; “106 native-owner/coordinator/manager-cycle/integration tests passed
  in8.068s”; “plans3/3, sprints115current126archived”.
- `bootstrap-deliverables-audit-20260916.md`: no test count. It is a
  state-and-code audit; its evidence is the live coordinator audit table
  (`begin_manager 187`, `manager_decision 120`, `dispatched 134`,
  `acknowledged 135`, `verified 130`, `owner_complete 1`, `owner_archive 1`,
  `owner_successor 1`, among others).

## Excluded

This record does not cover Slack alerting (deliverable 4), recurrence install
(deliverable 5), Task Node writeback or PF-79 beta work. It claims no recurring,
functional or human acceptance beyond what the four QA records state.
