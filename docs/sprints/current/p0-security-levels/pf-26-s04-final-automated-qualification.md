---
sprint_id: "PF-26-S04"
title: "Final integrated automated security qualification"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-26"
execution_order: 28
owner: "Jim Ricketts"
lane: "qualification"
write_scope: "qa/release"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-26-S01, PF-23-S02, PF-23-S03, PF-25-S01, PF-25-S02, PF-28-S01, PF-29-S02, PF-30-S02"
created: 2026-08-27
updated: 2026-08-27
---

# PF-26-S04 — Final integrated automated security qualification

## Execution mandate

- Deliver: Qualify one frozen integrated candidate with full automated, adversarial, platform, and standards evidence.
- Excludes: Harness construction, runtime fixes, final true-TUI workflows, human acceptance, and release publication.

## Plan linkage

- Plan: [P0 security levels](../../../plans/active/p0-security-levels.md).
- Feature: `PF-26`; see the plan's adopted contract and requirement traceability.
- Acceptance advanced: Qualify one frozen integrated candidate with full automated, adversarial, platform, and standards evidence.

## Code boundaries

- Existing (paths below `codex-rs/`): `scripts/{security-level-compat,security-level-adversarial,security-level-standards-check}` from S01.
- Planned: `qa/release/<version>/security/` candidate manifest and final automated evidence.
- Tests: sibling unit/integration regressions and the PF-26 fixture matrix.

## Preconditions

- [ ] Listed dependencies are completed and archived.
- [ ] Read applicable root and nested AGENTS instructions; plan remains active.
- [ ] Allocate exact owner/worktree/branch/base and literal write scope in the plan; check lane/slot conflicts.

## Done

- [x] Approved feature contract decomposed into this single-feature draft.

## Remaining

- [ ] Pin version, candidate commit/binary digests, dependency commits, environment, and exact baseline; reject stale or mixed-candidate evidence.
- [ ] Run complete affected suites including codex-core, security-policy, config, vault, network-proxy, and tui; no filtering around failures.
- [ ] Run compatibility, all-source/all-sink canaries, protected-action/race/taint fixtures, and real browser isolation/egress probes.
- [ ] Collect Linux/macOS/Windows results for promised controls; record backend support versus fail-closed unavailable behavior explicitly.
- [ ] Complete the versioned standards/design-requirement crosswalk with no missing applicable case or unsupported security claim.
- [ ] Obtain independent security review with no open critical finding; return fixes to owning sprints and requalify the resulting candidate.

## Verification

- [ ] Verify formatting/fixes precede final tests; any later runtime change invalidates affected candidate evidence.
- [ ] Run `python3 scripts/security-level-compat --baseline <commit> --candidate <binary> --output <dir>`.
- [ ] Run `python3 scripts/security-level-adversarial --candidate <binary> --output <dir>`.
- [ ] Run `python3 scripts/security-level-standards-check --manifest qa/release/<version>/security/standards-crosswalk.yaml`.
- [ ] TUI applicability: PF-26-S02 repeats final integrated workflows after this sprint; component interactive sprints must already have their own actual-key proof.

## Exit evidence

- [ ] Final candidate and dependency commits, commands, platform results, and artifact paths recorded.
- [ ] Applicable actual-key proof is linked; no missing test is relabeled a pass.
- [ ] Ledgers reflect reality; completed record is archived and plan evidence linked.
