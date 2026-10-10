---
sprint_id: "PF-60-S04"
title: "Cost-accounting acceptance and handoff"
status: in_progress
plan_file: "docs/plans/active/portfolio-agent-cost-accounting.md"
plan_feature: "PF-60"
execution_order: 5
owner: "Codex accounting lane (acct-s04); Codex coordinator receives; Travis Good accepts"
parallel_lane: "accounting-pf60-s04-acceptance"
write_scope: "codex-rs/core/src/accounting.rs, codex-rs/core/src/accounting_extensions.rs, codex-rs/core/src/accounting_extensions_tests.rs, codex-rs/core/src/codex_thread.rs, codex-rs/core/src/state/service.rs, codex-rs/core/src/session/, codex-rs/core/tests/suite/accounting_responses.rs, codex-rs/memories/write/src/runtime.rs, codex-rs/memories/write/src/startup_tests.rs, codex-rs/tui/src/chatwidget/tokens.rs, codex-rs/tui/src/chatwidget/tokens_tests.rs, codex-rs/tui/src/chatwidget/tokens/, qa/portfolio/agent-cost-accounting/qualification.md, qa/portfolio/agent-cost-accounting/pf-60-s04/, qa/demos/specs/pf60s04-side-conversation-cost.toml, qa/demos/specs/pf60s04-basis-only-request.toml, qa/demos/specs/pf60s04-parent-child-journey.toml, qa/demos/specs/pf60s04-restart-history.toml, qa/demos/specs/pf60s04-cost-date-bounds.toml, qa/demos/index/PF-60-S04.md, docs/sprints/current/portfolio-agent-cost-accounting/pf-60-s04-cost-accounting-acceptance-and-handoff.md"
integration_gate: "Small PRs to main, merged by the lane only when every check is green. Each code slice: focused tests with and without developer-accounting, Linux clippy -D warnings on the RTX box, one Opus 5.5 High read-only review. Codex coordinator receives; independent code-blind acceptance, then Travis."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/pf60-s04-20261010"
branch: "work/pf60-s04-20261010"
base_commit: "5e2298a12900b07c1e55805dba8d79854eb803cd"
depends_on: "PF-60-S03, PF-60-S05"
created: 2026-09-09
updated: 2026-10-10
---

# PF-60-S04 — Cost-accounting acceptance and handoff

**Status: in progress** (activated 2026-10-10; S03 and S05 are completed and archived). Travis asked for the accounting work to finish sprint by sprint with demo videos. This sprint closes two gaps S05 handed over, then qualifies every cost flow on one recorded binary and hands off.

## Execution mandate

- Deliver: `qa/portfolio/agent-cost-accounting/qualification.md`; all cost flows pass on one recorded binary; unknown and estimated values stay visibly distinct.
- **Gap (i), close:** `/side` conversations and memory consolidation are paid but not recorded. Record them like guardian reviews: under the conversation they belong to, with a turn label naming what they are (`side:`, `consolidation:`). An ephemeral session with no persisted conversation behind it (`exec --ephemeral`) stays excluded with its one log warning.
- **Gap (ii), close:** a request with a billing basis but no price shows a "Price source" line and price metadata on its detail page, as if it had a price. Show plainly that it has no price and records only its basis.
- **(iii) AC10 waiver re-check:** S05's waiver (a) held until #361 was fixed; #369 merged. Re-run one real OpenAI API-key request on the final binary and recompute it at published prices. Pass lifts waiver (a) for Travis to confirm.
- **(iv) #368 price follow-ups:** documented as limitations only. They change prices, which this sprint excludes.
- Excludes: changing prices, rebilling historical customers, collecting prompts, restoring legacy Plan allowances, silently converting allowance to cash, shipping `/cost` to users.

## Plan linkage

- Plan: [Unified agent cost and usage accounting](../../../plans/active/portfolio-agent-cost-accounting.md); feature `PF-60`; acceptance: all cost flows pass on one recorded binary; unknown/estimated values remain visibly distinct.

## Code boundaries

- Gap (i): `core/src/session/mod.rs::accounting_owner`, `core/src/accounting.rs` (`AccountingOwner`, `attach_scopes` turn label), `core/src/accounting_extensions.rs` (extension labels), the consolidation agent's start in `memories/write/src/runtime.rs` and the hook it needs on `core/src/codex_thread.rs` / `core/src/state/service.rs`.
- Gap (ii): `tui/src/chatwidget/tokens.rs::attempt_text`.
- Evidence: `qa/portfolio/agent-cost-accounting/pf-60-s04/`. The literal file list is `write_scope`.

## Preconditions

- [x] 2026-10-10 Plan active; S03 and S05 completed and archived; worktree, branch, 40-character base and literal write scope allocated and recorded in the plan; receiving owner named.
- [x] 2026-10-10 Inputs: vault labels `claude-plan-test-token`, `provider/zai_api_key`, `provider/kimi_api_key`, `openai-api-key`, used only in disposable homes with `CORBANU_TEST_NO_NATIVE_KEYRING=1`; small live spend only; no external writes or financial actions.

## Done

- [x] Draft sprint created and linked to one feature.
- [x] 2026-10-10 Activated with gaps (i)-(iv) folded in.

## Remaining

- [ ] **Slice A (gap i):** side conversations and memory consolidation recorded under their conversation; regression tests.
- [ ] **Slice B (gap ii):** basis-only request detail without "Price source"; regression tests.
- [ ] Build one debug binary (`developer-accounting`) from the final main commit and record its commit and digest.
- [ ] Three-provider parent/child journeys (Claude plan, GLM 5.2 on `zai`, Kimi k3 on `kimi-code`, plus OpenAI `gpt-5.4` on an API key) through the tmux harness with actual keys.
- [ ] Cancellation, missing price, duplicate retry, restart/resume, historical inspection and `/cost` date bounds; reconcile against golden totals.
- [ ] AC10 re-check (iii); #368 limitations (iv).
- [ ] Write `qualification.md`: expected vs actual, counterexamples, limitations, handoff; demo videos and the narrated reel (built, not published).
- [ ] Independent code-blind acceptance, then Travis's named acceptance; archive.

## Verification

- [ ] Slice A: from `codex-rs`, `just test -p codex-core --features developer-accounting -E 'test(accounting) | test(guardian)'` and `just test -p codex-core -E 'test(accounting) | test(guardian)'`; `just test -p codex-memories-write`.
- [ ] Slice B: from `codex-rs`, `just test -p codex-tui --features developer-accounting -E 'test(tokens) | test(cost) | test(accounting)'` and `just test -p codex-tui -E 'test(tokens) | test(cost)'`.
- [ ] Each slice: Linux `cargo clippy --all-targets -- -D warnings` for the touched crates, with and without `developer-accounting`, on the RTX box; one Opus 5.5 High read-only review.
- [ ] Focused tmux: from `codex-rs`, `CORBANU_TMUX_REQUIRED=1 just test -p codex-tui --test all tmux --retries 0`.
- [ ] Integration: from repo root, `python3 docs/plans/check.py; python3 docs/sprints/check.py`; `git diff --check`.
- [ ] True TUI on the recorded binary: actual-key success, cancel, recovery/resume and historical inspection for every affected path.
- [ ] Expected versus actual results and nonzero test counts recorded; no unchecked assumption is converted into a pass.

## Exit evidence

- [ ] Output commit/digest and input provenance recorded; checks linked to that final tree.
- [ ] Independent code-blind acceptance recorded; Travis accepts the bounded output.
- [ ] Handoff includes changed scope, contracts, known gaps and required combined-tree evidence.
- [ ] Done/Remaining ledgers updated honestly; accepted record archived under `docs/sprints/archive/portfolio-agent-cost-accounting/` and plan backlinks updated.
