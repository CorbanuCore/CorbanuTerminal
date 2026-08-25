---
sprint_id: "PF-20-S01"
title: "Versioned security persistence reconciliation"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-20"
execution_order: 6
owner: "Jim Ricketts"
worktree: "/home/pfrpc/repos/CorbanuTerminal-security-levels"
branch: "feat/p0-security-levels"
base_commit: "3c1b2f6cbe11657ff4e3b72b11db029c9e7a92eb"
depends_on: "PF-15-S01"
created: 2026-08-24
updated: 2026-08-24
---

# PF-20-S01 — Versioned security persistence reconciliation

## Execution mandate

- Deliver: verify typed, versioned level persistence with explicit corrupt/unknown-state failure.
- Excludes: TUI confirmation, effective runtime policy, downgrade invalidation, and audit persistence.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md)
- Feature: `PF-20`
- Acceptance advanced: restart restores a known level without transiently weakening policy.

## Code boundaries

- Existing: `codex-rs/config/src/config_toml.rs`; `codex-rs/core/src/config/{mod,edit}.rs`
- Generated/build: `codex-rs/core/config.schema.json`; Cargo and Bazel dependency files
- Tests: `codex-rs/core/src/config/{config_tests,edit_tests}.rs`

## Preconditions

- [ ] PF-15-S01 is completed and archived.
- [ ] Exact worktree coordinates match the plan.
- [ ] Read `codex-rs/AGENTS.md` and `codex-rs/core/AGENTS.md` before corrective work.

## Done

- [x] Sprint record is linked to PF-20.
- [x] Commit `0e3f2dfd92` added versioned config, schema, editing, and persistence tests.

## Remaining

- [ ] Review legacy absence, explicit Permissive, unknown version, corrupt level, and atomic-write behavior.
- [ ] Run `just write-config-schema` and prove the checked-in schema matches generated types.
- [ ] Run `just bazel-lock-update` if dependency metadata changes and include Bazel parity.
- [ ] Record any corrective commit and final changed paths.

## Verification

- [ ] Fix: `cd codex-rs && just fix -p codex-config && just fix -p codex-core`.
- [ ] Format: `cd codex-rs && just fmt`; then inspect the final diff.
- [ ] Focused tests: `cd codex-rs && just test -p codex-config && just test -p codex-core config::`.
- [ ] Schema: `cd codex-rs && just write-config-schema`; final diff is empty or intentional.
- [ ] TUI applicability: none; PF-24 owns interactive persistence.

## Exit evidence

- [ ] Commits, generated-schema result, and changed paths recorded.
- [ ] Test output linked under `qa/security-levels/sprints/PF-20-S01/`.
- [ ] Ledgers reflect reality and the completed record is archived.
