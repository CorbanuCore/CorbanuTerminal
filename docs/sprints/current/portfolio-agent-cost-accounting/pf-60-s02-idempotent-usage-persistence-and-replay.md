---
sprint_id: "PF-60-S02"
title: "Idempotent usage persistence and replay"
status: in_progress
plan_file: "docs/plans/active/portfolio-agent-cost-accounting.md"
plan_feature: "PF-60"
execution_order: 2
owner: "Astra High accounting exact pricing"
parallel_lane: "accounting-exact-pricing"
write_scope: "codex-rs/state/src/runtime/accounting.rs, codex-rs/state/src/runtime/accounting_pricing.rs, codex-rs/state/src/runtime/accounting_pricing_tests.rs, qa/portfolio/agent-cost-accounting/pf-60-s02/exact-pricing-increment.md"
integration_gate: "Codex management reviews the test-only exact quotation helper, serializes one child registration, reruns state and governance tests on the combined tree, and allocates later storage/production wiring separately; no worker commit/merge/push or live collection."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/accounting-pf60-s01-20260911"
branch: "workstream/accounting-pf60-s01-20260911"
base_commit: "c33d47f6ccd00a64fcb05a057472d8ca9f0139d4"
depends_on: "PF-60-S01"
created: 2026-09-09
updated: 2026-09-11
---

# PF-60-S02 — Idempotent usage persistence and replay

S01 is archived; the journal is reviewed/integrated. Exact quotation is the next
same-sprint increment. Reuse the idle worker after clean fast-forward; its name
preserves history, not S01 ownership. Record actual dispatch HEAD separately.
[Current allocation](../../../research/agent-cost-accounting/exact-pricing-allocation.md).

## Execution mandate

- Next increment: exact USD quotation of replayed observations using synthetic approved immutable snapshots, still test-only. Journal proof is accepted; full S02 still owes durable estimates, native wiring and complete golden totals.
- Excludes: Changing prices, rebilling historical customers, collecting prompts, restoring legacy Plan allowances, or silently converting allowance to cash.
- Budget proposal: 2–4 builder-days plus independent testing; not a commitment. Stop and re-slice if the bound is exceeded.

## Plan linkage

- Plan: [Unified agent cost and usage accounting](../../../plans/active/portfolio-agent-cost-accounting.md)
- Feature: `PF-60`; acceptance: Replay each fixture twice and after process restart; persisted and reconstructed totals equal the approved fixture exactly.
- Upstream/allocation: [plan record](../../../plans/active/portfolio-agent-cost-accounting.md#native-lifecycle-and-upstream-touch-record).

## Code boundaries

- Existing, read before work: `codex-rs/state/migrations/0041_provider_request_cache_usage.sql`; `codex-rs/app-server/src/request_processors/token_usage_replay.rs`; `codex-rs/tui/src/chatwidget/usage.rs`; `codex-rs/tui/src/token_usage.rs`.
- Planned output: front-matter paths only; accounting.rs gets only one child registration, serially delegated. No journal/types/SQL changes or concurrent state writer.
- Tests/evidence: new exact-pricing receipt only; preserve first-increment evidence. Native ABI, dependencies, production migrations and collectors excluded.

## Preconditions

- [x] Plan active under Travis's standing continuation authority; reservation transferred from completed S01, no fourth lane.
- [x] S01 accepted/archived after defaults approval, combined-tree fixture proof and independently reviewed technical handoff.
- [x] Exact worker/branch/base and four-file pricing scope match plan; parent must clean-fast-forward and record launch HEAD at dispatch.
- [x] Travis approved the default policy; synthetic/local tests only, no billing/live collection. Bound this allocation to one worker, at most 500 non-test lines, one independent review plus scoped corrections; no exhausted allowance reset.
- [x] Parent inspected native-state/pricing seams and registered test boundaries; exact current allocation linked above. New pricing test passes remain pending.

## Done

- [x] First test-only journal and native SQLite regressions reviewed clean and integrated at c33d47f6c (worker 3f39d7a65); 202 state tests pass on combined tree. [Receipt](../../../../qa/portfolio/agent-cost-accounting/pf-60-s02/first-increment.md).
- [x] Real on-disk close/reopen, duplicate/reordered revision, rollback, concurrent writer, unknown/zero and partial Anthropic regressions pass; no process-kill or production claim.

## Remaining

- [ ] Implement exact quotation, prospective snapshot selection, checked decimal arithmetic and explicit unknown/rounded values in current allocation; no live fetch or production caller.
- [ ] Test literal prices/unknowns/overflow/half-even and close/reopen quotations without changing journal records; return hashes and real results for independent review.
- [ ] After reviewing the first increment, manager allocates production migration/dispatch/presence/price/retention wiring and complete S02 golden tests separately; do not broaden this worker's scope.
- [ ] Record actual outputs, counterexamples, remaining limitations and a concrete next-sprint handoff; stop on changed scope.

## Verification

- [ ] Current increment: focused `just test -p codex-state runtime::accounting::pricing::tests`, journal selector and full state crate, then normal-library check; pinned offline commands in allocation.
- [ ] Integration: from repo root, `python3 docs/plans/check.py; python3 docs/sprints/check.py`; `git diff --check`.
- [ ] Resolve TUI applicability against the plan; record actual-key success, failure/cancel, recovery/resume and final binary evidence for every affected interactive path.
- [ ] Record expected versus actual results and nonzero test counts; no unchecked assumption is converted into a pass.

## Exit evidence

- [ ] Output commit/digest and input provenance recorded; checks linked to that final artifact/tree.
- [ ] Named human/receiving owner accepts the bounded output; needed go/no-go decision is recorded.
- [ ] Handoff includes changed scope, contracts, known gaps and required combined-tree evidence.
- [ ] Done/Remaining ledgers updated honestly; archive accepted record under `docs/sprints/archive/portfolio-agent-cost-accounting/` and update plan backlinks.
