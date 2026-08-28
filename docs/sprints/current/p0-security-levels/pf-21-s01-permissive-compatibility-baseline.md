---
sprint_id: "PF-21-S01"
title: "Permissive compatibility baseline reconciliation"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-21"
execution_order: 14
owner: "Jim Ricketts"
worktree: "/home/pfrpc/repos/CorbanuTerminal-security-levels"
branch: "feat/p0-security-levels"
base_commit: "3c1b2f6cbe11657ff4e3b72b11db029c9e7a92eb"
depends_on: "PF-16-S01, PF-20-S01"
created: 2026-08-24
updated: 2026-08-28
---

# PF-21-S01 — Permissive compatibility baseline reconciliation

## Execution mandate

- Deliver: verify a frozen, versioned baseline proving Permissive preserves current behavior.
- Excludes: Moderate/Aggressive controls, `/security` TUI, and release-level live-repository QA.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md)
- Feature: `PF-21`
- Reconciliation: [source decisions and archive mapping](../../../plans/security-source-reconciliation.md).
- Product citation: **P0 `/security` levels** — “Existing approval, sandbox, vault, wallet, tool, network, and agent policies are unchanged.”
- Acceptance advanced: existing installations gain no silent approval, vault, tool, network, or agent-policy change.

## Code boundaries

- Existing: `qa/security-levels/permissive-baseline-v1.json`
- Tests: `codex-rs/core/src/agent/registry_tests.rs`; `codex-rs/vault/src/tests.rs`
- Planned harness: `scripts/security-level-compat`

## Preconditions

- [ ] PF-16-S01 and PF-20-S01 are completed and archived.
- [ ] Exact worktree coordinates match the plan.
- [ ] Baseline commit remains the plan's recorded pre-feature commit.

## Done

- [x] Sprint record is linked to PF-21.
- [x] Commit `220af8dae8` added the first baseline manifest and Core/vault compatibility tests.

## Remaining

- [ ] Keep the independent pre-feature baseline as an acceptance oracle. Add an upstream-aligned control and reviewed upstream-drift ledger; same-candidate feature-on/off tests are supplemental and cannot regenerate golden expectations.
- [ ] Record baseline/upstream/candidate commits, config and environment digests per run; explain intentional upstream differences with owner review and new independent control evidence before accepting drift.

- [ ] Freeze inherited environment/auth-helper, web.run history/native search, browser, MCP/plugin/child, wallet, clipboard/export and persisted-session behavior; added broker/screening/migration controls are opt-in above Permissive, not silent baseline changes.

- [ ] Audit the manifest against representative approval, vault, tool, network, and agent-spawn policy surfaces.
- [ ] Implement the baseline-versus-candidate harness without rewriting expected behavior from the candidate.
- [ ] Add adjacent profile, credential type, spawn-depth, and missing-config regressions.
- [ ] Record any corrective commit and exact baseline/candidate hashes.

## Verification

- [ ] Fix: `cd codex-rs && just fix -p codex-core && just fix -p codex-vault`.
- [ ] Format: `cd codex-rs && just fmt`; then inspect the final diff.
- [ ] Focused tests: `cd codex-rs && just test -p codex-core permissive && just test -p codex-vault permissive`.
- [ ] Harness: `python3 scripts/security-level-compat --baseline 3c1b2f6cbe11657ff4e3b72b11db029c9e7a92eb --candidate <binary> --output <dir>`.
- [ ] TUI applicability deferred to PF-26-S02; this sprint freezes automated evidence only.

## Exit evidence

- [ ] Commits, manifest digest, and baseline/candidate hashes recorded.
- [ ] Output linked under `qa/security-levels/sprints/PF-21-S01/`.
- [ ] Ledgers reflect reality and the completed record is archived.
