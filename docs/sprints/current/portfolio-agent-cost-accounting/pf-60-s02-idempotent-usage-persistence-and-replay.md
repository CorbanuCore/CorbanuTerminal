---
sprint_id: "PF-60-S02"
title: "Idempotent usage persistence and replay"
status: in_progress
plan_file: "docs/plans/active/portfolio-agent-cost-accounting.md"
plan_feature: "PF-60"
execution_order: 2
owner: "Codex accounting latest-quote lane"
parallel_lane: "accounting-latest-quote"
write_scope: "codex-rs/state/src/runtime/accounting_estimates.rs, codex-rs/state/src/runtime/accounting_latest_quote_tests.rs, qa/portfolio/agent-cost-accounting/pf-60-s02/latest-quote-increment.md"
integration_gate: "Codex management reviews exact reader/no-repricing proof, serializes sibling test registration and reruns state/governance tests on combined tree. No retention mutation, production, worker commit/merge/push or live collection."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/accounting-pf60-s01-20260911"
branch: "workstream/accounting-pf60-s01-20260911"
base_commit: "99001e9b79676f78b6bd941125c2a8fcfca6d90e"
depends_on: "PF-60-S01"
created: 2026-09-09
updated: 2026-09-11
---

# PF-60-S02 — Idempotent usage persistence and replay

S01 is archived; journal/quotation/storage/contributions/deletion are integrated.
Exact compact values are reviewed/integrated; the worker is closed and previous
literal scope frozen. Latest reader reviewed/tested in staging; next allocation pending. The
[retention handoff](../../../research/agent-cost-accounting/retention-design-handoff.md)
records approved daily expiry; [latest-quote allocation](../../../research/agent-cost-accounting/latest-quote-allocation.md) owns the new exact mandate.

## Execution mandate

- Next: manager allocates atomic retention after the reviewed latest-quote prerequisite. Full S02 still owes compaction/retention, native owner/admission wiring and complete goldens.
- Excludes: Changing prices, rebilling historical customers, collecting prompts, restoring legacy Plan allowances, or silently converting allowance to cash.
- Budget proposal: 2–4 builder-days plus independent testing; not a commitment. Stop and re-slice if the bound is exceeded.

## Plan linkage

- Plan: [Unified agent cost and usage accounting](../../../plans/active/portfolio-agent-cost-accounting.md)
- Feature: `PF-60`; acceptance: Replay each fixture twice and after process restart; persisted and reconstructed totals equal the approved fixture exactly.
- Upstream/allocation: [plan record](../../../plans/active/portfolio-agent-cost-accounting.md#native-lifecycle-and-upstream-touch-record).

## Code boundaries

- Existing, read before work: `codex-rs/state/migrations/0041_provider_request_cache_usage.sql`; `codex-rs/app-server/src/request_processors/token_usage_replay.rs`; `codex-rs/tui/src/chatwidget/usage.rs`; `codex-rs/tui/src/token_usage.rs`.
- Output: front-matter paths are the three-file reader allocation; reviewed B1 paths remain frozen. Parent must separately allocate any retention mutation.
- Tests/evidence: accepted compact-values receipt; preserve it. Native ABI, dependencies, production migrations and collectors remain excluded.

## Preconditions

- [x] Plan active under Travis's standing continuation authority; reservation transferred from completed S01, no fourth lane.
- [x] S01 accepted/archived after defaults approval, combined-tree fixture proof and independently reviewed technical handoff.
- [x] Exact worker/branch/base and three-file reader scope match plan; parent must clean-fast-forward and record launch HEAD at dispatch.
- [x] Travis approved the default policy; synthetic/local tests only, no billing/live collection. Bound this allocation to one worker, at most 500 non-test lines, one independent review plus scoped corrections; no exhausted allowance reset.
- [x] Parent inspected DayTotals/Decimal visibility and amount/rate parsing distinction; B1 is now reviewed and tested, receipt below.

## Done

- [x] Latest quote8b6d629b6 reviewed clean and combined staging04ba6b8b7 passes242state/80TaskNode tests; reader only, no retention mutation. [Evidence](../../../../qa/initiative-control/native-staging-2026-09-12.md). Canonical receiving transfer/publication pending.

- [x] Travis approved conservative UTC-day-start+365-day aggregate expiry; no reapproval needed. Detail90/replay365 policies unchanged; S03 requested range/interval filters remain draft.
- [x] First test-only journal and native SQLite regressions reviewed clean and integrated at c33d47f6c (worker 3f39d7a65); 202 state tests pass on combined tree. [Receipt](../../../../qa/portfolio/agent-cost-accounting/pf-60-s02/first-increment.md).
- [x] Real on-disk close/reopen, duplicate/reordered revision, rollback, concurrent writer, unknown/zero and partial Anthropic regressions pass; no process-kill or production claim.
- [x] Exact quotation e8ffdad4e reviewed clean and integrated at 486d2fb94; 214 state tests pass there. Prospective snapshots, exact arithmetic/half-even, unknowns and journal replay covered. [Receipt](../../../../qa/portfolio/agent-cost-accounting/pf-60-s02/exact-pricing-increment.md).
- [x] Immutable storage d898fbac0 reviewed clean and integrated at56295f668; all220 state tests pass there. Atomic snapshot/binding/evidence versions, corruption/reopen proof. [Receipt](../../../../qa/portfolio/agent-cost-accounting/pf-60-s02/estimate-storage-increment.md).

- [x] Contributions/deletion b08fff66d reviewed clean and integrated at3898eaa65;226 state tests pass there. Exact current totals, atomic all-version deletion/shared snapshot retention, deterministic serialization orders and two disk reopens; no simultaneous lock-contention/pruning/native owner claim. [Receipt](../../../../qa/portfolio/agent-cost-accounting/pf-60-s02/contribution-deletion-increment.md).

- [x] Exact compact-value codec c8d46d709 independently reviewed clean and integrated;237 state tests pass on receiving tree. Canonical u128/24-place USD, all seven metric populations, checked composition and malformed/overflow rejection; no DB mutation. [Receipt](../../../../qa/portfolio/agent-cost-accounting/pf-60-s02/compact-values-increment.md).

## Remaining

- [ ] Transfer reviewed/tested staging checkpoint into canonical receiving after its pending publication resolves, preserving the exact candidate.
- [ ] Manager resolves daily/rolling expiry coverage, stale/unquoted source and late-import semantics before allocating any compaction mutation; no repeated defaults question.
- [ ] After reviewing the first increment, manager allocates production migration/dispatch/presence/price/retention wiring and complete S02 golden tests separately; do not broaden this worker's scope.
- [ ] Record actual outputs, counterexamples, remaining limitations and a concrete next-sprint handoff; stop on changed scope.

## Verification

- [x] B1:11 compact-value tests and four existing selectors passed; combined237 state/73 Task Node tests and normal-library check passed atc8d46d709. One non-accounting LEAK marker disclosed in manager handoff; no leak-clean claim.
- [ ] Integration: from repo root, `python3 docs/plans/check.py; python3 docs/sprints/check.py`; `git diff --check`.
- [ ] Resolve TUI applicability against the plan; record actual-key success, failure/cancel, recovery/resume and final binary evidence for every affected interactive path.
- [ ] Record expected versus actual results and nonzero test counts; no unchecked assumption is converted into a pass.

## Exit evidence

- [ ] Output commit/digest and input provenance recorded; checks linked to that final artifact/tree.
- [ ] Named human/receiving owner accepts the bounded output; needed go/no-go decision is recorded.
- [ ] Handoff includes changed scope, contracts, known gaps and required combined-tree evidence.
- [ ] Done/Remaining ledgers updated honestly; archive accepted record under `docs/sprints/archive/portfolio-agent-cost-accounting/` and update plan backlinks.
