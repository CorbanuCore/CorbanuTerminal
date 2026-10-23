# #461 and #462 fix evidence (2026-10-11)

Fixes [#461](https://github.com/CorbanuCore/CorbanuTerminal/issues/461) and
[#462](https://github.com/CorbanuCore/CorbanuTerminal/issues/462), found by the
[S04 independent acceptance](../independent-acceptance-s04-20261011/README.md).

- **Before:** debug build of `origin/main` at `5849adee3b`.
- **After:** debug build of the fix branch.
- **Environment:** macOS arm64, disposable homes under `env -i`, `CORBANU_TEST_NO_NATIVE_KEYRING=1`, private
  `tmux -L` socket ([lib.sh](captures/lib.sh)). Synthetic values only, except one live turn.

## What changed

- **#461:** while an onboarding text field has focus, every printable key is text, including in an empty field. That
  covers the API key, the Claude token and code, and a named account's name and value. The same holds while the value
  is being saved. Ctrl+C and Ctrl+D still exit. On exit, the TUI also flushes terminal input it never read. That only
  covers bytes already queued; bytes that arrive later can still reach the shell, so the real protection is that `q`
  no longer quits.
- **/providers audit:** no change needed. Its text fields (`VaultSecretEntryView`, `CustomPromptView`) already
  receive every key except Ctrl+C, Ctrl+R and Ctrl+U, and app shortcuts are off while a view is open.
- **#462 /status:** a non-OpenAI provider's stored key now shows as "`<provider>` API key configured" instead of
  "OpenAI API key configured".
- **#462 loop:** config can name an API-key provider (Kimi, Z.AI, or `forced_login_method = "api"`). Onboarding now
  begins setup for that provider when it opens on its key entry, and when you choose "Provide your … API key". Saving
  the key then continues to chat.
  - Before, setup began for `openai`, or not at all, so the save was ignored. It only worked when background provider
    discovery finished after the save.
  - Onboarding also no longer falls back to saving the key to the first listed provider while the screen names
    another one.

## Results

| Check | Before | After |
| --- | --- | --- |
| **#461:** type `qa-probe` in the onboarding *Add another account* name field | TUI exits, rc 0 | Name shows `qa-probe` |
| **#461:** type `abc-q-…` in the masked account key field | TUI exits | 31 `•` |
| **#461:** type `qsynthetic-…` in the empty default-key field | TUI exits | 25 `•` |
| **#462:** `/status` in a Z.AI-only home | `OpenAI API key configured (run corbanu login …)` | `Z.AI API key configured` |
| **#462:** config names `kimi-code`; save the key, then Enter | Back to "Provide your Kimi Code API key" | Trust prompt, then chat on Kimi Code K3 |
| **#462 live:** fresh home, `-m glm-5.3-flash -c model_provider="zai"`, real Z.AI key pasted through a deleted tmux buffer | n/a | Continues to chat; `• pong`; `/status`: `Z.AI API key configured` |

- **Captures:** [461-before](captures/461-before.txt), [461-after](captures/461-after.txt),
  [462-status-before](captures/462-status-before.txt), [462-status-after](captures/462-status-after.txt),
  [462-loop-before](captures/462-loop-before.txt), [462-loop-after](captures/462-loop-after.txt),
  [462-live-after](captures/462-live-after.txt).
- **Scripts:** [qa461.sh](captures/qa461.sh), [qa462status.sh](captures/qa462status.sh),
  [qa462loop.sh](captures/qa462loop.sh) (its synthetic key originally had an `sk-` prefix), [live.sh](captures/live.sh).

## Regression tests

The new tmux tests are in `tui/tests/suite/multi_provider_onboarding.rs`:

- `tmux_onboarding_text_fields_type_q_without_quitting` types the values key by key, not as a bracketed paste:
  - a Z.AI key starting with `q`;
  - the account name `qa-quota`;
  - a masked key full of `q`.

  It also scans the scrollback and the home for the values.
- `tmux_preset_provider_key_entry_continues_to_chat` covers the #462 loop.

Both fail against the before binary (`CARGO_BIN_EXE_corbanu=<before>`): the first because the pane dies, the second
with "onboarding looped back to key entry". Both pass on the fix.

The unit tests are in `onboarding_screen.rs`, `auth.rs` and `status/tests.rs`. They cover:

- every text field with `q`;
- the saving states;
- Ctrl+C;
- `q` in the provider list;
- the #462 Begin paths;
- `/status` naming.

## Video

[#461 fixed](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-84-s04-pf84-461-onboarding-q-f575311571d1-2026-10-11.mp4).
It types a `q`-leading default key, then the name `qa-quota`, then a masked key full of `q`, and onboarding saves the
account. The recording, the cast and the disposable home have no hits for the synthetic values.
