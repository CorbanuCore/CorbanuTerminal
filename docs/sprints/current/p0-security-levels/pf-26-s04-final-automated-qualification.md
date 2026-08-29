---
sprint_id: "PF-26-S04"
title: "Final integrated automated security qualification"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-26"
execution_order: 74
owner: "Jim Ricketts"
lane: "qualification"
write_scope: "qa/release"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-26-S01, PF-13-S07, PF-21-S02, PF-23-S03, PF-25-S02, PF-36-S02, PF-41-S02"
created: 2026-08-27
updated: 2026-08-27
---

# PF-26-S04 — Final integrated automated security qualification

## Execution mandate

- Deliver: Qualify one frozen integrated candidate with full automated, adversarial, platform, and standards evidence.
- Excludes: Harness construction, runtime fixes, final true-TUI workflows, human acceptance, and release publication.

## Plan linkage

- Upstream: [plan touch record](../../../plans/active/p0-security-levels.md#upstream-touch-record); resolve this sprint's adapter rows.
- Plan: [P0 security levels](../../../plans/active/p0-security-levels.md).
- Feature: `PF-26`; see the plan's adopted contract and requirement traceability.
- Acceptance advanced: Qualify one frozen integrated candidate with full automated, adversarial, platform, and standards evidence.

## Code boundaries

- Existing (repository-root paths): `scripts/{security-level-compat,security-level-adversarial,security-level-standards-check}` from S01; exact schemas and preparation commands are in `qa/security-levels/fixtures/README.md`.
- Planned: `qa/release/<version>/security/` candidate manifest and final automated evidence.
- Tests: sibling unit/integration regressions and the PF-26 fixture matrix.

## Preconditions

- [ ] Plan upstream baseline, adapter ownership, and exact contract tests are resolved before readiness.
- [ ] Listed dependencies are completed and archived.
- [ ] Read applicable root and nested AGENTS instructions; plan remains active.
- [ ] Allocate exact owner/worktree/branch/base and literal write scope in the plan; check lane/slot conflicts.

## Done

- [x] Approved feature contract decomposed into this single-feature draft.

## Remaining

- [ ] Pin version, candidate commit/binary digests, dependency commits, environment, and exact baseline; reject stale or mixed-candidate evidence.
- [ ] Run complete affected suites including codex-core, security-policy, config, vault, network-proxy, browser-isolation and tui; no filtering around failures.
- [ ] Run compatibility, all-source/all-sink canaries, protected-action/race/taint fixtures, and real browser isolation/egress probes.
- [ ] Audit every upstream-touch row and ingress adapter against the final candidate; run native schema/child/history/transport-recovery contracts and separate non-secret task-integrity assertions.
- [ ] Record verified upstream and fork SHAs, per-adapter retain/adapt/remove disposition, exact commands and evidence; unresolved compatibility cannot be marked qualified.
- [ ] Collect Linux/macOS/Windows results for promised controls; record backend support versus fail-closed unavailable behavior explicitly.
- [ ] Verify PF-30-S03 runtime reuse, missing-engine/image setup, stopped/stalled owned-service recovery, elevation/cancel, secret-free authentication handling, and no Permissive setup on the final candidate.
- [ ] Complete the versioned standards/design-requirement crosswalk with no missing applicable case or unsupported security claim.
- [ ] Obtain independent security review with no open critical finding; return fixes to owning sprints and requalify the resulting candidate.
- [ ] Record Travis's selected independent reviewer provider/model and exact reviewed candidate; routine Autoreview is supporting evidence, not a substitute for that selection or Travis's human acceptance.

## Verification

- [ ] Record applicable upstream adapter evidence or justified non-applicability; structural checks alone are not qualification.
- [ ] Verify formatting/fixes precede final tests; any later runtime change invalidates affected candidate evidence.
- [ ] Run `python3 scripts/security-level-compat --baseline <commit> --candidate <binary> --output <dir>`.
- [ ] Run `python3 scripts/security-level-adversarial --bundle <prepared-dir> --observations <host-run.json> --candidate <binary> --source-commit <sha> --platform <platform> --not-before <UTC> --output <dir>`; retain trusted build/run provenance, not model-authored verdicts.
- [ ] Run `python3 scripts/security-level-standards-check --manifest qa/release/<version>/security/standards-crosswalk.json --candidate <binary> --source-commit <sha> --platform <platform> --not-before <UTC>`; S01's planning check is not qualification.
- [ ] TUI applicability: PF-26-S02 repeats final integrated workflows after this sprint; component interactive sprints must already have their own actual-key proof.

## Exit evidence

- [ ] Final candidate and dependency commits, commands, platform results, and artifact paths recorded.
- [ ] Applicable actual-key proof is linked; no missing test is relabeled a pass.
- [ ] Ledgers reflect reality; completed record is archived and plan evidence linked.
