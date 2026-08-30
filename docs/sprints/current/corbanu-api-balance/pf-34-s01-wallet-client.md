---
sprint_id: "PF-34-S01"
title: "Rust wallet Corbanu API client"
status: in_progress
plan_file: "docs/plans/active/corbanu-api-balance.md"
plan_feature: "PF-34"
execution_order: 6
owner: "Jim Ricketts"
lane: "terminal-wallet"
write_scope: "codex-rs/wallet/src/corbanu_api.rs, codex-rs/wallet/src/corbanu_api_tests.rs, codex-rs/wallet/src/envelope.rs, codex-rs/wallet/src/lib.rs, codex-rs/wallet/src/payment.rs, codex-rs/wallet-daemon/src/client.rs, codex-rs/wallet-daemon/src/protocol.rs, codex-rs/wallet-daemon/src/protocol_tests.rs, codex-rs/wallet-daemon/src/server.rs"
worktree: "/home/pfrpc/repos/CorbanuTerminal"
branch: "feat/corbanu-api-wallet"
base_commit: "4ff38e974b4e63cebffc5d608c5584e2d453cf1b"
depends_on: "PF-32-S02"
created: 2026-08-30
updated: 2026-08-30
---

# PF-34-S01 — Rust wallet Corbanu API client

## Execution mandate

- Deliver: narrowly scoped wallet-daemon operations for account inspection, arbitrary USDC top-up, API-key creation, and API-key revocation through operation-bound challenges.
- Excludes: TUI rendering, model picker/provider mapping, deployment, and legacy-period removal.

## Plan linkage

- Plan: [Corbanu API balance and keys](../../../plans/active/corbanu-api-balance.md)
- Feature: `PF-34`
- Acceptance advanced: the Terminal wallet can safely drive every Corbanu API account mutation without exposing seed material.

## Code boundaries

- Existing: `codex-rs/wallet/src/payment.rs`, `codex-rs/wallet-daemon/src/{protocol,client,server}.rs`
- Planned: typed Corbanu API account operation and result types in the same wallet boundary
- Tests: wallet payment and wallet-daemon protocol/server tests

## Preconditions

- [x] Plan is active.
- [x] PF-32-S02 is completed and archived.
- [x] Worktree, branch, and base commit are exact and match the plan.
- [x] Terminal-wallet lane and literal write scope have no active collision.
- [x] The product-owned backend challenge adapter is recorded in the plan; no upstream patch is required.

## Done

- [x] Sprint record created and linked to PF-34.

## Remaining

- [ ] Add typed account, model-price, key-summary, top-up, and mutation result contracts.
- [ ] Execute operation-bound challenges and exact x402 top-ups inside the unlocked wallet boundary.
- [ ] Expose scoped daemon requests with one-action and cancellation behavior.
- [ ] Add protocol and tamper/failure regressions.

## Verification

- [ ] Focused test: `just test -p codex-wallet -p codex-wallet-daemon`
- [ ] Integration test: wallet daemon end-to-end operation tests.
- [x] TUI applicability resolved: not yet; rendering and true-TUI proof belong to PF-34-S02.
- [ ] Integrated interfaces and candidate commit match backend `cd79361d8b4f286291556a641288757d0451f52c`.
- [x] Upstream adapter contract: product-owned wallet boundary only.

## Exit evidence

- [ ] Implementation commit recorded.
- [ ] Final-tree test output linked.
- [ ] `Done` and `Remaining` ledgers reflect reality.
- [ ] Completed record moved to `docs/sprints/archive/corbanu-api-balance/`.
