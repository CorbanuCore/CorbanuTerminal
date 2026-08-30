---
sprint_id: "PF-33-S02"
title: "Customer-safe inference response boundary"
status: completed
plan_file: "docs/plans/active/corbanu-api-balance.md"
plan_feature: "PF-33"
execution_order: 4
owner: "Jim Ricketts"
lane: "backend"
write_scope: "src/app.ts, src/customer-response.ts, tests/api-balance.test.ts, tests/customer-response.test.ts, tests/pricing.test.ts"
worktree: "/home/pfrpc/repos/CorbanuAPI"
branch: "feat/corbanu-api-balance"
base_commit: "66097f417815bb094f070bd9733007d27be98725"
depends_on: "PF-33-S01"
created: 2026-08-30
updated: 2026-08-30
---

# PF-33-S02 — Customer-safe inference response boundary

## Execution mandate

- Deliver: structured sanitization for successful and failed JSON/SSE Corbanu API responses so public model identity is stable and internal routing/cost metadata cannot cross the gateway.
- Excludes: Terminal UI, payment UX, deployment, and changes to legacy-plan response compatibility.

## Plan linkage

- Plan: [Corbanu API balance and keys](../../../plans/active/corbanu-api-balance.md)
- Feature: `PF-33`
- Acceptance advanced: the customer response exposes Corbanu identity and token usage without wholesale routing metadata.

## Code boundaries

- Existing: `src/app.ts::proxyUpstreamRequest`
- Added: `src/customer-response.ts`
- Tests: `tests/customer-response.test.ts`, `tests/api-balance.test.ts`, `tests/pricing.test.ts`

## Preconditions

- [x] Plan is active.
- [x] PF-33-S01 is completed and archived.
- [x] Worktree, branch, and base commit match the active plan.
- [x] Backend lane and literal write scope had no active collision.
- [x] Live GLM 5.3 Flash evidence reproduced the generalized transparent-proxy disclosure boundary.

## Done

- [x] Sprint record created and linked to PF-33.
- [x] Live request proved correct at-cost settlement: $0.0000103 upstream cost became an 11-microdollar debit.
- [x] Sanitized structured JSON and SSE responses without regex or prompt-text inspection.
- [x] Rewrote upstream model identity to the selected public Corbanu model.
- [x] Added adjacent JSON, SSE, error, cost, and vendor-metadata regressions.
- [x] Repeated live GLM 5.3 Flash requests and verified provider-neutral customer bodies.

## Remaining

- [x] None.

## Verification

- [x] Focused tests: 12 passed, 0 failed across customer response and API balance suites.
- [x] Full suite: 104 passed, 0 failed; typecheck and build pass.
- [x] Live GLM 5.3 Flash: HTTP 200, public model `corbanu/glm-5.3-flash`, expected assistant output, standard usage, and no routing/vendor/wholesale-cost metadata.
- [x] Live at-cost ledger: two sanitized probes debited 32 microdollars total and left zero reserved balance.
- [x] TUI applicability resolved: not applicable; this correction is the backend response boundary.
- [x] Integrated interfaces and candidate commit match PF-33-S01 evidence.
- [x] No upstream Codex patch is involved.

## Exit evidence

- [x] Implementation commit: `778b4b33445aa452dce09ab416e520e6b4aaeab1`.
- [x] Final-tree automated and live-smoke output recorded.
- [x] `Done` and `Remaining` ledgers reflect reality.
- [x] Completed record moved to `docs/sprints/archive/corbanu-api-balance/`.
