---
sprint_id: "PF-28-S01"
title: "Cross-surface confidentiality and safe environments"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-28"
execution_order: 16
owner: "Jim Ricketts"
lane: "confidentiality"
write_scope: "codex-rs/core/src/security/confidentiality, codex-rs/core/src/exec.rs, codex-rs/core/src/client.rs, codex-rs/core/src/tools/registry.rs, codex-rs/network-proxy/src/credential_broker.rs"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-13-S05, PF-27-S01, PF-26-S01"
created: 2026-08-27
updated: 2026-08-27
---

# PF-28-S01 — Cross-surface confidentiality and safe environments

## Execution mandate

- Deliver: Keep protected values out of all model-visible and inherited surfaces in Moderate/Aggressive.
- Excludes: Changes to Permissive, new credential providers, automatic plaintext migration, browser runtime, and source labeling.

## Plan linkage

- Plan: [P0 security levels](../../../plans/active/p0-security-levels.md).
- Feature: `PF-28`; see the plan's adopted contract and requirement traceability.
- Acceptance advanced: Keep protected values out of all model-visible and inherited surfaces in Moderate/Aggressive.

## Code boundaries

- Existing (paths below `codex-rs/`): `core/src/{exec,client}.rs`; `core/src/tools/registry.rs`; `network-proxy/src/credential_broker.rs`.
- Planned: `core/src/security/confidentiality/` adapters, sink inventory, and canary regressions.
- Tests: sibling unit/integration regressions and the PF-26 fixture matrix.

## Preconditions

- [ ] Listed dependencies are completed and archived.
- [ ] Read applicable root and nested AGENTS instructions; plan remains active.
- [ ] Allocate exact owner/worktree/branch/base and literal write scope in the plan; check lane/slot conflicts.

## Done

- [x] Approved feature contract decomposed into this single-feature draft.

## Remaining

- [ ] Inventory model requests, tool results, errors/reflected provider responses, traces, logs, exports, artifacts, and child environments; assign every sink an adapter/test.
- [ ] Reuse the PF-13 broker and registered-value redaction; never serialize plaintext into generic model or child inputs.
- [ ] Centralize exact-value and conservative pattern redaction before visible/persisted output; test encoded/reflected error paths and failure cleanup.
- [ ] Build allowlisted child environments in stronger modes; preserve existing Permissive environment and helper behavior.
- [ ] Define narrow derived financial views; deny raw protected financial results, unknown classes, and unsupported credential paths.
- [ ] Detect legacy plaintext credential paths and block/quarantine their use in stronger modes without deleting or migrating user files.
- [ ] Prove the canary absent across every inventoried sink and unbound destination; output suppression must not hide structured denial/recovery.

## Verification

- [ ] Run `just fix -p <affected-crate>` and `just fmt` in `codex-rs` before final tests.
- [ ] Run focused and affected integration tests with `just test -p <affected-crate>`; never direct `cargo test`.
- [ ] Record exact dependency commits, candidate, commands, artifacts, and missing coverage; no pass by code presence.
- [ ] Run PF-13 canary and early PF-26 confidentiality fixtures, including subprocess and reflected-response negatives.
- [ ] TUI applicability: exercise a denied disclosure and safe retry in a true PTY; record actual keys and checkpoints.

## Exit evidence

- [ ] Final candidate and dependency commits, commands, platform results, and artifact paths recorded.
- [ ] Applicable actual-key proof is linked; no missing test is relabeled a pass.
- [ ] Ledgers reflect reality; completed record is archived and plan evidence linked.
