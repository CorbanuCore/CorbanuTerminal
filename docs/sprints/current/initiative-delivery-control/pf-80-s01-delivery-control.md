---
sprint_id: "PF-80-S01"
title: "Native Task Node delivery-control integration"
status: in_progress
plan_file: "docs/plans/active/initiative-delivery-control.md"
plan_feature: "PF-80"
execution_order: 1
owner: "Astra High Task Node kickoff"
parallel_lane: "tasknode-control-port"
write_scope: "scripts/initiative_control/, docs/research/tasknode-integration/, qa/initiative-control/pf-80-s01/"
integration_gate: "Codex management audits the source-port manifest and literal worker diff, reruns control and governance suites on the combined tree, reviews the immutable queue migration/one-event preview, and owns CI registration and any later live source cutover; no worker main push or external mutation."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/tasknode-pf80-s01-20260911"
branch: "workstream/tasknode-pf80-s01-20260911"
base_commit: "295aed26e53b17f919f7199ae1c9748b1b1250ba"
depends_on: "none"
created: 2026-09-10
updated: 2026-09-11
---

# PF-80-S01 — Native Task Node delivery-control integration

Allocated before post-merge dispatch; status reserves the lane, not proof a
worker has started. Manager receipt records native agent ID and actual HEAD.

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

## Preconditions

- [x] Travis selected workstream 3 and authorized an Astra High kickoff after main planning merge.
- [x] Native account and three Proposed personal targets verified; this is not public-beta or live-progress authority.
- [x] Exact owner, worker worktree/base and disjoint scope allocated; dispatch waits for the verified main merge and worker fast-forward.

## Done

- [x] Recovery source implemented and independently reviewed the internal dashboard; historical evidence retained without claiming a main-tree port pass.
- [x] Verified native account setup and generated task IDs; no seed/password needed, no task accepted or completed.
- [x] Recorded collision-free PF-80 receiving identity and two dependent PF-79 beta drafts.

## Remaining

- [ ] Manifest and port only the existing internal tooling; retain main's newer policy/tests and native API-balance semantics.
- [ ] Pin current Task Node source/contract; distinguish read-only capability checks from live entitlement and board grants. Prefer existing native transport over a second integration authority.
- [ ] Prepare redacted preview and a bounded one-event qualification path; retain immutable IDs, mapping checks, disable behavior and safe retry/restart semantics.
- [ ] Reconcile historical PF-76 reports/queued events with PF-80 explicitly; no silent replay, deletion or automatic flushing of the historical batch.
- [ ] Test disabled, stale/unsafe input, malformed/failed publication, expired/wrong-owner auth, duplicate send, timeout and changed-payload conflicts offline.
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
