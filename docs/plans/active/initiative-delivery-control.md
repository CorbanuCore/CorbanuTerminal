---
title: "3. Task Node integration and delivery control"
status: active
change_class: product-initiative
priority: P1
owner: "Codex Task Node integration lane; Travis Good accountable"
parallel_sprint_limit: 1
integration_owner: "Codex management"
activation_authority: "Travis Good"
activation_basis: "September 10 delivery control request; September 11 three-workstream selection, task creation, beta planning and Astra High kickoff"
target_release: "Internal operations; no Terminal or Desktop release authorized"
deadline: "TBD"
created: 2026-09-10
updated: 2026-09-11
product_spec:
  file: docs/corbanu-product-spec.md
  heading: "Internal delivery control — TO BUILD"
  requirement_excerpt: "Use sequential sprints per initiative"
implementation_worktrees:
  - path: "/Volumes/CorbanuDrive/Corbanu/worktrees/management-workstreams-20260911"
    branch: "integrate/management-workstreams-20260911"
    base_commit: "295aed26e53b17f919f7199ae1c9748b1b1250ba"
  - path: "/Volumes/CorbanuDrive/Corbanu/worktrees/tasknode-pf80-s01-20260911"
    branch: "workstream/tasknode-pf80-s01-20260911"
    base_commit: "295aed26e53b17f919f7199ae1c9748b1b1250ba"
---

# 3. Task Node integration and delivery control

## Activation record

Slot 3/3. Reuse native `/tasknode` and Campaign Tracker, not another auth system
or scheduler. The [main handoff](../main-workstreams-2026-09-11.md) owns source
and ID migration. Existing Linux dashboard operation is preserved separately.

## User pain

Parallel agent output exceeds Travis's one-hour daily review budget.

## Product intent and ideal flow

Open the
private overview, identify the next decision and inspect exact candidate evidence,
blockers, machine/run reports and human test plans. Workers cannot self-accept.

## Product linkage

**Internal delivery control — TO BUILD**, “Use sequential sprints per initiative”.
## Scope

PF-80 owns the internal projection/progress adapter; PF-79 owns the subsequent
public Desktop beta program. No fourth initiative, automatic merge, release,
reward, financial action or private-recording export is authorized.

PF-80 replaces only the recovery operations source's conflicting PF-76 identity.
Main's provider-persistence PF-76 and security PF-77/PF-78 remain untouched.
Historical QA and durable outbox IDs keep their original identity. Never silently
remap old events or replay them as new work; migrate only after reconciliation.

## Existing evidence versus main deliverable

Native account linking and three Proposed personal task targets were verified
in installed Corbanu 0.1.36. [Task receipts](../tasknode-workstream-tasks-2026-09-11.md)
and [setup checks](../../../qa/initiative-control/2026-09-11/tasknode-setup.md)
do not establish accepted tasks, current worker profile isolation or live posting.
The private dashboard, tested adapter and default-OFF checker exist on the
recovery source, not this main baseline. PF-80-S01 must port the bounded tooling,
qualify it here and prepare a reviewed first-event path. Do not copy the whole
recovery branch or any auth/session/state directory.

## Invariants

One manager owns shared plans, allocation and dashboard configuration. A worker
owns only its assigned implementation/research/QA paths and per-run report.
## Ownership and implementation worktrees

The first checkout above is the manager, the second the Astra High Task Node
worker. Fast-forward the worker to the merged planning commit before dispatch;
record actual HEAD in the kickoff receipt. No raw HTML, remote images, credentials or private logs in
the projection. Unknown/stale evidence stays visible. OFF hides discovery and
denies new work while preserving data and supported recovery.

## Useful code references

Reuse `codex-rs/cli/src/tasknode_cmd.rs`, native tasknode-session/profile and
Campaign Tracker contracts read-only initially. Port `scripts/initiative_control/`
from the inspected recovery source under an audited manifest; tests must run on
the receiving main tree. Keep `docs/plans/check.py` and `docs/sprints/check.py`
manager-owned. No Rust/upstream protocol modification is allocated by kickoff.
If one becomes necessary, return an exact scoped follow-up instead of widening
authority. [Feature delivery contract](../feature-delivery.md).

## Sprint execution map

| Feature | Sprint | State and acceptance |
| --- | --- | --- |
| PF-80 | [S01 native delivery-control integration](../../sprints/current/initiative-delivery-control/pf-80-s01-delivery-control.md) | Allocated for post-merge kickoff; port/requalify tooling, offline first-event preparation, then separately authorized live acceptance |
| PF-79 | [S01 Desktop beta channel/test contract](../../sprints/current/initiative-delivery-control/pf-79-s01-beta-channel-and-test-contract.md) | Draft; depends on PF-80-S01; Desktop source/permissions unresolved |
| PF-79 | [S02 public beta pilot](../../sprints/current/initiative-delivery-control/pf-79-s02-public-beta-pilot.md) | Draft; depends on S01; no public launch |

## Acceptance flows

Success: inspect a real plan, candidate and next human decision. Failure: invalid
reports or publication preserve last-good state. Recovery: restart and relink
through supported controls, retaining IDs and visible blockers.

## Implementation sequence

First audit/port the bounded internal tooling and pinned native progress
contract, then test report/queue/publication recovery, then prepare human-approved
first delivery. Reconcile every queued event before enabling anything; an
enrolled bulk flush is not a safe one-event qualification. Keep posting OFF
until the operator authorizes the account/destination/scope and exact payload.
Beta follows the [detailed contract](../tasknode-beta-program.md): isolated exact
Desktop candidate, public safe cards, independent testing and a bounded pilot.

## Automated evidence

Focused Python/governance suites, negative HTTP fixtures, retained last-good
publication and source-provenance checks must pass on the receiving tree.

## True-TUI evidence

Native
interactive changes require actual-key true-TUI proof; document-only evidence
does not qualify them.

## Human acceptance

Named human acceptance, entitlement, supported proposed-
target lifecycle and remote credential permission remain pending. Do not import
seeds or distribute the linked broad human session to workers.

## Live-repository applicability

TensorCash/Isometric are not applicable to the internal document projection;
runtime and release journeys retain the root applicability/qualification rules.
## Documentation

Unfinished contracts remain
plans/research/QA; finished product documentation waits for qualification.

## Dependencies, decisions, and blockers

Port/runtime review, supported public/credential authority and human acceptance
remain gates. No live post, beta launch or reward decision is authorized here.

## Release linkage

No release or benchmark completion is asserted. Root release policy is unchanged.

## Completion

- [x] Exact independent worker coordinates, literal write scope and combined-tree gate recorded; dispatch follows main merge.
- [ ] Main port and focused tests/review complete; existing source evidence not relabeled as a new pass.
- [ ] Human accepts dashboard/recovery, target lifecycle, entitlement and exact first delivery; posting remains OFF until then.
- [ ] Desktop repository/channel owner, public board scope, evidence destination, cohort and reward policy resolved before beta readiness.
- [ ] Sequential sprint evidence and required human/release linkage complete before archive or shipping claims.
