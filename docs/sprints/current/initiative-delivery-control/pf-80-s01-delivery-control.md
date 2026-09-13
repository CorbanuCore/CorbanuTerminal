---
sprint_id: "PF-80-S01"
title: "Native Task Node delivery-control integration"
status: in_progress
plan_file: "docs/plans/active/initiative-delivery-control.md"
plan_feature: "PF-80"
execution_order: 1
owner: "James - Codex Task Node bounded transport lane"
parallel_lane: "tasknode-owned-ingress"
write_scope: "codex-rs/responses-api-proxy/src/lib.rs, codex-rs/responses-api-proxy/src/synthetic.rs, codex-rs/responses-api-proxy/src/synthetic_io.rs, codex-rs/responses-api-proxy/src/synthetic_exchange.rs, codex-rs/responses-api-proxy/src/synthetic_policy.rs, codex-rs/responses-api-proxy/src/synthetic_tests.rs, codex-rs/responses-api-proxy/src/synthetic_policy_tests.rs, codex-rs/responses-api-proxy/tests/synthetic_cli.rs, codex-rs/responses-api-proxy/README.md, codex-rs/responses-api-proxy/Cargo.toml"
integration_gate: "Exact owned-ingress allocation: source authoring only until manager grants serialized dependency/build lease; parent owns root aliases and derived locks. Actual socket/CLI/default compatibility tests and Cargo/Bazel parity, one new material review plus correction; preserve prior history. Fixture-only internal N/A, real isolated executor and Slack phone/CAS/native ACK remain separate mandatory gates. Live OFF."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/tasknode-owned-ingress-20260913"
branch: "workstream/tasknode-owned-ingress-20260913"
base_commit: "6dde60fface29b0678b792926cbcff7bc8cc542e"
depends_on: "none"
created: 2026-09-10
updated: 2026-09-12
---

# PF-80-S01 — Native Task Node delivery-control integration

Current mandate: [bounded synthetic owned ingress](../../../research/tasknode-integration/owned-ingress-allocation-20260913.md), ten exact crate-local paths1250target/1500total850non-test hard. James authors code now; parent serializes dependency/build lease after accounting correction. No real model/service/Slack qualification claim. Earlier supervisor/isolation work stays accepted history.
Port/preparation/engine/adapter/reconciliation are reviewed and integrated locally.
[Native validity allocation](../../../research/tasknode-integration/native-validity-allocation.md)
and [recovery handoff](../../../research/tasknode-integration/recovery-decision-handoff.md).

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
- Previous native/projection/reply scopes are frozen. Front matter allocates real Slack transport/manager callable; linked allocation owns exact shared manager registration. No live authority inferred.

## Preconditions

- [x] Travis selected workstream 3 and authorized an Astra High kickoff after main planning merge.
- [x] Native account and three Proposed personal targets verified; this is not public-beta or live-progress authority.
- [x] Exact owner, worker worktree/base and disjoint scope allocated; dispatch waits for the verified main merge and worker fast-forward.

## Done
- [x] Exact reviewed supervisor c7a1b9690 received92d6eb261;261 receiving tests pass187.549s, Facilities/governance pass. Review08 clean,09 unspent; internal-only N/A, actual Slack and independent functional acceptance remain open.
- [x] Gated real-SDK transport/shared status3561ebcc2 reviewed clean (combined06);244 staging tests pass, receiving243 plus exact source-guard rerun1 pass. [Integration limits/evidence](../../../../qa/initiative-control/status-display/slack-integration-20260912.md). Missing-fence and actual live/phone/ACK qualification remain; no S01 completion.
- [x] Offline recovery730577f1d integrated9877c058d after clean review04;169 receiving Python/Facilities/governance pass. Manager accepts1506/638; original1151/1274/1412 and three failed reviews remain. [Review ledger](../../../../qa/initiative-control/status-display/integration-20260912-1356.md).
- [x] Corrected F01 age1a6d9c8df reviewed/integrated at60c5671b9;122Python/Facilities Node pass. Initial review P2 ID collision corrected, one added corrective review clean; prior usage retained. [Receiving proof](../../../../qa/initiative-control/status-display/integration-20260912-1232.md). Actual browser/evidence qualification remains separate.
- [x] All26 original browser cases executed at72854ec77; independent check verified714filehashes,24supported/DEC021partial/DEC025advisory. F01 and Facilities phone-table observation retained; original evidence not relabeled as acceptance of newer code.
- [x] Feed72854ec77 accepted590lines/281non-test after reproduced destination-less export failure, compatibility correction and clean additional review;111combined tests pass including actual server. [Receipt](../../../../qa/initiative-control/pf-80-s01/decision-projection/feed-receipt.md). Prior reviews preserved; independent evidence remains pending.
- [x] Offline revision-bound decisions4ff73485c accepted after clean correction review;777lines,100combinedPython tests pass. Original VoltaDEC001..026 preserved; full UI/feed proof remains. [Receipt](../../../../qa/initiative-control/pf-80-s01/decision-projection/receipt.md).
- [x] Canonical receiving reconciled at 87e31f521672e627e6230d48fc16a4cfaa7ff44c; native validity remains private/no caller. Independent intent-only [design DEC-001..026](../../../../qa/initiative-control/pf-80-s01/decision-projection/design-proposal.md) frozen before implementation. Integrator authorizes +3 design/code/evidence reviews, preserving earlier usage.

- [x] Private native validitybda5b35b4 reviewed clean; combined staging04ba6b8b7 passes80TaskNode/242state tests. [Evidence](../../../../qa/initiative-control/native-staging-2026-09-12.md). No enabled caller; canonical receiving transfer/publication pending.

- [x] Travis approved this Mac/existing publisher for now and native server-backed no-expiry validity design; no credential/enrollment/post permission inferred.
- [x] Recovery source implemented and independently reviewed the internal dashboard; historical evidence retained without claiming a main-tree port pass.
- [x] Verified native account setup and generated task IDs; no seed/password needed, no task accepted or completed.
- [x] Recorded collision-free PF-80 receiving identity and two dependent PF-79 beta drafts.
- [x] Audited port, pinned first-party contract, offline preview and conservative PF-76 hold reviewed; candidate 8640ca452 integrated locally at 0415a00dc.
- [x] Combined baseline passed 54 focused tests plus both governance checkers; no native/live or human acceptance inferred.
- [x] Pure native immutable-goal preparation and Python goldens reviewed clean, worker 2b281975a integrated at c33d47f6c; all 42 native crate tests and 54 Python tests pass on combined tree. [Receipt](../../../../qa/initiative-control/pf-80-s01/native-preparation/handoff.md).
- [x] Selected-ID/full-digest, schema, mapping, byte-limit, secret/control and explicit-time negatives pass; two pinned cross-language goldens pass. Advisory-only, no sender.
- [x] Bound one-event engine 7e25982b5 reviewed clean and integrated at 486d2fb94; all 55 crate tests pass there. Fixture authority, fencing/cancel/unknown outcomes only. [Receipt](../../../../qa/initiative-control/pf-80-s01/native-one-event/receipt.md).
- [x] Native adapter d7bf73a52 reviewed clean and integrated at56295f668; all63 crate tests pass there. Actual build/codec/resolver with synthetic inputs, no send. [Original receipt](../../../../qa/initiative-control/pf-80-s01/native-adapter/receipt.md); manager allocation reconciliation in handoff.

- [x] Exact-goal reconciliation177ec93fc reviewed clean and integrated at3898eaa65;73 native tests pass there. In-memory exact-ID GET, strict injected observation and preserved404uncertainty; no send/retry/live claim. [Receipt](../../../../qa/initiative-control/pf-80-s01/native-reconciliation/receipt.md).

## Remaining
- [ ] Implement allocated bounded fixture-only ingress, then qualify integrated supervisor/recovery with actual isolated executor and Slack identity/connection/phone/restart/CAS/native ACK. Manager owns dependencies/lease/live prerequisites; OFF until qualified.
- [ ] Resolve retained DEC021 mixed-feed useful-content failure and baseline-provenance limit without rewriting cases; [completed independent check](../../../../qa/initiative-control/status-display/age-evidence-20260912.md) supports F01 age/collision only. No human-readiness claim or blocking independent Slack implementation; DEC025 remains advisory.
- [ ] Manager scopes accessible recovery/input/retention and restart/cross-process fences before the later CLI allocation; this is manager work, not unanswered product approval.
- [ ] Reconcile historical PF-76 reports/queued events with PF-80 explicitly; no silent replay, deletion or automatic flushing of the historical batch.
- [ ] Give manager exact credential-scope/destination, enrollment, task-lifecycle and first-payload decisions; remain OFF if missing authority. Never inspect real credential stores as a worker.
- [ ] Under Travis's September12 progress-writeback authorization, verify technical/identity/mapping/redaction/recovery gates, then qualify one bounded live progress event and recovery before ongoing enablement. No task acceptance/rewards/public beta; fixtures are not delivery proof.
- [ ] Manager receives reviewed commit/evidence and controls source-sync cutover; existing remote service stays untouched by worker.

## Verification
- [ ] Apply [independent isolated execution](../../../../qa/code-blind-functional/isolated-execution.md) to affected functional handoff; record schema-2 proof or integrator-accepted internal-only N/A and later gate. Historical tests are not upgraded.
- [ ] Focused: `python3 -m unittest discover -s scripts/initiative_control -p 'test_*.py'`; install only pinned requirements in a disposable venv.
- [ ] Native: session_validity plus existing delivery/recovery selectors, full crate via just test and normal-library check after guarded formatting; production-compiled private module, no public caller.
- [ ] Governance: `python3 docs/plans/check.py`, `python3 docs/sprints/check.py`, `git diff --check`.
- [ ] Independent final-tree review and source manifest; inspect any registration/config changes before integration.
- [ ] Actual-key native `/tasknode` success/cancel/expiry/relink if interactive runtime changes; otherwise record not applicable with reason.
- [ ] Named human accepts dashboard/recovery and first-live-event evidence; all protected/financial actions remain excluded.

## Exit evidence
- [ ] Port commit, hashes, actual nonzero test counts, negative cases and candidate/contract pins recorded.
- [ ] Historical IDs, pending queue and receiving state reconciled without changing old receipts.
- [ ] Human/live gates complete before marking sprint completed or allowing PF-79 readiness.
- [ ] Manager audits combined tree, updates Done/Remaining and archives only on actual acceptance.
