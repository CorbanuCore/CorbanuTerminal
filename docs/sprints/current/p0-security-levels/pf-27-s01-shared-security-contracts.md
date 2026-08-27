---
sprint_id: "PF-27-S01"
title: "Shared security integration contracts"
status: in_progress
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-27"
execution_order: 14
owner: "Jim Ricketts"
lane: "contracts"
write_scope: "codex-rs/security-policy, codex-rs/protocol, codex-rs/core/src/security, codex-rs/tui/src/lib.rs, codex-rs/tui/src/security, codex-rs/tui/src/bottom_pane/mod.rs, codex-rs/tui/src/bottom_pane/security_view.rs, codex-rs/network-proxy/src/lib.rs, codex-rs/network-proxy/src/browser_policy.rs, codex-rs/Cargo.lock, MODULE.bazel.lock"
worktree: "/Users/travisgood/Documents/ChatGPT/corbanu-pf27-s01"
branch: "codex/pf-27-shared-security-contracts"
base_commit: "ea7d4bec720098f6e0994fcfcc59e272108f7e70"
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
- Planned: `security-policy/src/{integration,provenance,action_context}.rs`; `protocol/src/security.rs`; Core `security/{integration,trusted_requests}.rs`; TUI `security/mod.rs`, with sibling tests and module registrations.
- Tests: sibling unit/integration regressions and the PF-26 fixture matrix.

## Preconditions

- [x] Plan upstream baseline, adapter ownership, and exact contract tests resolved in the PF-27 execution contract.
- [x] PF-17/19/20/22 dependencies are completed and archived at the recorded base.
- [x] Read applicable root and nested AGENTS instructions; plan remains active.
- [x] Allocated separate worktree/branch/base; contracts scope is disjoint from PF-13 qualification.

## Done

- [x] Approved feature contract decomposed into this single-feature draft.
- [x] Recorded upstream ancestry, consumer footprints, dependency direction, and contract-test commands in the plan.
- [x] Inventoried existing policy/grant/revocation/persistence types; reuse recorded in `qa/security-levels/sprints/PF-27-S01/evidence.md`.
- [x] Defined versioned requested/effective inspector facts and independent control health; first slice passes all 29 policy tests, including 8 new contract regressions.

## Remaining

- [ ] Land shared module/test registration and any selected backend Cargo/Bazel manifest/lock changes serially; allocate exact paths before edits so browser/content/confidentiality consumers are disjoint.
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
