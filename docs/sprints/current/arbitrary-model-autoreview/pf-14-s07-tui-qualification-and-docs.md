---
sprint_id: "PF-14-S07"
title: "TUI qualification and finished documentation"
status: draft
plan_file: "docs/plans/proposed/arbitrary-model-autoreview.md"
plan_feature: "PF-14"
execution_order: 7
owner: "Jim Ricketts"
lane: "autoreview"
write_scope: "UNALLOCATED"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-14-S06"
created: 2026-08-24
updated: 2026-08-27
---

# PF-14-S07 — TUI qualification and finished documentation

## Execution mandate

- Deliver: one complete, human-accepted Autoreview TUI flow with release evidence and finished docs.
- Excludes: new Core policy, providers, model panels, and automatic finding application.

## Plan linkage

- Upstream: [plan touch record](../../../plans/proposed/arbitrary-model-autoreview.md#native-lifecycle-and-upstream-touch-record); resolve this sprint's adapter rows.
- Plan: [Arbitrary-model Autoreview](../../../plans/proposed/arbitrary-model-autoreview.md)
- Feature: `PF-14`
- Acceptance advanced: all plan acceptance flows pass in true TUI against both default live repositories.

## Code boundaries

- Existing: `codex-rs/tui/src/app/thread_routing.rs`; `codex-rs/tui/src/spawn_orchestration.rs`
- Planned: `codex-rs/tui/src/chatwidget/autoreview.rs`; `docs/features/autoreview.md`
- Tests: `codex-rs/tui/src/chatwidget/tests.rs`; `qa/release/<version>/`

## Preconditions

- [ ] Allocate literal implementation/test/registration scopes and check cross-plan collisions before readiness.
- [ ] Plan upstream baseline, adapter ownership, and exact contract tests are resolved before readiness.
- [ ] Plan is active.
- [ ] Dependencies are completed.
- [ ] Worktree, branch, and base commit are exact and match the plan.

## Done

- [x] Sprint record created and linked to PF-14.

## Remaining

- [ ] Render host-owned target/provider/model/effort selection, third-party disclosure, scan state, exact route, packet progress, elapsed time, usage, cancel, failure, and structured findings.
- [ ] Keep Autoreview visually and semantically distinct from `/review` and Guardian approval review.
- [ ] Add TUI state, routing, snapshot, keyboard, cancellation, pane switching, restart, and persisted-result regressions.
- [ ] Launch with `RUST_LOG=trace just codex -c log_dir=<private-temp>`; send prompt text and Enter separately for every flow.
- [ ] Run exact-runtime success, missing-auth, secret-refusal, malformed-result, cancel, and return/inspect flows in an isolated TensorCash worktree.
- [ ] Run mixed visual/code review, cancel, and return/inspect flows in an isolated Isometric Game worktree.
- [ ] Re-run inherited `/review`, Guardian, provider selection, and spawn TUI flows to prove no behavior collision.
- [ ] Obtain human acceptance; run the full performance campaign if this candidate is a due third release.
- [ ] After acceptance only, add `docs/features/autoreview.md`, update skills/provider/spawn docs and MkDocs navigation, and link final candidate evidence.

## Verification

- [ ] Run fix/format before final tests; execute Rust commands below from `codex-rs`.
- [ ] Record applicable upstream adapter evidence or justified non-applicability; structural checks alone are not qualification.
- [ ] Focused test: `just test -p codex-tui autoreview`
- [ ] Core regression: `just test -p codex-core multi_agents_tests`
- [ ] Docs: `mkdocs build --strict`
- [ ] True-TUI evidence: recordings, traces, provider-call ledger, unchanged-diff hashes, and human signoff linked.

## Exit evidence

- [ ] Implementation and documentation commits recorded.
- [ ] Final-tree automated and true-TUI evidence linked.
- [ ] Release record and benchmark decision linked.
- [ ] `Done` and `Remaining` ledgers reflect reality.
- [ ] Completed record moved to `docs/sprints/archive/arbitrary-model-autoreview/`.
