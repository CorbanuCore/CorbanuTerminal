# PR #50 Fix Handoff — 2026-07-17

## Context

Repo: `/home/pfrpc/repos/PfTerminal`, branch `feat/kimi-code-provider` (tracks origin, PR #50 open at https://github.com/agtico/PfTerminal/pull/50).
Mandate: `PR50-REQUESTED-FIXES.md` (repo root) lists 5 required fixes + merge gates. Fixes 1, 2, 4 are **implemented in the working tree, uncommitted**. Fixes 3 and 5 are **not started**.

Commit already on the branch: `03b975114` (Kimi Code K3 provider + Nazgul binding tightening + doctor KIMI_API_KEY env + benchmark rollout matching).

## What's done (uncommitted, compiles, tests green)

### Fix 1 — doctor vault auth (COMPLETE)
- `codex-rs/cli/src/doctor.rs`: `provider_specific_auth_check` now takes `&Config`, falls back to vault via new `provider_key_stored()` (presence-only, secret never rendered). Reachability: `provider_reachability_plan` resolves the *active* provider's credential (env OR vault) and passes `active_provider_key_present` into `provider_auth_reachability_mode_from_auth` (new 4th param; `NotRequired` → `ApiKey` when resolved).
- `codex-rs/login/src/lib.rs`: re-exports `provider_api_key_from_auth_storage`.
- Regression tests in doctor.rs (real provider-key storage path via `login_with_provider_api_key` into temp home): `doctor_auth_passes_with_vault_only_kimi_credential`, `doctor_auth_fails_actionably_without_kimi_credential`, `doctor_kimi_credential_does_not_satisfy_openai_provider`, `provider_reachability_mode_uses_active_provider_stored_key`. Also asserts credential value never appears in output. Env-only case covered by pre-existing env-var branch tests.
- Verified: `cargo test -p codex-cli --bin pfterminal -- doctor::tests` → 44 passed.

### Fix 2 — plan-dependent context (COMPLETE)
- `codex-rs/models-manager/models.json` k3 entry: `context_window` 1048576→**262144** (conservative Moderato default), `max_context_window` stays 1048576, description documents `model_context_window = 1048576` opt-in for Allegretto+. Auto-compaction derives from effective window via existing `with_config_overrides` (model_info.rs) + `turn_context.model_context_window()`. Resume preserved (config-based).
- 401 entitlement classification: new `CodexErr::PlanEntitlementExceeded(String)` (protocol/src/error.rs, non-retryable, maps to new `CodexErrorInfo::PlanEntitlementExceeded` in protocol.rs + app-server-protocol shared.rs + analytics facts.rs `CodexErrKind`). `codex-api/src/api_bridge.rs` maps 401 + `indicates_plan_entitlement_rejection(body)` (semantic heuristic: context/token + plan/quota/exceed language) → PlanEntitlementExceeded with actionable message; plain auth 401s keep UnexpectedStatus path. Kimi skips `handle_unauthorized` recovery (env_key provider → `unauthorized_recovery()` returns None), so it flows through map_api_error.
- Test: `map_api_error_classifies_entitlement_401_separately_from_bad_credential` — green.
- Full `cargo check --workspace` passes.

### Fix 4 — K3 reasoning levels (MOSTLY COMPLETE, tests pending)
- models.json: `supported_reasoning_levels` = low/high/max, default stays `max`.
- `codex-rs/core/src/client.rs`: replaced forced `"max"` with `kimi_code_reasoning_effort()` mapping — None/Minimal/Low/light/min→low, Medium/High→high, XHigh/max/ultra/unknown-custom→max, None→max (K3's default). Unsupported values are normalized locally (never sent → no remote 400). Effort preserved across resume via existing `turn_context` logic (supported_reasoning_levels now contains user's effort).
- **STILL NEEDED for Fix 4**: request-shape tests (low/high/max/default) in `codex-rs/core/src/client_tests.rs` (extend `kimi_code_k3_chat_uses_required_max_reasoning_and_standard_tools` pattern — it currently asserts `Some("max")` for default and still passes); picker snapshot showing all 3 levels (models-manager `manager_tests.rs::bundled_models_json_contains_kimi_code_k3` asserts old values: context_window Some(1_048_576) and single max level — **MUST UPDATE** to 262_144 + 3 levels, will fail now); note in PR description about prompt-cache invalidation on effort change.

## What's NOT started

### Fix 3 — completion guard (biggest item, see mandate §3)
- `codex-rs/codex-api/src/endpoint/chat_completions.rs`: `ChatChoice.finish_reason` parsed but discarded — plumb it into normalized completion event (currently always `end_turn: None`). Handle stop/length/content_filter/tool_calls/unknown.
- `codex-rs/core/src/session/turn.rs`: response with no tool follow-up ends turn — add bounded, provider-neutral completion guard distinguishing final answers from prospective progress narration (use structured/semantic classification, NOT regex on sentences). Bound continuation attempts, one actionable error on exhaustion, persist completion metadata to rollout.
- Regressions required: final text answer completes; paraphrased progress narration doesn't; `length` finish never → successful task_complete; tool call continues loop; bounded stop without flooding; resume/replay stable.
- Live acceptance: real TUI K3 multi-step coding objective on temp workspace (see mandate §3 "Required live acceptance").

### Fix 5 — CI real-tui just (small)
- Failed job: https://github.com/agtico/PfTerminal/actions/runs/29587681799/job/87908612262 — `just: command not found`. Find the workflow (`.github/workflows/`), install pinned just (e.g. `taiki-e/install-action@just` or extractions/setup-just) or invoke the underlying command directly. Must execute+pass, not be optional.

## Merge gates (mandate §Final)
1. All fixes + regressions  2. real-tui CI green  3. focused suites pass  4. live K3 agentic acceptance from fresh temp home  5. `git diff --check`  6. secret scan (no real/test credentials in commits/logs/snapshots)  7. Update PR #50 description: effective context behavior (256K default, 1M opt-in), supported reasoning levels, live acceptance evidence, honest limitations.

## State / gotchas
- Uncommitted fix work: 10 files (see `git status`). Commit these before pushing; branch tracks origin so `git push` updates PR #50 directly.
- Untracked junk to NOT commit: `canonical-assets/`, `test-results/`, `PR50-REQUESTED-FIXES.md` (mandate, keep local or delete).
- Kimi vault key for live testing: `API_KEY="$(pfterminal vault auth-helper provider/kimi_api_key)"` — never print.
- Pre-existing failures on main (NOT yours): `app::tests::assignment_bad_target_retries_durable_worker_once_then_pauses`, `duplicate_live_native_replacements_are_pruned`, stack overflow in `discard_side_thread_removes_agent_navigation_entry`.
- `kimi-code` provider: base https://api.kimi.com/coding/v1, env KIMI_API_KEY, wire Chat, id `kimi-code`, model slug `k3`. Provider key env-id doubles as vault label id (`KIMI_API_KEY` → `provider/kimi_api_key`).
- Build/test speed: use `cargo test -p <crate> -- <filter>`; full workspace check ~1min after warm.
