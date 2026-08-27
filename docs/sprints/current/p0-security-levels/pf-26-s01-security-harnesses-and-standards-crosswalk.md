---
sprint_id: "PF-26-S01"
title: "Security harnesses and standards crosswalk"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-26"
execution_order: 15
owner: "Jim Ricketts"
lane: "harness"
write_scope: "scripts/security-level-compat, scripts/security-level-adversarial, scripts/security-level-standards-check, scripts/test_security_level_compat.py, scripts/test_security_level_adversarial.py, scripts/test_security_level_standards_check.py, qa/security-levels/fixtures"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-21-S01, PF-27-S01"
created: 2026-08-24
updated: 2026-08-27
---

# PF-26-S01 — Security harnesses and standards crosswalk

## Execution mandate

- Deliver: early compatibility/adversarial fixtures, runners, and crosswalk schema for implementation feedback; not final candidate qualification.
- Excludes: final release-candidate runs, runtime fixes, live-repository QA, and human acceptance; PF-26-S04 owns final automated proof.

## Plan linkage

- Upstream: [plan touch record](../../../plans/active/p0-security-levels.md#upstream-touch-record); resolve this sprint's adapter rows.
- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md)
- Feature: `PF-26`
- Acceptance advanced: every adopted control has an executable fixture and named owner; product results remain pending until final qualification.

## Code boundaries

- Planned: `scripts/{security-level-compat,security-level-adversarial,security-level-standards-check}`
- Planned: `qa/security-levels/fixtures/`; final manifest instantiated by PF-26-S04.
- Tests: Python runner/checker self-tests; Rust contract-fixture tests only where affected

## Preconditions

- [ ] Plan upstream baseline, adapter ownership, and exact contract tests are resolved before readiness.
- [ ] Every listed dependency is completed and archived.
- [ ] Harness and PF-27 contract commits are pinned; a final product candidate is not required.
- [ ] Exact worktree coordinates match the active plan.

- [ ] Allocate lane/worktree/base in the plan and validate disjoint write scopes before readiness.

## Done

- [x] Sprint record is linked only to PF-26.

## Remaining

- [ ] Extend the existing compatibility runner and freeze the baseline; do not overwrite PF-21 evidence.
- [ ] Build synthetic hostile-source fixtures, unique canaries, fake financial actions, and all-source/all-sink assertions.
- [ ] Require an adapter/owner/support/fixture entry for every ingress; add non-secret task hijacking, test-weakening, review-output injection, and benign controls with separate task-integrity and policy results.
- [ ] Publish upstream-adapter fixtures for wire schemas, native children, compaction/memory/resume, current authority after reconnect, and duplicate-action detection using fake executors.
- [ ] Provide local capture-proxy and scanner fixtures for the PF-13 true-TUI contract without live provider credentials.
- [ ] Build provenance/compaction/child/memory, reflected-error/environment, browser containment/egress, and forced-classifier-miss cases.
- [ ] Define versioned standards/design crosswalk schema with explicit pending/unavailable/failed/passed results; missing evidence never passes.
- [ ] Implement runners against PF-27 interfaces; fixture/runner self-tests must pass even while product cases remain explicitly pending.
- [ ] Publish commands and fixture digests for feature lanes; leave final integrated qualification to PF-26-S04.

## Verification

- [ ] Record applicable upstream adapter evidence or justified non-applicability; structural checks alone are not qualification.
- [ ] Rust fix/format precedes final affected tests; inspect the exact candidate diff.
- [ ] Harness tests: `python3 -m unittest discover -s scripts -p 'test_security_level_*.py'`.
- [ ] Final affected Rust suites use `cd codex-rs && just test -p <affected-project>`; no direct `cargo test`.
- [ ] Crosswalk checker self-tests reject missing, failed, stale, or mixed-candidate evidence.
- [ ] TUI applicability: none; PF-26-S02 owns interactive evidence.

## Exit evidence

- [ ] Harness/contract commits, commands, fixture digests, and self-test results recorded.
- [ ] Every applicable control has a fixture/owner; unfinished product results remain pending, not accepted.
- [ ] Early harness evidence linked under `qa/security-levels/sprints/PF-26-S01/`.
- [ ] Ledgers reflect reality and the completed record is archived.
