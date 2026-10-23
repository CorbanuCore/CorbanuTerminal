# Functional executor report — group G17b

Executor role only. This is not human acceptance.

## Identity and isolation preflight

- Frozen cases SHA-256 observed: `cebdc554c916cd96e3b46f4893d2c0bc46db6f69bb5b337aa5098e55e566cda4` (matches required).
- Amendments/new-cases SHA-256 observed: `ec4bf3ece913e36f2c24bd7786750b3f6af604644da7f1fcb479045b88a854c9` (matches required).
- Candidate binary SHA-256 observed for `/opt/agms/pkg/bin/corbanu`: `9fe390351b2e2bd08fad3b210697f0e95156910baa829d1b09423008eb8be5d1` (matches required).
- Coordinator preflight files existed and were `ok: true`: `preflight-summary.json`, `preflight-launcher.json`, and `preflight-tmux-child.json`.
- I ran `agms-probe all > $AGMS_RUN/evidence/preflight-executor.json; echo rc=$?` before case work. The command returned rc 0 and the JSON has top-level `"ok": true` at 2026-10-12T12:15:07Z. It also independently observed the required packet and candidate digests.
- Product identity seen in UI on startup and after setup restart: `Corbanu Terminal (v0.1.48)`.

## AGG-27

### Case interpretation used

Frozen case text:

- Starting conditions: Aggressive active.
- Actions: (1) On `zai`, ask `Reply with exactly: pong`; (2) `/model`, switch to `zai-anthropic` GLM 5.3 Flash, ask the same; (3) restart and repeat on each route.
- Expected: every turn completes with `pong`; no broker or refusal error appears; model and provider choices are unchanged by the level.
- Group note: the two relevant routes are `zai` and `zai-anthropic`; `zai-plain-http` is not used.

### Setup and starting conditions

- Requested: establish Aggressive active before the case, then perform the case actions.
- Actual:
  - 2026-10-12T12:16:32Z: started a fresh Corbanu Terminal session in the fixture workspace with `agms-tui start`.
  - Screen `setup-01-start`: trust prompt for the fixture workspace, showing `› 1. Yes, continue` and `2. No, quit`.
  - Pressed Enter on the displayed default to accept the workspace trust.
  - Screen `setup-02-after-trust`: main prompt with footer `Z.AI GLM 5.3 Flash via zai default`.
  - 2026-10-12T12:16:44Z: typed `/security`, then pressed Enter.
  - Screen `setup-03-security`: `Active in this session: Permissive`; displayed choices Permissive, Moderate (not available yet), and Aggressive.
  - Selected Aggressive and pressed Enter. Screen `setup-05-aggressive-confirm` opened `Switch to Aggressive?` and showed a confirmation/review with `enter confirm and save`.
  - 2026-10-12T12:16:56Z: pressed Enter to confirm. Screen `setup-06-after-confirm` showed `Saved: Aggressive (nested agents: refuse)`, `Active now in this session: agent commands cannot run ...`, restart-only changes, and `Press r to restart now`.
  - Pressed `r`. Screen `setup-07-restart-start` showed the restarted main prompt with footer `Z.AI GLM 5.3 Flash via zai default`.
  - 2026-10-12T12:17:14Z: opened `/security` after restart. Screen `setup-08-security-after-restart` showed `Active in this session: Aggressive`, `Core's level: Aggressive`, and `Aggressive (active)`.
- Security level at transitions:
  - Before /security selection: Permissive.
  - In Aggressive confirmation screen: transition target Aggressive.
  - After confirm: Aggressive saved/active now.
  - After `r` restart and /security inspection: Aggressive.
- Positive checkpoint: the restarted session's footer retained `Z.AI GLM 5.3 Flash via zai default`.

### Fixture state

- Before case (`agms-tui fixture pre`) and after blocked case (`agms-tui fixture post`): workspace files are unchanged in visible text. In particular, `notes.txt` remained `buy coffee\nreview budget\n`, `.env` remained synthetic, and no new project files appeared. The full recorded fixture snapshots are retained by the TUI fixture labels `pre` and `post`.
- No requested case action involved modifying fixture files, and no modification was observed.

### Requested versus actual actions

1. **Requested:** On `zai`, ask `Reply with exactly: pong`.  
   **Actual:** 2026-10-12T12:17:22Z: from the main prompt after Aggressive restart, typed the exact prompt text without Enter, then pressed Enter separately. Screen `turn-01-sent` showed the request and the model reply `pong`. Footer remained `Z.AI GLM 5.3 Flash via zai default`.  
   **Checkpoints:** Request text visible; exact visible reply `pong`; provider and model visible.

2. **Requested:** Open `/model`, switch to `zai-anthropic` GLM 5.3 Flash, then ask the same prompt.  
   **Actual:** 2026-10-12T12:17:29Z: typed `/model`, then pressed Enter. Screen `model-01-menu` showed `Select Model and Effort` with `1. Z.AI GLM 5.3 Flash (current)`, GLM 5.3, and GLM 5.2. It did **not** display a route/provider selector for `zai-anthropic`. Pressing Enter selected the displayed current model and then opened a reasoning-level screen (`model-02-after-model-choice`); selecting that screen's displayed option returned to the main prompt with `model: glm-5.3-flash low` and the error:  
   `■ Cannot save `glm-5.3-flash` because provider `zai` is inactive or unavailable.`  
   A subsequent `/model` attempt displayed `• No additional models are available right now.` rather than a menu with a route/provider control (`model-04-reopen` through `model-07-blocker-missing-provider-control`). The required control or choice to switch to `zai-anthropic` was not present. This is an exact, visible missing required control, so the case was stopped at this point.
   - Side observation: the reasoning label changed to `low` through the displayed model/reasoning menus. The model identity remained GLM 5.3 Flash and provider remained `zai`; the requested provider switch never occurred.

3. **Requested:** Restart and repeat on each route.  
   **Actual:** not exercised, because the required `/model` route switch to `zai-anthropic` could not be performed.

### Product refusals or declines

- No model-generated refusal or decline was observed. The first model turn returned `pong`.
- The UI error seen during the `/model` attempt was: `■ Cannot save `glm-5.3-flash` because provider `zai` is inactive or unavailable.` This is a provider-save/UI error, not identified as a broker or model refusal.

### Expected-result outcomes

- **Every turn completes with `pong` — blocked.** The first `zai` turn completed with the visible reply `pong`; the `zai-anthropic` turn and restart repeats were not exercised because the route switch was unavailable.
- **No broker or refusal error appears — not exercised for the full case.** The first turn showed neither. The later `/model` attempt showed the provider-save error above; remaining turns were not exercised.
- **Model and provider choices are unchanged by the level — partially passed for the first checkpoint; otherwise not exercised.** After the Aggressive setup restart, the UI continued to show `Z.AI GLM 5.3 Flash via zai default`. No turn on `zai-anthropic` was reached.

### Stopping reason and exact observation

- Stopped AGG-27 as **blocked** on a missing required control.
- Exact observation on `model-07-blocker-missing-provider-control` at `2026-10-12T12:19:07Z`:
  - Header: `model: glm-5.3-flash low`.
  - Prior exchange: `› Reply with exactly: pong` and `• pong`.
  - Error: `■ Cannot save `glm-5.3-flash` because provider `zai` is inactive or unavailable.`
  - Notice: `• No additional models are available right now.`
  - Footer: `Z.AI GLM 5.3 Flash via zai low`.
  - No route choice for `zai-anthropic` was displayed.

### Ambiguity / partial observations preserved

- The first route worked before the `/model` attempt, despite the later provider-save message saying `zai` was inactive or unavailable.
- The visible `/model` menu offered model choices but no explicit route/provider choice. I did not invent an undocumented command or key to bypass this.
- The reasoning effort changed from an unspecified initial state to `low` through the displayed `/model` flow. I did not continue manipulating the UI after observing the missing provider control.

### Case result

`VERDICT AGG-27: blocked`
