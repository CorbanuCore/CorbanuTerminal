---
sprint_id: "PF-30-S01"
title: "Isolated public-web acquisition runtime"
status: in_progress
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-30"
execution_order: 19
owner: "Jim Ricketts"
lane: "browser"
write_scope: "codex-rs/browser-isolation, codex-rs/core/src/security/browser_isolation, codex-rs/network-proxy/src/browser_policy.rs, codex-rs/network-proxy/src/browser_policy_tests.rs, codex-rs/Cargo.toml, codex-rs/Cargo.lock, codex-rs/core/Cargo.toml, MODULE.bazel.lock"
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
- Planned: `browser-isolation/` crate (engine, broker, worker, quarantine, lifecycle and tests); thin `core/src/security/browser_isolation/` adapter; `network-proxy/src/browser_policy.rs` and sibling tests.
- Serial registrations: workspace `Cargo.toml`/`Cargo.lock`, `core/Cargo.toml`, `MODULE.bazel.lock`; new crate `BUILD.bazel` includes the fixed worker/build recipe as compile data. No PF-29 or public tool changes.
- Tests: sibling unit/integration regressions and the PF-26 fixture matrix.

## Preconditions

- [x] Plan upstream baseline, adapter ownership, and exact contract tests resolved in the runtime record linked below.
- [x] PF-27-S01 and PF-26-S01 completed and archived at the allocated base.
- [x] Root/Rust/Core instructions read; active plan and two-slot use checked.
- [x] Exact owner/worktree/branch/base and literal write scope allocated in the plan.
- [x] Podman/Docker backend, Scrapling source/image pins and all-platform pending matrix recorded.
- [x] Exact full-sprint crate/manifest/lockfile scope amended before implementation, with S01 serial ownership; PF-29 remains draft.
- [x] Existing PF-27 module registrations consumed; content adapters and shared TUI/protocol files excluded.

## Done

- [x] Approved feature contract decomposed into this single-feature draft.
- [x] Recorded runtime decisions/preflight; with later user authorization recovered Mac Docker and installed Linux Podman. Real qualification failures: `qa/security-levels/sprints/PF-30-S01/platform-qualification-2026-08-27.md`.
- [x] User requested end-to-end S01 implementation and selected Fable High; Autoreview must use `--engine claude --model claude-fable-5 --thinking high` without fallback.
- [x] Implemented the internal backend draft, pinned derivative image recipe, networkless worker/broker, owned lifecycle, quarantine and thin Core live-policy/health adapter; no public activation.
- [x] Added protocol, destination, ownership, bounded-restart, cancellation, promotion and native policy-epoch regression tests. Checkpoint evidence: `qa/security-levels/sprints/PF-30-S01/implementation-evidence.md`.
- [x] Fixed the validated DNS-denial ordering and Podman image-ID findings, added regressions, and obtained a clean Fable 5 High fix-cycle review via Computer Use. Evidence: `qa/security-levels/sprints/PF-30-S01/review-fixes.md`; the initial false positive remains withdrawn.
- [x] Fixed Docker PID/UTS arguments, Podman capability verification and undetected disabled seccomp; regressions, real Mac/Linux smokes, negative confinement checks and Fable High scoped review pass. Evidence: `qa/security-levels/sprints/PF-30-S01/platform-fixes.md`.

Reviewable stages: (1) crate registration and connection policy; (2) engine
selection, pinned image recipe and owned-container lifecycle; (3) bounded broker,
Scrapling worker and quarantine; (4) Core health/epoch adapter and final real
backend probes. Keep each code stage below 500 changed lines where practical;
review/test each independently and review the final integrated candidate.

## Remaining

The backend remains incomplete despite its earlier clean scoped review.
Mac/Linux backend smokes now pass; full qualification and Windows remain open.

- [ ] Resolve the residual native allowlist/DNS ordering with the integration owner before DNS qualification; allocate any scope expansion first. This is separate from the fixed explicit-denial ordering.
- [ ] Close review coverage gaps for broker redirects, worker request-handler branches and engine endpoint-selection JSON; measure real engine payload sizes before changing the bounded output cap.
- [ ] Implement the selected container or hardened-sandbox adapter with a pinned runtime, disposable profile, read-only base, and resource limits.
- [ ] Preserve existing engine selection; plan install/start/pull/restart/verify actions, bound recovery, and reject unowned service collisions. S03 owns actual installer consent/elevation UI.
- [ ] Exclude host IPC, vault, credential inheritance, host profiles, and unrestricted mounts; limit outputs to bounded acquisition data.
- [ ] Enforce destinations at URL, DNS/IP, redirect, and connection boundaries; deny private/link-local/metadata endpoints and rebinding bypasses.
- [ ] Quarantine downloads and expose a bounded, explicitly approved file-promotion interface; no direct workspace write.
- [ ] Report effective backend health and degradation using PF-27; unavailable setup denies acquisition without an unsandboxed fallback.
- [ ] Test containment probes, egress bypasses, timeout/cancel cleanup, crash/restart, and artifact isolation on the recorded platform matrix.

## Verification

- [ ] Record applicable upstream adapter evidence or justified non-applicability; structural checks alone are not qualification.
- [x] Run `just fix -p <affected-crate>` and `just fmt` in `codex-rs` before final tests.
- [x] Run focused and affected integration tests with `just test -p <affected-crate>`; never direct `cargo test`. 272 selected Rust tests and six worker tests on each host passed; see platform-fixes for the exact filter and fingerprints. Full backend qualification remains open below.
- [ ] Record exact dependency commits, candidate, commands, artifacts, and missing coverage; no pass by code presence.
- [ ] Run real backend probes and early PF-26 isolation fixtures, not only mocked policy tests.
- [ ] Prove the backend conforms to PF-27 native adapter contracts without requiring PF-29 implementation; S02 owns the content/facade join.
- [ ] TUI applicability: none; this internal backend is not exposed until PF-30-S02.

## Exit evidence

- [ ] Final candidate and dependency commits, commands, platform results, and artifact paths recorded.
- [ ] Applicable actual-key proof is linked; no missing test is relabeled a pass.
- [ ] Ledgers reflect reality; completed record is archived and plan evidence linked.
