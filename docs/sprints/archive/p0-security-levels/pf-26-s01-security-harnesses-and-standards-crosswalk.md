---
sprint_id: "PF-26-S01"
title: "Security harnesses and standards crosswalk"
status: completed
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-26"
execution_order: 15
owner: "Jim Ricketts"
lane: "harness"
write_scope: "scripts/security-level-compat, scripts/security-level-adversarial, scripts/security-level-standards-check, scripts/security_level_compat.py, scripts/security_level_evidence.py, scripts/security_level_adversarial.py, scripts/security_level_capture.py, scripts/security_level_standards_check.py, scripts/test_security_level_compat.py, scripts/test_security_level_evidence.py, scripts/test_security_level_adversarial.py, scripts/test_security_level_capture.py, scripts/test_security_level_standards_check.py, qa/security-levels/fixtures, qa/security-levels/sprints/PF-26-S01"
worktree: "/Users/travisgood/Documents/ChatGPT/corbanu-pf26-s01"
branch: "codex/pf-26-security-harnesses"
base_commit: "cb808c30c0058c101597ab2ada3da16238565c5e"
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

- Delivered: the three `scripts/security-level-*` entrypoints, corresponding Python modules/tests in write_scope.
- Delivered: `qa/security-levels/fixtures/`; final candidate manifest remains PF-26-S04 work.
- Tests: Python runner/checker self-tests; Rust contract-fixture tests only where affected

## Preconditions

- [x] Plan upstream baseline, adapter ownership, and exact contract tests are resolved in the PF-26-S01 execution contract.
- [x] PF-21-S01 and PF-27-S01 are completed and archived.
- [x] Harness base/PF-27 completion is pinned above; accepted baseline bytes are frozen before extending runners.
- [x] Exact worktree coordinates match the active plan.

- [x] Harness allocation is disjoint from PF-13 qualification; shared metadata updates are serial.

## Done

- [x] Sprint record is linked only to PF-26.

- [x] Extended compatibility preparation and froze accepted baseline bytes without overwriting PF-21 evidence.
- [x] Added hostile/benign fixtures, unique canaries, fake actions and all-source/all-sink assertions.
- [x] Published ten source-class owner/support/fixture rows, task hijacking/test weakening/review injection, and independent task/policy/confidentiality outcomes; native hook registration remains consumer work.
- [x] Published all seven pinned PF-27 wire/child/lineage/authority/health adapter contracts with exact required test selectors and expectations.
- [x] Added loopback HTTP capture/scanner fixtures for future PF-13 test TLS/TUI integration; no live credentials or forwarding.
- [x] Added provenance/lineage, reflected error/environment, browser containment/egress and forced-classifier-miss cases.
- [x] Added versioned standards/design crosswalk, 65 explicit pending result slots, and fail-closed evidence checks.
- [x] Ran 39 harness self-tests against the PF-27 contract catalog; product results remain pending.
- [x] Published commands, schemas and fixture digests for consumers; final qualification remains PF-26-S04.

## Remaining

None in S01. PF-26-S04/S02/S03 remain dependency-gated; no integrated security, platform, true-TUI or release acceptance is claimed.

## Verification

- [x] Historical PF-27 source hashes/selectors checked; stale upstream path references flagged in handoff; no native adapter qualification claim.
- [x] Ruff formatting/lint preceded final tests; exact source digests and diff inspected. Rust fix/format N/A: no Rust edits.
- [x] `python3 -m unittest discover -s scripts -p 'test_security_level_*.py'`: 39 passed; existing credential-canary tests: 6 passed.
- [x] Rust suites N/A for Python-only harness changes; no Rust build or direct `cargo test` performed.
- [x] Crosswalk tests reject missing/failed/stale/mixed-candidate evidence and omitted native contract-test proof; canary tests include uppercase/mixed-case hex.
- [x] TUI applicability: none; PF-26-S02 owns interactive evidence. No live-repository or human acceptance claimed.

## Exit evidence

- [x] Harness code commit `bed9c5bfeece2414cbf7e3f54af09fcb646959ed`; contract base above; commands/digests/results retained.
- [x] All 16 controls have fixtures/owners; unfinished product results remain pending.
- [x] [Final evidence](../../../../qa/security-levels/sprints/PF-26-S01/evidence.md), final-checks JSON and clean Autoreview retained; both accepted findings fixed in scope.
- [x] Ledgers complete and record archived; PF-26 as a whole remains unfinished.
