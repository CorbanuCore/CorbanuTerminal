---
sprint_id: "PF-60-S02"
title: "Idempotent usage persistence and replay"
status: in_progress
plan_file: "docs/plans/active/portfolio-agent-cost-accounting.md"
plan_feature: "PF-60"
execution_order: 2
owner: "Astra High accounting estimate storage"
parallel_lane: "accounting-estimate-storage"
write_scope: "codex-rs/state/src/runtime/accounting_pricing.rs, codex-rs/state/src/runtime/accounting_types.rs, codex-rs/state/src/runtime/accounting_estimates.rs, codex-rs/state/src/runtime/accounting_estimates_tests.rs, qa/portfolio/agent-cost-accounting/pf-60-s02/estimate-storage-increment.md"
integration_gate: "Codex management reviews test-only immutable estimate storage, serializes exact derives/registration, reruns state and governance tests on the combined tree, and allocates retention/deletion and production wiring later; no worker commit/merge/push or live collection."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/accounting-pf60-s01-20260911"
branch: "workstream/accounting-pf60-s01-20260911"
base_commit: "486d2fb9481c01f7d73a6f8d9992c6e1b96359dd"
depends_on: "PF-60-S01"
created: 2026-09-09
updated: 2026-09-11
---

# PF-60-S02 — Idempotent usage persistence and replay

S01 is archived; journal and quotation are reviewed/integrated. Immutable estimate
storage is next. Reuse the idle worker after clean fast-forward; its name
preserves history, not S01 ownership. Record actual dispatch HEAD separately.
[Current allocation](../../../research/agent-cost-accounting/estimate-storage-allocation.md).

## Execution mandate

- Next increment: test-only immutable snapshot/estimate storage over native journal, transactional conflicts and exact close/reopen. Full S02 still owes retention/deletion, native wiring and complete golden totals.
- Excludes: Changing prices, rebilling historical customers, collecting prompts, restoring legacy Plan allowances, or silently converting allowance to cash.
- Budget proposal: 2–4 builder-days plus independent testing; not a commitment. Stop and re-slice if the bound is exceeded.

## Plan linkage

- Plan: [Unified agent cost and usage accounting](../../../plans/active/portfolio-agent-cost-accounting.md)
- Feature: `PF-60`; acceptance: Replay each fixture twice and after process restart; persisted and reconstructed totals equal the approved fixture exactly.
- Upstream/allocation: [plan record](../../../plans/active/portfolio-agent-cost-accounting.md#native-lifecycle-and-upstream-touch-record).

## Code boundaries

- Existing, read before work: `codex-rs/state/migrations/0041_provider_request_cache_usage.sql`; `codex-rs/app-server/src/request_processors/token_usage_replay.rs`; `codex-rs/tui/src/chatwidget/usage.rs`; `codex-rs/tui/src/token_usage.rs`.
- Planned output: front-matter paths only; pricing.rs exact Serialize/child registration and Usage Serialize only in types. New fixture DDL in child, no existing journal SQL/type behavior changes.
- Tests/evidence: new estimate-storage receipt; preserve earlier evidence. Native ABI, dependencies, production migrations and collectors excluded.

## Preconditions

- [x] Plan active under Travis's standing continuation authority; reservation transferred from completed S01, no fourth lane.
- [x] S01 accepted/archived after defaults approval, combined-tree fixture proof and independently reviewed technical handoff.
- [x] Exact worker/branch/base and five-file storage scope match plan; parent must clean-fast-forward and record launch HEAD at dispatch.
- [x] Travis approved the default policy; synthetic/local tests only, no billing/live collection. Bound this allocation to one worker, at most 500 non-test lines, one independent review plus scoped corrections; no exhausted allowance reset.
- [x] Parent inspected native-state/pricing visibility and serialization seams; current allocation linked above. New storage test passes remain pending.

## Done

- [x] First test-only journal and native SQLite regressions reviewed clean and integrated at c33d47f6c (worker 3f39d7a65); 202 state tests pass on combined tree. [Receipt](../../../../qa/portfolio/agent-cost-accounting/pf-60-s02/first-increment.md).
- [x] Real on-disk close/reopen, duplicate/reordered revision, rollback, concurrent writer, unknown/zero and partial Anthropic regressions pass; no process-kill or production claim.
- [x] Exact quotation e8ffdad4e reviewed clean and integrated at 486d2fb94; 214 state tests pass there. Prospective snapshots, exact arithmetic/half-even, unknowns and journal replay covered. [Receipt](../../../../qa/portfolio/agent-cost-accounting/pf-60-s02/exact-pricing-increment.md).

## Remaining

- [ ] Implement current immutable snapshot/binding/evidence-version storage, atomic rollback, integrity-checked reads and real reopen tests; return actual evidence for independent review.
- [ ] After reviewing the first increment, manager allocates production migration/dispatch/presence/price/retention wiring and complete S02 golden tests separately; do not broaden this worker's scope.
- [ ] Record actual outputs, counterexamples, remaining limitations and a concrete next-sprint handoff; stop on changed scope.

## Verification

- [ ] Current increment: storage::tests selector under runtime::accounting::pricing, pricing/journal selectors, full state and normal-library check through pinned offline commands in allocation.
- [ ] Integration: from repo root, `python3 docs/plans/check.py; python3 docs/sprints/check.py`; `git diff --check`.
- [ ] Resolve TUI applicability against the plan; record actual-key success, failure/cancel, recovery/resume and final binary evidence for every affected interactive path.
- [ ] Record expected versus actual results and nonzero test counts; no unchecked assumption is converted into a pass.

## Exit evidence

- [ ] Output commit/digest and input provenance recorded; checks linked to that final artifact/tree.
- [ ] Named human/receiving owner accepts the bounded output; needed go/no-go decision is recorded.
- [ ] Handoff includes changed scope, contracts, known gaps and required combined-tree evidence.
- [ ] Done/Remaining ledgers updated honestly; archive accepted record under `docs/sprints/archive/portfolio-agent-cost-accounting/` and update plan backlinks.
