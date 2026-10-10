---
sprint_id: "PF-53-S01"
title: "Multi-provider onboarding"
status: completed
plan_file: "docs/plans/active/unified-provider-auth.md"
plan_feature: "PF-53"
execution_order: 12
owner: "GPT-5.6 Sol high implementation agent"
parallel_lane: "provider-auth-serial"
write_scope: "codex-rs/provider-auth/src/api_key_flow.rs; codex-rs/provider-auth/src/api_key_flow_tests.rs; codex-rs/provider-auth/src/auth_flow.rs; codex-rs/provider-auth/src/auth_flow_tests.rs; codex-rs/provider-auth/src/lib.rs; codex-rs/tui/src/provider_auth_effect_executor.rs; codex-rs/tui/src/provider_status_host.rs; codex-rs/tui/src/provider_account_auth_host.rs; codex-rs/tui/src/provider_account_auth_host_tests.rs; codex-rs/tui/src/onboarding/provider_setup.rs; codex-rs/tui/src/onboarding/provider_setup_tests.rs; codex-rs/tui/src/onboarding/mod.rs; codex-rs/tui/src/onboarding/auth.rs; codex-rs/tui/src/onboarding/onboarding_screen.rs; codex-rs/tui/src/onboarding/snapshots/; codex-rs/tui/src/config_update.rs; codex-rs/tui/src/lib.rs; codex-rs/tui/src/app.rs; codex-rs/tui/src/app_event.rs; codex-rs/tui/src/app/event_dispatch.rs; codex-rs/tui/src/app/test_support.rs; codex-rs/tui/src/app/tests.rs; codex-rs/tui/src/chatwidget.rs; codex-rs/tui/src/chatwidget/constructor.rs; codex-rs/tui/src/chatwidget/protocol.rs; codex-rs/tui/src/chatwidget/provider_credentials.rs; codex-rs/tui/src/chatwidget/wallet_menu.rs; codex-rs/tui/src/chatwidget/wallet_receipt.rs; codex-rs/tui/src/bottom_pane/vault_secret_entry.rs; codex-rs/tui/src/bottom_pane/wallet_recovery.rs; codex-rs/tui/tests/suite/multi_provider_onboarding.rs; codex-rs/tui/tests/suite/mod.rs; docs/sprints/current/unified-provider-auth/pf-53-s01-multi-provider-onboarding.md"
integration_gate: "PF-53 configure-many onboarding, reusable API-key/status/account hosts, typed App-level deferred Corbanu continuation, correlated nonblocking Plan credential persistence, typed receipt selection policy, and true-TMUX tests only; wallet/Plan behavior is otherwise reused; bottom-pane additions are cancellation callback seams only; chatwidget.rs and constructor.rs add only Claude adapter visibility plus monotonic/current Plan-persistence correlation fields; app test fixtures only initialize PF-53 fields to None; no /providers deactivation UI, startup heuristic removal, credential removal, PF-54+, or untyped secret/event history"
worktree: "/home/pfrpc/repos/worktrees/corbanu-main-f7356a94e0"
branch: "feat/unified-provider-auth"
base_commit: "f7356a94e032234022a462d65b576a7de2854859"
depends_on: "PF-52-S01"
created: 2026-09-01
updated: 2026-09-01
---

# PF-53-S01 — Multi-provider onboarding

## Execution mandate

- Deliver configure-many onboarding with **Done** and deferred Corbanu Plan execution.
- Exclude later deactivation controls and startup-wide heuristic removal.

## Plan linkage

- Plan: [Unified provider onboarding and management](../../../plans/active/unified-provider-auth.md).
- Feature: `PF-53`.
- Acceptance advanced: fresh users can set up multiple providers before optional Plan onboarding.

## Code boundaries

- Shared provider-auth contracts feed renderer-neutral setup/session decisions into onboarding and App hosts.
- Qualified login/vault and PF-51/PF-52 adapters retain credential custody and account-flow settlement.
- Existing wallet/Plan views execute the typed deferred continuation; focused callback seams preserve cancellation.

## Preconditions

- [x] Plan is active; PF-52-S01 completed at `a723374834` and archived at `1fe8d4ffb9`.
- [x] Exact serial allocation and all narrow scope additions were authorized.
- [x] Wallet/Plan remains behind its existing protected boundary; no wallet logic was duplicated.

## Done

- [x] Shared PF-48 catalog/PF-49 metadata statuses render before and after every attempt.
- [x] Generic API-key effects map `ApiKeyStorage` to qualified persistence and remain reusable by PF-54.
- [x] **Done** requires a usable provider or queued Plan; forced single-provider policy still terminates.
- [x] Corbanu queues only, launches after **Done**, activates after success, and does not replace a usable current provider.
- [x] Cancel continues with a fallback or returns the only-Plan path to the same shared provider host.
- [x] Existing current selection is preserved; fresh profiles persist the first usable success with a compatible model.
- [x] PF-51/PF-52 account reducers/adapters provide correlated success, failure, cancellation, and stale-result handling.
- [x] Qualified account discovery is metadata-only; selected-provider reconciliation avoids N× vault reads.
- [x] Late account metadata cannot resolve/re-render over an active authentication challenge.
- [x] Fresh, locked, and unlocked wallet continuations preserve cancellation through every protected view.
- [x] Snapshot, reducer, adapter, App-boundary, and five true-TMUX scenarios were added.
- [x] Narrow `vault_secret_entry.rs` and `wallet_recovery.rs` additions only expose cancellation callbacks.
- [x] `chatwidget.rs` and `chatwidget/constructor.rs` additionally hold only monotonic/current attempt state for correlated, nonblocking Plan credential persistence; `wallet_receipt.rs` applies the typed settlement selection policy so deferred onboarding retains activation ownership; App fixtures only initialize their own PF-53 fields to `None`.
- [x] The full five-case true-TMUX matrix, final scoped gates, and primary integration review completed on the exact final candidate.

## Remaining

- [x] Primary reviewed and integrated the final tree in implementation commit `30b595034b`; this completed record is archived with no implementation item remaining.

## Verification

- [x] `CARGO_INCREMENTAL=0 cargo test -p codex-provider-auth --tests -- --nocapture`: 49 passed.
- [x] `cargo test -p codex-tui provider_setup --lib -- --nocapture`: 9 passed.
- [x] `cargo test -p codex-tui provider_account_auth_host --lib -- --nocapture`: 2 passed.
- [x] Focused multi-provider picker snapshot: 1 passed.
- [x] App lazy-refresh regression: 1 passed; resolver was not invoked during `Authenticating`.
- [x] Plan settlement boundary regressions passed: immediate nonblocking receipt, redacted completion, stale-attempt rejection, deferred-vs-wallet selection policy, bounded optional enrichment, targeted correlated Corbanu status, single authoritative receipt ownership, and complete cancellation teardown.
- [x] Final `just fix -p codex-provider-auth`, `just fix -p codex-tui`, `cargo fmt --all -- --check`, and `git diff --check` passed. Provider-auth passed 49/49; provider setup 9/9; account host 2/2; focused settlement and cancellation regressions passed.
- [x] Full serial zero-retry TMUX matrix passed 9/9 in 537.22s: four harness-safety tests and all five required product journeys.
- [x] Governance passed with `python3 docs/plans/check.py` (2/2 active) and `python3 docs/sprints/check.py` (62 current/108 archived before PF-53 archival).
- [x] The broad 3,889-test TUI library run reached unrelated advanced-reasoning coverage, then aborted on the pre-existing `fork_current_session_preserves_conversation_ultra` stack overflow; no PF-53 test failed before the abort.
- [x] Primary-owned implementation commit `30b595034b` records the exact tested tree before archival.

## Exit evidence

- [x] Exact final candidate binary SHA-256: `e73de13086a70cc319c6f8a997c013653ae4a0553d84e68e7a96181cfa48d2a8`.
- [x] TMUX harness records binary hash, key events, view/log/request artifacts, generated-canary scans, and session cleanup.
- [x] New production modules remain below 500 lines: account host 357, executor 245, status host 375, session 305.
- [x] Encrypted-vault save regression was reproduced and fixed by selected-provider-only metadata resolution.
- [x] Final success artifacts are `pf53-configure-many-restart-request`, `pf53-deferred-fallback-cancel`, `pf53-fresh-wallet-plan-success`, `pf53-locked-wallet-failure-retry`, and `pf53-only-plan-return`; each records the exact final binary hash plus viewport, scrollback, isolated config/log, and canary checks.
- [x] Primary review confirmed literal write scope, secret-free typed events/snapshots, targeted provider reconciliation, active-by-default eligibility, first-success/current-provider invariants, and no PF-54 deactivation behavior.
- [x] Implementation commit `30b595034b` is recorded and this completed ledger is archived.
