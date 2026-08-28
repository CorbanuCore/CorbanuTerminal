---
sprint_id: "PF-27-S03"
title: "Platform containment contract and probes"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-27"
execution_order: 2
owner: "Jim Ricketts"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "none"
created: 2026-08-28
updated: 2026-08-28
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

- Planned: codex-rs/secret-broker/src/platform_contract.rs; scripts/security-platform-probe; qa/security-levels/platform/
- Existing integration paths are read-only until the named consumer sprint; shared manifests/lockfiles require serialized ownership.

## Preconditions

- [ ] Plan active; dependencies in front matter completed and archived.
- [ ] Assign a named execution owner and exact plan-matching worktree/branch/base; run the sprint checker before readiness.
- [ ] Read root and nearest implementation AGENTS.md; reserve disjoint write scope and receiving integration gate if parallel.

## Done

- [x] Bounded preparation/foundation mandate created from the accepted review; no implementation or platform acceptance claimed.

## Remaining

- [ ] Record a Linux/macOS/Windows matrix: trusted identities/processes, agent shell/plugin/MCP/child threat, filesystem/config/IPC/network access, inherited handles, process-memory/debug permissions, signing/entitlements and elevation actually required. Same-user process separation alone is not proof.
- [ ] Implement bounded capability probes from the untrusted execution context with synthetic process, file, handle and IPC canaries; distinguish supported, unsupported and untested mechanisms. Record OS/engine versions and rerun triggers; do not equate installation, notarization or a configuration flag with containment.
- [ ] Specify authenticated human controller IPC and protected policy-store ownership, including delete/rename/symlink/rollback/restart attacks; PF-20 implements the store and PF-27-S01/S02 implement and qualify the production boundary.
- [ ] Version the capability-result schema and fixture protocol; no runtime secret resolution, host-wide trust changes or automatic administrator setup. Any needed elevation uses human approval without password persistence.
- [ ] Require reviewed per-OS mechanism choices and successful design probes before this contract completes; unavailable target access is not a platform pass. Update estimates with measured feasibility, not universal OS assumptions.

## Verification

- [ ] Run affected format/fix tools before final tests; record exact commands and actual test counts.
- [ ] Run the planned probe with synthetic canaries on Linux/macOS/Windows; record target versions, expected denial, actual results and unsupported configurations. Run schema/fixture tests and wrong-identity/stale-result cases.
- [ ] TUI applicability: none for this pure preparation/foundation boundary; user-facing consumer sprints retain true-TUI proof.
- [ ] Verify no runtime route or profile becomes available from fixture-only preparation.

## Exit evidence

- [ ] Commit, contract/fixture versions, owner review and final-tree outputs under `qa/security-levels/sprints/PF-27-S03/`.
- [ ] PF-27-S01/S02 rerun all probes against the final actual launch path; this contract cannot activate a protected mode.
- [ ] Record integration handoff and scope audit; complete all ledgers before archive and update plan/navigation.
