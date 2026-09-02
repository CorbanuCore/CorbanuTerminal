---
sprint_id: "PF-42-S01"
title: "Plan gateway payload limit"
status: completed
plan_file: "docs/plans/active/corbanu-plan-large-image-requests.md"
plan_feature: "PF-42"
execution_order: 1
owner: "Jim Ricketts — Corbanu Plan gateway lane"
parallel_lane: "plan-gateway-payload"
write_scope: "src/app.ts, tests/app.test.ts, tests/xapi-routing.test.ts, README.md, docs/plans/active/corbanu-plan-large-image-requests.md, docs/sprints/current/corbanu-plan-large-image-requests/"
integration_gate: "Jim Ricketts receives the private CorbanuPlan branch, audits the exact 30 MiB error/documentation contract, reruns the complete private-service suite and governance checks, and requires deployment plus live image qualification before release."
worktree: "/Volumes/CorbanuDrive/Corbanu/CorbanuPlan"
branch: "fix/30mb-inference-body-limit"
base_commit: "e333167b7a98c26053ef76fe62070da2edc83285"
depends_on: "none"
created: 2026-08-30
updated: 2026-08-30
---

# PF-42-S01 — Plan gateway payload limit

## Execution mandate

- Deliver: Raise authenticated chat and Messages JSON bodies from 2 MiB to 30 MiB with accurate rejection and regression coverage.
- Excludes: client transcoding, unbounded bodies, billing/routing changes, deployment and release publication.

## Plan linkage

- Plan: [Corbanu Plan large-image requests](../../../plans/active/corbanu-plan-large-image-requests.md).
- Feature: `PF-42`.
- Acceptance advanced: Fable receives valid image-bearing requests above the former 2 MiB cap.

## Code boundaries

- Existing: private `CorbanuPlan/src/app.ts::MAX_PROXY_BODY_BYTES` and Express error middleware.
- Planned: private gateway constant/error text plus README security property.
- Tests: private `tests/app.test.ts` and `tests/xapi-routing.test.ts`.

## Preconditions

- [x] Plan is active and the product decision accepts the authenticated-service resource tradeoff.
- [x] Dependencies are completed; this sprint has none.
- [x] Worktree, branch and base commit are exact and match the plan.
- [x] Parallel owner, lane, disjoint scopes and receiving integration gate are recorded.

## Done

- [x] Sprint record created and linked to the single PF-42 feature.
- [x] Private deployable repository located and checked out on the recorded branch.
- [x] Raised the authenticated chat and Messages cap to 30 MiB and derived the exact 413 text from the same constant.
- [x] Updated the private service security property to document the 30 MiB cap.
- [x] Added a Fable image-bearing Messages regression above 2 MiB and moved the upper-bound rejection regression to 30 MiB.
- [x] Focused regressions pass 2/2; the full private-service suite passes 64/64 and the production build passes.
- [x] Plan and sprint governance checks pass on the final documentation tree.
- [x] Corrected Opus 5 Max review returned `CLEAN` after the accurate account-limit label, zero-copy proxy body and stronger body-integrity regression were added.
- [x] Private gateway implementation committed as `35698c4d5d5ef8c596041450fb415a699a39a5e9`.
- [x] Unchanged test-typecheck baseline debt is explicitly deferred; production build typechecking passes.

## Remaining

None. Deployment, live TUI qualification and human acceptance remain plan/release gates rather than private gateway implementation tasks.

## Verification

- [x] Focused tests cover the former failure, accurate account limit and new upper bound; 3/3 pass.
- [x] Full private-service tests pass 65/65 and `tsc -p tsconfig.build.json` passes on the final tree.
- [x] Full test typecheck disposition is recorded: current failures are unchanged baseline errors in `tests/store.test.ts` and existing `seen[0]` accesses in `tests/xapi-routing.test.ts`; the production build typecheck passes.
- [x] `python3 docs/plans/check.py && python3 docs/sprints/check.py` passes.
- [x] TUI applicability is resolved: post-deployment Isometric Game image success, over-limit failure and smaller-request recovery remain explicit release blockers.

## Exit evidence

- [x] Implementation commit recorded: `35698c4d5d5ef8c596041450fb415a699a39a5e9`.
- [x] Final-tree focused, full-suite, build and governance results are summarized in the active plan.
- [x] Handoff records private branch `fix/30mb-inference-body-limit`, the four-file scope, focused/full/build commands and clean Opus review.
- [x] `Done` and `Remaining` ledgers reflect reality; live/TUI and human gates remain in the active plan.
