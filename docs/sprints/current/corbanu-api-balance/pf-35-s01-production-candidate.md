---
sprint_id: "PF-35-S01"
title: "Corbanu API production candidate"
status: in_progress
plan_file: "docs/plans/active/corbanu-api-balance.md"
plan_feature: "PF-35"
execution_order: 8
owner: "Jim Ricketts"
lane: "production-candidate"
write_scope: "Dockerfile, fly.toml, src, tests"
worktree: "/home/pfrpc/repos/CorbanuAPI"
branch: "feat/corbanu-api-balance"
base_commit: "66097f417815bb094f070bd9733007d27be98725"
depends_on: "PF-32-S02, PF-33-S02"
created: 2026-08-30
updated: 2026-08-30
---

# PF-35-S01 — Corbanu API production candidate

## Execution mandate

- Deliver: deploy the private Corbanu API backend candidate, retire all legacy-plan authorization and state, and prove production health, wallet-auth, catalog, price, persistence, and provider-neutral inference boundaries.
- Excludes: public release publication, final Terminal human acceptance, and declaring the plan complete.

## Plan linkage

- Plan: [Corbanu API balance and keys](../../../plans/active/corbanu-api-balance.md)
- Feature: `PF-35`
- Acceptance advanced: a production-backed endpoint is available for the PF-34 wallet UI primary and recovery tests without exposing provider credentials or internal route identity.

## Code boundaries

- Existing: `Dockerfile`, `fly.toml`, `src/index.ts`, `src/config.ts`, `src/postgres-store.ts`
- Planned: no new implementation unless production qualification finds a generalized defect within the allocated backend scope.
- Tests: `tests/api-balance.test.ts`, `tests/customer-response.test.ts`, `tests/postgres-store.test.ts`, `tests/vercel-routing.test.ts`, `tests/wallet-auth.test.ts`

## Preconditions

- [x] Plan is active and explicitly permits two disjoint active lanes.
- [x] PF-32-S02 and PF-33-S02 are completed and archived.
- [x] Worktree, branch, and base commit are exact and match the plan.
- [x] Lane and literal write scopes are allocated; PF-34 uses a distinct worktree and disjoint scope.
- [x] Plan upstream-touch rows cover the provider adapter and customer-response boundary.

## Done

- [x] Sprint record created and linked to PF-35.
- [x] Fly authentication restored and the production app, compatibility hostname, current health, and private repository visibility verified.
- [x] Final backend tree at `6cc7894` passes 107 tests, typecheck, and production build.
- [x] Staged the protected Vercel gateway credential from the encrypted local vault without printing its value.
- [x] Deployed image `deployment-01M19ES7MMEKBX1MMDD6QR0AJ7`; Fly machines `2870663bd33de8` and `784503db5e0398` are version 13 with passing readiness checks.
- [x] Production smoke confirms `/readyz` 200, six provider-neutral at-cost catalog entries, GLM 5.3 Flash recommended at `$0.15` input, no vendor metadata, legacy `/v1/plans` retirement, and a no-store signed-wallet challenge.
- [x] Exercised the deployed read-only flow in the real Terminal at `865ea2edd2`; `/wallet` renders the zero-dollar account, arbitrary top-up, key actions, and all six prices without a catalog error.
- [x] Audited and transactionally deleted 17 legacy periods, 20 weekly windows, 1,910 legacy inference rows, 34 legacy credentials, one legacy-funded xAPI tenant record, and dependent legacy limit/accounting rows.
- [x] Post-delete production audit reports zero legacy periods, windows, inference rows, credentials, and legacy xAPI funding; Corbanu API account, top-up, and inference tables were unchanged.

## Remaining

- [ ] Named human tester unlocks the production wallet and completes top-up, create/revoke, one-time reveal, and recovery checks.
- [ ] Run one low-cost inference call after the human creates a funded Corbanu API key; no balance was fabricated for qualification.
- [ ] Complete final persistence/compliance/release qualification before public release.

## Verification

- [x] Focused and full backend suites pass on the deployed commit.
- [x] Production health and database-backed startup are stable across both machines.
- [x] Customer catalog responses expose Corbanu model and price identities without vendor metadata.
- [x] TUI applicability is resolved by the production true-PTY run recorded in PF-34-S02.
- [x] Integrated interfaces match Terminal backend client `cd79361d8b4f286291556a641288757d0451f52c`, read-only catalog fix `865ea2edd2`, and API-only wallet commit `66ff6579d7`.
- [x] Production database retirement audit and API-only `/wallet` true-PTY evidence show legacy state cannot reappear through supported product flows.
- [ ] Human-funded inference and wallet key-lifecycle acceptance pass against production.

## Exit evidence

- [x] Deployment image `deployment-01M19ES7MMEKBX1MMDD6QR0AJ7` and source commit `6cc7894` recorded.
- [x] Production smoke output recorded without secrets or plaintext customer keys.
- [x] `Done` and `Remaining` ledgers reflect reality.
- [ ] Completed record moved to `docs/sprints/archive/corbanu-api-balance/` after PF-34 handoff.
