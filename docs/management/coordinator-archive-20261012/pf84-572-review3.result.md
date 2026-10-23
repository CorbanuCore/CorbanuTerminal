**Verdict: approve with fixes.** Nothing blocks the merge, and R2 and the ordering race are fixed. R1 is still not fixed in the real `/model` flow. The new test passes only because it sets `app.config.model` itself, which the app never does when the user picks a model.

I only read the code; I didn't build or run anything.

## Blocking
None.

## Should-fix

**R1 (still open). Removing the last Bedrock account can still delete the current model from the list.** `tui/src/app/provider_management_status.rs:131–132`, `tui/src/app/event_dispatch.rs:2091`

- **Why the fix doesn't take effect:** when the user picks a model in `/model` (`UpdateModelSelection`), the app updates `self.config.model_provider_id`, `config.model_provider` and `chat_widget.set_model`. It never updates `self.config.model`.
- `PersistModelSelection` writes the choice to disk but doesn't reload `self.config`. As a result, `self.config.model` keeps the model the session started with.
- **The failing case:**
  1. The session is on OpenAI with `gpt-5.5` and picks `amazon-bedrock:openai.gpt-5.4`.
  2. The user removes the last Bedrock account.
  3. `sync_runtime_models(.., Some("gpt-5.5"), "amazon-bedrock")` runs.
  4. `resolve_model_for_provider` falls through to `_ => model` and returns `gpt-5.5`, so `openai.gpt-5.4` is never put back.
- **Why the test passes anyway:** `provider_accounts_tests.rs:196–197` sets `app.config.model` directly.
- **Second problem, even when the model does come back:** `include_runtime_model` re-creates it as a bare "Configured runtime provider" entry. The Bedrock entry was removed first, so the display name and the reasoning and service-tier lists are lost.
  - The restored entry isn't tracked by `set_provider_models` either. When an account is added again, the full Bedrock entry is skipped and the bare one stays.
- **Fix:** keep the current entry inside `apply_named_bedrock_models` / `set_provider_models`.
  - Pass the current pair `(self.chat_widget.current_model(), self.config.model_provider_id)`.
  - When removing, keep that id, and keep it recorded as added by `set_provider_models`.
  - The cheaper alternative, passing `chat_widget.current_model()` as the preferred model to `sync_runtime_models`, fixes the model disappearing but not the lost metadata.
- **Test:** select the model through the `UpdateModelSelection` path, or at least don't set `config.model`. Then assert the kept entry still has `supported_reasoning_efforts`.

## Nits
- **Spawn order** (`core/src/tools/handlers/multi_agents_common.rs:375–380`): `apply_requested_spawn_agent_model_overrides` fetches `selected_model_info` before the `corrected_catalog_provider` switch (lines 384–403). That means it uses the parent's provider, not the child's. Fetch it after the correction. This is harmless today, because `find_spawn_agent_model_name` only resolves models the session's own manager lists.
- **`TurnContext::with_model`** (`core/src/session/turn_context.rs:527`), used at `turn.rs:1420`, still calls `get_model_info` directly. This is the one remaining call site. It's fine for now (I covered it in round 2); for consistency it could use the new helper.
- **Policy refresh on the UI thread:** `sync_provider_manager_model_policy` now runs `refresh_provider_policy` on the UI thread after every check: startup, every `/providers` open, and every `Finished` event, including device-code progress. That reads files on the UI thread, which matches the existing `open_provider_manager` and persistence paths. It's acceptable; skipping the sync when `offered` hasn't changed would cut the repeats.
- **Duplicate vault reads** when opening `/providers` (from round 2): not addressed.
- **N2:** still open, as you noted.

## Checks that pass
- **Ordering race:**
  - `begin_provider_models_check` runs on the UI thread before the job is spawned, and `is_latest_provider_models_check` runs on the UI thread before applying (`model_catalog.rs:191–199`), so stale answers are dropped.
  - A `None` from the newest check leaves the list unchanged.
  - The stale-check case in the test is correct.
- **Vault error:** an error now gives `None` and leaves the list unchanged. A vault that was never set up still gives `Some(false)`, which is correct.
- **No lock held during file reads in the job:** the job only reads the vault and sends the event.
- **R2:**
  - `model_info_for_config` covers `new_turn`, `/review`, all four spawn sites and both guardian sites.
  - The early return keeps every non-Bedrock and non-fallback path exactly as before.
  - The Bedrock fallback uses a static catalog with `auth_manager=None`, so there's no network call.
  - The guardian sites pass the turn's or spawn config, so the provider is the live one.
  - The unit test covers both branches.
- **Behaviour change in `turn_context.rs` is a pure refactor.** No `create_model_provider` import was left behind (the diff removes only the use).