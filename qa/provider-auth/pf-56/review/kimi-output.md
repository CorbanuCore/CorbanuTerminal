Review complete. Final report:

# PF-56 external review — unified provider authentication

## 1. Reviewer identity

Model: `moonshotai/kimi-k3` (Kimi 3.0), provider: `vercel`, effort: high. No subagents spawned; read-only inspection only (git/rg/sed); no builds, tests, credentials, or network flows.

## 2. Reviewed range and key paths

Range: `f7356a94e0..21cf3199f2` (PF-48 → PF-55), qualification allocation `cf201dff78`.

Key paths: `codex-rs/provider-auth/src/{lib,status,status_contract,eligibility,management,auth_flow,api_key_flow,openai_account_flow,openai_account_controller,claude_account_flow,claude_account_controller,claude_account_settlement,runtime_selection}.rs`; `codex-rs/app-server-client/src/provider_auth.rs`; `codex-rs/login/src/auth/{manager,provider_key_vault,external_bearer}.rs`; `codex-rs/model-provider-info/src/lib.rs` (`credential_source`); `codex-rs/tui/src/{startup_provider,model_catalog,provider_status_host,provider_management_host,provider_account_auth_host,provider_auth_effect_executor}.rs`; `codex-rs/tui/src/app/{event_dispatch,provider_management,provider_management_auth,provider_management_status}.rs`; `codex-rs/tui/src/chatwidget/{provider_model_policy,provider_manager,model_popups,claude_auth_adapter,wallet_menu,wallet_receipt}.rs`; `codex-rs/tui/src/onboarding/provider_setup.rs`; `codex-rs/tui/src/spawn_orchestration.rs`; suites `multi_provider_onboarding.rs`, `provider_management.rs`, `provider_convergence.rs`.

## 3. Findings (by severity)

### High — Provider policy evaluates against a stale pre-authorization status snapshot, so startup-authorized command providers remain unselectable and a current command provider blocks turns

- `codex-rs/tui/src/chatwidget/provider_model_policy.rs:23-27,40-71`; `codex-rs/provider-auth/src/runtime_selection.rs:56-100,150-180`; `codex-rs/tui/src/startup_provider.rs:45-57`.
- Failure mode: `startup_provider::resolve` validates every `auth.command` provider and calls `host.set_runtime_authorizations(...)`, but `ProviderModelPolicy::new` captures `statuses = host.resolve()` **before** that call, and `ProviderRuntimeAuthorizations` is stored side-by-side, never re-applied. `ProviderRuntimeSelectionPolicy::assess` reads the snapshot's `availability == StatusOnly` and, with `NotChecked`, returns `Blocked{RuntimeAuthorizationRequired}` for `ModelPicker`/`AutomaticDefault` and `RequiresRuntimeAuthorization` for `ExplicitRequest`/`Resume`/`NativeSpawn`. `apply_to_status_catalog` (`runtime_selection.rs:56`) exists but is only exercised inside `ProviderStatusHost::resolve`, which the policy never re-invokes after authorization.
- Impact: a command-auth provider validated successfully at startup is hidden from the model picker (`model_popups.rs:256-260,459` keep only `Ready`); `UpdateModelSelection`/`PersistModelSelection` reject it (`event_dispatch.rs:1863-1874,4028-4048`); if it is the configured current provider, every `UserTurn`/`Review`/`Compact` is refused by the `current_requires_recovery` gate (`chatwidget.rs:1798-1810`) until some unrelated `refresh_provider_policy()` runs (only `event_dispatch.rs:547,714,2388` and `set_current_runtime`). This directly violates "successful command authorization permits normal selection" and makes `has_usable_provider` false for command-only setups, forcing onboarding (`startup_provider.rs:66-79,113-115`). Native-spawn gates happen to pass because they treat `RequiresRuntimeAuthorization` as non-blocking (`spawn_orchestration.rs:2354-2370,2395-2417,3067-3084`), so behavior is inconsistent across surfaces.
- Repair: construct the policy only after authorizations are applied (apply to the snapshot or call `policy.refresh()` after `set_runtime_authorizations`), or fold authorization into `assess` for command capability instead of a snapshot mutation, so every policy surface sees a single consistent view.

### High — API-key replace flow deadlocks permanently after the 120s timeout when the first refreshed status is already Configured

- `codex-rs/provider-auth/src/auth_flow.rs:402-407` (guard), `:364-387` (timeout), `:321-362` (persistence_finished).
- Failure mode: `TimeoutElapsed` moves `Settling → OutcomeUnknown` and issues exactly one `RefreshProviderStatus`. In `status_resolved`, a `Configured` status with `flow.intent == Replace` is swallowed (`replacement_is_unsettled` → `applied()`, no effect, no state change). If the correlated `PersistenceFinished` was already consumed (or the executor's persistence RPC reported `StorageUnavailable` while the write actually landed), nothing ever re-enters the controller: `commit_in_progress` rejects new starts and cancels, and `ProviderAuthEffectExecutor::persist_api_key` (`provider_auth_effect_executor.rs:144`) waits on `action_rx` forever.
- Impact: the spawned persistence task in `SaveSharedProviderApiKey`/`SaveProviderManagerApiKey` never completes → no `SharedProviderApiKeyFinished`/`ProviderManagerApiKeyFinished` → the TUI entry view never resolves; the manager session stays in `Authenticating`, its `Refresh` actions are rejected (`management.rs:196-209`), and the only recovery is process restart. Auth controller is renderer-independent and shared, so both hosts are affected.
- Repair: in the `OutcomeUnknown` + `Replace` + `Configured` branch, re-issue `RefreshProviderStatus` (bounded) or complete when the stored credential post-dates the attempt; and/or add a second deadline in `OutcomeUnknown` that transitions to `Failed{StorageUnavailable}` so hosts always settle.

### Medium — Deferred Corbanu Plan auto-selection (no usable fallback) is defeated by the stale policy and surfaces a spurious "inactive or unavailable" error

- `codex-rs/tui/src/chatwidget/wallet_menu.rs:1567-1574` (`select_pfterminal_plan_provider` when `!deferred.has_usable_fallback()`); `codex-rs/tui/src/app/event_dispatch.rs` `DeferredCorbanuPlanConfigured` handler (teardown without `model_catalog.refresh_provider_policy()`); gate at `event_dispatch.rs:1863-1874,4028-4048`.
- Failure mode: the deferred flow persists the plan credential and activates it via a throwaway `ProviderStatusHost`, but the model-catalog policy's snapshot still shows Corbanu Plan `NotConfigured`. The immediately dispatched `UpdateModelSelection`/`PersistModelSelection` are rejected by the selectability gate.
- Impact: a first-run user with no other usable provider who completes a deferred plan purchase ends the session with a stored, activated credential but **no current provider selected** and a misleading error; functionality only self-heals on next launch. The fallback case (preserve current) is pinned by tmux tests (`multi_provider_onboarding.rs:113-148,265-271`); the no-fallback selection path is not.
- Repair: refresh the model-catalog provider policy (or the shared status host's account metadata) in the `DeferredCorbanuPlanConfigured` handler before dispatching the selection events.

### Medium — Provider-manager eligibility changes never propagate to the model-catalog policy

- `codex-rs/tui/src/app/provider_management_status.rs:99-123` (manager-only refresh); absence of `model_catalog.refresh_provider_policy()` anywhere in `provider_management*.rs` / management transition handlers.
- Failure mode: deactivating/reactivating a non-current provider updates the eligibility file and the manager's own statuses, but the picker/selectability snapshot is untouched. Deactivated providers remain listed and selectable; choosing one then fails the `UpdateModelSelection` gate with a confusing error; reactivated providers stay hidden.
- Impact: stale picker state and contradictory errors for the rest of the session; the current provider itself cannot become stale-inactive (current deactivation forces replacement → `UpdateModelSelection` → refresh), which limits severity.
- Repair: refresh the model-catalog policy on `ProviderManagerPersistenceFinished` (or in `apply_provider_management_transition` after any applied mutation).

## 4. Test/evidence gaps

- No test pins that a startup-authorized command provider is immediately picker-selectable and usable as current without an intervening refresh (Finding 1). `provider_convergence.rs` covers visibility/no-enrollment-UI and blocked-when-failing, not the successful-authorization selection path through `ProviderModelPolicy`.
- No unit test for `OutcomeUnknown` + `Replace` + `Configured` arriving with no pending persistence result, and no second deadline or recovery from `OutcomeUnknown` in either API-key or Claude managed-token flows (Finding 2).
- No tmux/integration coverage for the deferred-plan **no-fallback** selection completing end-to-end in-session (Finding 3).
- No test that manager deactivation of a non-current provider removes it from `/model` and that reactivation restores it within the same session (Finding 4).
- Verified boundaries with no gaps found: no regex on LLM/provider call paths in the reviewed crates; secrets use zeroizing wrappers with redacted `Debug` and cross `AppEvent` only as opaque newtypes; deferred-plan preserve-current selection policy is pinned; resume session-specific recovery (`session_recovery_only`) and the unusable-current spawn/resume blocks are implemented and suite-tested.

## 5. Verdict

Not `CLEAN`. Two high-severity findings (stale-snapshot authorization gating; replace-flow `OutcomeUnknown` deadlock) and two medium findings (deferred-plan selection, manager→picker staleness) are actionable and in scope.

