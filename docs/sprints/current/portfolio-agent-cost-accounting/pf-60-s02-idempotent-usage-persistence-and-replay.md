---
sprint_id: "PF-60-S02"
title: "Idempotent usage persistence and replay"
status: in_progress
plan_file: "docs/plans/active/portfolio-agent-cost-accounting.md"
plan_feature: "PF-60"
execution_order: 2
owner: "Astra High accounting native journal"
parallel_lane: "accounting-native-journal"
write_scope: "codex-rs/state/src/runtime.rs, codex-rs/state/src/runtime/accounting.rs, codex-rs/state/src/runtime/accounting_types.rs, codex-rs/state/src/runtime/accounting_tests.rs, qa/portfolio/agent-cost-accounting/pf-60-s02/first-increment.md"
integration_gate: "Codex management reviews the exact test-only native-state journal, serializes its one module declaration, reruns affected state and governance tests on the combined tree, and allocates later production wiring separately; no worker commit/merge/push or live collection."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/accounting-pf60-s01-20260911"
branch: "workstream/accounting-pf60-s01-20260911"
base_commit: "265bf0c3e164e497d172f1a8e5a56bf4cd54ed46"
depends_on: "PF-60-S01"
created: 2026-09-09
updated: 2026-09-11
---

# PF-60-S02 — Idempotent usage persistence and replay

S01 is accepted/archived; the first native-state increment is now allocated. Reuse the
idle accounting worktree/branch after a clean fast-forward; its name preserves
history, not a claim that S01 is still the active sprint. Actual dispatch HEAD
is recorded separately. [Handoff](../../../research/agent-cost-accounting/s02-allocation.md).

## Execution mandate

- First increment: private native-state observation journal compiled only in tests; real disposable SQLite close/reopen and replay, no production migration or collection. Full S02 still owes native wiring and complete golden totals.
- Excludes: Changing prices, rebilling historical customers, collecting prompts, restoring legacy Plan allowances, or silently converting allowance to cash.
- Budget proposal: 2–4 builder-days plus independent testing; not a commitment. Stop and re-slice if the bound is exceeded.

## Plan linkage

- Plan: [Unified agent cost and usage accounting](../../../plans/active/portfolio-agent-cost-accounting.md)
- Feature: `PF-60`; acceptance: Replay each fixture twice and after process restart; persisted and reconstructed totals equal the approved fixture exactly.
- Upstream/allocation: [plan record](../../../plans/active/portfolio-agent-cost-accounting.md#native-lifecycle-and-upstream-touch-record).

## Code boundaries

- Existing, read before work: `codex-rs/state/migrations/0041_provider_request_cache_usage.sql`; `codex-rs/app-server/src/request_processors/token_usage_replay.rs`; `codex-rs/tui/src/chatwidget/usage.rs`; `codex-rs/tui/src/token_usage.rs`.
- Planned output: front-matter paths only; runtime.rs gets one test-only module declaration, delegated serially by the manager. No concurrent state/schema writer.
- Tests/evidence: only `qa/portfolio/agent-cost-accounting/pf-60-s02/first-increment.md`. Native ABI, manifests, lockfiles, production migrations and collectors are excluded.

## Preconditions

- [x] Plan active under Travis's standing continuation authority; reservation transferred from completed S01, no fourth lane.
- [x] S01 accepted/archived after defaults approval, combined-tree fixture proof and independently reviewed technical handoff.
- [x] Exact worker/branch/base and literal five-file scope match plan; clean fast-forward and actual launch HEAD recorded at dispatch.
- [x] Travis approved the default policy; synthetic/local tests only, no billing/live collection. Bound this allocation to one worker, at most 500 non-test lines, one independent review plus scoped corrections; no exhausted allowance reset.
- [x] Five files, native-state compatibility, test registration and nonempty intended commands resolved in the reviewed handoff; actual test passes remain pending.

## Done

- [x] Draft sprint created and linked to one feature; no implementation completed.

## Remaining

- [ ] Implement only the linked handoff's test-only begin_attempt/append_observation/read_observations journal; preserve exact identity, presence, conflicts and transactional replay.
- [ ] Add genuine on-disk close/reopen, duplicate/reordered revision, rollback, concurrent writer, unknown/zero and partial Anthropic regressions.
- [ ] After reviewing the first increment, manager allocates production migration/dispatch/presence/price/retention wiring and complete S02 golden tests separately; do not broaden this worker's scope.
- [ ] Record actual outputs, counterexamples, remaining limitations and a concrete next-sprint handoff; stop on changed scope.

## Verification

- [ ] First increment: from codex-rs, `CARGO_NET_OFFLINE=true just test -p codex-state runtime::accounting::tests`, then full affected crate `CARGO_NET_OFFLINE=true just test -p codex-state`; normal-build check `cargo check --offline --locked -p codex-state --lib`.
- [ ] Integration: from repo root, `python3 docs/plans/check.py; python3 docs/sprints/check.py`; `git diff --check`.
- [ ] Resolve TUI applicability against the plan; record actual-key success, failure/cancel, recovery/resume and final binary evidence for every affected interactive path.
- [ ] Record expected versus actual results and nonzero test counts; no unchecked assumption is converted into a pass.

## Exit evidence

- [ ] Output commit/digest and input provenance recorded; checks linked to that final artifact/tree.
- [ ] Named human/receiving owner accepts the bounded output; needed go/no-go decision is recorded.
- [ ] Handoff includes changed scope, contracts, known gaps and required combined-tree evidence.
- [ ] Done/Remaining ledgers updated honestly; archive accepted record under `docs/sprints/archive/portfolio-agent-cost-accounting/` and update plan backlinks.
