---
sprint_id: "PF-50-S01"
title: "Shared API-key flow controller"
status: completed
plan_file: "docs/plans/active/unified-provider-auth.md"
plan_feature: "PF-50"
execution_order: 9
owner: "GPT-5.6 Sol high implementation agent"
parallel_lane: "provider-auth-serial"
write_scope: "MODULE.bazel.lock, codex-rs/Cargo.lock, codex-rs/provider-auth/Cargo.toml, codex-rs/provider-auth/BUILD.bazel, codex-rs/provider-auth/src/lib.rs, codex-rs/provider-auth/src/auth_flow.rs, codex-rs/provider-auth/src/auth_flow_tests.rs, codex-rs/provider-auth/src/api_key_flow.rs, codex-rs/provider-auth/src/api_key_flow_tests.rs, docs/sprints/current/unified-provider-auth/pf-50-s01-api-key-flow-controller.md"
integration_gate: "Codex primary agent audits the reducer protocol, raw-secret effect boundary, commit-point cancellation, timeout reconciliation, stale-result rejection, custom-provider generality, and final affected tests before archiving PF-50-S01 and allocating PF-51-S01."
worktree: "/home/pfrpc/repos/worktrees/corbanu-main-f7356a94e0"
branch: "feat/unified-provider-auth"
base_commit: "f7356a94e032234022a462d65b576a7de2854859"
depends_on: "PF-49-S01"
created: 2026-09-01
updated: 2026-09-01
---

# PF-50-S01 — Shared API-key flow controller

## Execution mandate

- Deliver: renderer-independent typed provider-auth transitions and the API-key adapter.
- Excludes: OpenAI, Claude, Corbanu Plan, and full host rendering.

## Plan linkage

- Plan: [Unified provider onboarding and management](../../../plans/active/unified-provider-auth.md).
- Feature: `PF-50`.
- Acceptance advanced: API-key setup behaves identically from either host.

## Code boundaries

- Existing: onboarding API-key states and `chatwidget/provider_credentials.rs` vault entry.
- Planned: provider-auth flow state/actions/effects plus thin TUI event adapters.
- Tests: transition tables, cancellation races, stale completion, storage failure, and masking.

## Preconditions

- [x] Plan is active.
- [x] PF-49-S01 is completed and archived at implementation commit `5fcde1c1d9e6703d8618b23572abe69e44ada96d`.
- [x] Exact serial allocation matches the plan worktree, branch, base, owner, and lane.
- [x] App-server/vault persistence is the sole raw-secret effect boundary; Submit is the visible irreversible commit point, pre-dispatch cancel is inert, and the shared 120-second post-submit timeout becomes outcome-unknown until metadata reconciliation.

## Done

- [x] Draft sprint record created and linked to PF-50.
- [x] Added protocol version 1 renderer-independent `ProviderAuthController` actions, snapshots, effects, dispositions, typed failures, and monotonic correlated attempt identities.
- [x] Made Submit the irreversible raw-secret effect boundary: `ApiKeySecret`, its enclosing action/effect/transition, and all debug output are redacted; the secret is zeroizing, non-serializable, non-cloneable, and consumed into persistence.
- [x] Froze pre-dispatch cancellation, visible non-cancellable settling, the shared 120-second timeout, outcome-unknown reconciliation, definite-failure retry, and stale-result rejection semantics.
- [x] Added one generic catalog-derived `ApiKeyAuthTarget` for built-in, custom, and shared-`env_key` identities without provider-specific controller branches.
- [x] Returned only stable catalog/runtime identities plus PF-49 metadata status from completion and snapshots; no raw credential or arbitrary upstream error text survives the effect boundary.
- [x] Added a fake typed host adapter for the existing provider/API-key request shape without changing either renderer, app-server, login, or vault implementation.
- [x] Added deep-equality transition, environment-precedence, timeout/reconciliation, stale-attempt, redaction-canary, built-in/custom/shared target, and fake-host regression coverage.
- [x] Closed the whole timeout/result ordering race: correlated late settlement is accepted from outcome-unknown, metadata misses cannot unlock retry before settlement, definite rejected/unavailable settlement can unlock retry, and transient reconciling metadata remains non-retryable without emitting another persistence effect. Configured metadata may complete a timed-out Add, while a timed-out Replace waits for correlated stored settlement and a fresh configured reconciliation so the old credential cannot masquerade as replacement success.
- [x] Registered the existing workspace `zeroize` dependency in `codex-provider-auth`; Cargo lock changed and `just bazel-lock-update` confirmed no `MODULE.bazel.lock` delta was required.
- [x] Final implementation paths: `codex-rs/Cargo.lock`, `codex-rs/provider-auth/Cargo.toml`, `codex-rs/provider-auth/src/lib.rs`, `codex-rs/provider-auth/src/api_key_flow.rs`, `codex-rs/provider-auth/src/api_key_flow_tests.rs`, `codex-rs/provider-auth/src/auth_flow.rs`, and `codex-rs/provider-auth/src/auth_flow_tests.rs`.
- [x] Size audit: the two production modules are 102 and 452 physical lines and 498 combined nonblank/non-comment production lines after excluding test-module gates; the two test files are 94 and 511 physical lines, for 1,159 physical lines across the four new files. This exceeds the 800-line review guidance because the accepted secret boundary, reducer protocol, Add/Replace timeout-settlement ordering matrix, generic target adapter, and required security regressions form one hidden coherent contract; a mechanical split would leave either an unexported controller/adapter or a testless security-sensitive protocol.

## Remaining

- [x] Recorded primary-accepted implementation commit `6f90e89792d544453a45fe1c9444215476c8e8f3` and archived the sprint; renderer adoption remains PF-53/PF-54.

## Verification

- [x] `just bazel-lock-update` — passed; the existing Bazel dependency graph required no `MODULE.bazel.lock` change.
- [x] `just fix -p codex-provider-auth` then `just fmt` — passed before final affected verification.
- [x] `CARGO_INCREMENTAL=0 just test -p codex-provider-auth -j 1 --retries 0` — 28 passed, 0 failed after the final Add/Replace settlement-ordering fix, including all eleven controller/adapter tests.
- [x] `CARGO_INCREMENTAL=0 just test -p codex-login provider_api_key_metadata_reports_legacy_missing_and_suppressed_without_secret -j 1 --retries 0` — 1 passed, 185 skipped.
- [x] `CARGO_INCREMENTAL=0 just test -p codex-model-provider configured_provider_ -j 1 --retries 0` — both affected precedence/refresh tests passed (`configured_provider_prefers_env_key_over_stored_provider_key` and `configured_provider_observes_provider_key_saved_after_missing_lookup`); aggregate command was non-passing at 5 passed/1 failed because unrelated `configured_provider_models_manager_uses_provider_bearer_token` could not bind a wiremock OS port in the sandbox (`PermissionDenied`).
- [x] `CARGO_INCREMENTAL=0 cargo check -p codex-tui` — passed in 1m14s; reported one unrelated pre-existing `unused_mut` warning in `tui/src/chatwidget/claude_code_login.rs:280`.
- [x] `python3 docs/sprints/check.py`, `python3 docs/plans/check.py`, `python3 scripts/check_portable_skills.py`, and `git diff --check` — passed (`current 65/archived 105`, `active 2/2`, 25 portable skill files match, clean diff whitespace).
- [x] TUI applicability resolved: the minimal typed fake-host harness proves effect translation; full PTY flows and renderer adoption remain PF-53/PF-54.
- [x] Primary integration audit accepted the corrected settlement/reconciliation ordering, replacement proof boundary, redaction, custom-provider generality, and final 28-test contract suite.
- [x] Post-commit verification: governance checks passed against implementation commit `6f90e89792d544453a45fe1c9444215476c8e8f3` before archival.

## Exit evidence

- [x] Implementation commit `6f90e89792d544453a45fe1c9444215476c8e8f3`; primary integration accepted controller protocol version 1 as recorded above and in `PROVIDER_AUTH_FLOW_PROTOCOL_VERSION`.
- [x] Final-tree test commands and the redaction canary covering enclosing action/transition/effect values are recorded above.
- [x] Deep-equality tests freeze pre-submit cancellation, post-submit rejection, direct timeout-to-stored reconciliation, timeout-to-definite-failure retry, timeout-to-not-configured then late-stored recovery, stale settlement rejection, no duplicate persistence, transient reconciling metadata remaining non-retryable, and configured-after-timeout completing Add while Replace waits for late stored settlement plus fresh status.
- [x] `Done` and `Remaining` reflect the implementation tree awaiting primary audit.
- [x] Completed record archived under `docs/sprints/archive/unified-provider-auth/`.
