---
sprint_id: "PF-80-S01"
title: "Native Task Node delivery-control integration"
status: in_progress
plan_file: "docs/plans/active/initiative-delivery-control.md"
plan_feature: "PF-80"
execution_order: 1
owner: "Astra High Task Node native preparation"
parallel_lane: "tasknode-native-preparation"
write_scope: "codex-rs/tasknode-session/src/delivery_goal.rs, codex-rs/tasknode-session/src/delivery_goal_tests.rs, codex-rs/tasknode-session/src/lib.rs, qa/initiative-control/pf-80-s01/native-preparation/"
integration_gate: "Codex management audits the source-port manifest and literal worker diff, reruns control and governance suites on the combined tree, reviews the immutable queue migration/one-event preview, and owns CI registration and any later live source cutover; no worker main push or external mutation."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/tasknode-pf80-s01-20260911"
branch: "workstream/tasknode-pf80-s01-20260911"
base_commit: "0415a00dc3d3ee55a96662920c3eedbc0f0d4838"
depends_on: "none"
created: 2026-09-10
updated: 2026-09-11
---

# PF-80-S01 — Native Task Node delivery-control integration

Port returned, reviewed and integrated locally. This same sprint now allocates
native pure preparation; [handoff](../../../plans/workstream-manager-handoff-2026-09-11.md#task-node-next-assignment)
owns exact boundaries, upstream pins and tests. Agent identity is recorded at dispatch.

## Execution mandate

- Deliver: scoped main-tree port of existing internal projection/progress tooling, recoverable native contract and reviewed first-event preparation.
- Excludes: new scheduler, native auth redesign, automatic task acceptance/rewards, beta launch and broad credential distribution.
- Identity: recovery-source PF-76-S01 is this feature's historical name, not main's provider-persistence feature. Preserve old event IDs/receipts.

## Plan linkage

- [Workstream 3](../../../plans/active/initiative-delivery-control.md), PF-80.
- Product citation: **Internal delivery control — TO BUILD**, “Use sequential sprints per initiative”.
- [Main migration/handoff](../../../plans/main-workstreams-2026-09-11.md); [historical QA](../../../../qa/initiative-control/2026-09-11/verification.md).

## Code boundaries

- Existing native contracts: `codex-rs/cli/src/tasknode_cmd.rs`, native Campaign Tracker and tasknode-session/profile behavior, read-only for kickoff.
- Port: `scripts/initiative_control/` from the recovery source; audit every file, no state/auth/credential directories.
- Planned contract and tests: `docs/research/tasknode-integration/`, `qa/initiative-control/pf-80-s01/`, focused tests beside the ported scripts.
- Manager alone owns plan/sprint allocation, CI registration and live dashboard state; submit a handoff for any needed shared changes.
- Current worker: only front-matter paths; lib.rs gets one module declaration. The old Python allocation is closed. No CLI, Client, tracker, auth, manifests or lockfile writes.

## Preconditions

- [x] Travis selected workstream 3 and authorized an Astra High kickoff after main planning merge.
- [x] Native account and three Proposed personal targets verified; this is not public-beta or live-progress authority.
- [x] Exact owner, worker worktree/base and disjoint scope allocated; dispatch waits for the verified main merge and worker fast-forward.

## Done

- [x] Recovery source implemented and independently reviewed the internal dashboard; historical evidence retained without claiming a main-tree port pass.
- [x] Verified native account setup and generated task IDs; no seed/password needed, no task accepted or completed.
- [x] Recorded collision-free PF-80 receiving identity and two dependent PF-79 beta drafts.
- [x] Audited port, pinned first-party contract, offline preview and conservative PF-76 hold reviewed; candidate 8640ca452 integrated locally at 0415a00dc.
- [x] Combined baseline passed 54 focused tests plus both governance checkers; no native/live or human acceptance inferred.

## Remaining

- [ ] Implement and test the pure native immutable-goal preparation slice from the linked handoff; no production entry or sender.
- [ ] Manager reviews the returned native slice and allocates the subsequent profile-bound one-event transport within PF-80; do not delegate live authority.
- [ ] Reconcile historical PF-76 reports/queued events with PF-80 explicitly; no silent replay, deletion or automatic flushing of the historical batch.
- [ ] Preserve passing Python negative coverage; add native selected-ID/digest, schema, mapping, byte-limit and explicit-time negatives with cross-language goldens.
- [ ] Give manager exact credential-scope/destination, enrollment, task-lifecycle and first-payload decisions; remain OFF if missing authority. Never inspect real credential stores as a worker.
- [ ] After separate approval, qualify one live goal event and recovery; do not treat fixtures as delivery proof.
- [ ] Manager receives reviewed commit/evidence and controls source-sync cutover; existing remote service stays untouched by worker.

## Verification

- [ ] Focused: `python3 -m unittest discover -s scripts/initiative_control -p 'test_*.py'`; install only pinned requirements in a disposable venv.
- [ ] Governance: `python3 docs/plans/check.py`, `python3 docs/sprints/check.py`, `git diff --check`.
- [ ] Independent final-tree review and source manifest; inspect any registration/config changes before integration.
- [ ] Actual-key native `/tasknode` success/cancel/expiry/relink if interactive runtime changes; otherwise record not applicable with reason.
- [ ] Named human accepts dashboard/recovery and first-live-event evidence; all protected/financial actions remain excluded.

## Exit evidence

- [ ] Port commit, hashes, actual nonzero test counts, negative cases and candidate/contract pins recorded.
- [ ] Historical IDs, pending queue and receiving state reconciled without changing old receipts.
- [ ] Human/live gates complete before marking sprint completed or allowing PF-79 readiness.
- [ ] Manager audits combined tree, updates Done/Remaining and archives only on actual acceptance.
