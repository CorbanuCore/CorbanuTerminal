---
sprint_id: "PF-60-S02"
title: "Idempotent usage persistence and replay"
status: in_progress
plan_file: "docs/plans/active/portfolio-agent-cost-accounting.md"
plan_feature: "PF-60"
execution_order: 2
owner: "Codex accounting native-ownership lane"
parallel_lane: "accounting-native-ownership"
write_scope: "codex-rs/state/src/runtime/accounting.rs, codex-rs/state/src/runtime/accounting_native.rs, codex-rs/state/src/runtime/accounting_retention_atomic.rs, codex-rs/state/src/runtime/threads.rs, codex-rs/state/src/runtime/accounting_native_tests.rs, qa/portfolio/agent-cost-accounting/pf-60-s02/native-ownership-increment.md"
integration_gate: "Future six-path native ownership/deletion bridge only after parent allocation integration and dispatch; manager checks disjoint sole accounting threads.rs ownership. Target1200total/500non-test, estimate993/373; report overage before expansion. One new code review plus necessary correction, then combined receiving proof. Preserve A/B/C1/C2 and public deletion/no-schema behavior; cfg(test) accounting remains OFF normally, cross-database limits and90..365late-import gap retained. No partial API, schema, live actions or worker commits/pushes."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/accounting-pf60-s01-20260911"
branch: "workstream/accounting-pf60-s01-20260911"
base_commit: "d5608c58d91a75396fea78b447477c44091d4625"
depends_on: "PF-60-S01"
created: 2026-09-09
updated: 2026-09-12
---

# PF-60-S02 — Idempotent usage persistence and replay

S01 is archived; journal/quotation/storage/contributions, compact values and A/B/C1/C2 are integrated. Accepted scopes and receipts stay frozen. The [native ownership allocation](../../../research/agent-cost-accounting/native-ownership-allocation.md) owns the next mandate; [C2 allocation](../../../research/agent-cost-accounting/retention-coupled-next.md) is accepted history, not repeated work. The [retention handoff](../../../research/agent-cost-accounting/retention-design-handoff.md) retains approved daily expiry. This pass edits allocation documents only; source waits for parent integration and dispatch.

## Execution mandate

- Next: same-S02 native ownership/admission and coupled native deletion under the existing accounting cfg(test) gate. Full S02 still owes production integration,90..365-day late imports and complete native goldens; S03 stays draft.
- Excludes: Changing prices, rebilling historical customers, collecting prompts, restoring legacy Plan allowances, or silently converting allowance to cash.
- Approved target1200total/500non-test including receipt, estimate993/373; coherent native/accounting failure/concurrency proof justifies above800, not inherited C2 exception. Report measured overage to integrator before expanding; do not compress proof. One new code review plus necessary correction, no prior history reset.

## Plan linkage

- Plan: [Unified agent cost and usage accounting](../../../plans/active/portfolio-agent-cost-accounting.md)
- Feature: `PF-60`; acceptance: Replay each fixture twice and after process restart; persisted and reconstructed totals equal the approved fixture exactly.
- Upstream/allocation: [plan record](../../../plans/active/portfolio-agent-cost-accounting.md#native-lifecycle-and-upstream-touch-record).

## Code boundaries

- Existing, read before work: `codex-rs/state/migrations/0041_provider_request_cache_usage.sql`; `codex-rs/app-server/src/request_processors/token_usage_replay.rs`; `codex-rs/tui/src/chatwidget/usage.rs`; `codex-rs/tui/src/token_usage.rs`.
- Output: front-matter paths are exactly six future native bridge files, not permission to edit source in this documents-only pass. Manager verifies sole accounting threads.rs ownership under disjoint scope. No new auth principal/registry/partial transfer API; capture one time.
- Tests/evidence: preserve A/B/C1/C2 assertions and receipts. Public delete signatures/counts/missing-row semantics and normal no-schema behavior stay unchanged, with no blanket new failure/activation checks. Accounting hook remains cfg(test); separate logs/memory/goals stores are not covered by main-state atomicity. No schema/production installation, dependencies or collectors.

## Preconditions

- [x] Plan active under Travis's standing continuation authority; reservation transferred from completed S01, no fourth lane.
- [x] S01 accepted/archived after defaults approval, combined-tree fixture proof and independently reviewed technical handoff.
- [x] Parent reports clean worker fast-forwarded to d5608c58d91a75396fea78b447477c44091d4625; exact worker/branch/base and six future paths match plan/allocation. Historical C2 zero-edit scope correction consumed no passes.
- [x] Travis approved defaults; manager approved this native bridge and1200/500 target. Synthetic/local only, one worker, one new independent review plus necessary correction; prior failures/review usage retained, no billing/live authority.
- [ ] Parent checks/integrates these allocation documents and verifies disjoint ownership before source dispatch.
- [x] Parent inspected DayTotals/Decimal visibility and amount/rate parsing distinction; B1 is now reviewed and tested, receipt below.

## Done

- [x] C2 accepted: exact six-file b7a3466de integrated at d5608c58d91a75396fea78b447477c44091d4625. First independent Astra High c2-atomic-review.json exited0 clean; parent combined `just test -p codex-state -p codex-tasknode-session` passed346/346,0skipped13.556s, run06b633c7-51ae-4dd8-bf96-81a2665c2871. No LEAK marker in final summary; existing11 fixture dead-code warnings remain. [Exact receipt](../../../../qa/portfolio/agent-cost-accounting/pf-60-s02/atomic-retention-increment.md); c2-size-disposition.md accepts1519total/190non-test for that candidate, not800compliance. Original failures/review usage and seven-line interim C1 assertion removal/replacement mapping remain preserved.
- [x] C1 consumers accepted with recorded1121-line candidate exception, one clean Astra review and combined335passed/2leaky/0skipped9.285s at8cdb6dcf; integrated5406e3506 then reconciledafe535c06 with identical Rust tree. [Receipt](../../../../qa/portfolio/agent-cost-accounting/pf-60-s02/retention-consumers-increment.md). No repeated review or new product decision required.

- [x] Pure reduction B ac5a22d67 reviewed clean and integrated at7514be8ec; combined330 state/TaskNode tests pass9.928s, nextest178ce26c-71f5-4405-95b2-817ebdaa1a4b. [Receipt](../../../../qa/portfolio/agent-cost-accounting/pf-60-s02/retention-reduction-increment.md). Four paths507lines/131non-test; B code allowance used, unused correction retained. No mutation/runtime claim.
- [x] Read-only input A8eca814f8 reviewed clean and integrated atfd46c5897; combined325 state/TaskNode tests pass9.100s. No reduction/mutation claim. [Receipt](../../../../qa/portfolio/agent-cost-accounting/pf-60-s02/retention-plan-increment.md). Original904-line draft preserved; accepted A775lines.

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

- [ ] Implement the [native bridge](../../../research/agent-cost-accounting/native-ownership-allocation.md) after dispatch: native owner equality/existence admission and complete C2 deletion on the same held main-state transaction, preserving absent-schema behavior and cross-database limits.
- [ ] New native proof: full ten-accounting/three-native-table snapshots; owner/missing/replay/archived cases; multi-owner and post-accounting native DELETE/commit failures, two reopens/retry; both actual append/delete contention orders and read snapshots; separate-store cleanup failures, time/activation/late-import no-writes. No repeated C2 matrix or deleted proof.
- [ ] Production migration/dispatch/presence/original-price wiring, permanent deletion-fence qualification and complete native S02 goldens remain outside this bridge.90..365-day compact-only late import is explicitly unqualified, current guard unchanged, not waived; no new prerequisite chain. S03 stays draft.
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
