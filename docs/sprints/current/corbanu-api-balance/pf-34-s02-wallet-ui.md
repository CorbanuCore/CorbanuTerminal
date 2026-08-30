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
- [x] Replaced visible new tier-sale actions with the Corbanu API entry and account surface.
- [x] Added arbitrary amount entry, exact confirmation, operation-bound unlock continuation, and result handling.
- [x] Added balance, versioned at-cost prices, key summaries, and secure one-time secret reveal.
- [x] Added create/revoke actions and provider-neutral Corbanu API model selection.
- [x] Added funded, unfunded, amount-boundary, key-wire, legacy-sale, and provider-selection regressions.
- [x] Built the real Terminal and exercised `/wallet` -> `Corbanu API`, wallet-daemon startup, unlock prompt, top-up entry, exact confirmation, insufficient-funds disablement, and cancel against the wallet-bound disposable backend.

## Remaining

- [ ] Named human tester enters the wallet passcode and completes create/revoke plus one-time reveal against the disposable backend.
- [ ] Repeat the primary and recovery flows against the deployed backend in PF-35 after Fly authentication is restored.

## Verification

- [x] Focused tests: 14 wallet, 9 wallet-daemon, 58 provider, 3 wallet-API snapshots, and 23 wallet-menu tests pass.
- [x] Integration tests cover provider mapping, operation-preserving daemon IPC, backend public-key conversion, and exact microdollars.
- [ ] TUI applicability is resolved and non-secret checkpoints are recorded; human passcode/key lifecycle remains pending.
- [x] Integrated interfaces in `594d618306d922963cf6676d3600cd381922759c` match PF-34-S01 and backend `cd79361d8b4f286291556a641288757d0451f52c`.
- [x] Upstream picker/provider compatibility test passes with exact model-field selection.

## Exit evidence

- [x] Implementation commit: `594d618306d922963cf6676d3600cd381922759c`.
- [x] Final-tree focused tests and `just fix` pass; true-TUI log is `/tmp/corbanu-api-ui-qa-20260830/codex-tui.log` with no panic signature.
- [x] `Done` and `Remaining` ledgers reflect reality.
- [ ] Completed record moved to `docs/sprints/archive/corbanu-api-balance/`.
