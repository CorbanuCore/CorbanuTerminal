---
sprint_id: "PF-30-S01"
title: "Isolated public-web acquisition runtime"
status: in_progress
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-30"
execution_order: 19
owner: "Jim Ricketts"
lane: "browser"
write_scope: "codex-rs/core/src/security/browser_isolation, codex-rs/network-proxy/src/browser_policy.rs"
worktree: "/Users/travisgood/Documents/ChatGPT/corbanu-pf30-s01"
branch: "codex/pf-30-isolated-runtime"
base_commit: "9fc9c9106c8afd38aff48d0e5ad4a5f2552b723c"
depends_on: "PF-27-S01, PF-26-S01"
created: 2026-08-27
updated: 2026-08-27
---

# PF-30-S01 — Isolated public-web acquisition runtime

## Execution mandate

- Deliver: Provide an ephemeral containment and egress backend, including internal runtime-selection and bounded service-readiness policy, for unauthenticated public-web acquisition.
- Excludes: Authenticated login, new search providers, content sanitization, host-browser automation, and public tool activation.

Scheduling: approved parallel browser lane with PF-29 content and, when eligible,
PF-28 confidentiality after PF-27/PF-26-S01. No dependency on PF-13-S05; allocate
a distinct worktree/branch within the three-slot cap. Allocated and started on
2026-08-27. PF-29 remains allocated but draft until its native inventory resolves.

## Plan linkage

- Upstream: [plan touch record](../../../plans/active/p0-security-levels.md#upstream-touch-record); resolve this sprint's adapter rows.
- Plan: [P0 security levels](../../../plans/active/p0-security-levels.md).
- Feature: `PF-30`; see the plan's adopted contract and requirement traceability.
- Acceptance advanced: Provide an ephemeral containment and egress backend for unauthenticated public-web acquisition.

## Code boundaries

- Existing (paths below `codex-rs/`): `network-proxy/src/policy.rs` as a read-only composition reference.
- Planned: `core/src/security/browser_isolation/`; `network-proxy/src/browser_policy.rs`.
- Tests: sibling unit/integration regressions and the PF-26 fixture matrix.

## Preconditions

- [x] Plan upstream baseline, adapter ownership, and exact contract tests resolved in the runtime record linked below.
- [x] PF-27-S01 and PF-26-S01 completed and archived at the allocated base.
- [x] Root/Rust/Core instructions read; active plan and two-slot use checked.
- [x] Exact owner/worktree/branch/base and literal write scope allocated in the plan.
- [x] Podman/Docker backend, Scrapling source/image pins and all-platform pending matrix recorded.
- [x] Existing registered module boundaries used; no dependency/manifest changes authorized in this first stage.
- [x] PF-27 registrations consumed; content adapters and shared manifests excluded.

## Done

- [x] Approved feature contract decomposed into this single-feature draft.
- [x] Recorded user runtime/installation decisions and non-mutating Mac/Linux preflight in `qa/security-levels/sprints/PF-30-S01/runtime-selection.md`.

## Remaining

- [ ] Implement the selected container or hardened-sandbox adapter with a pinned runtime, disposable profile, read-only base, and resource limits.
- [ ] Preserve existing engine selection; plan install/start/pull/restart/verify actions, bound recovery, and reject unowned service collisions. S03 owns actual installer consent/elevation UI.
- [ ] Exclude host IPC, vault, credential inheritance, host profiles, and unrestricted mounts; limit outputs to bounded acquisition data.
- [ ] Enforce destinations at URL, DNS/IP, redirect, and connection boundaries; deny private/link-local/metadata endpoints and rebinding bypasses.
- [ ] Quarantine downloads and expose a bounded, explicitly approved file-promotion interface; no direct workspace write.
- [ ] Report effective backend health and degradation using PF-27; unavailable setup denies acquisition without an unsandboxed fallback.
- [ ] Test containment probes, egress bypasses, timeout/cancel cleanup, crash/restart, and artifact isolation on the recorded platform matrix.

## Verification

- [ ] Record applicable upstream adapter evidence or justified non-applicability; structural checks alone are not qualification.
- [ ] Run `just fix -p <affected-crate>` and `just fmt` in `codex-rs` before final tests.
- [ ] Run focused and affected integration tests with `just test -p <affected-crate>`; never direct `cargo test`.
- [ ] Record exact dependency commits, candidate, commands, artifacts, and missing coverage; no pass by code presence.
- [ ] Run real backend probes and early PF-26 isolation fixtures, not only mocked policy tests.
- [ ] Prove the backend conforms to PF-27 native adapter contracts without requiring PF-29 implementation; S02 owns the content/facade join.
- [ ] TUI applicability: none; this internal backend is not exposed until PF-30-S02.

## Exit evidence

- [ ] Final candidate and dependency commits, commands, platform results, and artifact paths recorded.
- [ ] Applicable actual-key proof is linked; no missing test is relabeled a pass.
- [ ] Ledgers reflect reality; completed record is archived and plan evidence linked.
