---
sprint_id: "PF-21-S01"
title: "Permissive compatibility baseline reconciliation"
status: completed
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-21"
execution_order: 7
owner: "Jim Ricketts"
worktree: "/Users/travisgood/Documents/ChatGPT/corbanu-security-levels"
branch: "feat/p0-security-levels"
base_commit: "7cc15ae0762664d6d01765de407329887da9f876"
depends_on: "PF-16-S01, PF-20-S01"
created: 2026-08-24
updated: 2026-08-25
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
- Planned harness: `scripts/security-level-compat`

## Preconditions

- [x] PF-16-S01 and PF-20-S01 are completed and archived.
- [x] Exact worktree coordinates match the plan.
- [x] Baseline commit remains the plan's recorded pre-feature commit.

## Record creation

- [x] Sprint record is linked to PF-21.
- [x] Commit `220af8dae8` added the first baseline manifest and Core/vault compatibility tests.

## Done

- [x] Audit the manifest against representative approval, vault, tool, network, and agent-spawn policy surfaces.
- [x] Implement the baseline-versus-candidate harness without rewriting expected behavior from the candidate.
- [x] Add adjacent profile, credential type, spawn-depth, and missing-config regressions.
- [x] Record any corrective commit and exact baseline/candidate hashes.

## Remaining

- None.

## Verification

- [x] Fix: `cd codex-rs && just fix -p codex-core && just fix -p codex-vault`.
- [x] Format: `cd codex-rs && just fmt`; then inspect the final diff.
- [x] Focused tests: `cd codex-rs && just test -p codex-core permissive && just test -p codex-vault permissive`.
- [x] Harness: `python3 scripts/security-level-compat --baseline 3c1b2f6cbe11657ff4e3b72b11db029c9e7a92eb --candidate <binary> --output <dir>`.
- [x] TUI applicability deferred to PF-26-S02; this sprint freezes automated evidence only.

## Exit evidence

- [x] Commits, manifest digest, and baseline/candidate hashes recorded.
- [x] Output linked under `qa/security-levels/sprints/PF-21-S01/`.
- [x] Ledgers reflect reality and the completed record is archived.
