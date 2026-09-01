---
sprint_id: "PF-51-S01"
title: "OpenAI account auth adapter"
status: in_progress
plan_file: "docs/plans/active/unified-provider-auth.md"
plan_feature: "PF-51"
execution_order: 10
owner: "GPT-5.6 Sol high implementation agent"
parallel_lane: "provider-auth-serial"
write_scope: "MODULE.bazel.lock; codex-rs/Cargo.lock; codex-rs/app-server-client/Cargo.toml; codex-rs/app-server-client/BUILD.bazel; codex-rs/app-server-client/src/lib.rs; codex-rs/app-server-client/src/provider_auth.rs; codex-rs/app-server-client/src/provider_auth_tests.rs; codex-rs/provider-auth/src/lib.rs; codex-rs/provider-auth/src/auth_flow.rs; codex-rs/provider-auth/src/openai_account_flow.rs; codex-rs/provider-auth/src/openai_account_controller.rs; codex-rs/provider-auth/src/openai_account_flow_tests.rs; docs/sprints/current/unified-provider-auth/pf-51-s01-openai-account-adapter.md"
integration_gate: "PF-51 OpenAI account controller and thin app-server-client adapter only; no app-server protocol, login, TUI host, onboarding, provider-management, or API-key behavior changes"
worktree: "/home/pfrpc/repos/worktrees/corbanu-main-f7356a94e0"
branch: "feat/unified-provider-auth"
base_commit: "f7356a94e032234022a462d65b576a7de2854859"
depends_on: "PF-50-S01"
created: 2026-09-01
updated: 2026-09-01
---

# PF-51-S01 — OpenAI account auth adapter

## Execution mandate

- Deliver: OpenAI browser/device account login as an adapter on the shared controller.
- Excludes: changing app-server login protocol, account policy, or API-key behavior.

## Plan linkage

- Active plan feature: `PF-51`; acceptance is shared host login semantics.

## Code boundaries

- Provider-auth controller plus thin app-server-client adapter and tests only.

## Preconditions

- [x] Plan is active.
- [x] PF-50-S01 is completed and archived.
- [x] Exact serial allocation matches the plan.
- [x] Existing app-server request/cancel semantics are preserved.

## Done

- [x] Draft sprint record created and linked to PF-51.
- [x] Added typed OpenAI actions/effects, secret-free snapshots, completion,
      recovery, and a thin app-server-client adapter without protocol changes.
- [x] Preserved browser/device login and device-only forced enrollment.
- [x] Added dual correlation, cancellation, stale rejection, PF-49 status
      settlement, restart recovery, redaction, and audit-race regressions.

## Remaining

- [ ] Commit the accepted implementation, rerun post-commit governance, and archive PF-51.

## Verification

- [ ] Post-commit plan/sprint validators and clean-tree check pass.

## Implementation and audit evidence

- Exact mappings are `Chatgpt`, `ChatgptDeviceCode`, and device-only
  `OpenaiProviderDeviceCode`; forced browser enrollment is typed-rejected.
- API-key status never satisfies account setup. Late cancellation, wrong
  login-bearing variants, transport uncertainty, and non-account settlement
  terminate through typed cancel/failure/recovery paths without orphaned login.
- Challenges are zeroizing, non-serializable, non-cloneable, and Debug-redacted;
  arbitrary server error text is discarded.
- Final production sizes after formatting: `openai_account_controller.rs` 489
  lines, `openai_account_flow.rs` 494 lines, and app-server-client
  `provider_auth.rs` 195 lines.
- Provider-auth passed 40/40; adapter 8/8; protocol filters 5/5 and 1/1; forced
  device app-server integration 1/1 (`ddafa2de-88bf-4c2a-be74-93009c91f47d`);
  TUI check passed. Fix/fmt/Bazel/governance/diff/canary checks passed.
- Changed paths are the scoped Cargo lock/manifest, app-server-client
  lib/adapter/tests, provider-auth lib/shared controller/OpenAI modules/tests,
  and this ledger. Bazel lock and BUILD required no diff.

## Exit evidence

- [ ] Implementation commit and app-server adapter contract recorded.
- [x] Final-tree tests linked.
- [x] Cancel/stale notification evidence recorded.
- [x] `Done` and `Remaining` reflect pre-commit reality.
- [ ] Completed record archived.
