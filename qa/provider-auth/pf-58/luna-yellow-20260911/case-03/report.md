# Case 3 — Inspect the unified provider catalog

Status: **BLOCKED**

Candidate SHA-256: `4a8eba7b10199ea49aee42a720687b63cb4b1b2194c2ce164d1b9f6f88ee510e`  
Platform: macOS arm64

## Summary

The candidate launched to the main TUI. `/providers`, `/wallet`, and `/model` opened promptly. Every provider visible in the provider catalog was opened and inspected without exposing credential values. Corbanu API was present. No legacy Corbanu Plan purchase, recovery, or details action was visible.

The case is blocked because the supplied profile has no local wallet, and the `/model` catalog exposed only five OpenAI models; Ambient could not be selected. No credential, wallet, or plan-changing action was performed.

## Observed checks

- **Passed — startup and prompt:** The TUI reached its main screen after the visible directory-trust confirmation. Evidence: `00-startup.txt`, `01-ready.txt`.
- **Passed — provider catalog:** `/providers` opened promptly and listed OpenAI, Claude Account, Corbanu API, Anthropic, Ambient, Kimi Code, Z.AI, and DeepSeek. Evidence: `02-providers-initial.txt`.
- **Passed — provider inspection:** OpenAI showed `Enabled · configured`; Claude Account showed `Unavailable · current · unavailable`; Corbanu API was present and showed `Not configured · unavailable`; the API-key entries showed setup choices and no secret values. Evidence: `03-openai.txt`, `04-claude-account.txt`, `05-corbanu-api.txt`, `06-anthropic.txt`, `07-ambient-provider.txt`, `08-kimi-code.txt`, `09-zai.txt`, `10-deepseek.txt`.
- **Passed — Corbanu API presence:** The entry was present in the unified provider catalog. Evidence: `02-providers-initial.txt`, `05-corbanu-api.txt`.
- **Passed — wallet safety surface:** `/wallet` opened promptly and showed only `Create wallet` and `Restore wallet` for the no-wallet state; no legacy plan-sale, recovery, or details action was visible. Evidence: `12-wallet-initial.txt`.
- **Blocked — wallet data/API surface:** The wallet screen showed `No local wallet`, so dollar balance, top-up, model prices, API-key management, and a wallet-level Corbanu API entry could not be inspected. Creating or restoring a wallet was not authorized. Evidence: `12-wallet-initial.txt`.
- **Blocked — Ambient model catalog:** The visible `/model` list contained only five OpenAI models. Down navigation wrapped from item 5 to item 1, and Right produced no visible catalog change. Ambient, GLM 5.2, and Kimi 2.7 were not exposed, so the Ambient-specific and cross-provider Kimi checks could not be evaluated. Evidence: `13-model-initial.txt`, `14-model-after-openai.txt`, `15-model-openai-end.txt`, `16-model-right.txt`.
- **Passed — secret handling:** No credential value, login code, wallet recovery material, or browser content was entered or recorded. Evidence: provider and wallet checkpoints above.

## Blockers

- The profile has no local wallet; wallet creation/restoration was outside the authorized actions.
- Ambient is displayed by `/providers` as `Not configured · unavailable` and is absent from the visible `/model` catalog. No authorized credential setup was available to change that state.
- Because Ambient was not selectable, the required GLM 5.2-only check and the comparison with other providers’ Kimi models remain unresolved.

## Test-instruction ambiguity

- The instruction says to inspect a Corbanu API entry in `/wallet`, but the observed no-wallet screen had no such entry and offered only create/restore actions.
- The instruction says to select Ambient in `/model`, but the visible menu had no provider selector and only OpenAI models while Ambient was unavailable in `/providers`.

## Limitations

- Results are limited to the supplied compiled candidate on macOS arm64; no Linux inference was made.
- The live validity of copied managed credentials was not assumed or tested with a model request.
- No wallet creation/restoration, purchase, recovery, permission change, browser login, native-keyring prompt, or credential entry was performed.
- The supplied harness disables startup update checks; update behavior was not used as evidence.
- `/vault` was not opened because the supplied action sequence specified `/providers`, `/wallet`, and `/model`; no vault hang was observed or ruled out.

The target tmux session was exited cleanly after the observations.
