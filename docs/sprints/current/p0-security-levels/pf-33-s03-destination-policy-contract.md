---
sprint_id: "PF-33-S03"
title: "Pure destination-policy contract"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-33"
execution_order: 18
owner: "Jim Ricketts"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "none"
created: 2026-08-28
updated: 2026-08-28
---

# PF-33-S03 — Pure destination-policy contract

## Execution mandate

- Deliver: Freeze pure URL, address-set and redirect decisions for later connection enforcement.
- Excludes: protected-mode activation, adjacent feature implementation and Permissive behavior changes.

## Plan linkage

- Plan: [P0 security levels](../../../plans/active/p0-security-levels.md#pf-33).
- Feature: `PF-33`.
- Product citation: **Reconciled security scope — TO BUILD** — “Unknown or unsupported protected paths fail visibly rather than falling back to raw secrets or unscreened execution.”
- Acceptance advanced: [accepted architecture refinements](../../../plans/security-architecture-refinements-2026-08-28.md).
- Source input: [OpenClaw source review](../../../plans/openclaw-source-review-2026-08-28.md) at `13adff02ca3897768d80d2bca18f5acf08c55d91`; reference behavior is not candidate acceptance.

## Code boundaries

- Planned: codex-rs/network-proxy/src/destination_contract.rs; codex-rs/network-proxy/tests/destination_contract.rs
- Existing integration paths are read-only until the named consumer sprint; shared manifests/lockfiles require serialized ownership.

## Preconditions

- [ ] Plan active; dependencies in front matter completed and archived.
- [ ] Assign a named execution owner and exact plan-matching worktree/branch/base; run the sprint checker before readiness.
- [ ] Read root and nearest implementation AGENTS.md; reserve disjoint write scope and receiving integration gate if parallel.

## Done

- [x] Bounded preparation/foundation mandate created from the accepted review; no implementation or platform acceptance claimed.

## Remaining

- [ ] Define normalized scheme/host/port/method/path, DNS-answer and redirect-decision types with versioned fixtures; no live socket or broker hooks in this sprint.
- [ ] Distinguish absent restrictions, explicit empty deny-all, wildcard public scope and explicit private-service authorization. Moderate public retrieval need not acquire a blanket per-host grant; Aggressive still requires its narrow grants.
- [ ] Keep private provider exceptions separate from public retrieval. Specify approved address/identity sets and change/revalidation rules; TLS identity alone does not authorize a private destination.
- [ ] Add table/property tests for IDNA, userinfo, suffix/trailing-dot confusion, unusual IPv4, mapped IPv6, mixed/private answers, downgrade redirects, credential/body replay and malformed policy.
- [ ] Freeze pure decision fixtures and ownership; PF-33-S01/S02 must wire actual DNS, retries, redirects and peer checks without replacing the decision contract with a permissive adapter.

## Verification

- [ ] Run affected format/fix tools before final tests; record exact commands and actual test counts.
- [ ] Run planned destination-contract unit/property tests with synthetic addresses; prove empty/absent/wildcard/private-policy polarity and bounded normalization.
- [ ] TUI applicability: none for this pure preparation/foundation boundary; user-facing consumer sprints retain true-TUI proof.
- [ ] Verify no runtime route or profile becomes available from fixture-only preparation.

## Exit evidence

- [ ] Commit, contract/fixture versions, owner review and final-tree outputs under `qa/security-levels/sprints/PF-33-S03/`.
- [ ] No claim of SSRF prevention until PF-33-S01/S02 real resolver, connected-peer and alternate-egress qualification completes.
- [ ] Record integration handoff and scope audit; complete all ledgers before archive and update plan/navigation.
