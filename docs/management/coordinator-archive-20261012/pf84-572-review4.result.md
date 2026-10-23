**Verdict: approve.** R1 is fixed in the real `/model` flow, and the two nits are done. Nothing blocks the merge. I only read the code; I didn't build or run anything.

## R1: fixed
- **Keeping the current model.** `set_provider_models` builds the id it keeps as `amazon-bedrock:<model>`, the same id format it gives Bedrock entries when it adds them. It keeps that id only if it was added earlier (`previous.contains`), so a model in the base catalog is never claimed. The kept entry stays in the list untouched, with its Bedrock display name, reasoning efforts and service tiers.
- **Removal later.** The kept id stays in the record of what `set_provider_models` added. When an account is added again, the same Bedrock model is skipped, so there's no duplicate and the metadata isn't lost. Once the session leaves Bedrock, the next check passes `None` and the entry is removed. The catalog test covers keeping it and the later removal.
- **Where the model comes from.** `named_bedrock_account_checked` reads it from `chat_widget.current_model()`, and only when `config.model_provider_id` is Bedrock. That provider id is the one `UpdateModelSelection` updates. `current_model()` also follows the collaboration mask, which `set_model` updates, so the test's `set_model` matches how the picker sets the model. The out-of-date `config.model` is no longer read on this path.
- **Removing `sync_runtime_models` from this handler is safe.** Withdrawing only removes ids recorded for Bedrock, so a session on another provider can't lose its current model here.
- **The test now matches the real flow.** It no longer sets `config.model`, and it checks that the kept entry still has `supported_reasoning_efforts`.

## Nits: done
- **Spawn order:** `selected_model_info` is now fetched after the child's provider correction, using the corrected `config`.
- **`TurnContext::with_model`** now uses `model_info_for_config` with the updated config. That covers the last direct call site.

## Remaining findings, ranked (none block the merge)
1. **Low, follow-up: `/providers` can still add a wrong Bedrock row.** `open_provider_manager` calls `sync_provider_manager_model_policy` (`provider_management_status.rs:152`), which still passes the out-of-date `config.model`.
   - `main` has the same call, but this PR makes the triggering case reachable: a session that starts on OpenAI with `gpt-5.5` picks `amazon-bedrock:openai.gpt-5.4` from `/model`, then opens `/providers`.
   - Then `sync_runtime_models(.., Some("gpt-5.5"), "amazon-bedrock")` runs. `resolve_model_for_provider` has no Bedrock case, so it falls through and returns `gpt-5.5`.
   - I don't think `corrected_catalog_provider` rewrites an OpenAI model name, so `include_runtime_model` would probably copy OpenAI's preset in as `amazon-bedrock:gpt-5.5`. That row would likely fail if picked. I didn't confirm this by running it.
   - **Fix:** pass the chat widget's current model there too, or skip that runtime sync on Bedrock.
2. **Nit (from round 3):** the policy refresh still runs on the UI thread after every check. It's acceptable, and skipping it when `offered` hasn't changed would cut the repeats.
3. **Nit (from round 2):** opening `/providers` still reads the vault twice.
4. **N2:** still open, as you noted earlier.