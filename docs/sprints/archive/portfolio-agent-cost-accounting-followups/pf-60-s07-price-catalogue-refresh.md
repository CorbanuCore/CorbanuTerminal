---
sprint_id: "PF-60-S07"
title: "Price catalogue refresh (#368)"
status: completed
plan_file: "docs/plans/completed/main-2026-10-20-portfolio-agent-cost-accounting-followups.md"
plan_feature: "PF-60"
execution_order: 2
owner: "Codex worker pf60-next; Travis Good accepts"
parallel_lane: "accounting"
write_scope: "codex-rs/core/src/config/mod.rs, codex-rs/core/src/accounting.rs, codex-rs/core/src/accounting_prices.rs, codex-rs/core/src/accounting_chat.rs, codex-rs/core/src/accounting_transport.rs, codex-rs/core/src/accounting_basis_tests.rs, codex-rs/core/src/accounting_chat_tests.rs, codex-rs/core/tests/suite/accounting_chat.rs, codex-rs/core/tests/suite/accounting_chat_recovery.rs, codex-rs/tui/src/chatwidget/tokens.rs, codex-rs/tui/src/chatwidget/tokens/charge.rs, codex-rs/tui/src/chatwidget/tokens/charge_tests.rs, docs/provider-accounts.md, docs/sprints/current/portfolio-agent-cost-accounting-followups/pf-60-s07-price-catalogue-refresh.md, docs/sprints/current/portfolio-agent-cost-accounting-followups/pf-60-s06-side-and-fork-attribution.md, docs/plans/completed/main-2026-10-20-portfolio-agent-cost-accounting-followups.md, qa/portfolio/agent-cost-accounting/issue-368/, qa/portfolio/agent-cost-accounting/pf-60-s07/, qa/demos/index/PF-60-S07.md, qa/demos/specs/"
integration_gate: "One PR to main: fail-before/pass-after tests, focused just test with and without developer-accounting, macOS and Linux clippy -D warnings, independent Opus 5.5 High review to APPROVE on the exact head, tmux check in a disposable home, SOP video; Travis accepts"
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/pf60-s07"
branch: "pf60-s07/moonshot-route-pricing"
base_commit: "df3d8c5d1c521228a8dbb1035831af572afe1284"
depends_on: "PF-60-S04"
created: 2026-10-10
updated: 2026-10-16
---

# PF-60-S07 — Price catalogue refresh (#368)

**Status: completed** (accepted by Travis 2026-10-15; code-blind run in PF-60-S10 accepted 2026-10-16; see Closure).

## Closure — 2026-10-16

Completed. Travis accepted S07 on 2026-10-15 (20:55Z, "1-6 as recommended"; comment on #400). PF-60-S10's isolated code-blind run ([record](../../../../qa/portfolio/agent-cost-accounting/followups-acceptance-20261016/README.md), PR #909, merged `223927bc26`) was accepted by Travis on 2026-10-16 as recorded, 29 passed, 1 failed, 7 blocked of 37 (decision D24, "all as recommended"; comment on [#898](https://github.com/CorbanuCore/CorbanuTerminal/issues/898)). With it he decided D25 (the S10 gaps are re-checked in #898's acceptance run, not a full S10 re-run) and D26 (the record's last evidence check, r3, failed on wording only; accepted without another round).

- **Real Moonshot requests** (after the sprint, key in the vault): one real turn each on `kimi-k3` and `kimi-k2.7-code`
  showed that Moonshot leaves out zero cache counts, so `/cost` showed only a lower bound. Fixed by PR #859 (merged
  `3ba6c460c5`); evidence and the committed stand-in script in PR #862 (merged `762d3956e9`,
  [readings](../../../../qa/portfolio/agent-cost-accounting/issue-368/moonshot-real-20261015/README.md)). Spend about $0.048; rates re-read and unchanged.
- **S10 results** ([dispositions](../../../../qa/portfolio/agent-cost-accounting/followups-acceptance-20261016/dispositions.md)): passed ACC-08 and ACC-09 (real Moonshot K3 and K2.7 Code priced from
  Moonshot's rates), ACC-19 and ACC-20 (`gpt-6.1-sol`, GPT-5.6 Sol), ACC-23, 25, 26 and 28 (OpenRouter, Vercel and the
  Corbanu API priced from their reported charge), ACC-24 (reasoning above output kept as not reported) and ACC-37 (no
  charge: a labelled estimate, a lower bound or no price, never $0). Blocked: ACC-22 (advisory; the product doesn't
  advertise a priority tier for `gpt-6-luna`, so the no-price branch was never reached) and ACC-35 (the optional
  Moonshot cache read; D3 was not granted during the run).
- **Carried:** a real Moonshot cache hit is checked in the next isolated live run (Travis 2026-10-16, D3, under
  $0.01). #368 stays open for BigModel (deferred by Travis 2026-10-14) and DeepSeek's 2027 holiday dates (nightly-ci
  fails from 2026-12-01 until they are added); the Sol re-read is due before 2026-11-21. #786 (reasoning above output)
  closes with this archive: Chat routes are fixed here (#812) and Responses routes by #853; its two remaining rare cases
  (split and cumulative reports) stay tracked in #814.

## Execution mandate

- Deliver: item (iii) from Travis's 2026-10-10 decision, the five price items of #368, so that each request is priced
  from the provider's current published rate or plainly shows "no price". The exceptions are OpenRouter, Vercel and the Corbanu API,
  which are priced from the charge they report (AC1).
- **Deadline:** the Sol promotion item (AC4) must land before 2026-11-21.
- Excludes: re-pricing history; changing what Corbanu charges; Grok tiers (#127) and OpenRouter caps (#129).

## Plan linkage

- Plan: [Accounting follow-ups after PF-60](../../../plans/completed/main-2026-10-20-portfolio-agent-cost-accounting-followups.md); feature `PF-60`.
- Acceptance advanced: no request shows a stale or guessed price.

## Code boundaries

- Existing: `codex-rs/core/src/accounting_prices.rs`; `codex-rs/models-manager/models.json`;
  `codex-rs/model-provider-info/src/` (routes); `scripts/check_openai_api_prices.py`; the OpenRouter `billed_usd`
  parse in the accounting path.
- Evidence: `qa/portfolio/agent-cost-accounting/issue-361/README.md` (2026-10-09 readings) and a new dated reading.

## Preconditions

- [x] Plan is active (Travis, 2026-10-14; slot 2).
- [x] Dependencies are completed (PF-60-S04 archived 2026-10-10).
- [x] Worktree, branch, and base commit are exact and match the plan.
- [x] Travis chooses the Moonshot/BigModel scope (2026-10-14): Moonshot by route first, BigModel deferred; Vercel and
      the Corbanu API priced from their reported charges by AC1's rule; #786 option (a).
- [x] Travis chooses the OpenRouter rule for AC1 (2026-10-14).

## Done

- [x] Draft sprint created 2026-10-10 from Travis's decision; linked to one plan feature.
- [x] AC3: `gpt-6.1-sol` has a row at OpenAI's published rates, cache writes included.
- [x] 2026-10-14 data upkeep (#762; readings in `qa/portfolio/agent-cost-accounting/issue-368/`). AC4: Sol re-read
      before 2026-11-21, a test pins the promotion end, the calendar check warns from 2026-10-22. AC5: DeepSeek's
      calendar matches its page through 2026-12-31.
- [x] AC1 OpenRouter (Travis 2026-10-14; PR #784): priced from `usage.cost`, labelled "charged by OpenRouter", each
      page states its `Price basis`; the catalogue figure stays a labelled estimate; the ledger is unchanged.
- [x] AC1 extended (Travis 2026-10-14): Vercel (`gatewayCost`) and the Corbanu API (`usage.corbanu_charge_usd`) by
      the same rule.
- [x] AC2 Moonshot (Travis 2026-10-14: by route first): a provider configured at `https://api.moonshot.ai/v1` is
      priced from Moonshot's published rates (K3, K2.7 Code, K2.7 Code HighSpeed, K2.6), keyed by the route. K3 cache
      writes by TTL: this client names none, so the five-minute rate (a test pins the body). BigModel deferred by
      Travis; it stays "no price".
- [x] #786 (Travis, option a), Chat routes: reasoning above output in one usage report is recorded as not reported;
      the rest, charge included, is kept. Responses routes and split reports: #814.
- [x] AC6: new rates cite source and read date; prices bind at admission and the charge rule is display-only, so
      history is not re-priced. [Gate](../../../../qa/portfolio/agent-cost-accounting/pf-60-s07/gate.md).
- [x] Merged: PR #812 as `b519fb088d` (head `b8311ff490`); review APPROVE on that head.

## Remaining

- [x] A real Moonshot request recomputed from provider-reported usage: done 2026-10-15/16 with a key in the vault
      (#859, #862; Closure), and again in PF-60-S10 (ACC-08, ACC-09). A real cache hit is carried (D3).
- [x] DeepSeek's 2027 holiday dates: carried on #368 until the State Council publishes them (nightly-ci fails from
      2026-12-01 until then).
- [x] Code-blind functional design frozen before test-result disclosure: PF-60-S10 froze it at `220c99ed59`
      ([S10](pf-60-s10-isolated-code-blind-acceptance.md)).

## Verification

- [x] Focused: `just test -p codex-core -E 'test(accounting) | test(spawn)'` 357/357 with `developer-accounting`,
      352/352 without; `just test -p codex-tui --features developer-accounting -E 'test(charge) | test(tokens) | test(cost)'`
      131/132 (the local-only `cost_scope` tmux test; see the gate).
- [x] Real requests recomputed from provider-reported usage at the cited prices: Corbanu API and Vercel charges read
      back exactly; Moonshot first against the stand-in, then for real (#859, #862; Closure).
- [x] TUI: `/cost` pages for each changed route captured in a tmux run
      ([captures](../../../../qa/portfolio/agent-cost-accounting/pf-60-s07/tmux-20261014/)); two demo videos.
- [x] Code-blind cases executed independently in PF-60-S10 under the enforced gate (Closure).

## Exit evidence

- [x] Implementation commits recorded: PR #812 (`b519fb088d`), with #762 and #784 before it and #859/#862 after it.
- [x] Final-tree test output and dated price readings linked (gate; `qa/portfolio/agent-cost-accounting/issue-368/`).
- [x] Code-blind handoff checker: it stops at S10's first unresolved case, as expected; Travis's acceptance of S10 as
      recorded (2026-10-16, D24) is the recorded agreement. No automatic human acceptance.
- [x] Parallel handoff: not applicable (the plan ran one sprint at a time).
- [x] `Done` and `Remaining` ledgers reflect reality; #368 kept open for the carried items (Closure).
- [x] Completed record moved to `docs/sprints/archive/portfolio-agent-cost-accounting-followups/` (2026-10-16).
