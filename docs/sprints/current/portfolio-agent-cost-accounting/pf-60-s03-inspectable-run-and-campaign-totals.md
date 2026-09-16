---
sprint_id: "PF-60-S03"
title: "Inspectable run and campaign totals"
status: draft
plan_file: "docs/plans/active/portfolio-agent-cost-accounting.md"
plan_feature: "PF-60"
execution_order: 3
owner: "Astra High accounting inspection lane; Fable receives and integrates; Travis remains product acceptance authority"
parallel_lane: "accounting-pf60-s03-inspection"
write_scope: "codex-rs/state/src/runtime/accounting_store.rs, codex-rs/state/src/runtime/accounting_estimates.rs, codex-rs/state/src/runtime/accounting_lifecycle.rs, codex-rs/state/src/runtime/accounting_pricing.rs, codex-rs/state/src/runtime/accounting_store_tests.rs, codex-rs/state/tests/accounting_store.rs, codex-rs/tui/src/chatwidget/usage.rs, codex-rs/tui/src/chatwidget/tokens.rs, codex-rs/tui/src/chatwidget/tokens_tests.rs, codex-rs/tui/src/chatwidget.rs, codex-rs/tui/src/chatwidget/constructor.rs, codex-rs/tui/src/chatwidget/slash_dispatch.rs, codex-rs/tui/src/chatwidget/tests/usage.rs, codex-rs/tui/src/chatwidget/tests/slash_commands.rs, codex-rs/tui/src/app_event.rs, codex-rs/tui/src/app/event_dispatch.rs, codex-rs/tui/src/app/tests.rs, codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__usage_command_menu.snap, codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__usage_command_menu_without_resets.snap, codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__usage_command_menu_before_reset_refresh.snap, codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__usage_command_with_invalid_view_reports_usage.snap"
integration_gate: "Fable receives onto integrate/management-workstreams-20260911 after S02 archival and an explicit PF-83 overlap handoff on the three shared TUI paths; audits literal scope and size against the frozen allocation, takes one independent Opus 5.0 High review plus necessary scoped correction, and reruns the combined state, TUI and accounting gates on the receiving tree. True-TUI proof and independent code-blind design and execution are owed by this sprint and are not deferred silently."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/acct-inspect-20260915"
branch: "bootstrap/acct-inspect-20260915"
base_commit: "124c3d68d14ddef34c1bffac157cbcd8e5074158"
depends_on: "PF-60-S02"
created: 2026-09-09
updated: 2026-09-11
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
- [x] Travis requested user-selected ranges and grouping intervals; recorded below without starting S03 or extending retention.
- [x] 2026-09-15 `acct-inspect-impl-01` verified its frozen brief and launch checkout, then stopped before code: this sprint remains draft with a placeholder base and the active plan omits its inspection coordinates. [Stopped return and all 34 case dispositions](../../../../qa/portfolio/agent-cost-accounting/pf-60-s03/acct-inspect-impl-01-return.md); no implementation or functional acceptance claimed.

## Remaining
- [ ] Fable reconciles S03-A allocation and exact worktree/branch/base in the active plan and this sprint, records executable status, reruns both governance checkers and redispatches; the active plan is outside `acct-inspect-impl-01` writable scope.
- [ ] Three TUI paths in the write scope overlap PF-83-S01 and were **released to this lane on 2026-09-15** because no PF-83 action held them: codex-rs/tui/src/app/event_dispatch.rs, codex-rs/tui/src/app_event.rs, codex-rs/tui/src/app/tests.rs. The release is recorded in that sprint. Re-check before launch rather than relying on this line, since PF-83 may re-reserve.
- [ ] **Inherited from PF-60-S02 at its archival, transferred rather than discharged:** Corbanu plan gateway economics, startup prewarm, auxiliary collection, legacy evidence acquisition and complete application coverage all remain unqualified, and anonymous 365-day fencing is not permanent. Six leaky markers remain across the accounting suite (two in `accounting::policy_tests`, three in `accounting_anthropic`, one in `accounting_chat_native_mismatched_endpoint`); the three diagnosed in `acct-cleanup-04` showed no concrete fixture defect and were deliberately left rather than given speculative cleanup. Also inherited: the one disclosed review P3, that the provider-publication regression pins ordering against contributor emission rather than against the state lock.

- [ ] Render root versus descendant totals, provider/model breakdown, estimate/billed distinctions and freshness.
- [ ] Add custom start/end filtering and selectable grouping intervals (proposed hour/day/week/month). Show requested range, effective coverage, timezone and bucket boundaries explicitly; use half-open start-inclusive/end-exclusive queries and reject reversed/invalid ranges.
- [ ] Offer sub-day precision only where retained raw detail supports it. Older UTC-day aggregates cannot supply hourly values or arbitrary partial-day amounts: show unsupported precision/partial coverage and offer whole-UTC-day bounds, never silently round, prorate, fabricate zeros or extend detail retention.
- [ ] Show oldest retained aggregate day and separate 90-day drill-down cutoff. For mixed raw/compact ranges preserve exact sums, unknown populations, estimate labels and no double count. Manager freezes timezone/week/month alignment and exact query allocation before readiness.
- [ ] Keep historical sessions inspectable; expose missing-price and unavailable-backend states with user-visible next steps.
- [ ] Add narrow-screen, mixed-provider, no-usage, stale-estimate and current-command regression tests.
- [ ] Test exact boundaries, partial days, timezone/DST transitions, empty versus unavailable ranges, deleted history, mixed retention coverage and interval switching through supported user controls; no hidden storage repair.
- [ ] Record actual outputs, counterexamples, remaining limitations and a concrete next-sprint handoff; stop on changed scope.

## Verification
- [ ] **Inherited from PF-60-S02:** [isolated code-blind functional execution](../../../../qa/code-blind-functional/isolated-execution.md) with real keys and negative controls, independent evidence review of that execution, and true-TUI qualification. S02 accepted an internal-stage N/A for its own increments only, which deferred these here; it did not discharge them. This sprint is the first user-facing unit, so they fall due.

- [ ] Focused: From codex-rs: just test -p codex-tui usage
- [ ] Integration: from repo root, `python3 docs/plans/check.py; python3 docs/sprints/check.py`; `git diff --check`.
- [ ] Resolve TUI applicability against the plan; record actual-key success, failure/cancel, recovery/resume and final binary evidence for every affected interactive path.
- [ ] Record expected versus actual results and nonzero test counts; no unchecked assumption is converted into a pass.

## Exit evidence

- [ ] Output commit/digest and input provenance recorded; checks linked to that final artifact/tree.
- [ ] Named human/receiving owner accepts the bounded output; needed go/no-go decision is recorded.
- [ ] Handoff includes changed scope, contracts, known gaps and required combined-tree evidence.
- [ ] Done/Remaining ledgers updated honestly; archive accepted record under `docs/sprints/archive/portfolio-agent-cost-accounting/` and update plan backlinks.
