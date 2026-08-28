---
sprint_id: "PF-27-S01"
title: "Shared security integration contracts"
status: completed
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
- [x] Landed serial shared registration and protocol → existing security-policy Cargo edge; Bazel lock regeneration passed without a lockfile change. No backend dependency selected.
- [x] Defined host-issued identity, source kinds, sticky bounded taint, and unknown-origin denial.
- [x] Defined immutable action context and issuance-epoch grant checks; Core runtime incarnation rejects stale authority after restart/resume.
- [x] Registered Core/TUI seams and transition/grant/revoke intents; only the existing trusted controller mints non-wire confirmation capabilities. Receipt/consumption does not mutate policy.
- [x] Published consumer contracts and exact reserved ownership for PF-23/24/25/28/29/30; unavailable controls never imply protection.
- [x] Added schema/version, spoofing, non-widening, post-read, expiry, revocation, and native inheritance regressions; Permissive remains unchanged.
- [x] Published seven conformance fixture definitions and verified every referenced contract test passed; native consumer qualification remains pending under its own sprints.

## Remaining

- None.

## Verification

- [x] Recorded verified inherited upstream baseline and retained/adapted seam decisions; no new upstream upgrade or native backend qualification claimed.
- [x] Ran scoped `just fix`, `just fmt`, and `just fmt-check` before final affected tests.
- [x] Final candidate: policy 39, protocol 281, network-proxy 214, Core security 26, Core inheritance 3, TUI state 2 tests passed. Core/TUI filters are explicit in evidence; not a full-workspace claim.
- [x] Exact candidate/dependencies, commands, JUnit artifacts, source fingerprints, and clean full-sprint Autoreview recorded in `qa/security-levels/sprints/PF-27-S01/evidence.md`.
- [x] TUI/live-repository applicability: none; no command, key handler, rendering, or native consumer is activated. Actual-key proof remains required for downstream interactive sprints.

## Exit evidence

- [x] Code candidate `faa8ed6d39bf30db1b2fe982a69661a108e00a71`; macOS arm64 results and dependency commits recorded; later closure commits change only evidence/navigation.
- [x] No actual-key flow applies to this contract-only sprint; Linux/Windows, native containment, human acceptance, and release gates are explicitly not claimed.
- [x] Completed record archived; plan points to completion evidence. PF-13 qualification is unchanged; no dependent sprint was activated.
