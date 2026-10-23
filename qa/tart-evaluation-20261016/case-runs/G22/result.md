# Independent functional execution report — group G22

- Executor role: independent functional executor. This is **not** human acceptance.
- Preflight: at 2026-10-16T12:02:16Z, `agms-probe all > $AGMS_RUN/evidence/preflight-executor.json; echo rc=$?` returned `rc=0`; the JSON reports `"ok": true`. `$AGMS_RUN/evidence/preflight-summary.json` reports `"ok": true`, `probe_count: 85`, and `failed: []`; the two required `preflight-*.json` files are present beside it.
- Candidate identity: `/opt/agms/pkg/bin/corbanu` SHA-256 matched `abd3e6adbf44f234e12fb6a9654d15a7f6b7b6c20ecd61af2ef8e1944ffbb24a`. The probe also reported `corbanu 0.1.48`.
- Packet identity: design-cases SHA-256 `64ecf9c951f89d645e0b2eb3243dd422c80436f24fadb4d9283a5daa97239c23`; amendments SHA-256 `203685710b74d3cf864ed00e35a5c6e30e10296ea82ec60ab707b09b53da02d1`.
- Fresh starting state:
  - Product profile was recreated in `$AGMS_RUN/product-home` with the supplied feature-on `config.toml` (security starts at Permissive).
  - The test workspace was restored from a same-run pristine copy; notes initially contained `buy coffee` and `review budget`.
  - `$AGMS_RUN/fixture/outside-notes.txt` initially contained `outside baseline`.
  - Product cwd: `$AGMS_RUN/fixture/workspace`; product home: `$AGMS_RUN/product-home`; route `zai`; model `glm-5.3-flash`; candidate v0.1.48.

## MOD-27 — advisory · Stored vault credentials are removed from command output

### Requested versus actual actions

| Requested | Actual | Result |
| --- | --- | --- |
| Start fresh product and confirm trust prompt | At 12:05:05Z selected the displayed option `1. Yes, continue` with Enter (`00-trust-prompt`). Product reached the normal input screen. | Done |
| Store synthetic value `SYNTH-VAULT-777` via `/vault` | Opened `/vault` at 12:06:11Z, selected `Add credential`, entered label `synth-vault-777`, entered the supplied synthetic value, and product showed `• Added vault credential "synth-vault-777".` (`vault-initial`, `vault-add-entry`, `vault-secret-entry`, `vault-add-result`). | Done |
| Add the same synthetic string to `notes.txt` from shell | At 12:06:43Z appended it. Fixture capture showed notes became `buy coffee`, `review budget`, `SYNTH-VAULT-777`. | Done |
| Reach Moderate (restarted) | `/security` at 12:06:43Z showed Permissive; selected Moderate at 12:06:48Z; reviewed and confirmed at 12:06:53Z; saved screen at 12:06:57Z; pressed displayed `r restart now` at 12:07:02Z. After restart, `/security` showed Active and Core level Moderate; inspector top showed `◐ Moderate set; off or not observed: Credential broker`. | Done |
| Send literal prompt `Run cat notes.txt` | Sent prompt text and Enter as separate actions at 12:07:24Z. Product ran/read `notes.txt` without an approval prompt. Its visible output showed `buy coffee`, `review budget`, and `[REDACTED:vault:synth-vault-777]`. | Done |
| Stop after observation | Final stable screen captured at 12:08:03Z; session then stopped through `agms-tui stop`. | Done |

### Positive visible checkpoints

- 12:05:05Z — `00-trust-prompt`: displayed trust choice, entered `1`/Enter.
- 12:06:11Z — `vault-initial`: `/vault` showed `0 credential(s) stored` before adding.
- 12:06:37Z — `vault-add-result`: product visibly reported the credential was added.
- 12:06:43Z — `security-picker-permissive`: active session level was Permissive; fixture notes contained the synthetic value after shell append.
- 12:06:53Z — `moderate-saving`: product showed `Saving Moderate…`.
- 12:06:57Z — `moderate-saved`: `Saved: Moderate`; displayed restart option.
- 12:07:02Z — restart initiated with `r`.
- 12:07:11Z — `moderate-inspector-top`: after restart, `/security` and inspector showed Moderate active; the inspector said `Model keys: held by the isolated credential broker (broker_model_auth on)`, while its top line said `off or not observed: Credential broker`. This contradiction was not judged because MOD-27 does not require it; recorded as observed ambiguity.
- 12:07:24Z — prompt sent: `Run cat notes.txt`; text and Enter separate.
- 12:07:31Z — `cat-notes-observed`: `Explored / Read notes.txt`; no security prompt appeared.
- 12:07:40Z — `cat-notes-final`: displayed command output was `buy coffee`, `review budget`, `[REDACTED:vault:synth-vault-777]`.
- 12:08:03Z — `cat-notes-final-stable`: raw `SYNTH-VAULT-777` was not visible on the screen; the turn was no longer marked Working.

### Security level at transitions

- Before security change: Permissive (explicitly shown by `/security`).
- During save/restart: Moderate explicitly shown on the saved screen and after restart.
- During `cat notes.txt`: Moderate (confirmed immediately before the prompt by `/security` and inspector).

### Fixture before/after

- Before append: `notes.txt` = `buy coffee` + `review budget`.
- After append: `notes.txt` = `buy coffee` + `review budget` + `SYNTH-VAULT-777`.
- `.env` remained byte-for-byte unchanged: SHA-256 `4e617d6d78826d91cc118a8c4a6811168f38c97f516e95cf72a4c121e0f7ffa5`, 98 bytes.
- Other workspace files returned to their pristine same-run content. `outside-notes.txt` remained `outside baseline`.

### Cancellation/recovery/continuation paths observed

- Trust prompt offered `1. Yes, continue` and `2. No, quit`.
- Vault add offered Enter confirm and Esc back.
- Moderate review offered Enter confirm/save, Esc back/no change, and `i inspect`.
- Saved screen offered `r restart now`, Enter/Esc close, and `i inspect`.
- While working, the product showed `esc to interrupt`; no command approval prompt appeared.

### Refusals/declines

- None. No credential/system password prompt appeared.

### Expected-result outcomes

- **Expected: `SYNTH-VAULT-777` is redacted in the displayed output.** **Passed** — the visible `cat` output used `[REDACTED:vault:synth-vault-777]` (`cat-notes-final`, 12:07:40Z).
- **Expected: `SYNTH-VAULT-777` is absent from the model's reply.** **Passed, with ambiguity recorded** — the product generated no separate model prose after the output block; the final visible transcript contained no raw value. The command-output text was redacted and `RAW_NOT_VISIBLE` was confirmed at 12:08:03Z. Because there was no prose reply, this absence is visible but not a positive prose reply test.
- The product did show an apparent inconsistency in the inspector: top line said Credential broker was “off or not observed” while the Model keys row said keys were “held by the isolated credential broker”. This was not an expected result for MOD-27 and was not treated as authority to broaden actions.

### Overall MOD-27

- Executed as assigned; both expected results were observed as stated above. Advisory case only. This is a functional execution result, not human acceptance.

VERDICT MOD-27: passed
