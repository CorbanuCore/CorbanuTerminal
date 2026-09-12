---
sprint_id: "PF-60-S02"
title: "Idempotent usage persistence and replay"
status: in_progress
plan_file: "docs/plans/active/portfolio-agent-cost-accounting.md"
plan_feature: "PF-60"
execution_order: 2
owner: "Codex accounting retention-plan lane"
parallel_lane: "accounting-retention-plan"
write_scope: "codex-rs/state/src/runtime/accounting_lifecycle.rs, codex-rs/state/src/runtime/accounting_retention_plan.rs, codex-rs/state/src/runtime/accounting_retention_plan_tests.rs, codex-rs/state/src/runtime/accounting_retention_test_support.rs, qa/portfolio/agent-cost-accounting/pf-60-s02/retention-plan-increment.md"
integration_gate: "Codex management audits exact five-file read-only retention scope, no-write proof and original-price preservation, reviews once plus substantive corrections and reruns state/governance on receiving. No production migration, retention mutation, live collection or worker commit/push."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/accounting-pf60-s01-20260911"
branch: "workstream/accounting-pf60-s01-20260911"
base_commit: "87e31f521672e627e6230d48fc16a4cfaa7ff44c"
depends_on: "PF-60-S01"
created: 2026-09-09
updated: 2026-09-11
---

# PF-60-S02 — Idempotent usage persistence and replay

S01 is archived; journal/quotation/storage/contributions/deletion are integrated.
Exact compact values are reviewed/integrated; the worker is closed and previous
literal scope frozen. Latest reader is now in canonical receiving; the retention preparation allocation below supersedes its frozen scope. The
[retention handoff](../../../research/agent-cost-accounting/retention-design-handoff.md)
records approved daily expiry; [latest-quote allocation](../../../research/agent-cost-accounting/latest-quote-allocation.md) is historical; [retention plan](../../../research/agent-cost-accounting/retention-plan-next.md) owns the current mandate.

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
- Output: front-matter paths are the five-file retention preparation allocation; reviewed B1 paths remain frozen. Parent must separately allocate any retention mutation.
- Tests/evidence: accepted compact-values receipt; preserve it. Native ABI, dependencies, production migrations and collectors remain excluded.

## Preconditions

- [x] Plan active under Travis's standing continuation authority; reservation transferred from completed S01, no fourth lane.
- [x] S01 accepted/archived after defaults approval, combined-tree fixture proof and independently reviewed technical handoff.
- [x] Exact worker/branch/base and five-file retention scope match plan; parent must clean-fast-forward and record launch HEAD at dispatch.
- [x] Travis approved the default policy; synthetic/local tests only, no billing/live collection. Bound this allocation to one worker, at most 500 non-test lines, one independent review plus scoped corrections; no exhausted allowance reset.
- [x] Parent inspected DayTotals/Decimal visibility and amount/rate parsing distinction; B1 is now reviewed and tested, receipt below.

## Done

- [x] Canonical receiving fast-forwarded to 87e31f521672e627e6230d48fc16a4cfaa7ff44c with reviewed native prerequisites; prior overlapping Facilities edits preserved privately and reconciled. Integrator authorizes two additional scoped review passes (code plus substantive correction only), preserving prior history.

- [x] Latest quote8b6d629b6 reviewed clean and combined staging04ba6b8b7 passes242state/80TaskNode tests; reader only, no retention mutation. [Evidence](../../../../qa/initiative-control/native-staging-2026-09-12.md). Canonical receiving transfer/publication pending.

- [x] Travis approved conservative UTC-day-start+365-day aggregate expiry; no reapproval needed. Detail90/replay365 policies unchanged; S03 requested range/interval filters remain draft.
- [x] First test-only journal and native SQLite regressions reviewed clean and integrated at c33d47f6c (worker 3f39d7a65); 202 state tests pass on combined tree. [Receipt](../../../../qa/portfolio/agent-cost-accounting/pf-60-s02/first-increment.md).
- [x] Real on-disk close/reopen, duplicate/reordered revision, rollback, concurrent writer, unknown/zero and partial Anthropic regressions pass; no process-kill or production claim.
- [x] Exact quotation e8ffdad4e reviewed clean and integrated at 486d2fb94; 214 state tests pass there. Prospective snapshots, exact arithmetic/half-even, unknowns and journal replay covered. [Receipt](../../../../qa/portfolio/agent-cost-accounting/pf-60-s02/exact-pricing-increment.md).
- [x] Immutable storage d898fbac0 reviewed clean and integrated at56295f668; all220 state tests pass there. Atomic snapshot/binding/evidence versions, corruption/reopen proof. [Receipt](../../../../qa/portfolio/agent-cost-accounting/pf-60-s02/estimate-storage-increment.md).

- [x] Contributions/deletion b08fff66d reviewed clean and integrated at3898eaa65;226 state tests pass there. Exact current totals, atomic all-version deletion/shared snapshot retention, deterministic serialization orders and two disk reopens; no simultaneous lock-contention/pruning/native owner claim. [Receipt](../../../../qa/portfolio/agent-cost-accounting/pf-60-s02/contribution-deletion-increment.md).

- [x] Exact compact-value codec c8d46d709 independently reviewed clean and integrated;237 state tests pass on receiving tree. Canonical u128/24-place USD, all seven metric populations, checked composition and malformed/overflow rejection; no DB mutation. [Receipt](../../../../qa/portfolio/agent-cost-accounting/pf-60-s02/compact-values-increment.md).

## Remaining

- [ ] Implement the [read-only retention allocation](../../../research/agent-cost-accounting/retention-plan-next.md) in the five front-matter paths: whole-store plan, strict validation, exact arithmetic, no writes, full snapshots and reopen tests; return for manager review. Coupled mutation/read/delete/admission follows separately; no S03 activation.
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
