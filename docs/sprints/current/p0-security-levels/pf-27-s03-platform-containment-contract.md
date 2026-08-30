---
sprint_id: "PF-27-S03"
title: "Platform containment contract and probes"
status: in_progress
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-27"
execution_order: 16
owner: "Codex foundation/platform lane"
parallel_lane: "foundation-platform"
write_scope: "codex-rs/secret-broker/, codex-rs/Cargo.toml, codex-rs/Cargo.lock, BUILD.bazel, MODULE.bazel.lock, .github/workflows/security-platform-contract.yml, scripts/security-platform-probe, scripts/security_platform_probe.py, scripts/test_security_platform_probe.py, qa/security-levels/platform/, qa/security-levels/sprints/PF-27-S03/, docs/plans/active/p0-security-levels.md, docs/sprints/current/p0-security-levels/, docs/sprints/archive/p0-security-levels/pf-27-s03-platform-containment-contract.md, docs/sprints/index.md, mkdocs.yml"
integration_gate: "Jim Ricketts receives the PF-27-S03 candidate at G1, audits the literal scope, exclusively registers codex-secret-broker Cargo/Bazel workspace surfaces after codex-content-security, reruns schema/probe/governance checks on the combined tree, then archives the sprint before a consumer can use the contract."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/p0-security-foundation-platform"
branch: "feat/p0-security-foundation-platform"
base_commit: "1907d99aed9714f05a5f54fca1703658017d616c"
depends_on: "none"
created: 2026-08-28
updated: 2026-08-30
---

# PF-27-S03 — Platform containment contract and probes

## Execution mandate

- Deliver: Freeze the trusted controller/broker and untrusted worker boundary with executable synthetic probes, before broker implementation.
- Excludes: protected-mode activation, adjacent feature implementation and Permissive behavior changes.

## Plan linkage

- Plan: [P0 security levels](../../../plans/active/p0-security-levels.md#pf-27).
- Feature: `PF-27`.
- Product citation: **Reconciled security scope — TO BUILD** — “Unknown or unsupported protected paths fail visibly rather than falling back to raw secrets or unscreened execution.”
- Acceptance advanced: [accepted architecture refinements](../../../plans/security-architecture-refinements-2026-08-28.md).
- Source input: [OpenClaw source review](../../../plans/openclaw-source-review-2026-08-28.md) at `13adff02ca3897768d80d2bca18f5acf08c55d91`; reference behavior is not candidate acceptance.

## Code boundaries

- Planned: codex-rs/secret-broker/src/platform_contract.rs; scripts/security-platform-probe; scripts/security_platform_probe.py; scripts/test_security_platform_probe.py; qa/security-levels/platform/
- G1 registration: codex-rs/secret-broker/{Cargo.toml,BUILD.bazel,src/lib.rs}; codex-rs/{Cargo.toml,Cargo.lock}; MODULE.bazel.lock. The integration owner alone changes these shared surfaces, after the completed codex-content-security registration.
- Scope amendment (2026-08-30): the fifth independent review showed that the extensionless probe was omitted by standard Ruff and test discovery. The integration owner added only the conventional linted Python implementation and companion discovery test; the extensionless command remains a stable shim. No shared manifest, runtime route, or consumer surface is added.
- G1 scope amendment (2026-08-30): after merging the completed PF-34-S04 registration, the integration owner reserved and registers the standalone `codex-secret-broker` crate on Cargo/Bazel surfaces. The crate exports only the frozen platform contract; no runtime consumer or protected-mode route is added.
- Post-G1 review amendment (2026-08-30): the integration owner expanded the serialized scope to bind Rust tests to frozen schema/result fixtures, add an opaque activation witness, cover remaining rejection classes, add three-OS CI, make the IPC socket probe independent of long temporary paths, reconcile owner/state/count narratives, and perform the required archive/navigation transition. This remains contract and evidence work only.

## Preconditions

- [x] Plan active; dependencies in front matter are `none`.
- [x] Named execution owner and exact plan-matching worktree/branch/base are allocated at post-handoff `main`; both governance checkers pass.
- [x] Read root and nearest implementation AGENTS.md; reserved disjoint write scope and the integration-owner-only G1 registration gate.

## Done

- [x] Bounded preparation/foundation mandate created from the accepted review; no implementation or platform acceptance claimed.
- [x] Implemented a versioned result schema and bounded cross-platform synthetic probe for process, filesystem/config, inherited handle, IPC, network, process-memory/debug, signing/entitlement, elevation and protected-store boundaries; unknown/stale/wrong-target/false-eligibility input fails closed.
- [x] Specified authenticated human-controller IPC and protected policy-store ownership, including replay, delete, rename, symlink/reparse, rollback and restart attacks; fixture preparation creates no runtime route.
- [x] Recorded reviewed-input mechanism candidates for Linux, macOS and Windows without treating installation, signing, same-user separation or proxy configuration as containment.
- [x] Executed the SHA-bound probe on macOS, Linux, and Windows with 10/10 capability records and 8/8 contract regressions per target; all three correctly remain ineligible because observed same-user bypasses are unsupported.
- [x] Recorded the Linux/macOS/Windows identity, worker-threat, filesystem/config/IPC/network, handle, debug, signing/entitlement, elevation and store matrix from direct measurements; unsupported and untested Windows mechanisms remain explicit rather than inferred.
- [x] Ran ten independent TMUX + Corbanu Terminal + Claude Opus 5.0 Max reviews; repaired all thirty-four findings/coverage gaps across process/handle/elevation/signing/network/IPC probes, schema parity, macOS helper resolution, Windows/unknown-OS boot and token identity, Rust identity/envelope validation, strict versus archival validation and eligibility enforcement, stable malformed-input handling, standard lint/test discovery, reproducible artifact compaction, and positive eligibility discrimination, then regenerated exact-SHA macOS/Linux/Windows evidence. The tenth complete three-platform review reported no findings.
- [x] Obtained a clean independent review of the final three-platform evidence tree and reran governance before integration.

## Remaining

- [ ] Complete the integration-owner shared-surface audit and archive the sprint at G1.

## Verification

- [x] Run affected format/fix tools before final tests; repository-standard `format.py --check` and focused Ruff discovery pass; Python self-test passes 8/8, discovered Python tests pass 6/6, and the registered Rust activation-gate tests pass 9/9 through both nextest and Bazel.
- [x] Run the planned probe with synthetic canaries on Linux/macOS/Windows; record target versions, expected denial, actual results and unsupported configurations. Run schema/fixture tests and wrong-identity/stale-result cases.
- [x] Record a clean independent review of the final three-platform evidence tree and rerun governance before archive.
- [ ] Rerun the final focused and governance checks on the integration target before archive.
- [x] TUI applicability: none for this pure preparation/foundation boundary; user-facing consumer sprints retain true-TUI proof.
- [x] Verify no runtime route or profile becomes available from fixture-only preparation; Cargo/Bazel register only the standalone contract crate, no workspace crate consumes it, and the probe creates only bounded temporary synthetic fixtures.

## Exit evidence

- [x] Commit, contract/fixture versions, owner review and final-tree outputs under `qa/security-levels/sprints/PF-27-S03/`.
- [x] Record that PF-27-S04/S02 must rerun all probes against the final actual launch path; this contract cannot activate a protected mode.
- [ ] Record integration handoff and scope audit; complete all ledgers before archive and update plan/navigation.
