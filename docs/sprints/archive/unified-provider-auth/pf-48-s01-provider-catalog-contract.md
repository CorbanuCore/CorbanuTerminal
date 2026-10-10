---
sprint_id: "PF-48-S01"
title: "Provider catalog contract"
status: completed
plan_file: "docs/plans/active/unified-provider-auth.md"
plan_feature: "PF-48"
execution_order: 7
owner: "GPT-5.6 Sol high implementation agent"
parallel_lane: "provider-auth-serial"
write_scope: "MODULE.bazel.lock, codex-rs/Cargo.toml, codex-rs/Cargo.lock, codex-rs/provider-auth/, codex-rs/model-provider-info/src/lib.rs, codex-rs/model-provider-info/src/model_provider_info_tests.rs, docs/sprints/current/unified-provider-auth/pf-48-s01-provider-catalog-contract.md"
integration_gate: "Codex primary agent audits the typed contract, provider coverage, dependency direction, and focused tests before archiving PF-48-S01 and allocating PF-49-S01."
worktree: "/home/pfrpc/repos/worktrees/corbanu-main-f7356a94e0"
branch: "feat/unified-provider-auth"
base_commit: "f7356a94e032234022a462d65b576a7de2854859"
depends_on: "PF-47-S01"
created: 2026-09-01
updated: 2026-09-01
---

# PF-48-S01 — Provider catalog contract

## Execution mandate

- Deliver: one typed, renderer-independent provider catalog and setup-capability contract.
- Excludes: credential status, eligibility persistence, auth execution, and TUI migration.

## Plan linkage

- Plan: [Unified provider onboarding and management](../../../plans/active/unified-provider-auth.md).
- Feature: `PF-48`.
- Acceptance advanced: onboarding and `/providers` can consume one provider inventory.

## Code boundaries

- Existing: `codex-rs/model-provider-info/src/lib.rs` provider registry and auth metadata.
- Planned: `codex-rs/provider-auth/` catalog identity, ordering, labels, and capabilities; workspace/Cargo/Bazel lock registration.
- Tests: provider-auth contract tests and model-provider-info compatibility tests.

## Preconditions

- [x] Plan is active.
- [x] PF-47-S01 is completed and archived.
- [x] Worktree, branch, and base commit match the plan.
- [x] Read nearest implementation `AGENTS.md` files and reverified local upstream tip `ba6cf9c69277caec51a4c12c5b7401a9920930e0`.

## Done

- [x] Sprint record created and linked to PF-48.
- [x] Added and registered `codex-provider-auth` without duplicating runtime transport definitions.
- [x] Defined distinct catalog/runtime identities and ordered, non-empty setup capabilities for API key, OpenAI account, Claude account, Corbanu Plan, local, command-auth, and status-only providers.
- [x] Derived the catalog from resolved `ModelProviderInfo` values with deterministic built-in/custom ordering, shared-`env_key` deduplication, and runtime-alias grouping.
- [x] Kept unsupported custom command-auth providers visible with `CommandAuthSetup::StatusOnly` and no renderer-dispatchable command data.
- [x] Added adjacent custom-provider, shared-key, multi-method OpenAI, Corbanu alias, blank-identity, command-auth, and Claude/local/AWS built-in-shape regression tests.
- [x] Recorded the changed paths, frozen interface, finalization commands, and final-tree verification results below.

## Remaining

- [x] Recorded implementation commit `7936d83859dc5f192c7966d071058a7c23410c4f` and archived the primary-accepted sprint.

## Verification

- [x] Focused test: `CARGO_INCREMENTAL=0 just test -p codex-provider-auth -j 1 --retries 0` — 7 passed, 0 skipped, exit 0.
- [x] Integration test: `CARGO_INCREMENTAL=0 just test -p codex-model-provider-info -j 1 --retries 0` — 58 passed, 0 skipped, exit 0; `cargo check -p codex-tui` — exit 0 in 1m53s with one pre-existing `unused_mut` warning in `tui/src/chatwidget/claude_code_login.rs:280`.
- [x] TUI applicability resolved: no renderer change; later host sprints own true-TMUX proof.
- [x] Primary integration acceptance: pass 1 found a built-in-shape coverage gap; the focused regression closes it and passes.
- [x] Post-commit verification: governance checks passed against recorded implementation commit `7936d83859dc5f192c7966d071058a7c23410c4f` before archival.

## Exit evidence

- [x] Implementation commit `7936d83859dc5f192c7966d071058a7c23410c4f`; frozen catalog contract is accepted and recorded below.
- [x] Final-tree test commands and exact summaries are recorded below.
- [x] Primary integration review pass 1 found one test-coverage gap; it is addressed without consuming an additional formal review pass.
- [x] `Done` and `Remaining` reflect reality.
- [x] Completed record moved to `docs/sprints/archive/unified-provider-auth/`.

## Frozen catalog contract

- `ProviderCatalog::from_runtime_providers(&HashMap<String, ModelProviderInfo>)` derives the ordered inventory from resolved runtime definitions.
- `ProviderCatalogId` is product-facing setup identity; `ProviderRuntimeId` preserves each transport/compatibility identity. Corbanu is cataloged as `corbanu-plan` while legacy `pfterminal-plan` runtime IDs remain attached to the entry.
- `ProviderCatalogEntry` exposes a display label, every deduplicated runtime ID, and `ProviderSetupCapabilities { primary, alternatives }`, which is non-empty and supports OpenAI account plus API key on one row.
- `ProviderSetupCapability` covers OpenAI account, API key storage boundary, Claude account, Corbanu Plan, Ollama/LM Studio local runtimes, status-only command auth, and non-interactive status reasons.
- Custom `env_key` providers are included automatically; entries sharing an environment key or canonical Corbanu identity are deterministically grouped without losing runtime IDs.

## Changed paths

- Workspace registration: `codex-rs/Cargo.toml`, `codex-rs/Cargo.lock`, `codex-rs/provider-auth/Cargo.toml`, and `codex-rs/provider-auth/BUILD.bazel`.
- Contract and tests: `codex-rs/provider-auth/src/lib.rs`, `codex-rs/provider-auth/src/provider_catalog_tests.rs`, `codex-rs/model-provider-info/src/lib.rs`, and `codex-rs/model-provider-info/src/model_provider_info_tests.rs`.
- `docs/sprints/current/unified-provider-auth/pf-48-s01-provider-catalog-contract.md`

## Final-tree evidence

- `just bazel-lock-update` — exit 0; no `MODULE.bazel.lock` delta was produced. Bazel reported existing direct-version warnings for `platforms` and `rules_cc`.
- `just fix -p codex-provider-auth` — exit 0 after the catalog-group repair and pass-1 built-in-shape regression addition.
- `just fmt` — exit 0. The initial sandboxed attempt could not write the external uv cache; the authorized rerun passed and was the final formatter run.
- `CARGO_INCREMENTAL=0 just test -p codex-provider-auth -j 1 --retries 0` — exit 0; 7 passed, 0 skipped.
- Codex primary independently reran the focused provider-auth suite after pass-1 closure — exit 0; 7 passed, 0 skipped.
- `CARGO_INCREMENTAL=0 just test -p codex-model-provider-info -j 1 --retries 0` — exit 0; 58 passed, 0 skipped.
- `cargo check -p codex-tui` — exit 0 in 1m53s; one pre-existing unrelated `unused_mut` warning at `tui/src/chatwidget/claude_code_login.rs:280`.
- `python3 docs/plans/check.py` and `python3 docs/sprints/check.py` — exit 0; active plans 2/2, current sprints 67, archived sprints 103.
- `git diff --check` — exit 0.
