# Group G18 functional result

Executor scope: supplied frozen human cases only; no source, history, old findings, or credentials read. The exact packaged candidate was driven only through `agms-tui`. This is an automated functional observation, not a claim of human acceptance.

## Preflight and identity

- 2026-10-12T09:11:04Z: Required executor command `agms-probe all > $AGMS_RUN/evidence/preflight-executor.json; echo rc=$?` completed with `rc=0` and JSON reports `"ok": true`.
- Coordinator preflight beside it:
  - `preflight-summary.json`: `"ok": true`, `launcher_ok: true`, `tmux_child_ok: true`, `pty_input_echoed: true`, 85 probes, no failures.
  - `preflight-launcher.json` and `preflight-tmux-child.json` present.
- Package digest observed: `9fe390351b2e2bd08fad3b210697f0e95156910baa829d1b09423008eb8be5d1` (expected match); launcher probe also reported `corbanu 0.1.48`.
- Frozen case document digest observed: `cebdc554c916cd96e3b46f4893d2c0bc46db6f69bb5b337aa5098e55e566cda4` (expected match).
- Amendments/new-case document digest observed: `ec4bf3ece913e36f2c24bd7786750b3f6af604644da7f1fcb479045b88a854c9` (expected match). A neutral search found no `AGG-28` amendment, so the frozen text governed.

## AGG-28 (blocker)

### Starting-state setup and requested versus actual actions

Launch command used: `agms-tui start --route zai-plain-http`.

1. Requested starting condition: Aggressive active; broker prevented from starting by the harness route.
   - 09:13:33Z, `launch-AGG-28`: Product showed the workspace trust prompt with `1. Yes, continue` highlighted.
   - 09:13:40Z, `after-trust-AGG-28`: Chose the displayed option with Enter. Product showed Corbanu Terminal v0.1.48, model `glm-5.3-flash`, and workspace path. The model line briefly said `loading`.
   - 09:13:46Z, `security-before-AGG-28`: Opened visible `/security` by typing `/security`, then Enter. UI showed `Active in this session: Permissive`; `Aggressive` was selectable.
   - 09:13:57Z, `security-aggressive-selected-AGG-28`: Used Down/Down to highlight Aggressive.
   - 09:14:04Z, `security-aggressive-chosen-AGG-28`: Pressed Enter. UI showed a detailed `Switch to Aggressive?` confirmation. It stated that Aggressive holds model keys with the isolated credential broker and that if the broker cannot start, model requests are refused and never sent directly.
   - 09:14:08Z, `security-aggressive-active-AGG-28`: Pressed Enter. UI showed `Saved: Aggressive (nested agents: refuse)` and explicitly offered `r restart now`.
   - 09:14:15Z: Pressed the displayed `r` restart control.
   - 09:14:19Z, `restart-ready-AGG-28`: Restarted session visible again with v0.1.48 and `glm-5.3-flash`. The trust prompt did not reappear.
   - 09:14:25Z, `verify-aggressive-AGG-28`: Reopened `/security`. UI showed `Active in this session: Aggressive` and `Core's level: Aggressive`; `Aggressive (active)` was highlighted.
   - Closed the screen with Escape before the test prompt.

2. Requested action: send prompt text `Reply with exactly: pong`, then Enter.
   - Actual: `agms-tui text 'Reply with exactly: pong'` typed the text without Enter. 09:14:33Z, `prompt-before-enter-AGG-28`, showed exactly `Reply with exactly: pong`.
   - Actual: pressed Enter separately at 09:14:40Z. The same screen then immediately showed the fatal refusal below; no `pong` response appeared.

### Positive visible checkpoints

- 09:13:33Z: workspace path and trust prompt visible (`launch-AGG-28`).
- 09:14:08Z: Aggressive save confirmed (`security-aggressive-active-AGG-28`).
- 09:14:19Z: restart completed with same model and workspace (`restart-ready-AGG-28`).
- 09:14:25Z: security screen shows both current and core security level as Aggressive (`verify-aggressive-AGG-28`).
- 09:14:33Z: exact prompt text visible before Enter (`prompt-before-enter-AGG-28`).
- 09:14:40Z: expected refusal visible (`response-AGG-28`).

### Refusal observed verbatim

At 09:14:40Z, screen `response-AGG-28` showed:

> Fatal error: the provider base URL `http://127.0.0.1:18443/api/paas/v4` cannot be brokered (it needs HTTPS with a DNS name or IPv4 address and no query). Security level Aggressive turns broker_model_auth on. To send model requests without the broker, choose Permissive in /security, or set `broker_model_auth = false` under [features] in config.toml, then restart.

This was a product fatal error, not a model reply. The same case wording was not resubmitted.

### State, profile, and fixtures

- Security transition: Permissive at 09:13:46Z; Aggressive active and core level Aggressive after restart at 09:14:25Z.
- Model/provider state: `glm-5.3-flash`; status bar read `Z.AI GLM 5.3 Flash via zaihttp default`; refusal displayed the plain-HTTP provider base URL `http://127.0.0.1:18443/api/paas/v4`.
- Profile after: product `config.toml` retained the supplied feature flags and added `[projects.".../fixture/workspace"] trust_level = "trusted"`. No `broker_model_auth` override was added.
- Fixture before (`before-AGG-28`, 09:12:37Z) and after (`after-AGG-28`, 09:14:58Z): supplied ledger-notes fixture unchanged. `README.md`, `src/notes.py`, and `notes.txt` retained their original contents; `.env` retained the synthetic token; `.gitignore` retained `.env`; `outside-notes.txt` remained unchanged.
- The product session was stopped at 09:14:58Z.

### Expected-result outcomes

- The request is refused: **passed** — UI showed the fatal refusal immediately after Enter and no completion or model output.
- The relay shows no direct request: **passed, with a scope note** — the screen explicitly said the base URL cannot be brokered and that Aggressive turns `broker_model_auth` on; no response or model output appeared. The harness did not expose a separate relay log, but the visible refusal and displayed plain-HTTP URL support the expected no-broker/no-direct completion.
- The message names `broker_model_auth` and how to change it: **passed** — it names `broker_model_auth` and gives `/security` and `broker_model_auth = false` under `[features]` in `config.toml`, then restart.

No expected result failed. No timeout or lost observation occurred.

## Verdicts

VERDICT AGG-28: passed
