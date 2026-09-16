---
sprint_id: "PF-60-S03"
title: "Inspectable run and campaign totals"
status: in_progress
plan_file: "docs/plans/active/portfolio-agent-cost-accounting.md"
plan_feature: "PF-60"
execution_order: 3
owner: "Astra High accounting inspection lane; Fable receives and integrates; Travis remains product acceptance authority"
parallel_lane: "accounting-pf60-s03-inspection"
write_scope: "codex-rs/core/src/config/mod.rs, codex-rs/core/src/accounting.rs, codex-rs/state/src/runtime/accounting_native.rs, qa/portfolio/agent-cost-accounting/pf-60-s03/, codex-rs/state/src/runtime/accounting_store.rs, codex-rs/state/src/runtime/accounting_estimates.rs, codex-rs/state/src/runtime/accounting_lifecycle.rs, codex-rs/state/src/runtime/accounting_pricing.rs, codex-rs/state/src/runtime/accounting_store_tests.rs, codex-rs/state/tests/accounting_store.rs, codex-rs/tui/src/chatwidget/usage.rs, codex-rs/tui/src/chatwidget/tokens.rs, codex-rs/tui/src/chatwidget/tokens_tests.rs, codex-rs/tui/src/chatwidget.rs, codex-rs/tui/src/chatwidget/constructor.rs, codex-rs/tui/src/chatwidget/slash_dispatch.rs, codex-rs/tui/src/chatwidget/tests/usage.rs, codex-rs/tui/src/chatwidget/tests/slash_commands.rs, codex-rs/tui/src/app_event.rs, codex-rs/tui/src/app/event_dispatch.rs, codex-rs/tui/src/app/tests.rs, codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__usage_command_menu.snap, codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__usage_command_menu_without_resets.snap, codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__usage_command_menu_before_reset_refresh.snap, codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__usage_command_with_invalid_view_reports_usage.snap"
integration_gate: "Fable receives onto integrate/management-workstreams-20260911 after S02 archival and an explicit PF-83 overlap handoff on the three shared TUI paths; audits literal scope and size against the frozen allocation, takes one independent Opus 5.0 High review plus necessary scoped correction, and reruns the combined state, TUI and accounting gates on the receiving tree. True-TUI proof and independent code-blind design and execution are owed by this sprint and are not deferred silently."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/acct-inspect-20260915"
branch: "bootstrap/acct-inspect-20260915"
base_commit: "5105e3ce44d31fd16a35f6cb6900ef6abf6168a8"
depends_on: "PF-60-S02"
created: 2026-09-09
updated: 2026-09-16
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

- [x] Plan active after explicit authority decision; global WIP and occupied lanes reconciled.
- [x] Dependencies completed and archived; Accepted PF-60-S02 output and its explicit go/no-go; no unresolved decision that changes this mandate.
- [x] Exact worktree/branch/40-character base match plan; literal scopes and receiving owner are allocated. `base_commit` names the commit the lane branches from; the exact launch commit is carried in the frozen allocation inputs and must equal the checkout HEAD.
- [ ] Inputs, disclosure rights and spend/time limits approved; external writes and live financial actions remain excluded.
- [ ] Resolve exact changed files, native compatibility and nonempty test commands before any code sprint starts.

## Done

- [x] Draft sprint created and linked to one feature; no implementation completed.
- [x] Travis requested user-selected ranges and grouping intervals; recorded below without starting S03 or extending retention.
- [x] 2026-09-16 The one disclosed review finding is closed rather than carried: a stale contribution now outranks maintenance lag, so a day that is both behind and stale says so. Twenty production lines, its test proven to fail without the change, and the companion test kept deliberately to pin plain lag as a distinct state. Independent review returned clean. Receiving gates rerun on the merged tree.
- [x] 2026-09-16 Read-only recorded-request inspector received at the integration tip: 2,382 source/test lines in the 21 allocated paths, all 34 named tests passing, independent Opus 5.0 High review confirming it is read-only in fact, plus one scoped correction closing its P2. Receiving gates on the merged tree: state+tasknode 407/407, focused TUI 19/19. Full-suite delta [attributed against this base](../../../../qa/portfolio/agent-cost-accounting/pf-60-s03/full-suite-attribution-20260916.md) - zero regressions.
- [x] 2026-09-16 Sprint activated from the verified S02 receiving commit `533a16077`; records reconciled and both governance checkers rerun by Fable, whose fault the stop below was.
- [x] 2026-09-15 `acct-inspect-impl-01` verified its frozen brief and launch checkout, then stopped before code: this sprint remains draft with a placeholder base and the active plan omits its inspection coordinates. [Stopped return and all 34 case dispositions](../../../../qa/portfolio/agent-cost-accounting/pf-60-s03/acct-inspect-impl-01-return.md); no implementation or functional acceptance claimed.

- [x] 2026-09-15 `acct-inspect-impl-02` passed launch preflight, then stopped when required broad formatting changed 74 unallocated paths. All source edits restored; unvalidated draft, scope incident and all 34 unexecuted cases are preserved in the [return](../../../../qa/portfolio/agent-cost-accounting/pf-60-s03/acct-inspect-impl-02-return.md). No implementation or functional gate completed.

- [x] 2026-09-15 `acct-inspect-impl-03` verified the frozen brief/launch coordinates and implemented the scoped read-only UTC-day inspector with collection OFF. The [increment receipt](../../../../qa/portfolio/agent-cost-accounting/pf-60-s03/acct-inspect-impl-03-return.md) records all 34 named tests passing, the 406-test state/TaskNode pass (one existing leaky marker), and failed full TUI regression (3980 passed, 93 failed, 47 timed out, 8 skipped). Functional/receiving acceptance remains open.

- [x] 2026-09-15 `acct-inspect-fix-04` corrected the independent review's single P2: checkpoint lag now renders separately from stale contributions. The [correction receipt](../../../../qa/portfolio/agent-cost-accounting/pf-60-s03/acct-inspect-fix-04-return.md) preserves the failing-before/passing-after regression, all 34 original cases passing, 19/19 focused TUI and 407/407 state/TaskNode results. Receiving and functional qualification remain open.

- [x] 2026-09-15 `acct-inspect-breakdown-06` implemented the bounded S03-B root/descendant and provider/model breakdowns, separate unknown-parent population, estimate-only billing distinction and inherited freshness diagnostics. The [worker receipt](../../../../qa/portfolio/agent-cost-accounting/pf-60-s03/acct-inspect-breakdown-06-return.md) records final verification and limitations. Positive billed-amount comparison has no retained settlement seam; receiving and functional acceptance remain open.

- [x] 2026-09-16 `acct-inspect-fix-07` corrected unknown-ancestry unavailability attribution, bounded candidate count before repeated inspection, and proved partial group-price coverage against the requested mutant. The [correction receipt](../../../../qa/portfolio/agent-cost-accounting/pf-60-s03/acct-inspect-fix-07-return.md) records reproduced failures and 442/442 final tests, including all 52 inspector tests. Repeated whole-store validation below the cap and all functional/receiving gates remain open.
- [x] 2026-09-16 **Code-blind guest run and its usability pass.** The package built from the integration tip ran in the sealed guest and a real parent-and-child inference pair completed, with all three source hosts denied from inside the run. Four findings came from watching real screens: the unavailable reason is now the first line and wraps to the rendered width rather than a fixed 26 columns; every short-form state names the requested UTC day; and the child-watcher refusal is deliberately kept because the boundary is input ownership, not whether a command mutates. Receiving gates 464/464. [Findings](../../../../qa/portfolio/agent-cost-accounting/pf-60-s03/no-activation-path-20260916.md).
- [x] 2026-09-16 **S03-C received**: user-chosen ranges with hour, ISO week and calendar month grouping on the frozen UTC alignment. Review checked the alignment where it breaks - ISO weeks across a year boundary, a week crossing a month, February in a leap year - found no local time anywhere, confirmed the out-of-window hour refusal is a superset of the removal rule so a permitted window can never be missing swept rows, and confirmed the mixed-range withholding is a genuine necessity rather than an unimplemented sum behind a refusal. No P1 or P2. Receiving gates on the merged tree: 454/454.
- [x] 2026-09-16 **S03-B received on the integration tip.** Held out of integration until its review P2 was closed, because that P2 was a regression against already received behaviour: an unrelated thread's unavailability and retention coverage could overwrite the root's own view. Estimate versus billed stays unimplemented because no billed or settlement column exists anywhere in the ledger, confirmed by review rather than assumed, and every page says so. Receiving gates on the merged tree: 442/442.
- [x] 2026-09-16 **Bucket alignment frozen, manager decision, no product authority needed.** Everything is UTC, because the retained aggregates are UTC days and any other zone would require re-bucketing evidence that was never kept that way; the displayed timezone is stated on every view rather than assumed. Weeks are ISO-8601, Monday to Sunday. Months are calendar UTC months. Hour buckets are offered only inside the 90-day drill-down window where raw per-request detail is actually retained, and are refused with the reason elsewhere rather than synthesised from a day aggregate. A bucket that is only partly covered by retained data is labelled as partial and never totalled.

- [x] 2026-09-16 `acct-inspect-ranges-08` implemented S03-C custom half-open UTC ranges and hour/day/ISO-week/calendar-month grouping within the frozen paths. Requested/effective coverage, partial-bucket exclusion, precision refusals and retained cutoff facts are explicit. [Worker receipt](../../../../qa/portfolio/agent-cost-accounting/pf-60-s03/acct-inspect-ranges-08-return.md) records checks, exact reconciliation and conservative range limits; independent receiving/functional gates remain open.
- [x] 2026-09-16 `acct-inspect-fix-09` corrected the three S03-C P3 findings: coverage explicitly names aggregate retention, the range overview discloses unknown ancestry before its total, and a store-level ISO-week test checks day slices, raw expiry and aggregate expiry. Both named loop/window mutants fail the new test. The [revision receipt](../../../../qa/portfolio/agent-cost-accounting/pf-60-s03/acct-inspect-fix-09-return.md) preserves before/after results and final checks; receiving and functional qualification remain open. Received on the integration tip; receiving gates on the merged tree 457/457.

## Remaining
- [ ] **Disclosed and unresolved: the reader is conservative in a way that can refuse a small day.** The inherited whole-store validator visits unrelated history, and the caps - 10,000 retained rows, 4 MiB combined serialized input, 512 selected attempts, 4,096 observations - are applied store-wide before materialization, so a small selected day can return TooLarge because of evidence that has nothing to do with it. The boundary fixtures prove rejection at 513 attempts and 4,097 observations, but they do not prove a valid 10,000-row result or a maximal valid packet. No schema or index change was made. Receiving is complete; this limit is not.
- [ ] Three TUI paths in the write scope overlap PF-83-S01 and were **released to this lane on 2026-09-15** because no PF-83 action held them: codex-rs/tui/src/app/event_dispatch.rs, codex-rs/tui/src/app_event.rs, codex-rs/tui/src/app/tests.rs. The release is recorded in that sprint. Re-check before launch rather than relying on this line, since PF-83 may re-reserve.
- [ ] **Inherited from PF-60-S02 at its archival, transferred rather than discharged:** Corbanu plan gateway economics, startup prewarm, auxiliary collection, legacy evidence acquisition and complete application coverage all remain unqualified, and anonymous 365-day fencing is not permanent. Six leaky markers remain across the accounting suite (two in `accounting::policy_tests`, three in `accounting_anthropic`, one in `accounting_chat_native_mismatched_endpoint`); the three diagnosed in `acct-cleanup-04` showed no concrete fixture defect and were deliberately left rather than given speculative cleanup. Also inherited: the one disclosed review P3, that the provider-publication regression pins ordering against contributor emission rather than against the state lock.

- [ ] **Disclosed from S03-B review and deferred:** the new candidate bound counts every thread with an attempt on the day, including unrelated ones, so a busy host can refuse a small root's day as TooLarge. It refuses rather than truncating and never invents an amount; a narrower bound needs the retained-reader rework this allocation excluded. Functional qualification of the S03-B views is also still open.
- [ ] **Unblocked 2026-09-16: Travis answered `accounting-collection-activation-20260916` with developer-only activation.** The inspector's data paths could not be functionally qualified because collection had no activation outside tests - `Config::accounting` defaults to `Disabled`, is set non-`Disabled` only in four test modules, and has no TOML key, environment variable or command; the ledger is separately gated on being explicitly installed, so `/usage requests` had exactly one reachable output on any machine. The answer is an activation a developer can turn on deliberately and that is **unreachable in a shipped binary**, which is why the write scope above now includes `core/src/config/mod.rs`, `core/src/accounting.rs` and `state/src/runtime/accounting_native.rs` - owner-authorised new work, recorded here rather than added silently. Shipping `/usage requests` to users is explicitly **not** authorised by this answer and stays held. All four findings from the code-blind guest run are closed.
- [ ] Qualify mixed raw/compact range explanations, separate oldest aggregate/cutoff facts, unknown populations and exact inspectable-bucket reconciliation; compact attribution and full-range totals remain unavailable.
- [ ] Keep historical sessions inspectable; expose missing-price and unavailable-backend states with user-visible next steps.
- [ ] Add narrow-screen, mixed-provider, no-usage, stale-estimate and current-command regression tests.
- [ ] Test exact boundaries, partial days, timezone/DST transitions, empty versus unavailable ranges, deleted history, mixed retention coverage and interval switching through supported user controls; no hidden storage repair.
- [ ] Record actual outputs, counterexamples, remaining limitations and a concrete next-sprint handoff; stop on changed scope.

## Verification
- [ ] **Inherited from PF-60-S02:** [isolated code-blind functional execution](../../../../qa/code-blind-functional/isolated-execution.md) with real keys and negative controls, independent evidence review of that execution, and true-TUI qualification. S02 accepted an internal-stage N/A for its own increments only, which deferred these here; it did not discharge them. This sprint is the first user-facing unit, so they fall due.

- [x] Focused, as specified: `just test -p codex-tui usage` passes 91/91. Recorded because I had been receiving against a filter of my own - `accounting_inspect_` plus the state and tasknode packages - which added coverage but was not the gate written here. The union of both is 553/553 and is the number future receipts should use.
- [x] Integration: `python3 docs/plans/check.py` and `python3 docs/sprints/check.py` pass and `git diff --check` is clean on the integration tip.
- [ ] Resolve TUI applicability against the plan; record actual-key success, failure/cancel, recovery/resume and final binary evidence for every affected interactive path.
- [ ] Record expected versus actual results and nonzero test counts; no unchecked assumption is converted into a pass.

## Exit evidence

- [ ] Output commit/digest and input provenance recorded; checks linked to that final artifact/tree.
- [ ] Named human/receiving owner accepts the bounded output; needed go/no-go decision is recorded.
- [ ] Handoff includes changed scope, contracts, known gaps and required combined-tree evidence.
- [ ] Done/Remaining ledgers updated honestly; archive accepted record under `docs/sprints/archive/portfolio-agent-cost-accounting/` and update plan backlinks.
