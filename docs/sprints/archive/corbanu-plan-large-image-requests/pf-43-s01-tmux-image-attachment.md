---
sprint_id: "PF-43-S01"
title: "Tmux image attachment qualification"
status: completed
plan_file: "docs/plans/active/corbanu-plan-large-image-requests.md"
plan_feature: "PF-43"
execution_order: 2
owner: "Codex — Tmux qualification lane"
parallel_lane: "tmux-image-qualification"
write_scope: "codex-rs/tui/tests/support/tmux.rs, codex-rs/tui/tests/support/tmux_tests.rs, codex-rs/tui/tests/suite/large_image.rs, codex-rs/tui/tests/suite/mod.rs, docs/tmuxHarness.md, docs/plans/active/corbanu-plan-large-image-requests.md, docs/sprints/current/corbanu-plan-large-image-requests/"
integration_gate: "Jim Ricketts receives the public main worktree, audits that the new harness action emits only a bracketed paste event, reruns the focused support and large-image selectors with zero retries, and keeps live-provider, deployment and human gates explicit."
worktree: "/Volumes/CorbanuDrive/Corbanu/CorbanuTerminal"
branch: "main"
base_commit: "9ccecfc4d2c3c47b7aeea90bdb4889cae5bfe4d6"
depends_on: "none"
created: 2026-08-30
updated: 2026-08-30
---

# PF-43-S01 — Tmux image attachment qualification

## Execution mandate

- Deliver: Add a typed bracketed-paste action to the private tmux harness and prove a >2 MiB image request through the real Corbanu TUI against a deterministic local Fable boundary.
- Excludes: clipboard mutation, production deployment, live-provider spend, billing/reservation changes and release publication.

## Plan linkage

- Plan: [Corbanu Plan large-image requests](../../../plans/active/corbanu-plan-large-image-requests.md).
- Feature: `PF-43`.
- Acceptance advanced: Automated image attachment, send, visible success and same-session recovery can be driven with real keys in tmux.

## Code boundaries

- Existing: `codex-rs/tui/tests/support/tmux.rs::TmuxPane` literal/key input boundary.
- Planned: `TmuxPane::send_paste`, harness documentation and one Fable image scenario.
- Tests: `codex-rs/tui/tests/support/tmux_tests.rs` and `codex-rs/tui/tests/suite/large_image.rs`.

## Preconditions

- [x] Plan is active.
- [x] Dependencies are completed; this sprint has none.
- [x] Worktree, branch and base commit are exact and match the plan.
- [x] Owner, lane, scope and receiving integration gate are recorded.

## Done

- [x] Sprint record created and linked to the single PF-43 feature.
- [x] Added `TmuxPane::send_paste`, which emits one bracketed-paste byte sequence without using the system clipboard.
- [x] Redacted literal and paste contents from tmux command artifacts and rejected embedded escape characters.
- [x] Added exact PTY-byte and escape-rejection support tests.
- [x] Added a real-TUI Fable scenario with a deterministic PNG whose serialized `/messages` request exceeds 2 MiB.
- [x] Proved large-image success, small-image success and exactly one automatic retry after a synthetic 413.
- [x] Final Corbanu Terminal + tmux + `claude-opus-5-plan` at `max` review returned `CLEAN`.
- [x] Rebuilt the current candidate as `corbanu 0.1.35` before the final zero-retry tmux runs.

## Remaining

None. Deployment, live Isometric Game qualification and named human acceptance remain plan/release gates.

## Verification

- [x] `just fmt` and the compile-only `codex-tui` integration-test build pass.
- [x] Both focused paste support tests pass after formatting with zero retries, 2/2.
- [x] Tmux large-image selector passes with `CORBANU_TMUX_REQUIRED=1` and `--retries 0`, 1/1 in 5.977s.
- [x] Existing single-Enter tmux smoke selector passes with zero retries, 1/1 in 1.326s.
- [x] TUI applicability is exercised with paste text, literal prompt text and Enter sent as separate actions.
- [x] Full `just test` disposition is recorded: host contention produced unrelated keychain, loopback and app-server timeouts; 3,954 passed before the non-diagnostic run was interrupted.

## Exit evidence

- [x] Explicit uncommitted final-tree handoff recorded on observed public HEAD `e4f9761fc2dca509bff295788b0dfefa364f5685`; no public commit was requested.
- [x] Final-tree test output is summarized in the active plan; successful tmux scenarios emitted no failure bundle.
- [x] Handoff records the local Wiremock/Fable boundary, synthetic Plan key, scope audit and remaining live/deployment/human gates.
- [x] `Done` and `Remaining` ledgers reflect reality.
- [x] Completed record moved to `docs/sprints/archive/corbanu-plan-large-image-requests/`.
