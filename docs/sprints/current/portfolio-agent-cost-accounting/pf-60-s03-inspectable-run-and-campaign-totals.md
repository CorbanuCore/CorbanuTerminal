---
sprint_id: "PF-60-S03"
title: "Inspectable run and campaign totals"
status: draft
plan_file: "docs/plans/active/portfolio-agent-cost-accounting.md"
plan_feature: "PF-60"
execution_order: 3
owner: "Jim Ricketts (proposed)"
parallel_lane: "UNALLOCATED"
write_scope: "UNALLOCATED"
integration_gate: "UNALLOCATED"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-60-S02"
created: 2026-09-09
updated: 2026-09-09
---

# PF-60-S03 — Inspectable run and campaign totals

## Execution mandate

- Deliver: codex-rs/tui/src/chatwidget/usage.rs; The user can explain each displayed total using constituent requests without inspecting storage.
- Excludes: Changing prices, rebilling historical customers, collecting prompts, restoring legacy Plan allowances, or silently converting allowance to cash.
- Budget proposal: 2–4 builder-days plus independent testing; not a commitment. Stop and re-slice if the bound is exceeded.

## Plan linkage

- Plan: [Unified agent cost and usage accounting](../../../plans/active/portfolio-agent-cost-accounting.md)
- Feature: `PF-60`; acceptance: The user can explain each displayed total using constituent requests without inspecting storage.
- Upstream/allocation: [plan record](../../../plans/active/portfolio-agent-cost-accounting.md#native-lifecycle-and-upstream-touch-record).

## Code boundaries

- Existing, read before work: `codex-rs/state/migrations/0041_provider_request_cache_usage.sql`; `codex-rs/app-server/src/request_processors/token_usage_replay.rs`; `codex-rs/tui/src/chatwidget/usage.rs`; `codex-rs/tui/src/token_usage.rs`.
- Planned output: `codex-rs/tui/src/chatwidget/usage.rs`; no edits outside the allocated scope.
- Tests/evidence: `qa/portfolio/agent-cost-accounting/pf-60-s03/` (planned); final literal test/registration scope required before readiness.

## Preconditions

- [ ] Plan active after explicit authority decision; global WIP and occupied lanes reconciled.
- [ ] Dependencies completed and archived; Accepted PF-60-S02 output and its explicit go/no-go; no unresolved decision that changes this mandate.
- [ ] Exact worktree/branch/40-character base match plan; literal scopes and receiving owner are allocated.
- [ ] Inputs, disclosure rights and spend/time limits approved; external writes and live financial actions remain excluded.
- [ ] Resolve exact changed files, native compatibility and nonempty test commands before any code sprint starts.

## Done

- [x] Draft sprint created and linked to one feature; no implementation completed.

## Remaining

- [ ] Render root versus descendant totals, provider/model breakdown, estimate/billed distinctions and freshness.
- [ ] Keep historical sessions inspectable; expose missing-price and unavailable-backend states with user-visible next steps.
- [ ] Add narrow-screen, mixed-provider, no-usage, stale-estimate and current-command regression tests.
- [ ] Record actual outputs, counterexamples, remaining limitations and a concrete next-sprint handoff; stop on changed scope.

## Verification

- [ ] Focused: From codex-rs: just test -p codex-tui usage
- [ ] Integration: from repo root, `python3 docs/plans/check.py; python3 docs/sprints/check.py`; `git diff --check`.
- [ ] Resolve TUI applicability against the plan; record actual-key success, failure/cancel, recovery/resume and final binary evidence for every affected interactive path.
- [ ] Record expected versus actual results and nonzero test counts; no unchecked assumption is converted into a pass.

## Exit evidence

- [ ] Output commit/digest and input provenance recorded; checks linked to that final artifact/tree.
- [ ] Named human/receiving owner accepts the bounded output; needed go/no-go decision is recorded.
- [ ] Handoff includes changed scope, contracts, known gaps and required combined-tree evidence.
- [ ] Done/Remaining ledgers updated honestly; archive accepted record under `docs/sprints/archive/portfolio-agent-cost-accounting/` and update plan backlinks.
