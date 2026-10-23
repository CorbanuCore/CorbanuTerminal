**Verdict: approve with fixes.** Nothing blocks the merge. All five round-1 findings are addressed. Running the vault check in the background did introduce one regression: an account change can remove the model the user is currently on. It also has a small ordering race. The S3 metadata fix covers normal turns only; `/review`, spawned agents and the guardian still get the fallback metadata.

## Blocking
None.

## Should-fix

**R1. Removing the last Bedrock account can delete the current model from the list** (`model_catalog.rs:196–197`, `provider_management_status.rs:89–101`)
- How it happens:
  1. The user picks `amazon-bedrock:openai.gpt-5.4` from the models this change adds.
  2. The selection handler (`event_dispatch.rs:~2060–2100`) never calls `sync_runtime_models`, so `set_provider_models` still owns that entry.
  3. The user removes the last Bedrock account.
  4. `provider_account_finished` → job → `set_provider_models(empty)` deletes the entry.
- Opening `/providers` doesn't repair it. `open_provider_manager` runs `sync_runtime_models` first, which does nothing because the entry still exists, and then the job deletes it.
- Result: `/model` loses the current model and its marking, and `provider_for_model` returns `None`. It reappears only after a later `sync_runtime_models`.
- **Ordering race:** concurrent jobs (startup, `/providers` open, every `Finished` event, including progress messages) each read the vault and then apply, with no ordering between them. A job that read the vault before a removal can apply after a newer job and bring the models back. Each job also uses a copy of `Config` taken when it was spawned.
- **Fix:**
  - The blocking job only computes the boolean and sends an `AppEvent`, tagged with a generation number.
  - The UI thread drops stale events, applies `set_provider_models`, then runs `sync_runtime_models` with the live `self.config`.
  - Alternatively, `set_provider_models` keeps the preset matching the current (model, provider) instead of removing it.
- Add a catalog test: `set_provider_models` → current model selected → `set_provider_models(empty)` keeps the current model.

**R2. The S3 metadata fix only covers `new_turn`** (`core/src/session/turn_context.rs:1108–1124`)
These places still ask the session's models manager directly, so a session that started on another provider still gets fallback metadata for `openai.gpt-*`:
- **`/review`:** `session/review.rs:16–20`. A review on Bedrock after the switch shows the "metadata not found" warning and the reduced tool set.
- **Spawning agents:** `tools/handlers/multi_agents_common.rs:320–323`, `488–492` and `528–532`. With fallback metadata, `supported_reasoning_levels` and `service_tiers` are empty. A spawn with `model_provider=amazon-bedrock` and an explicit `reasoning_effort` (or service tier) is therefore rejected with an empty "Supported reasoning efforts:" list, and the default effort becomes `None`.
- **Guardian:** `guardian/review.rs:791–797` and `guardian/review_session.rs:726–733`.

These paths existed before. This change makes them easier to reach because the spawn and `/model` pickers now offer Bedrock models (N3).

**Fix:** move the fallback into one `Session` helper, e.g. `model_info_for(provider_info, model, cfg)`, and call it from these sites. At minimum cover review and spawn; otherwise list them as known limits.

**Things I checked in the S3 fix that are fine:**
- The Bedrock manager it uses is a static catalog, so there's no network call.
- `with_config_overrides` still applies because the same `per_turn_config` is passed.
- `config_model_catalog=None` is right, since the user's catalog has already missed the model.
- Bundled `comp_hash` is `None` for gpt-5.4, 5.5 and 5.6, so the change adds no new trigger for compaction on a model switch (`turn.rs:1369`).
- `with_model` at `turn.rs:1420` for the previous model still gets fallback metadata (context window 272k), which happens to match Bedrock's.

## Nits
- **Policy lock held during file reads.** The job calls `refresh_provider_policy`, which holds the `provider_policy` write lock while `host.resolve()` reads auth storage, the vault key ids and the eligibility store (`model_catalog.rs:128–137`). On the UI thread, `provider_policy()` (taken for every preset in `preset_is_selectable` when `/model` renders) waits on that lock. Statuses don't depend on which models are in the list, so either drop this refresh or run it on the UI thread together with the R1 event.
- **Duplicate vault work.**
  - Opening `/providers` now reads the vault twice in parallel: `load_named_accounts` and `holds_named_bedrock_account`.
  - Every `Finished` event, including the ChatGPT device-code progress message, starts a vault read.
  - Deriving the boolean from the `accounts` rows already handled in `provider_manager_statuses_resolved` would remove both.
- **A transient vault error now removes the models** (`provider_named_accounts.rs:150–157`). Before, an error only meant "don't add". Consider leaving the list unchanged on `Err`.
- **Tests.**
  - Nothing covers the core fallback. A unit test is cheap: a Bedrock provider plus a manager without the slug should give non-fallback metadata.
  - The app test calls `apply_named_bedrock_models` directly, so the spawned and call-site paths aren't exercised.
  - `ProviderStatusHost::from_config` in the test follows the existing `popups_and_settings.rs` pattern, so it's acceptable.
- **Per-turn rebuild.** The fallback rebuilds Bedrock's static catalog (parsing the bundled models) on every fallback turn. It's negligible; a `OnceLock` would remove it.
- **N2 is still open.** The three `openai.gpt-5.6-*` ids still change visibility depending on whether a named account exists.

## Checks that pass
- **S1:** `set_provider_policy` now runs before the sync. Startup, `/providers` and the account handler all use `spawn_blocking`. N1 is fixed.
- **Locks:** no deadlock. `set_provider_models` takes `provider_model_ids` then `models`; every other function takes a single lock.
- **Interaction with 288fcd9740:**
  - Its `replace_listed_models` takes `listed_ids` then `models`, which doesn't conflict with the order above.
  - It keeps the added presets, because their `amazon-bedrock:<model>` ids are never in `listed_ids`.
  - If a refetched list contains the same (model, provider), the added copy is dropped. Its id stays recorded, so the later removal does nothing. That's harmless.
  - Expect trivial textual conflicts: both commits add a struct field and a `new()` initializer to `ModelCatalog`, and a line after `let model_catalog` in `app.rs`.
- **Other catalog functions:** `replace_gpu_models` only touches `gpu-*` entries. `include_runtime_model` skips any model `set_provider_models` already added, so ownership stays with `set_provider_models`; that's the cause of R1.
- **Snapshot behaviour:** a `/model` popup that is open when a job applies keeps its snapshot. Picking an entry that was just removed still goes through the policy check in `event_dispatch.rs:2061`, so nothing crashes.
- **S2 / S4:** removal deletes only the ids this path added. The catalog test proves the zai and listed entries survive. S4 now uses a real policy with `preset_is_selectable` and covers removal.
- **S5:** the doc line about the default account is accurate.