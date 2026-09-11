---
sprint_id: "PF-81-S01"
title: "Bounded screenshot and inference QA harness"
status: draft
plan_file: "docs/plans/active/initiative-delivery-control.md"
plan_feature: "PF-81"
execution_order: 4
owner: "Existing PF13/provider-reauth task; Travis Good accountable"
parallel_lane: "delivery-visual-qa-followup"
write_scope: "scripts/visual_test_harness/, qa/visual-test-harness/"
integration_gate: "Codex management audits only the scoped commit, reruns offline harness and governance tests on the combined tree, and confirms no live surface or endpoint is enabled; never merge the provider branch wholesale."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/provider-reauth-health"
branch: "feat/provider-reauth-health"
base_commit: "eb7af932bc8cc859122b5b2dbef223fbd45ea25c"
depends_on: "PF-80-S01"
created: 2026-09-11
updated: 2026-09-11
---

# PF-81-S01 — Bounded screenshot and inference QA harness

Draft-only destination, not execution authority. Godel retains PF-80-S01.
No additional agent/reviewer is allocated. Existing beta drafts are unchanged.

## Execution mandate

- Deliver: standalone command harness, setup instructions and deterministic screenshot/action fixtures for delivery-control QA.
- Excludes: Terminal or Desktop runtime changes, arbitrary-shell model actions, native application control, credential/Keychain access, live inference deployment and release.
- Computer Use Terminal denial remains effective; another driver, CLI, SSH hop or inference model must not bypass it.

## Plan linkage

- [Delivery control](../../../plans/active/initiative-delivery-control.md), feature PF-81.
- Product citation: **Internal delivery control — TO BUILD**, “standalone screenshot/inference QA infrastructure”.
- [Allocation and integration handoff](../../../plans/visual-test-harness-allocation-2026-09-11.md).

## Code boundaries

- Planned CLI, schemas, validators, bounded runner and isolated fixture driver: `scripts/visual_test_harness/`.
- Setup instructions, protocol, synthetic inputs, executable tests and evidence: `qa/visual-test-harness/`.
- Read existing delivery-control report contracts after PF-80-S01 acceptance; submit a separate manager proposal if a shared adapter needs changes.
- `humanTest.html`, PF-58 evidence, runtime, shared manifests, plan/sprint files and CI are outside worker scope. Preserve existing branch edits.

## Preconditions

- [x] Manager verified requested branch/path/HEAD and recorded the relayed human decision without interrupting any worker.
- [ ] PF-80-S01 completed and archived with required human/live gates; an offline artifact alone does not clear the dependency.
- [ ] Manager selects this follow-up, reconciles branch drift and updates matching plan/sprint base coordinates; no other reserved delivery-control sprint.
- [ ] Plan/sprint checkers pass and status is promoted to ready/in_progress before implementation.

## Done

- [x] Allocated a draft single-feature record with disjoint literal paths and a combined-tree integration gate.
- [x] Recorded that restricted native testing, human acceptance, inference access and new review capacity remain unproven.

## Remaining

- [ ] Freeze a versioned screenshot/request/action/result protocol: candidate and frame identity, dimensions, driver identity, allowed actions and explicit outcome types.
- [ ] Implement exact typed validation rejecting unknown keys/actions, malformed JSON, non-finite or out-of-range coordinates, stale frame IDs and unsupported targets before dispatch.
- [ ] Build an isolated fixture driver with no native control or shell execution; accept only validated actions against synthetic fixtures.
- [ ] Enforce operator-set action/time/payload/inference budgets and cancellation at every boundary; no unbounded retries or automatic resume after a stop/denial.
- [ ] Add fail-closed inference transport boundaries and recorded-response replay; default to fixture/offline mode, no implicit endpoint, ambient auth or model-provided destination.
- [ ] Document RTX endpoint setup and operator-controlled commands, exact endpoint/model pins, data permission, timeout/stop and cleanup requirements; instructions are not deployment or permission grants.
- [ ] Preserve redacted run manifests and per-action evidence with hashes, candidate/driver/model identity, requested versus executed actions and blocked/not-run/pass/fail outcomes.
- [ ] Document seeded successes/failures, cancellation and fresh-start recovery; never relabel synthetic or denied native cases as real product passes.

## Verification

- [ ] Focused planned command: `python3 -m unittest discover -s qa/visual-test-harness -p 'test_*.py'`; report actual nonzero test count.
- [ ] Cover invalid/unknown actions and keys, shell payloads, stale frames, coordinate bounds, malformed/oversize responses, timeout, endpoint denial, retry caps and cancel without another action.
- [ ] Prove fixture mode cannot access the network, credentials or native surfaces; restart does not silently replay an action.
- [ ] Record deterministic CLI success/failure/cancel/recovery with synthetic screenshots and replayed inference; setup instructions match actual CLI help.
- [ ] Run `python3 docs/plans/check.py`, `python3 docs/sprints/check.py` and `git diff --check` on the receiving tree.
- [ ] TUI/live-repository applicability: standalone fixture infrastructure only; no Terminal/native acceptance, TensorCash, Isometric or GPU benchmark pass claimed. A permitted live target needs a separate scoped gate.

## Exit evidence

- [ ] Scoped implementation commit, protocol version, fixture hashes, actual test output and command transcript linked.
- [ ] Independent review capacity and human acceptance resolved explicitly; exhausted PF-58 review allowance is not reused or silently replenished.
- [ ] Manager scope audit and combined-tree tests recorded; no broad provider-branch merge, main push, remote deployment or public evidence export by worker.
- [ ] Done/Remaining updated and accepted record archived; missing mandatory evidence stays open.
