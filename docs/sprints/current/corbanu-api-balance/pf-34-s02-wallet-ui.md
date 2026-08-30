---
sprint_id: "PF-34-S02"
title: "Corbanu API wallet UI"
status: in_progress
plan_file: "docs/plans/active/corbanu-api-balance.md"
plan_feature: "PF-34"
execution_order: 7
owner: "Jim Ricketts"
lane: "terminal-tui"
write_scope: "codex-rs/model-provider-info/src/lib.rs, codex-rs/model-provider-info/src/model_provider_info_tests.rs, codex-rs/tui/src/app/event_dispatch.rs, codex-rs/tui/src/app/tests.rs, codex-rs/tui/src/app/snapshots, codex-rs/tui/src/app_event.rs, codex-rs/tui/src/bottom_pane/mod.rs, codex-rs/tui/src/bottom_pane/snapshots, codex-rs/tui/src/chatwidget.rs, codex-rs/tui/src/chatwidget/model_popups.rs, codex-rs/tui/src/chatwidget/tests/popups_and_settings.rs, codex-rs/tui/src/chatwidget/snapshots, codex-rs/tui/src/chatwidget/wallet_account_actions.rs, codex-rs/tui/src/chatwidget/wallet_api.rs, codex-rs/tui/src/chatwidget/wallet_api_tests.rs, codex-rs/tui/src/chatwidget/wallet_menu.rs, codex-rs/tui/src/chatwidget/wallet_unlock.rs, codex-rs/tui/src/chatwidget/wallet_unlock_tests.rs, codex-rs/tui/src/crew_presets.rs, codex-rs/tui/src/crew_presets_tests.rs, codex-rs/tui/src/crew_state_tests.rs, codex-rs/tui/src/slash_command.rs, codex-rs/tui/src/snapshots, codex-rs/tui/src/spawn_crew.rs, codex-rs/tui/src/spawn_orchestration.rs, codex-rs/wallet/src/corbanu_api.rs, codex-rs/wallet/src/corbanu_api_tests.rs, codex-rs/wallet/src/payment.rs"
worktree: "/home/pfrpc/repos/CorbanuTerminal"
branch: "feat/corbanu-api-wallet"
base_commit: "4ff38e974b4e63cebffc5d608c5584e2d453cf1b"
depends_on: "PF-34-S01"
created: 2026-08-30
updated: 2026-08-30
---

# PF-34-S02 — Corbanu API wallet UI

## Execution mandate

- Deliver: replace all tier and legacy-plan UX with a `/wallet` Corbanu API surface for balance, at-cost prices, arbitrary top-up, key summaries, one-time secret reveal, create/revoke, provider selection, and a separate Corbanu API `/spawn` crew preset.
- Excludes: production database deletion, deployment, compliance sign-off, and release publication.

## Plan linkage

- Plan: [Corbanu API balance and keys](../../../plans/active/corbanu-api-balance.md)
- Feature: `PF-34`
- Acceptance advanced: a wallet user can visibly fund and manage Corbanu API access and select its provider-neutral models without plan tiers.

## Code boundaries

- Existing: `codex-rs/tui/src/chatwidget/wallet_menu.rs`, `wallet_unlock.rs`, `model_popups.rs`, `crew_presets.rs`, `spawn_orchestration.rs`
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
- [x] Corrected the production price-discovery boundary so locked, unfunded, stale-key, and legacy-key wallets load the public provider-neutral catalog before authenticated account details.
- [x] Exercised `/wallet` -> `Corbanu API` against the deployed backend and confirmed the zero-dollar balance, arbitrary top-up, key actions, and all six priced models render without exposing upstream vendors.
- [x] Removed the legacy status request and all Plan status, details, receipt, recovery, and mixed-credential copy from `/wallet`; stored credentials now expose only Corbanu API disconnect behavior.
- [x] Rebuilt `corbanu-debug` and exercised `/wallet` against the production database after legacy deletion; only Receive, Corbanu API, unlock, API disconnect, backup/removal, and refresh actions remain.
- [x] Generalized the x402 parser to preserve heterogeneous chain alternatives and select only the exact confirmed Solana offer; the production Solana-plus-Base challenge now passes typed parsing while retaining its wire fields.
- [x] Corrected the shared unlock-capability lifecycle so one-action grants are removed from TUI state when any signing request begins, while timed grants remain reusable; successful top-ups no longer poison the following account refresh with a daemon `capability_invalid` refusal.
- [x] Replaced the legacy four-row Plan relabeling with the six provider-neutral Corbanu API routes while preserving the supported Ambient GLM 5.2 option; Flash is marked recommended and GLM 5.3 is labeled as using balance faster.
- [x] Replaced `/wallet` pricing referrals in the model picker with each route's explicit input, cache-read, cache-write, and output rates, including the Luna/Sol 272K+ context schedule.

## Remaining

- [ ] Add a distinct Corbanu API crew quick start with Fable Nazgul, Luna Troll, and three Flash Orcs while preserving the existing Standard Crew.
- [ ] Add exact CrewSpec, event-routing, rendered-picker, and true-TUI evidence for the second preset.
- [ ] Named human tester enters the wallet passcode and completes create/revoke plus one-time reveal against the disposable backend.
- [ ] Named human tester repeats top-up, create/revoke, one-time reveal, inference, and recovery flows against the deployed backend.

## Verification

- [x] Focused tests: 15 wallet, 9 wallet-daemon, 58 provider, 4 wallet-API snapshots, and 17 API-only wallet-menu tests pass.
- [x] Integration tests cover provider mapping, operation-preserving daemon IPC, backend public-key conversion, and exact microdollars.
- [x] TUI applicability is resolved and non-secret read-only checkpoints are recorded; human passcode/key lifecycle remains pending.
- [x] Integrated interfaces in `594d618306d922963cf6676d3600cd381922759c`, `865ea2edd2`, and API-only wallet commit `66ff6579d7` match PF-34-S01 and backend `6cc7894`.
- [x] Upstream picker/provider compatibility test passes with exact model-field selection.
- [x] Capability-lifecycle and wallet API/unlock focus suite passes 13/13; the broad 3,842-test TUI run passes 3,811 and retains 31 known unrelated version/copy snapshot failures.
- [x] Corbanu catalog mapping, rendered seven-row picker snapshot, and existing cross-provider picker regression pass 3/3.
- [ ] Named human passcode, top-up, one-time reveal, create/revoke, inference, and recovery acceptance pass against production.

## Exit evidence

- [x] Implementation commits: `594d618306d922963cf6676d3600cd381922759c`, production catalog fix `865ea2edd2`, and legacy retirement `66ff6579d7`.
- [x] Final focused tests pass: 4 wallet-API and 17 API-only wallet-menu tests; the prior broad 3,847-test run passed 3,817 and exposed 30 unrelated pre-existing version-drift snapshots.
- [x] Production true-TUI log is `/tmp/corbanu-api-prod-qa-final.nuVmDv/codex-tui.log` with no panic, malformed-catalog, or unavailable-screen signature.
- [x] API-only wallet true-TUI log is `/tmp/corbanu-no-legacy-final.Zn0gYI/codex-tui.log` with no Plan copy, Plan-status request failure, panic, malformed, or unavailable signature.
- [x] Removed the deleted server credential from the local encrypted store through the API-only confirmation flow; `/tmp/corbanu-credential-clean-final.nzi7jm/codex-tui.log` records the successful disconnect notice and a reopened wallet without the disconnect action or legacy Plan surface.
- [x] The failed production `$20` top-up remained pending with no transaction and no credited balance; the installed `corbanu-debug` now matches the rebuilt parser-fix binary byte-for-byte. Human payment retry remains explicit and pending.
- [x] The production `$10` top-up settled as `NcLSGL398QdVTLoPpZQyUZ9vU9SC1fNwRaK3pCbBNtmfwuT1TKYk93Zk25TtKGAJgtMNuc9nbZYuh65B96QeiqE`; the subsequent stale-capability failure is covered by the one-action/timed lifecycle regressions.
- [x] True-TUI read-only recovery on `06e54c4431` in TensorCash opened `/wallet` -> `Corbanu API` and rendered `$10 available` plus the priced catalog without `capability_invalid`, `Unavailable`, or a second signing/payment request; exact installed binary SHA-256 is `cb4bfa6ae28a1ffbcfc7f19a4118655f00cd95dc526bb43cdf347a0054b6e604`, with trace log at `/tmp/corbanu-capability-qa.t06JLR/codex-tui.log`.
- [x] True-TUI `/model` verification on `fd6f3fde66` opened directly on `Corbanu API` and rendered Ambient GLM 5.2 unchanged alongside the six new routes, with no removed Kimi/direct legacy rows; installed binary SHA-256 is `6501287f8cb5199d02676c576fe0ee0c88559bca2d4068752eacf656b44e6a06`, with trace log at `/tmp/corbanu-ambient-picker-qa.Wq8Vio/codex-tui.log`.
- [x] True-TUI `/model` verification on `80ce83d9c3` rendered direct input, cache-read, cache-write, output, and applicable 272K+ rates inline with no `/wallet` referral; installed binary SHA-256 is `4fb1b0495f5875dd6421fbb97a8cda89bd37321e5ad95a18d85002238a41cfe1`, with trace log at `/tmp/corbanu-inline-prices-qa.MIZEot/codex-tui.log`.
- [x] `Done` and `Remaining` ledgers reflect reality.
- [ ] Completed record moved to `docs/sprints/archive/corbanu-api-balance/`.
