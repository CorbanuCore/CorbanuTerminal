---
sprint_id: "PF-55-S01"
title: "Expose Fable 5.1 through Claude Plan"
status: in_progress
plan_file: "docs/plans/active/claude-subscription-auth.md"
plan_feature: "PF-55"
execution_order: 7
owner: "Release owner"
parallel_lane: "serial-release"
write_scope: "codex-rs/core/src/client.rs, codex-rs/core/src/client_tests.rs, codex-rs/model-provider-info/src/lib.rs, codex-rs/model-provider-info/src/model_provider_info_tests.rs, codex-rs/model-provider/src/provider.rs, codex-rs/models-manager/models.json, codex-rs/models-manager/src/manager_tests.rs, codex-rs/tui/src/chatwidget/model_popups.rs, codex-rs/tui/src/chatwidget/tests/popups_and_settings.rs, codex-rs/tui/src/chatwidget/snapshots, docs/getting-started.md, docs/install.md, docs/plans/active/claude-subscription-auth.md, docs/sprints/current/claude-subscription-auth, qa/release/0.1.37"
integration_gate: "Release owner integrates onto release/0.1.37 and reruns model-provider-info, model-provider, models-manager, core client, TUI picker, formatting, and release-scope checks."
worktree: "/home/pfrpc/repos/CorbanuTerminal-0.1.37"
branch: "release/0.1.37"
base_commit: "c428cb1021e9287a5e7b6b5a5b4a7094713fbf51"
depends_on: "PF-47-S01"
created: 2026-09-01
updated: 2026-09-01
---

# PF-55-S01 — Expose Fable 5.1 through Claude Plan

## Execution mandate

- Deliver: make Fable 5.1 visible and selectable through the persisted Claude Plan credential, using the exact upstream `claude-fable-5-1` model.
- Excludes: changing the default crew, removing Fable 5, adding a metered Anthropic API catalog row, inventing direct-API pricing, or changing unrelated providers.

## Plan linkage

- Plan: [Reliable Claude subscription authentication](../../../plans/active/claude-subscription-auth.md)
- Feature: `PF-55`
- Acceptance advanced: a user with Claude subscription authentication can select Fable 5.1 and retain the exact provider/model pair through request dispatch.

## Code boundaries

- Existing: `codex-rs/model-provider-info/src/lib.rs`, `codex-rs/core/src/client.rs`
- Existing: `codex-rs/models-manager/models.json`, `codex-rs/tui/src/chatwidget/model_popups.rs`
- Tests: focused provider, request mapping, catalog, and TUI picker suites

## Preconditions

- [x] Claude subscription-auth plan is active.
- [x] PF-47-S01 is completed and archived.
- [x] Worktree, branch, and base commit are exact and match the plan.
- [x] Human release instruction explicitly includes Fable 5.1 and limits the release scope.

## Done

- [x] Added typed Fable 5.1 upstream and Claude Plan slugs.
- [x] Added exact catalog-provider correction and Plan-to-upstream request mapping.
- [x] Added the image-capable Fable 5.1 Plan catalog row without removing Fable 5.
- [x] Added provider, catalog, request-wire, semantic picker, and snapshot regressions.
- [x] Updated Claude Plan model documentation.

## Remaining

- [ ] Run final formatting and focused affected tests.
- [ ] Review and accept the Claude Plan picker snapshot.
- [ ] Record final candidate commit and release evidence.
- [ ] Move this completed sprint to the archive.

## Verification

- [ ] Format: `just fmt`.
- [ ] Provider routing: `just test -p codex-model-provider-info`.
- [ ] Static catalogs: `just test -p codex-model-provider` and `just test -p codex-models-manager`.
- [ ] Request mapping: focused `codex-core` client test.
- [ ] Picker: focused `codex-tui` model-picker test and reviewed snapshot.
- [ ] TUI applicability: required; exact Fable 5.1 row must render under Claude Plan.

## Exit evidence

- [ ] Implementation commit recorded.
- [ ] Final-tree test output summarized in `qa/release/0.1.37/RELEASE-CANDIDATE.md`.
- [ ] Release scope audit confirms no GLM 5.3 worktree changes or unrelated security implementation are included.
- [ ] `Done` and `Remaining` ledgers reflect reality.
- [ ] Completed record moved to `docs/sprints/archive/claude-subscription-auth/`.
