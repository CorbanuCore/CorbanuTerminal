# Q10 functional execution report — MOD-24

This is an independent functional-execution record only. It is not human acceptance.

## Preflight and candidate identity

- Execution start (UTC): 2026-10-16T12:09:36Z. All timestamps below are UTC.
- Expected package SHA-256: `cb42614c8cc194bf8fc5d598e61e49883384a6747051d92884d5c41f905b0d25`.
- Actual `/opt/agms/pkg/bin/corbanu` SHA-256: `cb42614c8cc194bf8fc5d598e61e49883384a6747051d92884d5c41f905b0d25` — match.
- Expected/actual frozen-case SHA-256: `64ecf9c951f89d645e0b2eb3243dd422c80436f24fadb4d9283a5daa97239c23` — match.
- Expected/actual amendments-v2 SHA-256: `203685710b74d3cf864ed00e35a5c6e30e10296ea82ec60ab707b09b53da02d1` — match.
- Expected/actual amendments-v3 SHA-256: `8ebf87cbddcfa2f67d7399aee4ff06ee7304b6835059f7fde2a31ed8db7bd4d3` — match.
- Coordinator preflight summary (`preflight-summary.json`) reported `ok: true`; `preflight-launcher.json` and `preflight-tmux-child.json` were also `ok: true`.
- Executor preflight: `agms-probe all > $AGMS_RUN/evidence/preflight-executor.json; echo rc=$?` returned rc=0. Its JSON reported `"ok": true`. It is saved beside this report.
- The coordinator preflight positive probe launched the exact package and reported `corbanu 0.1.48`.
- No source, old findings, credentials, relay log, or product history was read. No permission was bypassed. No live account outside test scope was used.

## Starting fixture and setup

- Product-home initial `config.toml` was the supplied Permissive configuration with security feature flags on, including `broker_model_auth` not set to false.
- Initial fixture listing (`agms-tui fixture initial-fixture`, 2026-10-16T12:09:36Z):
  - `workspace/.env`: credential-like, recorded only as 98 bytes, SHA-256 `4e617d6d78826d91cc118a8c4a6811168f38c97f516e95cf72a4c121e0f7ffa5`.
  - `workspace/README.md`: the supplied ledger-notes README.
  - `workspace/notes.txt`: `buy coffee\nreview budget\n`.
  - `outside-notes.txt`: `outside-the-project: synthetic file\n`.
- Workspace git status was clean before setup.

## MOD-24

**Case:** Model requests are refused, not sent directly, when the broker cannot be used.

### Requested versus actual actions

| Requested action | Actual action and UTC |
|---|---|
| Establish Moderate (restarted) | Started supplied product with `agms-tui start` (`zai`, GLM 5.3 Flash). Product showed the folder trust prompt; pressed Enter on the displayed “Yes, continue” option at 12:10:00. Initial main screen label `24-01-permissive-initial`; UI showed Permissive. |
| Change to Moderate and restart | Typed `/security`, then pressed Enter at 12:10:05. Picker showed “Active in this session: Permissive” (`24-02-picker-permissive`). Pressed Down, then Enter; Moderate was highlighted (`24-03-picker-moderate-highlight`) and the Moderate review appeared (`24-04-moderate-review`). Pressed Enter at 12:10:12; “Saved: Moderate” appeared (`24-05-moderate-saved`). Pressed `r`; restarted screen appeared by 12:10:21 (`24-06-after-moderate-restart`). |
| `/quit`, relaunch on `zai-plain-http` | Typed `/quit` and pressed Enter at 12:10:29, then `agms-tui stop` at 12:10:32. Started with `agms-tui start --route zai-plain-http`; product appeared at 12:10:34 and header/status showed `zaihttp` (`24-07-plain-http-start`). |
| Send “Reply with exactly: pong” | Typed prompt and Enter as separate actions. Prompt text was typed at 12:10:40; the “just before sending” time is 2026-10-16T12:10:40Z. Enter was pressed at 12:10:40. The product immediately showed the fatal broker error, visible in `24-08-model-request-result`; confirmed again in `24-08-model-request-result-confirmed` at 12:10:51. |
| `/security`, `i` | Typed `/security`, Enter at 12:10:51. Picker showed “Active in this session: Moderate” and “Core's level: Moderate” (`24-09-picker-after-error`). Pressed `i` at 12:10:51; inspector appeared (`24-10-inspector-top`). Scrolled with Down through the inspector by 12:11:10 (`24-11-inspector-middle`, `24-12-inspector-lower`, `24-13-inspector-bottom`). |
| `/quit`, relaunch on `zai` | Pressed Escape twice to return to the main screen (`24-14-back-to-main-before-quit`); typed `/quit` and Enter at 12:11:18; stopped at 12:11:21. Started with `agms-tui start --route zai` at 12:11:21; product appeared at 12:11:23 with `zai` (`24-15-relaunch-zai`). |
| Normal turn after relaunch | Typed `Reply with exactly: pong` and Enter as separate actions; Enter at 12:11:28. Product showed “Working (0s • esc to interrupt)” (`24-16-zai-normal-turn`); later showed exactly `• pong` (`24-16a-zai-working-or-result`, `24-16b-zai-normal-turn`) by 12:11:51. |
| End | Stopped the product with `agms-tui stop` at 12:12:32. |

### Positive visible checkpoints

- Initial session: `Active in this session: Permissive`.
- Moderate review: exact section “Model keys” said provider keys are held by the isolated credential broker from the next start and that requests are refused, never sent directly if it cannot start.
- Saved screen: “Saved: Moderate”; broker starts on restart; `Press r to restart now`.
- After `r`: new product session appeared.
- Plain HTTP relaunch: UI showed `Z.AI GLM 5.3 Flash via zaihttp default`.
- The product stayed running after the fatal model-request error; the main prompt and suggestions remained visible.
- `/security` after the error: `Active in this session: Moderate`, `Core's level: Moderate`, and `Moderate  (active)`.
- Inspector top: `▲ Moderate degraded: Model key broker`; Model keys row said “held by the isolated credential broker (broker_model_auth on)”.
- Inspector runtime row: `[degraded] Model key broker: the last model request was refused: its provider URL cannot be brokered (it needs HTTPS with a DNS name or IPv4 address); nothing was sent [observed: this process]`.
- Final relaunch on `zai`: UI showed `Z.AI GLM 5.3 Flash via zai default`; model turn completed with `• pong`.

### Product error/refusal

No model-generated decline was observed; the product refused before sending. The visible product error was:

> ■ Fatal error: the provider base URL `http://127.0.0.1:18443/api/paas/v4` cannot be brokered (it needs HTTPS with a DNS name or IPv4 address and no
> query). Security level Moderate turns broker_model_auth on. To send model requests without the broker, choose Permissive in /security, or set
> `broker_model_auth = false` under [features] in config.toml, then restart.

### Recorded relay window for independent verification

- Start of window (just before sending the plain-HTTP model request): **2026-10-16T12:10:40Z**.
- Product error was visible in the same second: **2026-10-16T12:10:40Z**.
- The independent evidence check must read the relay for this window. The executor could not and did not read the relay.

### State/profile and fixture after

- Final fixture listing (`agms-tui fixture after-fixture`, 2026-10-16T12:11:28Z) showed the same 98-byte `.env` and SHA-256 `4e617d6d78826d91cc118a8c4a6811168f38c97f516e95cf72a4c121e0f7ffa5` as before.
- `outside-notes.txt` remained `outside-the-project: synthetic file\n`; SHA-256 `d1dbe90b0861e5e51ddd1c718964f8b0b7050864ad86b90cbbe7513845aaf541`.
- Workspace git status remained clean.
- Product-home final `config.toml` SHA-256: `a870df92c7b596fdeb85741f7fb4d80b6065138d6a862a3421b8341a16b385c0`; size 514 bytes. Its visible contents did not add or change security settings during the case.
- Security-state files were not opened. Metadata only:
  - `security_level.toml`: 31 bytes, SHA-256 `1f61c9237d705e171894dbd3a276782040352c377361201e49255dac9a2f6408`.
  - `security_state.json`: 307 bytes, SHA-256 `5491b1b21128cd2831aed1f1266f4685748c233f4830dd4be5f56552267172ca`.
- Raw TUI evidence is in `$AGMS_RUN/evidence/actions.jsonl`, `screens.jsonl`, and `pty.raw`.

### Expected-result outcomes

1. **Error names the URL, says Moderate turns broker_model_auth on, and gives both ways to change it — passed.** The visible fatal error named `http://127.0.0.1:18443/api/paas/v4`, said Moderate turns `broker_model_auth` on, and offered both `/security` → Permissive and `broker_model_auth = false` in config followed by restart.
2. **No request reaches relay/provider between the two recorded times — not exercised by executor; no contrary observation.** Relay verification is reserved to the independent evidence check. Positive product-side evidence: the broker row says the last request was refused and “nothing was sent”, and the visible fatal error says it cannot be brokered. If the independent relay check contradicts this, this expected result must be re-judged.
3. **App keeps running — passed.** The prompt, previous turn, status bar, and later `/security`/`i` navigation remained available after the fatal error.
4. **Inspector shows broker degraded or off — passed.** Inspector showed `▲ Moderate degraded: Model key broker` and `[degraded] Model key broker`.
5. **Relaunch on `zai` gives a normal turn — passed.** A new `zai` session completed the requested turn with exactly `• pong`.

### Execution observations and ambiguity

- After the trust prompt’s Enter, a narrow wait pattern timed out, but the immediately captured screen was the normal main prompt; the case was not affected.
- The inspector’s “Recent denials” section said “None: nothing refused in this process”, while the Runtime controls section showed the last model request refused in the Model key broker row. This inconsistency is recorded without interpretation.
- No missing required control, credential or system-password prompt, unexpected profile access, lost observation, or budget exhaustion occurred.

**VERDICT MOD-24: passed**

Note: the relay expected result depends on the post-run independent evidence check; the passed verdict covers the directly executed visible checks, with that result recorded as not exercised by this executor.
