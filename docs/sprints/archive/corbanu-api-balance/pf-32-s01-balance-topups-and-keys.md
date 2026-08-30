---
sprint_id: "PF-32-S01"
title: "Dollar balance, arbitrary top-ups, and key lifecycle"
status: completed
plan_file: "docs/plans/active/corbanu-api-balance.md"
plan_feature: "PF-32"
execution_order: 2
owner: "Jim Ricketts"
lane: "backend"
write_scope: "src/app.ts, src/index.ts, src/store.ts, src/postgres-store.ts, src/x402.ts, src/money.ts, src/token.ts, tests/api-balance.test.ts, tests/postgres-store.test.ts"
worktree: "/home/pfrpc/repos/CorbanuAPI"
branch: "feat/corbanu-api-balance"
base_commit: "66097f417815bb094f070bd9733007d27be98725"
depends_on: "PF-31-S01"
created: 2026-08-30
updated: 2026-08-30
---

# PF-32-S01 — Dollar balance, arbitrary top-ups, and key lifecycle

## Execution mandate

- Deliver: Replace new tier sales with wallet-owned integer-microdollar balances, amount-bound USDC top-up intents, and multiple one-time-reveal API keys while preserving unexpired legacy periods.
- Excludes: Priced inference activation, provider price selection, Terminal UI, production deployment, and conversion of legacy token allowances.

## Plan linkage

- Plan: [Corbanu API balance and keys](../../../plans/active/corbanu-api-balance.md)
- Feature: `PF-32`
- Acceptance advanced: Settled canonical-USDC top-ups credit an idempotent shared wallet balance and authorize separately revocable keys without adding a new plan period.
- Upstream: [plan touch record](../../../plans/active/corbanu-api-balance.md#upstream-touch-record); standalone backend domain work.

## Code boundaries

- Store and key contracts: `src/store.ts`, `src/postgres-store.ts`, `src/token.ts`
- Payment and API boundaries: `src/x402.ts`, `src/app.ts`, `src/index.ts`
- Exact decimal parsing: planned `src/money.ts`
- Tests: planned `tests/api-balance.test.ts`; PostgreSQL fixtures in `tests/postgres-store.test.ts`

## Preconditions

- [x] Plan is active.
- [x] PF-31-S01 is completed and archived.
- [x] Worktree, branch, and base commit match the plan.
- [x] Backend lane and write scope do not collide with another sprint in this plan.
- [x] Shared-wallet balance, one-USDC-to-one-dollar credit, legacy grandfathering, and one-time plaintext key behavior are recorded product decisions.

## Done

- [x] Sprint record created and linked to PF-32.
- [x] Added exact microdollar parsing and formatting with no floating-point money.
- [x] Added idempotent top-up intents, settlements, and shared wallet balances to memory and PostgreSQL stores.
- [x] Permitted key creation for funded API accounts or active legacy periods; plaintext remains response-only.
- [x] Replaced public tier-sale payment routes with wallet-authenticated, amount-bound top-up creation and settlement routes.
- [x] Added wallet/key balance account responses while preserving unexpired legacy entitlement behavior.
- [x] Proved duplicate settlement, payer/amount/expiry rejection, concurrent first-key issuance, one-time reveal, multiple keys, and legacy compatibility.
- [x] Ran typecheck, focused tests, full package tests, and build.

## Remaining

- [x] None.

## Verification

- [x] Focused store/API tests: 31 passed, 0 failed.
- [x] PostgreSQL integration environment was unavailable (`TEST_DATABASE_URL` absent); typed schema/query coverage and concurrency fixture were added and compile, while the suite records the database group as skipped.
- [x] Full package suite: 92 passed, 0 failed; typecheck and build pass.
- [x] TUI applicability resolved: none; customer UI is PF-34.
- [x] Final candidate preserves legacy periods and removes static tier checkout from the x402 route registry; legacy paths are acknowledgment-only.

## Exit evidence

- [x] Implementation commit: `00a410be45d6f463e04d6342255df864af56a92b`.
- [x] Final-tree test output recorded above.
- [x] `Done` and `Remaining` ledgers reflect reality.
- [x] Completed record moved to `docs/sprints/archive/corbanu-api-balance/`.
