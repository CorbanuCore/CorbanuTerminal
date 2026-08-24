---
sprint_id: "PF-21-S01"
title: "Permissive compatibility baseline reconciliation"
status: completed
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-21"
execution_order: 7
owner: "Jim Ricketts"
worktree: "/home/pfrpc/repos/CorbanuTerminal-security-levels"
branch: "feat/p0-security-levels"
base_commit: "3c1b2f6cbe11657ff4e3b72b11db029c9e7a92eb"
depends_on: "PF-16-S01, PF-20-S01"
created: 2026-08-24
updated: 2026-08-24
---

# PF-21-S01 — Permissive compatibility baseline reconciliation

## Execution mandate

- Deliver: verify a frozen, versioned baseline proving Permissive preserves current behavior.
- Excludes: Moderate/Aggressive controls, `/security` TUI, and release-level live-repository QA.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md)
- Feature: `PF-21`
- Acceptance advanced: existing installations gain no silent approval, vault, tool, network, or agent-policy change.

## Code boundaries

- Existing: `qa/security-levels/permissive-baseline-v1.json`
- Tests: `codex-rs/core/src/agent/registry_tests.rs`; `codex-rs/vault/src/tests.rs`
- Harness: `scripts/security-level-compat`

## Preconditions

- [x] PF-16-S01 and PF-20-S01 are completed and archived.
- [x] Exact worktree coordinates match the plan.
- [x] Baseline commit remains the plan's recorded pre-feature commit.

## Done

- [x] Sprint record is linked to PF-21.
- [x] Commit `220af8dae8` added the first baseline manifest and Core/vault compatibility tests.

## Remaining

- [x] Audited the manifest across approval, vault, tool, network, and agent-spawn policy surfaces.
- [x] Implemented a source-hash-pinned baseline-versus-candidate harness.
- [x] Added adjacent profile, credential type, spawn-depth, tool-policy, and missing-config regressions.
- [x] Recorded corrective commit `fcaa84dfb8`, manifest digest, and baseline/candidate hashes.

## Verification

- [x] Fix: `cd codex-rs && just fix -p codex-core && just fix -p codex-vault`.
- [x] Format: `cd codex-rs && just fmt`; final diff inspected.
- [x] Focused tests: `cd codex-rs && just test -p codex-core permissive && just test -p codex-vault permissive`.
- [x] Policy probe: `cd codex-rs && just test -p codex-security-policy permissive_composition_preserves_every_frozen_surface_decision`.
- [x] Harness: `python3 scripts/security-level-compat --baseline 3c1b2f6cbe11657ff4e3b72b11db029c9e7a92eb --candidate codex-rs/target/debug/corbanu --output qa/security-levels/sprints/PF-21-S01/harness`.
- [x] TUI applicability deferred to PF-26-S02; this sprint freezes automated evidence only.

## Exit evidence

- [x] Commits, manifest digest, and baseline/candidate hashes recorded.
- [x] Output linked under `qa/security-levels/sprints/PF-21-S01/`.
- [x] Ledgers reflect reality and the completed record is archived.
