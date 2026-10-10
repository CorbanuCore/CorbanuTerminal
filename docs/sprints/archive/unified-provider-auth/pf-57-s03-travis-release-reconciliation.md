---
sprint_id: "PF-57-S03"
title: "Travis release-branch reconciliation"
status: completed
plan_file: "docs/plans/active/unified-provider-auth.md"
plan_feature: "PF-57"
execution_order: 18
owner: "Codex primary integration agent"
parallel_lane: "release-reconciliation"
write_scope: "codex-rs/.config/nextest.toml, codex-rs/cli/src/claude_oauth.rs, codex-rs/keyring-store/, codex-rs/models-manager/, codex-rs/provider-auth/, codex-rs/tui/, docs/getting-started.md, docs/install.md, docs/integrations/index.md, docs/plans/active/unified-provider-auth.md, docs/sprints/index.md, docs/sprints/current/unified-provider-auth/, docs/sprints/archive/unified-provider-auth/, humanTest.html, qa/release/0.1.38/"
integration_gate: "Preserve both source histories through a serial merge, inspect overlapping credential/login and test-harness seams, run formatted combined-tree domain/TUI/TMUX tests, and record exact evidence and release limits before handoff."
worktree: "/home/pfrpc/repos/worktrees/corbanu-release-0.1.38-reconcile"
branch: "integration/reconcile-release-0.1.38"
base_commit: "c9680a41e7940e20c8816201db37b32d001a1a6b"
depends_on: "PF-57-S02"
created: 2026-09-05
updated: 2026-09-05
---

# PF-57-S03 — Travis release-branch reconciliation

## Execution mandate

- Combine the user's completed repairs with Travis/IridiumMaster commits
  `f38dccd8bf39ebbb6fb87b67612a0cb6f2504cc3` and
  `07791288b6feeccfaee5a57c12452359cc666957` without rewriting either history.
- Preserve Claude login/token enrollment, actionable account recovery, isolated
  harness credentials and Ambient GLM-only selection/migration.
- Exclude site implementation, live credentials/payments, unrelated security
  lanes and new model-catalog choices. Publication authority is being clarified.

## Plan linkage

- Plan: [Unified provider authentication](../../../plans/active/unified-provider-auth.md).
- Feature: `PF-57`.
- Product citation: **Shipping MVP — LIVE** — “Encrypted `/vault`, masked entry,
  metadata-only inspection, and operational credential use without placing raw
  values in chat.” Related shipping capability: multi-provider inference.
- No upstream Codex merge: local upstream reference remains
  `ba6cf9c69277caec51a4c12c5b7401a9920930e0`. Fork `main` at
  `6dd9ad646beb4a7407521439411f436f21ea4af1` is included in the incoming branch.

## Code boundaries

- Incoming provider-auth/keyring/TUI changes and tests, Claude CLI probe, model
  catalog/retired pane migration, corresponding user docs and release evidence.
- Semantic overlap: wallet menu and multi-provider TMUX fixtures. Retain the
  PF-57-S02 protocol guard, direct legacy Lock, durable unlink and alias parity.
- Declared paths are disjoint from active PF-27-S04 and PF-35-S01 write scopes.

## Preconditions

- [x] User reconciliation mandate, active plan and archived PF-57-S02 verified.
- [x] Clean isolated worktree at the exact repair tip; canonical dirty tree untouched.
- [x] Root/Rust/TUI/bottom-pane instructions and development/test-tui skills read.

## Done

- [x] Fetched and identified the two-by-two divergence; main is already included.
- [x] Allocated the sole serial sprint and recorded exact merge inputs.
- [x] Merged both histories at `c37eb277d9f83ebcabe89e41cc81b9d3e92797a2`;
  the two overlapping files preserve both sides without semantic repair.
- [x] Completed source review and final Linux qualification; Astra found no new
  P0/P1 in its bounded combined-source check.

## Remaining

No sprint-scoped implementation or qualification remains. Release publication,
broader release gates and the previously disclosed privacy/updater/site issues
are separate from this bounded reconciliation.

## Verification

- [x] Scoped fix/format, then 625 selected tests passed through `just test` with no retries.
- [x] Real TMUX: 8 provider/catalog/alias application journeys plus 8 harness tests,
  all passing in one no-retry run, including failed and successful Claude token save.
- [x] Manual `just codex` startup/provider/token-entry/cancel/exit, temporary home,
  explicit trace logs and synthetic auth only; captured exact process hash.
- [x] Governance checks, portable-skill parity, installer/package tests and diff checks.
- [x] Final ancestry/diff distinguishes local Linux qualification from
  cross-platform, live-repository, named-human and due benchmark release evidence.

## Exit evidence

- [x] Final source commit, command/results, binary hashes and artifact paths in
  [reconciliation evidence](../../../../qa/release/0.1.38/travis-reconciliation.md).
- [x] Completed bounded reconciliation archived; no tag or release publication.
