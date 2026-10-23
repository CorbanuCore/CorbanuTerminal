# G25 functional execution report — MOD-34

Executor: independent functional executor. This report records observed outcomes only; it does not claim human acceptance.

## Preflight and identity

- Candidate digest: `abd3e6adbf44f234e12fb6a9654d15a7f6b7b6c20ecd61af2ef8e1944ffbb24a` — matched.
- Frozen cases digest: `64ecf9c951f89d645e0b2eb3243dd422c80436f24fadb4d9283a5daa97239c23` — matched.
- Amendments digest: `203685710b74d3cf864ed00e35a5c6e30e10296ea82ec60ab707b09b53da02d1` — matched.
- `agms-probe all` exited rc=0 and reported `"ok": true` at 2026-10-15T06:23:15Z; it included 85 probes.
- Coordinator `preflight-summary.json` reported `launcher_ok:true`, `tmux_child_ok:true`, `pty_input_echoed:true`, `failed:[]`, `probe_count:85`, `ok:true`; both accompanying `preflight-*.json` files were present.
- Outcome: passed.

## Case MOD-34 — broker_model_auth=false still wins under Moderate

### Starting state

- Fresh product profile and fresh `ledger-notes` project copy were used. Product home: `$AGMS_RUN/product-home`.
- Before first start, from the tester shell at 2026-10-15T06:25:41Z, added `broker_model_auth = false` under the existing `[features]` table. The resulting `[features]` table had `security_levels`, `protected_mode_preflight`, `secretless_agent_launch`, `isolated_credential_broker`, `secret_output_gate`, `source_envelopes`, `url_destination_policy`, and `contained_external_agents` all true, plus `broker_model_auth = false`.
- Fixture before product start: `workspace/.env` was shown by size/hash only as `98 bytes`, SHA-256 `4e617d6d78826d91cc118a8c4a6811168f38c97f516e95cf72a4c121e0f7ffa5`; its contents were not read. Other workspace files were `README.md`, `src/notes.py`, `notes.txt`, `.gitignore`, and `.git`; `../outside-notes.txt` was adjacent.

### Requested versus actual actions

1. Start and choose Moderate
   - Requested: start product; `/security`; Down to Moderate; Enter to read review; Enter; r.
   - 2026-10-15T06:25:56Z — launched with `agms-tui start` on route `zai`, model `glm-5.3-flash`. The first-start trust prompt appeared (`start-MOD34`) with `1. Yes, continue` / `2. No, quit`.
   - 2026-10-15T06:26:02Z — because the displayed native control required it, pressed Enter on the highlighted trust choice. Main start screen `after-trust-MOD34` showed v0.1.48, `glm-5.3-flash`, and the workspace. This extra trust confirmation is recorded.
   - 2026-10-15T06:26:09Z — typed `/security`, then pressed Enter separately. Picker `security-picker-MOD34` showed `Active in this session: Permissive` and `Permissive  (active)`.
   - 2026-10-15T06:26:13Z — pressed Down; `moderate-highlight-MOD34` showed `> Moderate`.
   - Same second — pressed Enter. Review `moderate-review-MOD34` showed `Switch to Moderate?`. The Model keys paragraph said `broker_model_auth = false in your own config keeps it off`; the final summary still said broker keys would start on restart.
   - 2026-10-15T06:26:20Z — pressed Enter. `moderate-saved-MOD34` showed `Saved: Moderate`, said Moderate controls were active in this session, and said broker keys start on restart unless `broker_model_auth` was off. It offered `r restart now`.
   - 2026-10-15T06:26:24Z — pressed r. Restart completed by `after-restart-2-MOD34` at 2026-10-15T06:26:34Z with this warning:
     `⚠ Security level Moderate: broker_model_auth is off (set in your configuration or by policy), so Corbanu reads provider keys itself instead of the credential broker.`
   - Security at transitions: Permissive in the first picker; Moderate on the saved screen; Moderate again after restart.

2. Inspect security after restart
   - Requested: `/security`, i; read Model keys and Level rows; scroll.
   - 2026-10-15T06:26:41Z — typed `/security`, then Enter separately. `security-picker-after-restart-MOD34` showed `Active in this session: Moderate`, `Core's level: Moderate`, and `Moderate  (active)`. It did not claim model keys were broker-held.
   - 2026-10-15T06:26:46Z — pressed i. Inspector `inspector-top-MOD34` header showed `◐ Moderate set; off or not observed: Model key broker, Credential broker`. Level rows showed `Saved: Moderate`, `Core config: Permissive`, `Active this run: Moderate`, and `Live policy: Moderate in force`. The Model keys row read exactly `Model keys: Corbanu reads provider keys itself (broker_model_auth off)`.
   - 2026-10-15T06:26:53Z — pressed Down four times through the inspector (`inspector-down1-MOD34` through `inspector-down4-MOD34`). No row claimed that model keys were held by the credential broker.
   - Ambiguity: the UI used the explicit partial-state marker `◐ Moderate set; off or not observed: ...`, but neither screen used the literal word `partial`.

3. Send test turn
   - Requested: Esc, Esc; send `Reply with exactly: pong`.
   - 2026-10-15T06:27:01Z — pressed Escape twice; `before-prompt-MOD34` returned to the main prompt with the same warning.
   - 2026-10-15T06:27:02Z — typed the exact prompt text, then pressed Enter separately.
   - The product showed `Working (0s • esc to interrupt)`.
   - 2026-10-15T06:27:10Z — `after-prompt-wait-MOD34` showed `• pong` and TPS `0.7 tok/s`; no broker error appeared.
   - At completion, the main screen still showed the same broker-off warning. No approval transition or model refusal occurred.

### Final state

- 2026-10-15T06:28:22Z — final screen `final-MOD34` still showed the broker-off warning and completed `pong`.
- Fixture after: unchanged; `.env` remained `98 bytes` with the same SHA-256, and the other recorded workspace/outside fixture contents matched before.
- Product config after: still had `broker_model_auth = false`. The product added a project trust entry, consistent with accepting the visible first-start trust prompt.
- No credential or system password prompt appeared; no required control was missing.

### Expected-result outcomes

- Moderate is saved and, after restart, active or shown as partial — **passed**. `Saved: Moderate` was observed; after restart the picker showed active Moderate and the inspector showed the partial-state marker.
- After restart, neither `/security` nor the inspector claims model keys are credential-broker-held, and the inspector model-key row says broker off — **passed**.
- Turn completes with `pong` and no broker error — **passed**.
- Startup warning and `/security` partial indication — **passed, with wording ambiguity noted**. The warning was exact. The inspector showed `◐ Moderate set; off or not observed: Model key broker, Credential broker`, but not the literal word `partial`; this marker was counted as its partial indication.

VERDICT MOD-34: passed
