---
sprint_id: "PF-32-S02"
title: "Terminal signed-wallet account operations"
status: completed
plan_file: "docs/plans/active/corbanu-api-balance.md"
plan_feature: "PF-32"
execution_order: 5
owner: "Jim Ricketts"
lane: "backend"
write_scope: "src/app.ts, src/wallet-auth.ts, tests/api-balance.test.ts, tests/wallet-auth.test.ts"
worktree: "/home/pfrpc/repos/CorbanuAPI"
branch: "feat/corbanu-api-balance"
base_commit: "66097f417815bb094f070bd9733007d27be98725"
depends_on: "PF-33-S02"
created: 2026-08-30
updated: 2026-08-30
---

# PF-32-S02 — Terminal signed-wallet account operations

## Execution mandate

- Deliver: replay-safe signed-wallet JSON operations for top-up intent creation, account inspection, key creation, and key revocation so the Rust wallet can drive the Corbanu API lifecycle.
- Excludes: payment settlement changes, inference routing, Terminal rendering, and deployment.

## Plan linkage

- Plan: [Corbanu API balance and keys](../../../plans/active/corbanu-api-balance.md)
- Feature: `PF-32`
- Acceptance advanced: an unlocked wallet can fund its balance and manage independently revocable API keys without exposing its seed.

## Code boundaries

- Existing: `src/app.ts::createGatewayApp`, `src/wallet-auth.ts::verifyWalletChallenge`
- Planned: signed-wallet operation request boundary in `src/app.ts`
- Tests: `tests/api-balance.test.ts`

## Preconditions

- [x] Plan is active.
- [x] PF-33-S02 is completed and archived.
- [x] Worktree, branch, and base commit are exact and match the plan.
- [x] Backend lane and literal write scope have no active collision.
- [x] Existing wallet-challenge contract is the product-owned adapter; no upstream Codex patch is involved.

## Done

- [x] Sprint record created and linked to PF-32.
- [x] Added one typed, replay-safe signed-wallet verification boundary for account operations.
- [x] Exposed top-up intent, account, create-key, and revoke-key operations through that boundary.
- [x] Added success, replay, changed-operation, changed-wallet, and revocation regressions.

## Remaining

- [x] None.

## Verification

- [x] Focused test: 14 passed, 0 failed across API balance and wallet-auth suites.
- [x] Integration test: 106 passed, 0 failed; typecheck and build pass.
- [x] TUI applicability resolved: supporting backend contract only; interactive proof belongs to PF-34.
- [x] Integrated interfaces and candidate commit match PF-33 evidence.
- [x] Upstream adapter contract: no upstream interface is changed.

## Exit evidence

- [x] Implementation commit: `cd79361d8b4f286291556a641288757d0451f52c`.
- [x] Final-tree focused and full-suite output recorded.
- [x] `Done` and `Remaining` ledgers reflect reality.
- [x] Completed record moved to `docs/sprints/archive/corbanu-api-balance/`.
