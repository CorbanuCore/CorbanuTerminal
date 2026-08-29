---
sprint_id: "PF-26-S03"
title: "Human acceptance, finished docs, and release evidence"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-26"
execution_order: 76
owner: "Jim Ricketts"
lane: "qualification"
write_scope: "qa/release, docs/features, docs/authentication.md, docs/slash_commands.md"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-26-S02"
created: 2026-08-24
updated: 2026-08-27
---

# PF-26-S03 — Human acceptance, finished docs, and release evidence

## Execution mandate

- Deliver: named human acceptance, candidate-verified finished documentation, and complete release linkage.
- Excludes: unaccepted feature claims, product-scope changes, benchmark waivers, and the final release decision itself.

## Plan linkage

- Upstream: [plan touch record](../../../plans/active/p0-security-levels.md#upstream-touch-record); resolve this sprint's adapter rows.
- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md)
- Feature: `PF-26`
- Acceptance advanced: users receive accurate security/vault guidance only after the final candidate passes required evidence.

## Code boundaries

- Planned docs: `docs/features/security.md`; updates to `docs/features/{vault,index}.md`, `docs/authentication.md`, and `docs/slash_commands.md`
- Evidence: `qa/release/<version>/security/human-acceptance.md`; release manifest/checklist
- Navigation: `mkdocs.yml` only after finished pages exist

## Preconditions

- [ ] Plan upstream baseline, adapter ownership, and exact contract tests are resolved before readiness.
- [ ] Every listed dependency is completed and archived.
- [ ] A named human tester and independent security reviewer are recorded.
- [ ] Candidate version/commit and benchmark state are fixed.

- [ ] Allocate lane/worktree/base in the plan and validate disjoint write scopes before readiness.

## Done

- [x] Sprint record is linked only to PF-26.
- [x] Travis Good named himself as human tester on 2026-08-27 and selected Fable High (`claude-fable-5`, high effort) as independent reviewer; no review or human acceptance recorded by these selections.

## Remaining

- [ ] Have the named human perform the accepted level, cancel, downgrade, grant, revoke, kill, restart, and credential workflows on the exact candidate.
- [ ] Record tester, date, candidate, flow, result, and artifact without secrets or protected financial values.
- [ ] Publish the finished `/security` user page with exact product-spec heading/excerpt and candidate-backed behavior only.
- [ ] Update vault/authentication guidance: Permissive raw helper status, Moderate/Aggressive broker-only behavior, exact limitations, and recovery.
- [ ] Update feature catalog, slash commands, MkDocs navigation, release record, benchmark tracker link, and remaining blockers.
- [ ] Run docs, plans, sprints, links, and release checks; block shipment on any hard-gate failure.

## Verification

- [ ] Record applicable upstream adapter evidence or justified non-applicability; structural checks alone are not qualification.
- [ ] Documentation build: `mkdocs build --strict`.
- [ ] Governance: `python3 docs/plans/check.py && python3 docs/sprints/check.py`.
- [ ] Human record matches the final candidate commit and required flows.
- [ ] Independent security review has no open P0 finding.
- [ ] Benchmark/release records are present and passing when due.

## Exit evidence

- [ ] Named human acceptance linked.
- [ ] Finished docs and exact product citations linked.
- [ ] Release and benchmark records linked with no hard blocker.
- [ ] Ledgers reflect reality and the completed record is archived.
