---
sprint_id: "PF-70-S05"
title: "Creator ownership and economics decision"
status: draft
plan_file: "docs/plans/proposed/portfolio-acceleration-index.md"
plan_feature: "PF-70"
execution_order: 5
owner: "Alex Good (index owner, proposed)"
parallel_lane: "UNALLOCATED"
write_scope: "UNALLOCATED"
integration_gate: "UNALLOCATED"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-70-S04"
created: 2026-09-10
updated: 2026-09-10
---

# PF-70-S05 — Creator ownership and economics decision

## Execution mandate

- Deliver: `docs/research/acceleration-index/creator-decision.md`; an optional creator-phase go/no-go backed by synthetic ownership, fee and leaderboard examples.
- Excludes: implementation, account claims, real trades, fee collection/payouts, token launch, marketing and invented commercial terms.
- Proposed timebox: 0.5–2 analyst-days after inputs; stop/re-slice rather than imply a launch deadline.

## Plan linkage

- Plan: [Index-creation API, replayability and creator ecosystem](../../../plans/proposed/portfolio-acceleration-index.md)
- Feature: `PF-70`; acceptance: creator rights, economics, dispute/abuse controls and leaderboard meaning are explicit or the phase is parked.

## Code boundaries

- Existing: S04 candidate contract/evidence, product specification and owner-approved service schema; read-only.
- Planned output: `docs/research/acceleration-index/creator-decision.md`.
- Tests/evidence: `qa/portfolio/acceleration-index/pf-70-s05/`; synthetic fixtures only.

## Preconditions

- [ ] Plan active; PF-70-S04 completed/archived; owner chooses whether to evaluate this optional phase.
- [ ] Exact worktree/branch/base and literal document/fixture scope allocated and matched to the plan.
- [ ] Alex/Travis identifies economic decision owner, review budget and qualified legal/commercial review requirements.

## Done

- [x] Draft linked to PF-70; no creator implementation or commercial action completed.

## Remaining

- [ ] Define claim identity, duplicate/copy claims, ownership changes, disputes and deletion; do not infer ownership from first API use.
- [ ] Propose fee attribution and reconciliation with explicit rates still undecided, reversals, refunds and self-trading/abuse cases.
- [ ] Define leaderboard eligibility, time windows, paper versus realized performance, cost treatment and anti-gaming criteria.
- [ ] Exercise synthetic duplicate-claim, replayed-trade, refunded-fee and manipulated-ranking examples; independent recomputation must reproduce ledger/ranking results.
- [ ] Record go/no-go and unresolved rights/terms; a go requires a separate scope/spec amendment and bounded creator implementation/qualification sprints before any code starts.

## Verification

- [ ] Reviewer traces each commercial claim to dated primary evidence or labels it an undecided proposal.
- [ ] Named human reviews synthetic cases and the no-go option; legal/regulatory conclusions cannot be supplied by model confidence.
- [ ] `python3 docs/plans/check.py`, `python3 docs/sprints/check.py` and `git diff --check`; no product UI qualification claimed for document-only scope.

## Exit evidence

- [ ] Accepted decision digest, source/fixture evidence, named owner and implementation-or-park handoff recorded.
- [ ] No real claim, trade, charge, payout or publication performed; creator functionality stays disabled.
- [ ] Done/Remaining accurate; accepted sprint archived and plan backlink updated.
