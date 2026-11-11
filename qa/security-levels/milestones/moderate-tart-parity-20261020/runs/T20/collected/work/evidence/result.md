# Independent functional execution — group T20

Executor: independent functional executor only. No source/history/credentials read; no code changed; no human acceptance claimed.

## Coordinator isolation and candidate identity preflight

Requested/actual:
- Verified frozen packet and amendment digests (requested values matched).
- Verified `/opt/agms/pkg/bin/corbanu` SHA-256 matched expected `15cb32e9407cc346286d39f24514bb391027db10d2aceb12c1c2798e482c4ae6`.
- Read coordinator summary `preflight-summary.json` and both `preflight-*.json` files beside it.
- Ran the required probe: `agms-probe all > $AGMS_RUN/evidence/preflight-executor.json; echo rc=$?`.

Observations:
- At 2026-10-20T17:00:24Z coordinator preflight summary: `"ok": true`; no failures; probe_count 99.
- At 2026-10-20T17:02:00Z executor probe result: `rc=0`; file content `"ok": true`.
- Package `--version` evidence in coordinator preflight: `corbanu 0.1.48`.

Preflight outcome: all assigned cases may proceed.

## MOD-24 · Model requests are refused, not sent directly, when the broker cannot be used

### State and fixture

- Supplied fresh profile/workspace were used. Product started in `$AGMS_RUN/fixture/workspace`.
- `agms-tui fixture pre` at 17:03:54Z showed `.env` as `<credential file: 98 bytes, sha256 4e617d6d78826d91cc118a8c4a6811168f38c97f516e95cf72a4c121e0f7ffa5>`; `agms-tui fixture post` at 17:07:36Z showed the same size/hash. No case fixture content was changed by the requested actions.
- No credential/system password prompt appeared. No approval prompt appeared. The product's model did not decline anything; the observed product-level broker refusal is not a model decline.

### Requested versus actual actions

| Requested | Actual |
| --- | --- |
| Start fresh product | Started at 17:03:54Z. At 17:04:00Z, S00-start showed the directory-trust prompt with `1. Yes, continue`. |
| Select Moderate and restart | Pressed Enter on the trust prompt (17:04:00–17:04:05Z). S01-permissive showed v0.1.48/GLM 5.3 Flash via zai; the security picker was not yet open, so no security level was displayed. `/security` + Enter at 17:04:13Z showed Active session Permissive (S02-security-picker). Down, Enter at 17:04:20Z displayed the Moderate review (S03-moderate-review). Enter at 17:04:30Z displayed Saved: Moderate and `r restart now` (S04-moderate-saved). `r` restarted at 17:04:37Z (S05-after-restart; header model briefly `loading`). `/security` + Enter at 17:04:48Z showed Active in session Moderate / Core's level Moderate (S06-security-after-restart). Esc returned to the normal input (17:04:57Z). |
| `/quit`, relaunch zai-plain-http | Typed `/quit` and pressed Enter at 17:04:57–17:05:00Z; product exited, so `agms-tui screen` returned `tmux error: no server running`. Started `agms-tui start --route zai-plain-http` at 17:05:08Z; S09-plain-http-start showed v0.1.48, GLM 5.3 Flash via `zaihttp`. |
| Send prompt text and Enter separately | Typed literal `Reply with exactly: pong` without Enter at 17:05:24Z; S10-prompt-before-enter showed it in the input. UTC just before sending: **2026-10-20T17:05:24Z**. Pressed Enter at 17:05:24Z; product error was visible immediately and at 17:05:24Z (S11-broker-error). |
| Inspect security | `/security` + Enter at 17:05:36Z showed Active/Core Moderate (S12-security-picker-after-error). Pressed `i`: S13-inspector-top showed `▲ Moderate degraded: Model key broker`. |
| `/quit`, relaunch zai | First attempt: pressed Escape once, typed `/quit` and Enter around 17:05:46–17:05:48Z, but the inspector remained shown and `agms-tui start` reported `a product session is already running`; no relaunch occurred. Pressed Escape twice; at 17:05:58Z S15-after-esc-attempts showed the normal input. Typed `/quit` + Enter at 17:06:04Z. `agms-tui start` then launched the default zai route at 17:06:09Z (S16-relaunch-zai, v0.1.48, GLM 5.3 Flash via zai). |
| Verify normal zai turn | At 17:06:18Z typed literal `Reply with exactly: pong` without Enter; pressed Enter at 17:06:20Z. An early screen (also labeled S17) showed `• Working (0s …)` at 17:06:22Z. At 17:07:00Z, S17-zai-normal-turn-after-15s showed `• pong` and `TPS: 2.3 tok/s`. `/security` at 17:07:35Z showed Active/Core Moderate (S18-zai-security-after-normal-turn). Esc, then `agms-tui stop` at 17:07:36Z. |

### Positive checkpoints

- **Moderate starting condition met:** security picker after restart displayed `Active in this session: Moderate` and `Core's level: Moderate` (S06-security-after-restart, 17:04:48Z).
- **Plain HTTP route active:** header showed `zaihttp` (S09-plain-http-start, 17:05:12Z).
- **Broker refusal text met:** S11-broker-error at 17:05:24Z displayed, verbatim: `■ Fatal error: the provider base URL \`http://127.0.0.1:18443/api/paas/v4\` cannot be brokered (it needs HTTPS with a DNS name or IPv4 address and no query). Security level Moderate turns broker_model_auth on. To send model requests without the broker, choose Permissive in /security, or set \`broker_model_auth = false\` under [features] in config.toml, then restart.`
- **App stayed running:** after the error, the UI remained usable; `/security`, `i`, Escape and another `/quit` could be operated.
- **Broker degraded:** S13-inspector-top at 17:05:36Z showed `▲ Moderate degraded: Model key broker`; the Level section showed Saved/Core/Live details and `Model keys: held by the isolated credential broker`.
- **Normal turn on default zai:** S17 result at 17:07:00Z displayed exactly `• pong`, with no security or broker error.
- **Level after normal turn:** S18 at 17:07:35Z showed Active/Core Moderate.

### Expected-result outcomes

1. **Product error names URL, Moderate/broker_model_auth, and both ways to change it: passed.** All three clauses are present in the S11 text.
2. **No request reaches relay/provider between recorded times: not exercised by executor.** Relay/provider evidence is explicitly outside executor visibility. Recorded window: request submitted at **2026-10-20T17:05:24Z**; error visible at **2026-10-20T17:05:24Z**. Independent relay-log check remains outstanding.
3. **App keeps running: passed.** Interactive security/inspector/navigation continued after the fatal error message.
4. **Inspector shows broker degraded or off: passed.** S13 showed `▲ Moderate degraded: Model key broker`.
5. **Relaunch on zai gives normal turn: passed.** The default zai route completed `pong` without security or broker error (S17, 17:07:00Z).

No product model refusal/decline was observed. No missing required control blocked execution. The first post-inspector `/quit` was initially ineffective; Escape was used as displayed, and the eventual quit/relaunch succeeded.

### MOD-24 executor verdict

All directly executable expected results passed; the relay/provider result is left for the independent evidence check and is not claimed by the executor.

VERDICT MOD-24: passed
