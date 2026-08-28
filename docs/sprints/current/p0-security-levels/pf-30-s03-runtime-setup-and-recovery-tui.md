---
sprint_id: "PF-30-S03"
title: "Container runtime setup and recovery TUI"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-30"
execution_order: 31
owner: "Jim Ricketts"
lane: "browser-setup"
write_scope: "codex-rs/core/src/security/browser_isolation, codex-rs/tui/src/security/runtime_setup.rs, codex-rs/tui/src/security/runtime_setup_tests.rs"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-30-S02, PF-24-S01"
created: 2026-08-27
updated: 2026-08-27
---

# PF-30-S03 — Container runtime setup and recovery TUI

## Execution mandate

- Deliver: Corbanu-guided Podman/Docker and pinned Scrapling setup, readiness checks, and bounded recovery on macOS/Linux/Windows.
- Excludes: Changing security policy, silently replacing existing engines, unrelated workload/daemon restart, authenticated browsing, and application password collection.

## Plan linkage

- Plan: [P0 security levels](../../../plans/active/p0-security-levels.md#browser-runtime-lifecycle-decision).
- Feature: `PF-30`.
- Acceptance advanced: **Moderate/Aggressive isolation and content provenance**, “If no runtime exists, offer a Corbanu-guided installation, then image setup and end-to-end verification.”
- Upstream: [touch record](../../../plans/active/p0-security-levels.md#upstream-touch-record); keep installer policy in browser-owned modules and native TUI adapters thin.

## Code boundaries

- Existing: Core browser-isolation API and TUI security view from the completed dependencies.
- Planned: `tui/src/security/runtime_setup.rs`, sibling tests, and platform-specific browser-owned installer adapters.
- Tests: sibling state/command tests, actual-key TUI setup, and isolated disposable-host tests.
- Shared TUI module/event/registration edits must be inventoried, added to scope, and serialized before readiness; paths above do not authorize undeclared edits.

## Preconditions

- [ ] Plan active; dependencies completed and archived; exact candidate contracts recorded.
- [ ] Allocate worktree/branch/base, literal implementation/test/registration paths, and free lane slot.
- [ ] Resolve upstream adapter rows, exact tests, installer pins/hashes/signatures and supported platform prerequisites.
- [ ] Record disposable test hosts and rollback scope; preserve installed runtimes and unrelated workloads.

## Done

- [x] User's 2026-08-27 installation and password-handling decision scoped to PF-30.
- [x] Added dependency joins into PF-24-S02 and PF-26-S04; Windows remains a required gate.

## Remaining

- [ ] Add stronger-mode readiness entrypoint and separate browser/content health; Permissive does no setup.
- [ ] Discover existing engines and reuse selection; prefer Podman only where appropriate, without context/config replacement.
- [ ] Present publisher/version, disk/download/VM requirements and host changes; require human consent before install/elevation.
- [ ] Use verified installers or signed distro packages; refuse altered downloads. No floating image tags or network-to-shell scripts.
- [ ] Delegate authentication to trusted OS UI or an uncaptured controlling terminal; never accept, log, persist, transmit to a model, or replay a password.
- [ ] Handle cancel, missing privileges, reboot/WSL requirements, partial installation and retry without widening authority.
- [ ] Pull missing image; start a stopped owned service; recover a stalled owned service within budget; never restart a shared daemon or unrelated workload.
- [ ] Run real acquisition plus containment probes before healthy status; distinguish process-running from effective protection.
- [ ] Revalidate on resume, remove only owned disposable state, and preserve the stronger level with visible denial on failed setup.

## Verification

- [ ] Run scoped `just fix`, `just fmt`, then affected `just test` suites; record exact commands and candidate.
- [ ] Exercise existing Podman, existing Docker, both, neither, unavailable engine, missing image, stopped/stalled service and ownership collision.
- [ ] Prove password absence in application/model/history/log/artifact outputs and ordinary cancellation from OS authentication.
- [ ] Run actual-key success/cancel/failure/recovery/resume in both disposable live repositories.
- [ ] Qualify macOS and Linux, then request Windows instructions from Travis and run Windows; no platform inferred passing.
- [ ] Verify Permissive compatibility and native TUI/API upgrade contracts on the final tree.

## Exit evidence

- [ ] Exact implementation/dependency commits, installer/image identities and platform evidence linked.
- [ ] Required actual-key evidence and secret-free artifacts recorded.
- [ ] Ledgers complete; archive sprint and link final evidence in plan. Human/release acceptance remains PF-26.
