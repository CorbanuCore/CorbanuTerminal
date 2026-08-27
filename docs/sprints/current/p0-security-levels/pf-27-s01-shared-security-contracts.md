---
sprint_id: "PF-27-S01"
title: "Shared security integration contracts"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-27"
execution_order: 14
owner: "Jim Ricketts"
lane: "contracts"
write_scope: "codex-rs/security-policy, codex-rs/protocol, codex-rs/core/src/security/mod.rs, codex-rs/tui/src/security/mod.rs"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-17-S01, PF-19-S01, PF-20-S01, PF-22-S01"
created: 2026-08-27
updated: 2026-08-27
---

# PF-27-S01 — Shared security integration contracts

## Execution mandate

- Deliver: Land typed shared interfaces that let isolated security lanes compose without inventing competing authority.
- Excludes: Credential resolution, browser backend, content adapters, profile activation, and final qualification.

## Plan linkage

- Upstream: [plan touch record](../../../plans/active/p0-security-levels.md#upstream-touch-record); resolve this sprint's adapter rows.
- Plan: [P0 security levels](../../../plans/active/p0-security-levels.md).
- Feature: `PF-27`; see the plan's adopted contract and requirement traceability.
- Acceptance advanced: Land typed shared interfaces that let isolated security lanes compose without inventing competing authority.

## Code boundaries

- Existing (paths below `codex-rs/`): `security-policy/src/{lib,grant,revocation}.rs`; `protocol/src/lib.rs`.
- Planned: `security-policy/src/integration.rs`; `protocol/src/security.rs`; Core/TUI security module registration.
- Tests: sibling unit/integration regressions and the PF-26 fixture matrix.

## Preconditions

- [ ] Plan upstream baseline, adapter ownership, and exact contract tests are resolved before readiness.
- [ ] Listed dependencies are completed and archived.
- [ ] Read applicable root and nested AGENTS instructions; plan remains active.
- [ ] Allocate exact owner/worktree/branch/base and literal write scope in the plan; check lane/slot conflicts.

## Done

- [x] Approved feature contract decomposed into this single-feature draft.

## Remaining

- [ ] Record the verified upstream ancestry and each consumer's native adapter footprint; record dependency direction and exact contract-test commands in the plan.
- [ ] Land shared module/test registration and any selected backend Cargo/Bazel manifest/lock changes serially; allocate exact paths before edits so browser/content/confidentiality consumers are disjoint.
- [ ] Inventory existing types; extend rather than duplicate policy, grant, revocation, and persistence contracts.
- [ ] Define versioned requested/effective policy facts, backend health/degradation, and secret-free inspector snapshots.
- [ ] Define host-issued source identity, source kinds, sticky derived-taint context, and unknown-origin deny semantics.
- [ ] Define immutable action context and authority-epoch interfaces for grant matching, invalidation, and resume.
- [ ] Register shared Core/TUI module seams and typed human-only transition/grant/revoke events; payloads cannot self-authenticate as human.
- [ ] Publish consumer contracts for PF-23/24/25/28/29/30 and record file ownership; unavailable implementations report unavailable, never protected.
- [ ] Add schema/serialization/unknown-version, authority spoofing, and non-widening contract tests; preserve Permissive.
- [ ] Define adapter conformance fixtures for native dispatch, provider schemas, context lineage, cancellation, and resume; assign consumers without creating a second policy or agent lifecycle.

## Verification

- [ ] Record applicable upstream adapter evidence or justified non-applicability; structural checks alone are not qualification.
- [ ] Run `just fix -p <affected-crate>` and `just fmt` in `codex-rs` before final tests.
- [ ] Run focused and affected integration tests with `just test -p <affected-crate>`; never direct `cargo test`.
- [ ] Record exact dependency commits, candidate, commands, artifacts, and missing coverage; no pass by code presence.
- [ ] TUI applicability: none; schemas and unavailable module seams do not expose a user interaction.

## Exit evidence

- [ ] Final candidate and dependency commits, commands, platform results, and artifact paths recorded.
- [ ] Applicable actual-key proof is linked; no missing test is relabeled a pass.
- [ ] Ledgers reflect reality; completed record is archived and plan evidence linked.
