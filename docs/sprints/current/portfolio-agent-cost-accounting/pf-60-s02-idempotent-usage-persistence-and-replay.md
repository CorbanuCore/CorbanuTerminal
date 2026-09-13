---
sprint_id: "PF-60-S02"
title: "Idempotent usage persistence and replay"
status: blocked
plan_file: "docs/plans/active/portfolio-agent-cost-accounting.md"
plan_feature: "PF-60"
execution_order: 2
owner: "Codex accounting contract-golden lane"
parallel_lane: "accounting-contract-goldens"
write_scope: "codex-rs/state/tests/accounting_contract_golden.rs, qa/portfolio/agent-cost-accounting/pf-60-s02/original-contract-native-golden.md"
integration_gate: "Compact import reviewed/received81d0f90e7; parent595shared/100Core/20focused/6external and format/Clippy/check PASS, old failures retained. Next exact two-path original-contract-native-golden-allocation.md target800/150 STOP900/200; one material review+necessary correction, manager receiving. S02 open, S03 dependent, collection OFF."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/accounting-contract-goldens-20260913"
branch: "workstream/accounting-contract-goldens-20260913"
base_commit: "81d0f90e77c1e9217a16e70fef1019ff9aa13753"
depends_on: "PF-60-S01"
created: 2026-09-09
updated: 2026-09-12
---

# PF-60-S02 — Idempotent usage persistence and replay

**User-requested management pause, September 13.** Goldens/combined proof accepted; worker closed. [Checkpoint and gates](../../../plans/management-pause-2026-09-13.md). No answer requested; continuation mandates below suspended. S03 not started.
17:22UTC: parent accepted review01's dropped role retry/timeout overrides P2 as
in-scope; first correction queued after the matched baseline run. Same20 paths
and existing allowance; native child override/mismatched-route proof required.

S01 is archived; journal/quotation/storage/contributions, compact values and A/B/C1/C2/native are integrated. Accepted scopes and receipts stay frozen. The [production store allocation](../../../research/agent-cost-accounting/production-store-allocation.md) is completed history; [native allocation](../../../research/agent-cost-accounting/native-ownership-allocation.md) and [C2 allocation](../../../research/agent-cost-accounting/retention-coupled-next.md) are accepted history, not repeated work. The [retention handoff](../../../research/agent-cost-accounting/retention-design-handoff.md) retains approved daily expiry. Current source authority is the exact two-path golden allocation and manager dispatch.

## Execution mandate

- Next: [Original S01 native golden](../../../research/agent-cost-accounting/original-contract-native-golden-allocation.md): three literal raw/reopen/compact parity tests on the normal library; accepted runtime/tests unchanged.
- Excludes: Changing prices, rebilling historical customers, collecting prompts, restoring legacy Plan allowances, or silently converting allowance to cash.
- Approved target800total/150non-test, STOP900/200; estimate700/110. Bounded contingency covers complete original-fixture parity/receipt, not inherited allowance; one material review plus necessary correction, prior usage retained.

## Plan linkage

- Plan: [Unified agent cost and usage accounting](../../../plans/active/portfolio-agent-cost-accounting.md)
- Feature: `PF-60`; acceptance: Replay each fixture twice and after process restart; persisted and reconstructed totals equal the approved fixture exactly.
- Upstream/allocation: [plan record](../../../plans/active/portfolio-agent-cost-accounting.md#native-lifecycle-and-upstream-touch-record).

## Code boundaries

- Existing, read before work: `codex-rs/state/migrations/0041_provider_request_cache_usage.sql`; `codex-rs/app-server/src/request_processors/token_usage_replay.rs`; `codex-rs/tui/src/chatwidget/usage.rs`; `codex-rs/tui/src/token_usage.rs`.
- Output: exact two test/receipt paths in front matter; no accepted-file/runtime/manifest/lock/Core writes. No overlap with PF27 or Slack. Bounds/review allowance are in the original-contract golden allocation.
- Tests/evidence: preserve A/B/C1/C2/native assertions, failures and receipts. Public delete signatures/counts/missing-row semantics and normal no-schema behavior stay unchanged, no blanket activation/failure checks. Normal-library facade gates fixture constructors cfg(test); default installs/collects nothing, installed-store deletion remains atomic even disabled. Separate logs/memory/goals stores remain outside main-state atomicity. No dependencies or real collectors.

## Preconditions

- [x] Plan active under Travis's standing continuation authority; reservation transferred from completed S01, no fourth lane.
- [x] S01 accepted/archived after defaults approval, combined-tree fixture proof and independently reviewed technical handoff.
- [x] Worker clean fast-forward to8725e1ff755a5fa974058f465f3553b3f9b884eb verified; exact worker/branch/base and17 future paths match plan/allocation. Historical C2 zero-edit scope correction consumed no passes.
- [x] Travis approved defaults; manager approved separate ledger/sequence, default no-install/collection OFF, typed facade and1900/950 target. One existing worker; one NEW code review plus necessary correction and one allocation review if needed, prior failures/review usage retained. No new product/billing/live authority.
- [x] Historical native allocation: parent read three documents; native-ownership-allocation-review exited0 clean. Integrated f13e9740a atce8a81980; checkers3/115/126, disjoint sole accounting threads.rs ownership. That six-path dispatch is completed, not current source authorization.
- [x] Parent read all three allocation documents; first production-store-allocation-review exit0 clean. Docs12032acc7 integrated atf32e99f08a6218539c7d2b9d82f38ca038cb3e2f; governance3/115/126pass, exact17-path scope disjoint and no accounting migration/ledger collision. Source dispatch now authorized; parent records actual launch HEAD separately.
- [x] Parent inspected DayTotals/Decimal visibility and amount/rate parsing distinction; B1 is now reviewed and tested, receipt below.

## Done
- [x] Compact import281ee4ec1 independently reviewed clean02, received81d0f90e7; parent595shared/100Core/20focused/6external plus format/Clippy/check pass. [Receiving proof](../../../../qa/portfolio/agent-cost-accounting/pf-60-s02/compact-late-import-receiving.md) preserves failures, original review and metadata addendum; no live/full-S02 claim.
- [x] Ten-path Core policy correction b320ef722 independently reviewed clean0.94 and received2cb69e429; exact combinedeb5855959 passes100focused18.334s/575shared31.075s, zero execution skips, Clippy/check. [Receiving proof](../../../../qa/initiative-control/status-display/combined-native-20260913.md); old failures/internal-only N/A retained.
- [x] Normal-library store6b2dbcab5 integrated at9518184ac after clean independent review03 (helper85655). Exact17 paths2057total/829non-test; second correction samples production deletion time after writer-lock acquisition, with real ordering regression. Receiving `just test -p codex-state -p codex-tasknode-session`:367passed,1leaky,0skipped,16.922s. [Receipt](../../../../qa/portfolio/agent-cost-accounting/pf-60-s02/production-store-increment.md) preserves287 worker tests, original failures and review history. Optional versioned installation, typed facade and atomic admission/observations accepted; collection remains OFF.
- [x] Native bridge accepted: exact six-file0eb98e8d3 integrated at8725e1ff755a5fa974058f465f3553b3f9b884eb. First independent Astra High native-ownership-code-review.json/txt, helper70866, exit0 clean. Parent receiving exec46485 combined `just test -p codex-state -p codex-tasknode-session` exit0:354/354passed,0skipped14.997s, nextest ef047f72-80c9-4700-863d-1f1e253b5d75; no final-summary LEAK,21fixture dead-code warnings(9duplicates). Parent governance3/115/126exit0. [Receipt](../../../../qa/portfolio/agent-cost-accounting/pf-60-s02/native-ownership-increment.md) retains950total/227non-test, worker failures/results and prior review usage; receiving evidence is parent-attributed, not rerun here.
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
- [ ] Execute original six-attempt S01 fixture through native ownership, normal storage/replay/two reopens and compact original-bundle parity; return exact two-path frozen candidate for review/receiving.
- [ ] Preserve accepted normal-library store tests and all original failed evidence as the unchanged dependency regression gate; do not re-author its installation/rollback matrix.
- [ ] Actual dispatch/presence/original-price path is qualified for direct Anthropic only; Responses HTTP/WS, Chat/Corbanu and auxiliary routes remain open. Legacy original-evidence acquisition is unqualified. Approved anonymous365-day fencing is not permanent owner revocation: recreated owner/new IDs and physical/cross-DB erasure remain limitations, not a new registry or human blocker. S03 stays draft.
- [ ] Record actual outputs, counterexamples, remaining limitations and a concrete next-sprint handoff; stop on changed scope.

## Verification
- [ ] Apply [independent isolated execution](../../../../qa/code-blind-functional/isolated-execution.md) to affected functional handoff; record schema-2 proof or integrator-accepted internal-only N/A and later gate. Historical tests are not upgraded.
- [x] B1:11 compact-value tests and four existing selectors passed; combined237 state/73 Task Node tests and normal-library check passed atc8d46d709. One non-accounting LEAK marker disclosed in manager handoff; no leak-clean claim.
- [ ] Integration: from repo root, `python3 docs/plans/check.py; python3 docs/sprints/check.py`; `git diff --check`.
- [ ] Resolve TUI applicability against the plan; record actual-key success, failure/cancel, recovery/resume and final binary evidence for every affected interactive path.
- [ ] Record expected versus actual results and nonzero test counts; no unchecked assumption is converted into a pass.

## Exit evidence

- [ ] Output commit/digest and input provenance recorded; checks linked to that final artifact/tree.
- [ ] Named human/receiving owner accepts the bounded output; needed go/no-go decision is recorded.
- [ ] Handoff includes changed scope, contracts, known gaps and required combined-tree evidence.
- [ ] Done/Remaining ledgers updated honestly; archive accepted record under `docs/sprints/archive/portfolio-agent-cost-accounting/` and update plan backlinks.
