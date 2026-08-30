---
sprint_id: "PF-31-S01"
title: "Provider-neutral Vercel backend adapter"
status: completed
plan_file: "docs/plans/active/corbanu-api-balance.md"
plan_feature: "PF-31"
execution_order: 1
owner: "Jim Ricketts"
lane: "backend"
write_scope: "src/config.ts, src/models.ts, src/vercel.ts, tests/config.test.ts, tests/vercel-routing.test.ts"
worktree: "/home/pfrpc/repos/CorbanuAPI"
branch: "feat/corbanu-api-balance"
base_commit: "66097f417815bb094f070bd9733007d27be98725"
depends_on: "none"
created: 2026-08-30
updated: 2026-08-30
---

# PF-31-S01 — Provider-neutral Vercel backend adapter

## Execution mandate

- Deliver: Add a protected Vercel upstream adapter and typed internal route definitions for GLM 5.3 Flash, GLM 5.3, GPT-5.6 Luna, and GPT-5.6 Sol without activating unfinished customer pricing or exposing the vendor publicly.
- Excludes: Dollar ledger, top-up endpoints, customer sell-price activation, xAPI changes, Terminal UI, deployment, and migration.

## Plan linkage

- Plan: [Corbanu API balance and keys](../../../plans/active/corbanu-api-balance.md)
- Feature: `PF-31`
- Acceptance advanced: Approved Vercel models have one typed internal route with protected credential substitution and no provider-name leakage.
- Upstream: [plan touch record](../../../plans/active/corbanu-api-balance.md#upstream-touch-record); standalone backend, no upstream Codex files.

## Code boundaries

- Existing: `src/models.ts`, `src/config.ts`
- Added: `src/vercel.ts`; route definitions remain non-customer-visible until pricing activation.
- Tests: `tests/config.test.ts`, `tests/vercel-routing.test.ts`

## Preconditions

- [x] Plan was active before implementation.
- [x] Dependencies were completed.
- [x] Worktree, branch, and base commit matched the plan.
- [x] Lane and write scope were allocated without a collision.
- [x] Live catalog verification established the four upstream model IDs.

## Done

- [x] Sprint record created and linked to one plan feature.
- [x] Added file-or-environment Vercel credential configuration without logging secret values.
- [x] Added a typed Vercel request target that translates internal route identity to the official upstream model ID.
- [x] Defined the four routes as inactive pending approved customer prices.
- [x] Proved authorization replacement, URL/path preservation, model translation, and provider-name absence from customer metadata.
- [x] Ran typecheck, focused tests, the full package test suite, and package build on the final sprint tree.

## Remaining

- [x] None.

## Verification

- [x] Focused tests: `corepack pnpm exec tsx --test --test-concurrency=1 tests/config.test.ts tests/vercel-routing.test.ts` — 12 passed.
- [x] Integration test: `corepack pnpm test` — 86 passed, 0 failed.
- [x] Typecheck/build: `corepack pnpm typecheck && corepack pnpm build` — passed.
- [x] TUI applicability resolved: none; this sprint exposes no customer-visible route or UI.
- [x] Integrated interfaces and candidate commit match dependency evidence.
- [x] Upstream adapter contracts pass; no upstream Codex patch is introduced.

## Exit evidence

- [x] Implementation commit: `ef31361e5becfabc971db7a3670ed340433f18ea`.
- [x] Final-tree test output recorded above.
- [x] `Done` and `Remaining` ledgers reflect reality.
- [x] Completed record moved to `docs/sprints/archive/corbanu-api-balance/`.
