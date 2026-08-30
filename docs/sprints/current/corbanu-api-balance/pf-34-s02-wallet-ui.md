---
sprint_id: "PF-34-S02"
title: "Corbanu API wallet UI"
status: in_progress
plan_file: "docs/plans/active/corbanu-api-balance.md"
plan_feature: "PF-34"
execution_order: 7
owner: "Jim Ricketts"
lane: "terminal-tui"
write_scope: "codex-rs/model-provider-info/src/lib.rs, codex-rs/model-provider-info/src/model_provider_info_tests.rs, codex-rs/tui/src/app/event_dispatch.rs, codex-rs/tui/src/app_event.rs, codex-rs/tui/src/bottom_pane/mod.rs, codex-rs/tui/src/bottom_pane/snapshots, codex-rs/tui/src/chatwidget.rs, codex-rs/tui/src/chatwidget/model_popups.rs, codex-rs/tui/src/chatwidget/wallet_api.rs, codex-rs/tui/src/chatwidget/wallet_api_tests.rs, codex-rs/tui/src/chatwidget/wallet_menu.rs, codex-rs/tui/src/chatwidget/wallet_unlock.rs, codex-rs/tui/src/chatwidget/wallet_unlock_tests.rs"
worktree: "/home/pfrpc/repos/CorbanuTerminal"
branch: "feat/corbanu-api-wallet"
base_commit: "4ff38e974b4e63cebffc5d608c5584e2d453cf1b"
depends_on: "PF-34-S01"
created: 2026-08-30
updated: 2026-08-30
---

# PF-34-S02 — Corbanu API wallet UI

## Execution mandate

- Deliver: replace new tier-sale UX with a `/wallet` Corbanu API surface for balance, at-cost prices, arbitrary top-up, key summaries, one-time secret reveal, create/revoke, and provider selection.
- Excludes: deleting grandfathered legacy periods, production deployment, compliance sign-off, and release publication.

## Plan linkage

- Plan: [Corbanu API balance and keys](../../../plans/active/corbanu-api-balance.md)
- Feature: `PF-34`
- Acceptance advanced: a wallet user can visibly fund and manage Corbanu API access and select its provider-neutral models without plan tiers.

## Code boundaries

- Existing: `codex-rs/tui/src/chatwidget/wallet_menu.rs`, `wallet_unlock.rs`, `model_popups.rs`
- Planned: `codex-rs/tui/src/chatwidget/wallet_api.rs` and focused tests
- Tests: TUI unit/snapshot tests, provider-resolution tests, and true-PTY flow

## Preconditions

- [x] Plan is active.
- [x] PF-34-S01 is completed and archived.
- [x] Worktree, branch, and base commit are exact and match the plan.
- [x] Terminal-TUI lane and literal write scope have no active collision.
- [x] Product-owned wallet and provider adapters are recorded; upstream picker interfaces remain native.

## Done

- [x] Sprint record created and linked to PF-34.

## Remaining

- [ ] Replace visible new tier-sale actions with the Corbanu API entry and account surface.
- [ ] Add amount entry, exact confirmation, unlock continuation, and result handling.
- [ ] Show balance, versioned at-cost prices, key summaries, and secure one-time secrets.
- [ ] Add create/revoke actions and Corbanu API provider/model selection.
- [ ] Add snapshot and adjacent state regressions.
- [ ] Pass the primary, cancel/failure, and recovery true-TUI flows with actual keys.

## Verification

- [ ] Focused test: `just test -p codex-model-provider-info -p codex-tui`
- [ ] Integration test: provider request-wire and wallet-daemon interaction tests.
- [ ] TUI applicability resolved; actual keys, checkpoints, and artifact recorded.
- [ ] Integrated interfaces and candidate commit match PF-34-S01 and backend evidence.
- [ ] Upstream picker/provider compatibility tests pass.

## Exit evidence

- [ ] Implementation commit recorded.
- [ ] Final-tree test output linked.
- [ ] `Done` and `Remaining` ledgers reflect reality.
- [ ] Completed record moved to `docs/sprints/archive/corbanu-api-balance/`.
