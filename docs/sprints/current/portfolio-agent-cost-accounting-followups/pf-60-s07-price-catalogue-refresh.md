---
sprint_id: "PF-60-S07"
title: "Price catalogue refresh (#368)"
status: draft
plan_file: "docs/plans/proposed/portfolio-agent-cost-accounting-followups.md"
plan_feature: "PF-60"
execution_order: 2
owner: "Codex accounting lane (proposed); Travis Good accepts"
parallel_lane: "UNALLOCATED"
write_scope: "UNALLOCATED"
integration_gate: "UNALLOCATED"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-60-S04"
created: 2026-10-10
updated: 2026-10-10
---

# PF-60-S07 — Price catalogue refresh (#368)

## Execution mandate

- Deliver: item (iii) from Travis's 2026-10-10 decision, the five price items of #368, so that each request is priced
  from the provider's current published rate or plainly shows "no price".
- **Deadline:** the Sol promotion item (AC4) must land before 2026-11-21.
- Excludes: re-pricing history; changing what Corbanu charges; Grok tiers (#127) and OpenRouter caps (#129).

## Plan linkage

- Plan: [Accounting follow-ups after PF-60](../../../plans/proposed/portfolio-agent-cost-accounting-followups.md); feature `PF-60`.
- Acceptance advanced: no request shows a stale or guessed price.

## Code boundaries

- Existing: `codex-rs/core/src/accounting_prices.rs`; `codex-rs/models-manager/models.json`;
  `codex-rs/model-provider-info/src/` (routes); `scripts/check_openai_api_prices.py`; the OpenRouter `billed_usd`
  parse in the accounting path.
- Evidence: `qa/portfolio/agent-cost-accounting/issue-361/README.md` (2026-10-09 readings) and a new dated reading.

## Preconditions

- [ ] Plan is active, or Travis authorises AC4 alone as a bounded fix before 2026-11-21.
- [ ] Dependencies are completed.
- [ ] Worktree, branch, and base commit are exact and match the plan.
- [ ] Travis chooses the OpenRouter rule for AC1.

## Done

- [x] Draft sprint created 2026-10-10 from Travis's decision; linked to one plan feature.

## Remaining

- [ ] AC1 OpenRouter: a request is priced from the charge OpenRouter reports for it when present, labelled as the
      provider's charge; without one, the catalogue rate is shown only as a labelled estimate, or "no price"
      (Travis's choice). A real request matches OpenRouter's reported charge exactly.
- [ ] AC2 Moonshot/Kimi Open Platform (`api.moonshot.ai/v1`) and BigModel (`open.bigmodel.cn/api/paas/v4`): rows from
      the official pricing pages, with K3 cache writes by TTL; a request on each recomputes exactly (or the route stays
      "no price" with the reason recorded if no key is available).
- [ ] AC3: `gpt-6.1-sol` has a row at OpenAI's published rates, cache writes included.
- [ ] AC4: the GPT-5.6 Sol price is re-read before 2026-11-21; a test pins the promotion end so work after it uses the
      new published price or shows "no price" with a reason, never the promotional price.
- [ ] AC5: DeepSeek's holiday and off-peak calendar after 2026-10-07 matches DeepSeek's page.
- [ ] AC6: every changed rate cites its source URL and read date; history is not re-priced (test).
- [ ] Code-blind functional design frozen before test-result disclosure.

## Verification

- [ ] Focused: `just test -p codex-core --features developer-accounting -E 'test(accounting_prices) | test(accounting)'`; `python3 scripts/check_openai_api_prices.py`.
- [ ] Real requests recomputed from provider-reported usage at the cited prices, to the micro-dollar.
- [ ] TUI: `/cost` pages for each changed route captured in a tmux run.
- [ ] Code-blind cases executed independently (in PF-60-S10 under the enforced gate).

## Exit evidence

- [ ] Implementation commit recorded.
- [ ] Final-tree test output and dated price readings linked.
- [ ] Code-blind handoff checker passes, or explicit limited-testing agreement/blocker recorded; no automatic human acceptance.
- [ ] Parallel handoff, if applicable: commit, contract versions, scope audit and combined-tree test evidence recorded.
- [ ] `Done` and `Remaining` ledgers reflect reality; #368 closed.
- [ ] Completed record moved to `docs/sprints/archive/portfolio-agent-cost-accounting-followups/`.
