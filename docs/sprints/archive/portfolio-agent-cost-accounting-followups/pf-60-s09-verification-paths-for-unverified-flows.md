---
sprint_id: "PF-60-S09"
title: "Verification paths for unverified flows"
status: completed
plan_file: "docs/plans/completed/main-2026-10-20-portfolio-agent-cost-accounting-followups.md"
plan_feature: "PF-60"
execution_order: 4
owner: "Codex worker pf60-next; Travis Good accepts"
parallel_lane: "accounting"
write_scope: "qa/portfolio/agent-cost-accounting/pf-60-s09/, qa/demos/index/PF-60-S09.md, qa/demos/specs/pf60s09-image-cost.toml, qa/demos/specs/pf60s09-chatgpt-login-cost.toml, docs/sprints/current/portfolio-agent-cost-accounting-followups/pf-60-s09-verification-paths-for-unverified-flows.md, docs/sprints/current/portfolio-agent-cost-accounting-followups/pf-60-s08-cost-page-clarity-and-reopen-warmup.md, docs/plans/completed/main-2026-10-20-portfolio-agent-cost-accounting-followups.md, docs/sprints/index.md"
integration_gate: "One PR to main: each flow run once with captures (PASS, FAIL with an issue, or Travis decision), drivers documented for S10, docs checkers, independent Opus 5.5 High review to APPROVE on the exact head, SOP videos; Travis accepts"
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/pf60-s09"
branch: "pf60-s09/verification-paths"
base_commit: "643838f2f4ed87dfe71be24f983a6dcc91709faa"
depends_on: "PF-60-S04"
created: 2026-10-10
updated: 2026-10-16
---

# PF-60-S09 — Verification paths for unverified flows

**Status: completed** (accepted by Travis 2026-10-15 with the realtime failure tracked in #824; code-blind run in
PF-60-S10 accepted 2026-10-16; see Closure).

## Closure — 2026-10-16

Completed. Travis accepted S09 on 2026-10-15 (20:55Z, "1-6 as recommended"; comment on #400), with AC3's realtime
failure tracked in #824, which he sent for a fix (option (a)). PF-60-S10's isolated code-blind run ([record](../../../../qa/portfolio/agent-cost-accounting/followups-acceptance-20261016/README.md), PR #909, merged `223927bc26`) was accepted by Travis on 2026-10-16 as recorded, 29 passed, 1 failed, 7 blocked of 37 (decision D24, "all as recommended"; comment on [#898](https://github.com/CorbanuCore/CorbanuTerminal/issues/898)). With it he decided D25 (the S10 gaps are re-checked in #898's acceptance run, not a full S10 re-run) and D26 (the record's last evidence check, r3, failed on wording only; accepted without another round).

- **Realtime after the sprint:** PR #882 (merged `a43d22feab`) records a WebSocket realtime session as one ledger
  request with tokens and cost unknown, not zero (ledger format 6, accepted 2026-10-16 as D14). It merged after S10's
  freeze, so S10 didn't cover it; one real realtime check on an OpenAI key runs in the next isolated live run (D16,
  under $0.05). #824 stays open for WebRTC on an API key; #889 holds the other follow-ups (its item 1 is D15).
- **S10 results** ([dispositions](../../../../qa/portfolio/agent-cost-accounting/followups-acceptance-20261016/dispositions.md)): passed ACC-30 to ACC-33 (ChatGPT sign-in through S09's stand-in: catalogued,
  uncatalogued, priority-tier and image turns covered by the subscription) and ACC-21 (a real API-key image priced at
  `gpt-image-2` rates).
- TensorCash stays not applicable until access is granted.

#402 closes with this archive.

## Execution mandate

- Deliver: item (iv) from Travis's 2026-10-10 decision (#402). Four flows were NOT VERIFIABLE in PF-60-S05 and
  PF-60-S04: ChatGPT login, image generation, realtime and TensorCash. Build a real way to verify each, run it once,
  and record PASS or FAIL.
- Excludes: changing how these flows are priced or recorded (a FAIL becomes an issue, not a fix here).

## Plan linkage

- Plan: [Accounting follow-ups after PF-60](../../../plans/completed/main-2026-10-20-portfolio-agent-cost-accounting-followups.md); feature `PF-60`.
- Acceptance advanced: no accounting flow is left unverified without Travis's recorded decision.

## Code boundaries

- Existing: `codex-rs/core/src/realtime_conversation.rs`; `codex-rs/ext/image-generation/src/backend.rs`;
  `codex-rs/core/src/tools/spec_plan.rs` (image tool offered to OpenAI API keys since #370); the tmux harness
  (`docs/tmuxHarness.md`) and the PF-60-S04 acceptance tools under `qa/portfolio/agent-cost-accounting/pf-60-s04/`.
- Planned: test drivers and fixtures under `qa/portfolio/agent-cost-accounting/`; vault labels for any new test account.

## Preconditions

- [x] Plan is active (Travis, 2026-10-14; slot 2).
- [x] Dependencies are completed (PF-60-S04 archived 2026-10-10).
- [x] Worktree, branch, and base commit are exact and match the plan.
- [x] Travis decides the verification paths (2026-10-14): ChatGPT login with a mock sign-in server only (his own login
      is never swapped); TensorCash not applicable until access is granted; image generation and realtime for real on
      the OpenAI API key (vault `openai-api-key`, small spend).

## Done

- [x] Draft sprint created 2026-10-10 from Travis's decision; linked to one plan feature.
- [x] AC1 ChatGPT login: **PASS (stand-in only)**. Device-code sign-in and turns through the client's built-in ChatGPT
      routes, redirected to a local stand-in; catalogued, uncatalogued and priority-tier turns all read "Covered by
      your subscription".
- [x] AC2 Image generation: **PASS**. A real API-key `gpt-image-2` request recomputes exactly at $5/M input and $30/M
      output ($0.007110); the ChatGPT-login image path (stand-in) is recorded as subscription.
- [x] AC3 Realtime: **FAIL, filed as #824**. A real WebSocket session isn't recorded; the WebRTC call is recorded but
      OpenAI refused it on this API key (v1: header; v3: no access); realtime has no token counts to recompute.
- [x] AC4 TensorCash: not applicable until access is granted (Travis, 2026-10-14).
- [x] Each driver is documented for the PF-60-S10 executor: [drivers](../../../../qa/portfolio/agent-cost-accounting/pf-60-s09/drivers/README.md);
      [gate](../../../../qa/portfolio/agent-cost-accounting/pf-60-s09/gate.md).

## Remaining

- [x] Code-blind functional design frozen before test-result disclosure: PF-60-S10 froze it at `220c99ed59`
      ([S10](pf-60-s10-isolated-code-blind-acceptance.md)).
- [x] Travis accepted S09 (2026-10-15; AC3 FAIL tracked in #824, fixed for WebSocket by #882); #402 closed with this
      archive.

## Verification

- [x] Each AC has captures and a recomputation, an issue, or Travis's recorded decision; no NOT VERIFIABLE left.
- [x] Key scan of all captures: 0 hits; disposable homes with `CORBANU_TEST_NO_NATIVE_KEYRING=1`.
- [x] Integration: `python3 docs/plans/check.py; python3 docs/sprints/check.py`; `git diff --check`.
- [x] Code-blind cases executed independently in PF-60-S10 under the enforced gate (Closure).

## Exit evidence

- [x] Driver commit and evidence directory recorded (PR #825; `qa/portfolio/agent-cost-accounting/pf-60-s09/`).
- [x] Per-flow PASS/FAIL linked; the FAIL filed as #824.
- [x] Code-blind handoff checker: it stops at S10's first unresolved case, as expected; Travis's acceptance of S10 as
      recorded (2026-10-16, D24) is the recorded agreement. No automatic human acceptance.
- [x] Parallel handoff: not applicable (the plan ran one sprint at a time).
- [x] `Done` and `Remaining` ledgers reflect reality; #402 closed with this archive.
- [x] Completed record moved to `docs/sprints/archive/portfolio-agent-cost-accounting-followups/` (2026-10-16).
