---
sprint_id: "PF-33-S01"
title: "Versioned at-cost metering and route activation"
status: completed
plan_file: "docs/plans/active/corbanu-api-balance.md"
plan_feature: "PF-33"
execution_order: 3
owner: "Jim Ricketts"
lane: "backend"
write_scope: "src/app.ts, src/config.ts, src/index.ts, src/money.ts, src/models.ts, src/postgres-store.ts, src/pricing.ts, src/store.ts, src/usage.ts, src/vercel.ts, src/xapi-tenant-accounts.ts, src/xapi.ts, tests/api-balance.test.ts, tests/config.test.ts, tests/postgres-store.test.ts, tests/pricing.test.ts, tests/vercel-routing.test.ts, tests/xapi-routing.test.ts, tests/xapi-tenant-accounts.test.ts, tests/usage.test.ts"
worktree: "/home/pfrpc/repos/CorbanuAPI"
branch: "feat/corbanu-api-balance"
base_commit: "66097f417815bb094f070bd9733007d27be98725"
depends_on: "PF-32-S01"
created: 2026-08-30
updated: 2026-08-30
---

# PF-33-S01 — Versioned at-cost metering and route activation

## Execution mandate

- Deliver: a provider-neutral six-model catalog with zero-markup versioned prices and atomic dollar metering.
- Excludes: Terminal wallet/key UI, deployment, migration execution, and release qualification.

## Plan linkage

- Plan: [Corbanu API balance and keys](../../../plans/active/corbanu-api-balance.md)
- Feature: `PF-33`
- Acceptance advanced: funded API keys can see and call priced routes; debit equals the pinned upstream schedule.

## Code boundaries

- Existing: `src/app.ts::createGatewayApp`, `src/store.ts::GatewayStore`, `src/postgres-store.ts`, `src/vercel.ts`, `src/models.ts`
- Added: `src/pricing.ts`
- Tests: `tests/pricing.test.ts`, balance/routing/usage/PostgreSQL regressions

## Preconditions

- [x] Plan is active.
- [x] PF-31-S01 and PF-32-S01 are completed and archived.
- [x] Worktree, branch, and base commit are exact and match the plan.
- [x] Lane and literal write scopes are allocated; active-slot and collision checks pass.
- [x] Plan upstream-touch rows cover the product-owned backend adapter and contract tests.
- [x] Alex Good approved upstream cost with zero markup on 2026-08-30.

## Done

- [x] Sprint record created and linked to PF-33.
- [x] Live Vercel and xAPI public catalogs were re-read on 2026-08-30 without exposing credentials.
- [x] Added immutable price-schedule versions and provider-neutral customer price metadata.
- [x] Added atomic microdollar reserve, settle, release, and request attribution in both stores.
- [x] Activated six routes only when their protected backend is configured.
- [x] Routed requests with fresh upstream authorization and exact usage settlement.
- [x] Added generalized price, concurrency, idempotency, insufficient-balance, and secret-boundary regressions.

## Remaining

- [x] None.

## Verification

- [x] Focused tests: 35 passed, 0 failed across pricing, API balance, Vercel, and xAPI routing.
- [x] Full package suite: 101 passed, 0 failed; typecheck and build pass.
- [x] PostgreSQL integration: 13 passed, 0 failed against disposable PostgreSQL 16, including concurrent reserve and stale-settlement coverage.
- [x] TUI applicability resolved: not applicable; PF-33 is backend-only and PF-34 owns interactive UI.
- [x] Integrated interfaces and candidate commit match PF-31/PF-32 evidence.
- [x] Product-owned adapter contracts pass; no upstream Codex patch is involved.
- [x] LLM request-path source guard passes with no regex-dependent request behavior.

## Exit evidence

- [x] Implementation commit: `6aa81161ece53b26915f05c3346a9ebe11b094fd`.
- [x] Final-tree test output recorded in this sprint.
- [x] `Done` and `Remaining` ledgers reflect reality.
- [x] Completed record moved to `docs/sprints/archive/corbanu-api-balance/`.
