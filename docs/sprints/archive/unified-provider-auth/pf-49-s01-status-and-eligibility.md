---
sprint_id: "PF-49-S01"
title: "Provider status and eligibility"
status: completed
plan_file: "docs/plans/active/unified-provider-auth.md"
plan_feature: "PF-49"
execution_order: 8
owner: "GPT-5.6 Sol high implementation agent"
parallel_lane: "provider-auth-serial"
write_scope: "MODULE.bazel.lock, codex-rs/Cargo.lock, codex-rs/provider-auth/, codex-rs/login/src/lib.rs, codex-rs/login/src/auth/manager.rs, codex-rs/login/src/auth/provider_key_vault.rs, codex-rs/login/src/auth/auth_tests.rs, docs/sprints/current/unified-provider-auth/pf-49-s01-status-and-eligibility.md"
integration_gate: "Codex primary agent audits metadata-only credential boundaries, eligibility migration/durability, redaction, dependency direction, and final affected tests before archiving PF-49-S01 and allocating PF-50-S01."
worktree: "/home/pfrpc/repos/worktrees/corbanu-main-f7356a94e0"
branch: "feat/unified-provider-auth"
base_commit: "f7356a94e032234022a462d65b576a7de2854859"
depends_on: "PF-48-S01"
created: 2026-09-01
updated: 2026-09-01
---

# PF-49-S01 — Provider status and eligibility

## Execution mandate

- Deliver: one metadata-only configured/status resolver and durable active/inactive policy.
- Excludes: renderer migration, credential setup execution, and credential deletion.

## Plan linkage

- Plan: [Unified provider onboarding and management](../../../plans/active/unified-provider-auth.md).
- Feature: `PF-49`.
- Acceptance advanced: both hosts and startup can read identical provider state.

## Code boundaries

- Existing: `codex-login::provider_api_key_from_auth_storage`, Claude/Plan status loaders, config edits.
- Planned: provider-auth status snapshots plus typed eligibility persistence/migration.
- Tests: source/status matrices, config round trips, migration, restart, and redaction.

## Preconditions

- [x] Plan is active.
- [x] PF-48-S01 is completed and archived at implementation commit `7936d83859dc5f192c7966d071058a7c23410c4f`.
- [x] Exact serial allocation matches the plan worktree, branch, base, owner, and lane.
- [x] Persistence and credential boundaries were reviewed before edits: eligibility is versioned metadata outside `config.toml` and credential stores expose presence/source only.

## Done

- [x] Draft sprint record created and linked to PF-49.
- [x] Added renderer-independent, secret-free configured, active/inactive, current, checking, unavailable, and recovery-required snapshots with ordered per-method status.
- [x] Added typed metadata adapters for API-key environment/vault, OpenAI account alternatives, Claude sources, Corbanu Plan, local providers, command auth, and status-only providers.
- [x] Enforced runtime API-key precedence: a present-invalid environment credential shadows managed storage and requires recovery; only a missing environment credential may fall back to managed storage.
- [x] Added versioned `provider-eligibility.json` persistence for stable inactive identities; missing state is active-by-default, unknown identities survive round trips, malformed/future versions fail visibly, and deterministic atomic writes are mode `0600` on Unix.
- [x] Migrated existing configured providers to active without changing `config.toml`, the current model, or credential stores, and kept environment-backed removal/config layering explicit.
- [x] Added failure, ambiguity, partial-state, precedence, restart, durability, and secret-canary coverage without retaining arbitrary upstream error strings.
- [x] Kept the Corbanu environment-alias limitation outside this sprint for PF-55 startup convergence.

## Remaining

- [x] Recorded primary-accepted implementation commit `5fcde1c1d9e6703d8618b23572abe69e44ada96d` and archived the sprint.

## Verification

- [x] `just bazel-lock-update` passed (invocation `ce075531-9b55-4128-9a97-520a36fcd9de`); the dependency graph required no `MODULE.bazel.lock` delta.
- [x] Final scoped sequence passed: `just fix -p codex-provider-auth`, `just fmt`, `git diff --check`, then `CARGO_INCREMENTAL=0 just test -p codex-provider-auth -j 1 --retries 0` (17 passed, 0 skipped; run `786d4533-7aa6-4665-af0f-a3ca8c2e7cc7`).
- [x] Focused login metadata tests passed: `provider_api_key_metadata_reports_legacy_missing_and_suppressed_without_secret` (1 passed) and `openai_auth_metadata_distinguishes_account_api_key_and_unsupported_auth` (1 passed).
- [x] Focused vault coverage passed: `CARGO_INCREMENTAL=0 just test -p codex-vault claude_auth -j 1 --retries 0` (21 passed, 34 skipped).
- [x] Affected non-network model-provider coverage passed inside the full run: `configured_provider_observes_provider_key_saved_after_missing_lookup` and `configured_provider_prefers_env_key_over_stored_provider_key`.
- [x] Unrestricted full `codex-login` rerun removed every WireMock failure: 186 tests ran, 184 passed within Nextest's 60-second per-test cap, and the two existing slow provider-vault tests timed out. Direct single-threaded executions without that cap then passed `provider_api_key_login_is_provider_scoped_and_not_primary_auth` in 95.49s and `delete_all_provider_keys_clears_provider_entries` in 115.87s; both PF-49 metadata tests passed in the full run.
- [x] Unrestricted full `codex-model-provider` rerun passed all 73 tests with 0 skipped, including both affected provider-key precedence/refresh tests.
- [x] TUI applicability resolved: snapshots remain hidden for later host sprints, typed fixtures are recorded, and `cargo check -p codex-tui` passed in 57.96s with one unrelated pre-existing `unused_mut` warning at `tui/src/chatwidget/claude_code_login.rs:280`.
- [x] Final governance passed: `python3 docs/plans/check.py` reported active 2/2 with 0 available slots, `python3 docs/sprints/check.py` reported 66 current and 104 archived, and final `git diff --check` exited 0.
- [x] Post-commit verification: governance checks passed against implementation commit `5fcde1c1d9e6703d8618b23572abe69e44ada96d` before archival.

## Exit evidence

- [x] Implementation commit `5fcde1c1d9e6703d8618b23572abe69e44ada96d`; the primary integration audit accepted the corrected precedence contract, metadata boundaries, durability, redaction, dependency direction, and final evidence.
- [x] State schema recorded: `ProviderMetadataSnapshot` feeds an ordered `ProviderStatusCatalog` with per-method state plus aggregate configuration, eligibility, current selection, and availability; eligibility v1 serializes only sorted stable inactive identities.
- [x] PF-49 implementation paths recorded: `codex-rs/Cargo.lock`; `codex-rs/login/src/lib.rs`; `codex-rs/login/src/auth/{manager.rs,provider_key_vault.rs,auth_tests.rs}`; `codex-rs/provider-auth/{Cargo.toml,src/lib.rs,src/eligibility.rs,src/eligibility_tests.rs,src/status.rs,src/status_contract.rs,src/status_tests.rs}`; and this sprint ledger. `MODULE.bazel.lock` was authorized but unchanged.
- [x] Migration, restart, malformed/future-version, unknown-identity, permission, precedence, and redaction fixtures are exercised by the 17-test provider-auth suite.
- [x] No raw credential or arbitrary upstream error appears in debug, error, snapshot, or serialized eligibility artifacts.
- [x] Change-size audit recorded: before the required final precedence regression, the five new PF-49 modules were approximately 1,435 lines including approximately 507 test lines; the final tree is 1,482 lines including 558 test lines. Production remains split coherently into `eligibility.rs` (245), `status.rs` (373), and `status_contract.rs` (306), each under 400 lines. Further mechanical commit splitting would leave the hidden renderer-independent contract temporarily unexported or testless, so PF-49 remains one coherent sprint.
- [x] `Done` and `Remaining` reflect the primary-review-ready tree and environmental rerun requirement.
- [x] Completed record archived under `docs/sprints/archive/unified-provider-auth/`.
