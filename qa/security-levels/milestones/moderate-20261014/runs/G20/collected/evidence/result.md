# G20 execution result — MOD-24

Independent functional execution only. The observations below are from the live candidate screens and the harness’s recorded actions; they are not human acceptance.

## Preflight and candidate identity

- Before any case work, the executor ran `agms-probe all > $AGMS_RUN/evidence/preflight-executor.json; echo rc=$?`. The command exited rc `0`; the file reports `ok: true`.
- Coordinator files were present and reported:
  - `preflight-summary.json`: `ok: true`, `launcher_ok: true`, `tmux_child_ok: true`, `pty_input_echoed: true`, `failed: []`.
  - `preflight-launcher.json`: `ok: true`.
  - `preflight-tmux-child.json`: `ok: true`.
- `/opt/agms/pkg/bin/corbanu` SHA-256: `abd3e6adbf44f234e12fb6a9654d15a7f6b7b6c20ecd61af2ef8e1944ffbb24a`; matched the expected value.
- `/opt/agms/packet/design-cases.md` SHA-256: `64ecf9c951f89d645e0b2eb3243dd422c80436f24fadb4d9283a5daa97239c23`; matched.
- `/opt/agms/packet/amendments.md` SHA-256: `203685710b74d3cf864ed00e35a5c6e30e10296ea82ec60ab707b09b53da02d1`; matched.
- Preflight package launch reported `corbanu 0.1.48`.

## Case MOD-24

### Starting state and fixture

- Supplied product-home was fresh except `config.toml`. Its visible `[features]` table had the Moderate-related feature flags enabled and no security-level override. The fixture workspace was the supplied `ledger-notes` project.
- No fixture files were modified by the executor or observed to change during this case.
- Fixture recorded after the plain-http error as label `before-mod24`: `workspace/.env` remained `<credential file: 98 bytes, sha256 4e617d6d78826d91cc118a8c4a6811168f38c97f516e95cf72a4c121e0f7ffa5>`; `outside-notes.txt`, `README.md`, `notes.txt`, `.gitignore`, `src/notes.py`, and git metadata were unchanged from the supplied fixture.
- After the final zai turn, label `after-mod24` showed the same fixture content and `.env` size/SHA-256.

### Requested versus actual actions

1. **Requested:** fresh profile and fresh test project; establish Moderate (restarted).  
   **Actual:** used the supplied fresh profile/project. At 05:27:02Z started on `zai`. At 05:27:07Z pressed Enter on the displayed trust option `1. Yes, continue` (`start-trust`). Typed `/security` and pressed Enter separately at 05:27:21Z (`security-permissive`, active Permissive). Pressed Down at 05:27:30Z (`picker-moderate-highlight`). Pressed Enter at 05:27:38Z (`moderate-review`), then Enter again at 05:27:44Z (`moderate-saved`), then `r` at 05:27:58Z (`after-moderate-restart`). The visible route was `zai`.
2. **Requested:** `/quit`, then relaunch on `zai-plain-http`.  
   **Actual:** typed `/quit` and pressed Enter separately at 05:28:07Z; the tmux session closed. At 05:28:26Z started with `--route zai-plain-http`; the status line showed `via zaihttp`.
3. **Requested:** send “Reply with exactly: pong”.  
   **Actual:** typed the exact text and pressed Enter separately at 05:28:34Z. A product fatal error appeared and was captured at 05:28:35Z (`plain-http-result`). The prompt did not complete with `pong`.
4. **Requested:** `/security`, `i`.  
   **Actual:** typed `/security` and pressed Enter separately at 05:28:50Z (`security-picker-after-error`); pressed `i` at 05:28:52Z (`inspector-moderate-top`). Scrolled with Down at 05:29:00–01Z (`inspector-broker-rows`).
5. **Requested:** `/quit`, then relaunch on `zai`.  
   **Actual:** after the inspector, an initial `/quit` attempt at 05:29:12Z occurred while the security picker was still open and did not quit. Escaping through the displayed screens eventually reached the main prompt at 05:30:28Z (`main-before-quit2`). Typed `/quit` and pressed Enter separately at 05:30:34Z; the tmux session closed. At 05:30:46Z started with the default `zai`.
6. **Requested/needed to show the normal turn:** sent the same literal wording, “Reply with exactly: pong”, with text and Enter separately at 05:30:52Z. The turn completed with `pong` by 05:31:13Z (`zai-relaunch-final`).

### Timestamps for the attempted model request

- Before sending the plain-http prompt: **2026-10-15T05:28:34Z**.
- Product error captured on screen: **2026-10-15T05:28:35Z**; the same error remained at 05:28:44Z.
- The executor cannot access the relay or provider log; no direct outbound-connection observation was possible. Independent relay verification must use the interval beginning 2026-10-15T05:28:34Z through the observed error time.

### Positive visible checkpoints

- `security-permissive`, 05:27:22Z: Security picker showed **Active in this session: Permissive**.
- `picker-moderate-highlight`, 05:27:32Z: Moderate was highlighted; Permissive remained shown as active.
- `moderate-review`, 05:27:41Z: “Switch to Moderate?”; footer offered “enter confirm and save · esc back, nothing changes · i inspect”.
- `moderate-saved`, 05:27:46Z: “Saved: Moderate”; text said broker-related model keys start when restarting, and offered “r restart now”.
- `after-moderate-restart`, 05:28:02Z: fresh product screen on `zai`; restarted state was reached.
- `plain-http-result`, 05:28:35Z: exact visible error:
  > Fatal error: the provider base URL `http://127.0.0.1:18443/api/paas/v4` cannot be brokered (it needs HTTPS with a DNS name or IPv4 address and no query). Security level Moderate turns broker_model_auth on. To send model requests without the broker, choose Permissive in /security, or set `broker_model_auth = false` under [features] in config.toml, then restart.
- `security-picker-after-error`, 05:28:52Z: **Active in this session: Moderate** and **Core's level: Moderate**; Moderate was shown as `(active)`. The app remained running.
- `inspector-moderate-top`, 05:28:54Z: top line read “◐ Moderate set; off or not observed: Credential broker”; Saved level was Moderate and active run was Moderate. The inspector also said “Model keys: held by the isolated credential broker (broker_model_auth on)”. `inspector-broker-rows`, 05:29:02Z, retained the broker-off/degraded top line and app remained running.
- `zai-relaunch-final`, 05:31:13Z: on `zai`, the turn displayed `pong` with no security or broker error.

### Security level at transitions

- Before choice: Permissive (`security-permissive`).
- During review: current/active level remained Permissive; review described Moderate after confirmation.
- Immediately after save: Moderate was active in the session per `moderate-saved`; broker startup deferred until restart.
- After Moderate restart: fresh screen did not print the level; the next `/security` screen and inspector both showed Moderate active. The inspector additionally showed “Core config: Permissive [configured: config layers]”.
- Plain-http relaunch: no immediate level line in the fresh screen, but `/security`/inspector after the error showed Moderate active.
- Final zai relaunch: normal turn completed; no `/security` check was requested or performed after this turn.

### Model refusals or declines

- None. The plain-http outcome was a product fatal broker error, not a model refusal/decline.

### Expected-result outcomes

1. **Product error names the unbrokerable URL, names Moderate turning `broker_model_auth` on, and gives both ways to change it:** **passed.** The error named `http://127.0.0.1:18443/api/paas/v4`; it named Moderate and both `/security` → Permissive and setting `broker_model_auth = false`, then restart.
2. **No request reaches the relay or provider between the recorded times:** **inconclusive from the executor.** No direct relay/provider observation is available to the executor. The relevant interval was recorded; relay evidence must be read by the independent evidence check.
3. **The app keeps running:** **passed.** After the fatal error, `/security`, the inspector, scrolling, Esc navigation, and later `/quit` remained operable.
4. **Inspector shows the broker as degraded or off:** **passed.** Inspector top line showed “off or not observed: Credential broker” with the ◐ symbol.
5. **Relaunch on `zai` gives a normal turn:** **passed.** After relaunch, “Reply with exactly: pong” produced `pong` without a security or broker error.

### Case verdict

VERDICT MOD-24: passed

Caveat: the relay/provider non-delivery expected result is inconclusive from the executor and depends on the independent relay check; all directly visible expected results passed.
