---
sprint_id: "PF-30-S02"
title: "Isolated acquisition integration and recovery"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-30"
execution_order: 20
owner: "Jim Ricketts"
lane: "browser"
write_scope: "codex-rs/core/src/security/browser_isolation, codex-rs/core/src/tools/handlers/web_run.rs, codex-rs/core/src/tools/spec.rs"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-30-S01, PF-29-S01"
created: 2026-08-27
updated: 2026-08-27
---

# PF-30-S02 — Isolated acquisition integration and recovery

## Execution mandate

- Deliver: Route stronger-mode public acquisition through isolation and untrusted-content handling with visible recovery.
- Excludes: Authenticated login, new providers, changes to Permissive, classifier training, and general browser-control handles.

## Plan linkage

- Plan: [P0 security levels](../../../plans/active/p0-security-levels.md).
- Feature: `PF-30`; see the plan's adopted contract and requirement traceability.
- Acceptance advanced: Route stronger-mode public acquisition through isolation and untrusted-content handling with visible recovery.

## Code boundaries

- Existing (paths below `codex-rs/`): `core/src/tools/spec.rs`; existing web facade located and recorded before readiness.
- Planned: `core/src/tools/handlers/web_run.rs` adapter; browser-isolation integration tests.
- Tests: sibling unit/integration regressions and the PF-26 fixture matrix.

## Preconditions

- [ ] Listed dependencies are completed and archived.
- [ ] Read applicable root and nested AGENTS instructions; plan remains active.
- [ ] Allocate exact owner/worktree/branch/base and literal write scope in the plan; check lane/slot conflicts.

## Done

- [x] Approved feature contract decomposed into this single-feature draft.

## Remaining

- [ ] Preserve the current agent-facing web facade while routing eligible Moderate/Aggressive acquisition through S01 and PF-29-S01.
- [ ] Inventory provider-native search, external browser/MCP tools, and direct/host-browser fallback paths; enforce the same boundary or deny unsupported paths.
- [ ] Keep Browser Isolation and External Content Firewall health separate; sanitized output retains untrusted provenance.
- [ ] Wire download quarantine and exact human-approved promotion; reject path traversal, symlinks, unapproved overwrite, and stale approvals.
- [ ] Make setup failure, crash, timeout, cancellation, and retry visible; teardown all disposable state and never retry through a host browser.
- [ ] Test successful hostile-page ingestion, denial, promotion cancel, recovery, and resume; compare Permissive to the frozen baseline.

## Verification

- [ ] Run `just fix -p <affected-crate>` and `just fmt` in `codex-rs` before final tests.
- [ ] Run focused and affected integration tests with `just test -p <affected-crate>`; never direct `cargo test`.
- [ ] Record exact dependency commits, candidate, commands, artifacts, and missing coverage; no pass by code presence.
- [ ] Run backend/platform and facade bypass fixtures against the real integrated runtime.
- [ ] TUI applicability: required in disposable TensorCash and Isometric Game worktrees; send actual keys for success/cancel/failure/recovery/resume.

## Exit evidence

- [ ] Final candidate and dependency commits, commands, platform results, and artifact paths recorded.
- [ ] Applicable actual-key proof is linked; no missing test is relabeled a pass.
- [ ] Ledgers reflect reality; completed record is archived and plan evidence linked.
