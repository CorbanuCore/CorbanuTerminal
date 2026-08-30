---
sprint_id: "PF-31-S01"
title: "Pinned retriever artifact and sandbox"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-31"
execution_order: 46
owner: "Jim Ricketts"
worktree: "/Users/travisgood/Documents/ChatGPT/corbanu-security-levels"
branch: "feat/p0-security-levels"
base_commit: "7cc15ae0762664d6d01765de407329887da9f876"
depends_on: "PF-33-S02, PF-27-S02, PF-31-S04"
created: 2026-08-28
updated: 2026-08-28
---

# PF-31-S01 — Pinned retriever artifact and sandbox

## Execution mandate

- Deliver: Public retrieval runs in a pinned isolated workload with no host secrets or browser session.
- Excludes: adjacent feature implementation, Permissive policy changes, and unlisted integrations.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md#pf-31).
- Feature: `PF-31`.
- Product citation: **Non-negotiable controls** — “Default to no secret export, arbitrary egress, clipboard exposure, or sensitive logging.”
- Acceptance advanced: Public retrieval runs in a pinned isolated workload with no host secrets or browser session.
- Sources and archive disposition: [PF-31 reconciliation](../../../plans/security-source-reconciliation.md#pf-31).

## Code boundaries

- Planned setup/recovery UI: `codex-rs/tui/src/security/retriever_setup.rs`; typed engine lifecycle adapter under `codex-rs/web-retriever/src/engine.rs` and tests.

- OpenClaw adoption reference: [OC-8](../../../plans/openclaw-source-review-2026-08-28.md#oc-8) at `13adff02ca3897768d80d2bca18f5acf08c55d91`; see the review for named functions, callers, tests and limits. Reference tests are not candidate evidence.

- Existing/foundation: codex-rs/sandboxing/src/{manager,spawn}.rs; codex-rs/exec-server/src/process_sandbox.rs.
- Planned: codex-rs/web-retriever/src/{main,sandbox}.rs; tools/web-retriever/{Containerfile,manifest.json}.
- Tests: planned colocated Rust test modules prefixed `pf_31_s01`; fixtures use synthetic secrets and fake services only.

## Preconditions

- [ ] All dependencies in front matter are completed and archived; plan remains active.
- [ ] Read root and nearest implementation-path AGENTS.md; verify exact plan/worktree coordinates.
- [ ] Confirm source pins, declared crate/module paths, and backend/API availability; unresolved security prerequisites block readiness.

## Done

- [x] New single-feature record reconciled with current ownership and archived design input; no implementation claimed.

## Remaining

- [ ] Consume PF-31-S04 pins and engine fixtures; implement the idempotent human install/start/restart/pull/test flow with existing-engine reuse and Corbanu-owned worker locking. Shared engines and unrelated containers are never replaced or restarted without separate authority.
- [ ] Exercise actual-key installation cancel/failure/retry, reuse, stalled-worker recovery and concurrent launches on Linux/macOS/Windows; explain elevation and never record passwords. Final engine state, mounts/network and health must match the approved containment contract.

- [ ] Verify observed image digest, user, mounts, engine target, network and resource policy separately from configured defaults; test mismatched/reused workers, required-session identity and failed provisioning without host fallback.

- [ ] Integrate the completed PF-31-S04 artifact manifest, pinned image/runtime/dependency digests, license inventory and SBOM; verify identity again at installation and startup.
- [ ] Mount no workspace, vault, wallet, host browser profile or IPC sockets; use ephemeral storage, unprivileged identity and bounded CPU/memory/time/file/process budgets.
- [ ] Restrict all worker networking to PF-33 policy; disable downloads/executables, arbitrary eval, extensions, persistent cookies and service-worker persistence by default.
- [ ] Probe actual isolation at startup and expose capability failures; Moderate/Aggressive refuse retrieval if the required backend is unavailable.
- [ ] Test forbidden mounts, profile theft, host IPC, process escape, resource exhaustion, invalid image signatures and missing runtime.
- [ ] Add named `pf_31_s01` regression tests; update affected Cargo/Bazel/lock/schema edges together without broadening this feature.

## Verification

- [ ] Run `cd codex-rs && just fix -p <affected-crate>` for each listed crate, then `just fmt`; inspect the final diff.
- [ ] Focused: `cd codex-rs && just test -p codex-web-retriever pf_31_s01 && just test -p codex-sandboxing pf_31_s01`; confirm tests actually ran.
- [ ] Integration: full affected crate suites via `just test -p <affected-crate>`; update Bazel locks when manifests change.
- [ ] TUI applicability: required for install/cancel/recovery and engine reuse; send actual keys separately from Enter and rerun final flows in PF-26-S02.
- [ ] Record candidate/commit, commands, expected/actual outcomes and safe artifact digests; no production credentials or funds.

## Exit evidence

- [ ] Implementation commit and final-tree outputs under `qa/security-levels/sprints/PF-31-S01/`.
- [ ] Acceptance and source-mapping assertions proven; applicable true-TUI keys/checkpoints captured after formatting.
- [ ] PF-26 final-candidate and both-live-repository requalification remains mandatory; no release-complete claim here.
- [ ] Done/Remaining reflect reality; completed record moved to the archive and plan/navigation updated.
