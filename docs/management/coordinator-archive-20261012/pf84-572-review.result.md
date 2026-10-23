**Verdict: approve with fixes.** I found no blocking problems. The fix works for its stated case, and nothing changes when the feature is off or there's no Bedrock account. Before merge, fix the vault reads that now block the UI thread (S1), and either fix or document the stale models (S2) and the downgraded model metadata (S3).

## Blocking
None.

## Should-fix

**S1. Vault reads now block the UI thread and runtime**
- The new lookup at `provider_management_status.rs:59–83`, through `provider_named_accounts.rs:141`, does real vault work, not a quick label check. `list_provider_accounts()` → `list()` → `with_storage_lock` + `load_index`, which decrypts `secrets/local.age` using the passphrase from the keyring (`vault/src/lib.rs:534,726`).
- It runs synchronously in three places:
  - **Startup**, inside async `App::run` (`app.rs:1263`).
  - **`open_provider_manager`** (`provider_management_status.rs:97` → `:49`). The same function then reads the vault again off-thread: `load_named_accounts` is deliberately run through `spawn_blocking`.
  - **`provider_account_finished`** (`provider_accounts.rs:206`). This handler fires for every `Finished` event, including name-validation errors and the ChatGPT device-code progress message. The code at `provider_accounts.rs:~269` already says "Vault and default-login reads block; keep them off the runtime."
- Safety is fine: no secret is read, a `NotInitialized` vault returns early (`key_storage` only checks for files), and the output gate already opens the vault per session, so a keyring prompt isn't new. The cost is that a held storage lock or a slow keyring now freezes the UI.
- Cheap fix: derive "has an AWS-profile Bedrock account" from the `accounts` rows already loaded in the background and handled in `provider_manager_statuses_resolved`; the rows carry `provider_id` and `kinds`. Do the startup check in `spawn_blocking`, or the first time `/model` or `/providers` opens.

**S2. Models stay listed after the account is removed**
- `include_provider_models` (`model_catalog.rs:186`) only ever adds. After the last Bedrock account is removed, the Bedrock models stay in `/model` until restart.
- Picking one then runs on the default Bedrock account (managed key, then `AWS_BEARER_TOKEN_BEDROCK`, then the AWS SDK chain). It isn't blocked, because Bedrock is StatusOnly and the policy treats that as Ready (`runtime_selection.rs:190–202`).
- Fix: record which ids this path added (e.g. in a set on `ModelCatalog`) and retain-remove them when no account remains. Remove only those ids, not models from `model/list` or `include_runtime_model`.

**S3. Fallback model metadata is a real downgrade**
- The session's models manager is fixed per thread. `construct_model_info_from_candidates` (`models-manager/src/manager.rs:700`) can't match `openai.gpt-5.4`, so it falls back to `model_info_from_slug`.
- Compared with bundled `gpt-5.4` (what a session started on Bedrock gets):
  - apply_patch tool: none instead of freeform
  - shell type: `Default` instead of `shell_command`
  - parallel tool calls: off instead of on
  - truncation: 10 000 bytes instead of 10 000 tokens
  - base instructions: generic
  - reasoning levels: empty
- Things that aren't worse: the context window is 272k either way (coincidence), and the chosen effort is still sent because `build_reasoning` passes it through (`client.rs:1908`). Compared with before, Bedrock was simply unreachable; compared with starting on Bedrock, it's clearly worse.
- Cheap fix (about 10 lines in core): when the turn's provider `is_amazon_bedrock()` and the session's manager returned fallback metadata, look the model up in Bedrock's static catalog instead. Either export a `pub fn amazon_bedrock_model_info(slug)` from `catalog.rs`, or call `create_model_provider(info).models_manager_without_cache(None).get_model_info(..)`; Bedrock's is a static manager, so there's no network call.
- Don't map `openai.gpt-5.4` → `gpt-5.4` in `corbanu_catalog_source_model`. The OpenAI context window could differ from Bedrock's.
- If you defer this, add it to the doc as a known limit.

**S4. The selectability assertion is vacuous**
- `make_test_app`'s catalog has no provider policy, so `provider_is_selectable` returns true without checking anything (`model_catalog.rs`, `is_none_or`). The test at `provider_accounts_tests.rs:173–176` therefore doesn't prove StatusOnly → Ready.
- Fix: set a policy (as `chatwidget/tests/popups_and_settings.rs:3922` does), or test `preset_is_selectable` on the added preset with a real policy.

**S5. The doc should cover two limits**
`docs/provider-accounts.md:115` should say:
- Picking a Bedrock model without choosing *Use for this session* (or `--account`, or a `[provider_accounts]` default) uses the default account.
- The fallback-metadata limit from S3, unless it's fixed.

## Nits
- **N1. Misplaced doc comment.** `provider_named_accounts.rs:137–140`: `load_rows`'s doc ("Lists named accounts…") now sits above `holds_named_account`, so it merges into that doc and `load_rows` loses it. Move it back.
- **N2. Bedrock sessions change slightly.**
  - Root cause: `canonical_catalog_provider` (`model-provider-info/src/lib.rs:431–436`) knows only `openai.gpt-5.5` and `openai.gpt-5.4`.
  - Before: Bedrock's `model/list` entries have no provider, so the three `openai.gpt-5.6-*` models fail `preset_is_selectable` and are hidden in a Bedrock session unless one is the current model.
  - With a named account: this path adds them with a provider, so Terra and Luna appear. A Bedrock session now shows different models depending on whether a named account exists.
  - There are no visible duplicates, since the picker dedupes by (model, provider) at `model_popups.rs:569`.
  - Consider adding the three 5.6 ids to `canonical_catalog_provider` so both cases agree.
- **N3. Spawned agents and default account.** The models also show in the spawn-agent picker (`spawn_orchestration.rs:2290` `provider_for_model` takes the first match, which is Bedrock). A spawned agent uses the default account unless the spawn sets `account`. That's fine, but worth knowing.

## Checks that pass
1. **Correctness**
   - **Duplicates:**
     - In a Bedrock session, 5.5 and 5.4 are skipped through the `canonical_catalog_provider` fallback.
     - If the current model is one of the 5.6 models, it's skipped: `include_runtime_model` gave it a provider first.
     - Unselected 5.6 models are added (see N2).
     - Re-running the include is idempotent (covered by the test).
   - **Current-model marking** (`model_popups.rs:684`) compares model and provider with `config.model_provider_id`, so after the switch it marks correctly.
   - **Default:** `is_default` is forced off.
   - **Grouping and switch:** the models land in the "other" tab labelled "… via amazon-bedrock". Switching uses `preset.provider_id` (`model_popups.rs:1016`), not slug lookup, so `openai.gpt-5.4` resolves to amazon-bedrock.
   - **Resume:** the thread resumes on Bedrock with Bedrock's own manager, so no metadata warning. I didn't check whether the named account itself survives resume; that's existing PF-84 behaviour, not this diff.
2. **Feature off / no account / Bedrock as default provider:** early returns, so behaviour is unchanged. The only exception is the N2 difference in a Bedrock session that also holds a named account.
3. **Vault:** labels only, no secret read; a `NotInitialized` vault is skipped and errors are logged rather than propagated. Cost is covered in S1.
4. **Overlap with 288fcd9740:** `replace_listed_models` keeps every preset whose id isn't in `listed_ids`. `mark_listed` takes the bootstrap `model/list` only, captured before the include, and the added ids (`amazon-bedrock:<model>`) never match listed slugs, so a refetch keeps the Bedrock models. Expect a trivial textual conflict at `app.rs:1262–1263`: both commits insert a line right after `let model_catalog = …`.
5. **Tests:** without the fix, both tests fail to compile; with the helper turned into a no-op, the "offered contains GPT-5.5" assertion fails. Apart from S4, the tests are meaningful. Neither covers the startup, `/providers` or after-add/remove call sites, or removal.