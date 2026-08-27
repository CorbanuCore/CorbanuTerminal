---
sprint_id: "PF-30-S01"
title: "Isolated public-web acquisition runtime"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-30"
execution_order: 19
owner: "Jim Ricketts"
lane: "browser"
write_scope: "codex-rs/core/src/security/browser_isolation, codex-rs/network-proxy/src/browser_policy.rs"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-27-S01, PF-26-S01"
created: 2026-08-27
updated: 2026-08-27
---

# PF-30-S01 — Isolated public-web acquisition runtime

## Execution mandate

- Deliver: Provide an ephemeral containment and egress backend for unauthenticated public-web acquisition.
- Excludes: Authenticated login, new search providers, content sanitization, host-browser automation, and public tool activation.

## Plan linkage

- Plan: [P0 security levels](../../../plans/active/p0-security-levels.md).
- Feature: `PF-30`; see the plan's adopted contract and requirement traceability.
- Acceptance advanced: Provide an ephemeral containment and egress backend for unauthenticated public-web acquisition.

## Code boundaries

- Existing (paths below `codex-rs/`): `network-proxy/src/policy.rs` as a read-only composition reference.
- Planned: `core/src/security/browser_isolation/`; `network-proxy/src/browser_policy.rs`.
- Tests: sibling unit/integration regressions and the PF-26 fixture matrix.

## Preconditions

- [ ] Listed dependencies are completed and archived.
- [ ] Read applicable root and nested AGENTS instructions; plan remains active.
- [ ] Allocate exact owner/worktree/branch/base and literal write scope in the plan; check lane/slot conflicts.
- [ ] Select and record the pinned backend/runtime and Linux/macOS/Windows support/fail-closed matrix before readiness.
- [ ] Record actual backend integration/dependency files in write_scope before readiness; shared registrations land serially.

## Done

- [x] Approved feature contract decomposed into this single-feature draft.

## Remaining

- [ ] Implement the selected container or hardened-sandbox adapter with a pinned runtime, disposable profile, read-only base, and resource limits.
- [ ] Exclude host IPC, vault, credential inheritance, host profiles, and unrestricted mounts; limit outputs to bounded acquisition data.
- [ ] Enforce destinations at URL, DNS/IP, redirect, and connection boundaries; deny private/link-local/metadata endpoints and rebinding bypasses.
- [ ] Quarantine downloads and expose a bounded, explicitly approved file-promotion interface; no direct workspace write.
- [ ] Report effective backend health and degradation using PF-27; unavailable setup denies acquisition without an unsandboxed fallback.
- [ ] Test containment probes, egress bypasses, timeout/cancel cleanup, crash/restart, and artifact isolation on the recorded platform matrix.

## Verification

- [ ] Run `just fix -p <affected-crate>` and `just fmt` in `codex-rs` before final tests.
- [ ] Run focused and affected integration tests with `just test -p <affected-crate>`; never direct `cargo test`.
- [ ] Record exact dependency commits, candidate, commands, artifacts, and missing coverage; no pass by code presence.
- [ ] Run real backend probes and early PF-26 isolation fixtures, not only mocked policy tests.
- [ ] TUI applicability: none; this internal backend is not exposed until PF-30-S02.

## Exit evidence

- [ ] Final candidate and dependency commits, commands, platform results, and artifact paths recorded.
- [ ] Applicable actual-key proof is linked; no missing test is relabeled a pass.
- [ ] Ledgers reflect reality; completed record is archived and plan evidence linked.
