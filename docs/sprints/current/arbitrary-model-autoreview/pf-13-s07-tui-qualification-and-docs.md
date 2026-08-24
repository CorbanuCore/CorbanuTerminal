---
sprint_id: "PF-13-S07"
title: "TUI qualification and finished documentation"
status: draft
plan_file: "docs/plans/proposed/arbitrary-model-autoreview.md"
plan_feature: "PF-13"
execution_order: 7
owner: "Jim Ricketts"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-13-S06"
created: 2026-08-24
updated: 2026-08-24
---

# PF-13-S07 — TUI qualification and finished documentation

## Execution mandate

- Deliver: one complete, human-accepted Autoreview TUI flow with release evidence and finished docs.
- Excludes: new Core policy, providers, model panels, and automatic finding application.

## Plan linkage

- Plan: [Arbitrary-model Autoreview](../../../plans/proposed/arbitrary-model-autoreview.md)
- Feature: `PF-13`
- Acceptance advanced: all plan acceptance flows pass in true TUI against both default live repositories.

## Code boundaries

- Existing: `codex-rs/tui/src/app/thread_routing.rs`; `codex-rs/tui/src/spawn_orchestration.rs`
- Planned: `codex-rs/tui/src/chatwidget/autoreview.rs`; `docs/features/autoreview.md`
- Tests: `codex-rs/tui/src/chatwidget/tests.rs`; `qa/release/<version>/`

## Preconditions

- [ ] Plan is active.
- [ ] Dependencies are completed.
- [ ] Worktree, branch, and base commit are exact and match the plan.

## Done

- [x] Sprint record created and linked to PF-13.

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

- [ ] Focused test: `cargo test -p codex-tui autoreview`
- [ ] Core regression: `cargo test -p codex-core multi_agents_tests`
- [ ] Docs: `mkdocs build --strict`
- [ ] True-TUI evidence: recordings, traces, provider-call ledger, unchanged-diff hashes, and human signoff linked.

## Exit evidence

- [ ] Implementation and documentation commits recorded.
- [ ] Final-tree automated and true-TUI evidence linked.
- [ ] Release record and benchmark decision linked.
- [ ] `Done` and `Remaining` ledgers reflect reality.
- [ ] Completed record moved to `docs/sprints/archive/arbitrary-model-autoreview/`.
