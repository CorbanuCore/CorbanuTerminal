---
sprint_id: "PF-55-S02"
title: "OpenAI Astra model selector"
status: completed
plan_file: "docs/plans/active/unified-provider-auth.md"
plan_feature: "PF-55"
execution_order: 19
owner: "Codex primary implementation agent"
parallel_lane: "astra-selector"
write_scope: "codex-rs/models-manager/models.json, codex-rs/models-manager/src/manager_tests.rs, codex-rs/tui/src/chatwidget/tests/popups_and_settings.rs, codex-rs/tui/src/chatwidget/snapshots/, codex-rs/tui/tests/suite/provider_management.rs, docs/features/model-providers.md, docs/plans/active/unified-provider-auth.md, docs/sprints/index.md, docs/sprints/current/unified-provider-auth/, docs/sprints/archive/unified-provider-auth/, qa/release/0.1.38/"
integration_gate: "Primary agent serially adds the sourced catalog entry, validates remote-overlay/default behavior, snapshots and real TUI selection/cancel/restart/request routing, then archives evidence and pushes the existing reconciled branch without releasing."
worktree: "/home/pfrpc/repos/worktrees/corbanu-release-0.1.38-reconcile"
branch: "integration/reconcile-release-0.1.38"
base_commit: "90e29701f26704225f31cee03234dc05e65bc484"
depends_on: "PF-57-S03"
created: 2026-09-05
updated: 2026-09-05
---

# PF-55-S02 — OpenAI Astra model selector

## Execution mandate

- Add `gpt-6-astra` to OpenAI's bundled manual model selector, with sourced
  capabilities and supported reasoning levels. Preserve current/default model.
- Exclude automatic model allocation, guessed billing, Corbanu API/Bedrock routes,
  credential changes, provider fallback, actual paid inference and publication.

## Plan linkage

- [Unified provider authentication](../../../plans/active/unified-provider-auth.md),
  feature `PF-55`, explicitly amended by the user's 2026-09-05 model request.
- Product citation: **Shipping MVP — LIVE**, “OpenAI, Anthropic/Claude Plan, Kimi,
  Z.AI, DeepSeek, OpenRouter, Ambient, Meta, Baseten, Vercel, Bedrock, Ollama,
  LM Studio, Corbanu Plan, and custom providers.”
- The existing provider/model routing, persistence and native Responses interfaces
  remain authoritative. Upstream baseline `ba6cf9c69277caec51a4c12c5b7401a9920930e0`
  is unchanged; no upstream merge or new runtime adapter is planned.

## Code boundaries

- Bundled catalog in models-manager; existing catalog/auth/cache behavior tests.
- Existing model-picker snapshots and provider-management TMUX fixture for exact
  Astra selection, cancellation, restart and loopback Responses request assertions.
- No credentials, security broker paths or other active sprint scopes overlap.

## Preconditions

- [x] User request and product linkage recorded in active plan; prior sprint archived.
- [x] Root, Rust, TUI policies and development/test-tui skills read; clean receiving tree.
- [x] OpenAI Docs fetched exact Astra model and migration parameter pages; no API-key
  setup tool is available, and no real API key is needed for loopback fixtures.

## Done

- [x] Confirmed missing bundled Astra entry and existing generic OpenAI route mapping.
- [x] Added sourced catalog entry and regression/snapshot/TMUX coverage.
- [x] Final scoped fix/format and 87 selected tests pass without retries.
- [x] Manual `just codex` verified Astra, Max, cancellation and saved High selection.
- [x] User documentation and exact source/binary/test evidence recorded.

## Remaining

None within PF-55-S02. Real account entitlement and broader release gates are
separate, explicitly disclosed in the evidence record.

## Verification

- [x] Scoped fix/format, model-manager tests and relevant TUI snapshots pass.
- [x] Real TMUX selection/cancel/restart and exact loopback model/effort request pass.
- [x] Manual `just codex` with temporary home, explicit trace log directory and fake auth.
- [x] Governance, portable-skill parity and final diff checks pass.

## Exit evidence

- [x] Source/docs and test commands/results in `qa/release/0.1.38/astra-selector.md`.
- [x] TensorCash/Isometric applicability: not a project coding-logic change; isolated
  selection/persistence/loopback routing proves this feature. Broader release-level
  live-repository, benchmark, cross-platform and named-human gates remain separate.
- [x] Completed sprint archived for integration handoff; no release or paid access claim.

Implementation commit: `6b17a2630f31f5447d2c53fa8f6a29b60407b42a`.
Final nextest run: `a6f3821f-aa09-49d0-a435-210c7c210586`, 87/87 passed.
[Evidence](../../../../qa/release/0.1.38/astra-selector.md).
