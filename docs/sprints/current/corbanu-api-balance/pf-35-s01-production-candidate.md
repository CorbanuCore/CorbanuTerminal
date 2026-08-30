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

- Deliver: deploy the private Corbanu API backend candidate behind the existing compatibility hostname and prove its production health, wallet-auth, catalog, price, persistence, and provider-neutral inference boundaries.
- Excludes: public release publication, deleting legacy entitlements, final Terminal human acceptance, and declaring the plan complete.

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

## Remaining

- [ ] Run final-tree backend tests, typecheck, and production build.
- [ ] Stage the protected Vercel credential without exposing its value.
- [ ] Deploy with rolling health checks and verify both production machines.
- [ ] Smoke-test readiness, provider-neutral catalog and prices, signed-wallet challenge, persistence, legacy compatibility, and one low-cost inference call.
- [ ] Record the candidate deployment and hand the endpoint to PF-34 human UI testing.

## Verification

- [ ] Focused and full backend suites pass on the deployed commit.
- [ ] Production health and database migration are stable across both machines.
- [ ] Customer responses expose Corbanu model and price identities without vendor metadata.
- [ ] TUI applicability is deferred to PF-34-S02 against this deployed candidate.
- [ ] Integrated interfaces match Terminal backend client commit `cd79361d8b4f286291556a641288757d0451f52c`.

## Exit evidence

- [ ] Deployment image and source commit recorded.
- [ ] Production smoke output recorded without secrets or plaintext customer keys.
- [ ] `Done` and `Remaining` ledgers reflect reality.
- [ ] Completed record moved to `docs/sprints/archive/corbanu-api-balance/` after PF-34 handoff.
