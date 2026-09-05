---
sprint_id: "PF-55-S03"
title: "Live Astra runtime compatibility and TUI harness"
status: in_progress
plan_file: "docs/plans/active/unified-provider-auth.md"
plan_feature: "PF-55"
execution_order: 20
owner: "Codex primary Astra runtime integration agent"
parallel_lane: "astra-runtime"
write_scope: "codex-rs/model-provider-info/, codex-rs/models-manager/, codex-rs/protocol/src/openai_models.rs, codex-rs/login/src/auth/default_client.rs, codex-rs/login/src/auth/default_client_tests.rs, codex-rs/tui/tests/suite/provider_management.rs, codex-rs/tui/src/chatwidget/tests/popups_and_settings.rs, codex-rs/tui/src/chatwidget/snapshots/, codex-rs/core/tests/suite/astra_runtime.rs, codex-rs/core/tests/suite/mod.rs, scripts/astra_tui_acceptance.py, scripts/test_astra_tui_acceptance.py, docs/features/model-providers.md, docs/plans/active/unified-provider-auth.md, docs/sprints/index.md, docs/sprints/current/unified-provider-auth/, docs/sprints/archive/unified-provider-auth/, qa/release/0.1.38/"
integration_gate: "Primary agent audits upstream Astra contracts, implements the compatible runtime, runs final affected regression tests and live TUI tool/cancel/restart/resume acceptance, records exact binary/evidence identities, then installs and pushes the reconciled candidate without publishing."
worktree: "/home/pfrpc/repos/worktrees/corbanu-release-0.1.38-reconcile"
branch: "integration/reconcile-release-0.1.38"
base_commit: "de26f9f3ccff5748b12633b995ada52570a9e161"
depends_on: "PF-55-S02"
created: 2026-09-05
updated: 2026-09-05
---

# PF-55-S03 — Live Astra runtime compatibility

## Execution mandate

- Deliver working `gpt-6-astra` in Corbanu, with any required upstream upgrade,
  a repeatable live harness and actual TUI proof. Mocks alone cannot qualify it.
- Preserve the reconciled provider/credential fixes, current profile and other
  providers. No release, new credential copying, payments or fallback model.

## Plan linkage

- [Unified provider authentication](../../../plans/active/unified-provider-auth.md), PF-55.
- Product: **Shipping MVP — LIVE**, “Multi-provider inference”; **Product
  principles**, “Maintain continuous Codex parity without removing Corbanu-specific behavior.”
- User mandate: live Astra failed with a newer-Codex error; do the necessary
  upgrade and test the real harness in TUI before claiming it works.

## Code boundaries

- Existing model-provider compatibility headers, bundled catalog and manager
  merge behavior; shared model protocol metadata and native login user-agent.
- Typed request/metadata regression fixtures and real TMUX acceptance script.
- Upstream baseline `ba6cf9c69277caec51a4c12c5b7401a9920930e0`; candidate
  `rust-v0.153.4` = `3d2ee51ca2d5db578f328aa75e20aa22c0197c9a` fetched from
  `https://github.com/openai/codex.git`. Exact integration disposition required.
- Wider Core/protocol changes or full upstream merge require explicit scope
  amendment and disjoint allocation before editing, not a change in the goal.

## Preconditions

- [x] User-authorized upgrade and live inference recorded; active plan updated.
- [x] Receiving branch is clean at the recorded base; prior sprint is archived.
- [x] Root/Rust/TUI/Core policies, development/test-TUI skills, OpenAI Docs and
  model-migration reference read. No API-key setup tool is available.
- [x] Current scope does not overlap active PF-27-S04/PF-35-S01 paths.

## Done

- [x] Confirmed local compatibility `0.144.1` and exact upstream Astra minimum
  `0.153.0`, native Code Mode/Responses Lite/unified-execution metadata.
- [x] Preserved user-provided 400 as failed live acceptance, not an auth failure.
- [x] Integrated the native Astra contract and shared discovery/inference version;
  preserved template-only instructions through personality overrides.
- [x] Added native request/header regressions and an opt-in structured live TUI
  harness. Final affected Rust checks: 445/445; harness checks: 12/12.
- [x] First live TensorCash diagnostic passed real Astra file/tool work,
  Escape cancellation/recovery and process restart/same-thread resume.

## Remaining

- [ ] Run actual Astra TUI responses and file/tool work, cancel/recovery,
  process restart and same-thread resume, with exact binary and structured evidence.
- [ ] Exercise both TensorCash and Isometric Game in disposable worktrees;
  identify exact paths/base commits before testing. No trading/backtest work.
- [ ] Update docs/qualification honestly, install the tested build, leave a
  separate human-test session with the approved normal profile, commit and push.

## Verification

- [ ] Model/provider/login/protocol and affected TUI regression checks pass.
- [ ] Full Core checks if upstream adapter integration requires them; distinguish
  unrelated failures from qualification and do not count skipped live tests as pass.
- [ ] True TUI final-binary live Astra tool/cancel/restart/resume evidence passes.
- [ ] Model identity and real response/tool events are asserted, not inferred
  from echoed prompts, picker labels or success strings in a mocked server.
- [ ] Governance, portable skills and final diff checks pass.

## Exit evidence

- [ ] `qa/release/0.1.38/astra-runtime.md` records source/upstream disposition,
  commands, hashes, actual runs and limitations. Prior selector evidence stays historical.
- [ ] Human rejection resolved in installed candidate; named-human follow-up
  and unrelated release/benchmark/cross-platform gates remain honestly disclosed.
- [ ] Completed sprint archived only after all live acceptance requirements pass.
