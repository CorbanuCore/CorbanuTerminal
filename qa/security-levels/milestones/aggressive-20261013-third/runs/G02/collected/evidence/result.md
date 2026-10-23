# G02 independent functional run — result

## Preflight and identity

- Coordinator preflight evidence reviewed:
  - `$AGMS_RUN/evidence/preflight-summary.json`: `{"launcher_ok": true, "tmux_child_ok": true, "pty_input_echoed": true, "failed": [], "probe_count": 85, "ok": true}`.
  - `$AGMS_RUN/evidence/preflight-launcher.json`: top-level `ok: true`, no failed probes.
  - `$AGMS_RUN/evidence/preflight-tmux-child.json`: top-level `ok: true`, no failed probes.
- Executor preflight command run before other work:
  - `agms-probe all > $AGMS_RUN/evidence/preflight-executor.json; echo rc=$?`
  - observed `rc=0` at 2026-10-13T17:31:48Z; JSON top-level `ok: true`; no failed probes.
- Candidate identity:
  - `/opt/agms/pkg/bin/corbanu` SHA-256 `ad288be98d80d5eb164c2fb34a040565dcc7590a12c58ab2ac3aaf25f0675043`.
  - `/opt/agms/pkg/bin/corbanu --version` printed `corbanu 0.1.48`.
- Frozen case file SHA-256 matched `cebdc554c916cd96e3b46f4893d2c0bc46db6f69bb5b337aa5098e55e566cda4`.
- Amendment file SHA-256 matched `c184f1bda104aa521441f3a34b2599c8bdb57335395abe188300915c0c6101b9`.
- No amendment or new case reference applied to AGG-29 or AGG-03 in the supplied amendment file.

## AGG-29

### Setup

- Baseline fixture (`agms-tui fixture baseline-pre-AGG-29`): notes were `buy coffee` and `review budget`; outside file was unchanged; `.env` contained only the supplied synthetic test token.
- Fresh Permissive session:
  - Started product via `agms-tui start`.
  - Approved the displayed trust prompt with Enter (visible option 1 selected).
  - Asked: `Run exactly this command and show its output: python3 src/notes.py list`.
  - Product ran it without an approval prompt in the initial Permissive state and displayed `buy coffee` and `review budget`; screen `AGG29-permissive-complete`.
- Changed to Aggressive:
  - `/security`, highlighted Aggressive, pressed Enter.
  - Confirmation screen `AGG29-aggressive-confirm` displayed the full switch description and `Preflight passed: no known raw-secret route stays open to agents or the model.`
  - Pressed Enter. Screen `AGG29-aggressive-saved` showed `Saved: Aggressive (nested agents: refuse)`, stated that Aggressive starts when restart, and offered `r restart now`.
  - Pressed `r`. Product restarted in same fixture workspace; screen `AGG29-after-restart`.
  - `/security` after the attempted resume showed `Active in this session: Aggressive`, `Core's level: Aggressive`, and `> Aggressive (active)`; screen `AGG29-security-verify`.

### Requested actions versus actual actions

1. `/resume, choose the Permissive-era session, Enter`
   - Actual: typed `/resume`, pressed Enter; the picker showed one session, `57s ago — Run exactly this command and show its output: python3 src/notes.py list`; pressed Enter (screen `AGG29-resume-list`).

### Observations

- During setup only, one 30-second `wait` pattern timed out (`Ready|Working|You can|Ask`), but the immediately captured screen showed the product main prompt; a later wait matched the completed Permissive command output. This was not treated as a case-result timeout.
- After Enter, the UI displayed:

  `■ Security level Aggressive: this conversation was recorded before protected mode was activated and may contain secrets. Start a new session instead.`

  Screen `AGG29-resume-outcome`.
- `/security` immediately afterwards showed the current session remained Aggressive (screen `AGG29-security-verify`).
- Fixture after case: unchanged from baseline (`agms-tui fixture baseline-post-AGG-29`).
- Product stopped after the case. No model decline occurred; the refusal was a visible security-level message.

### Expected-result outcomes

1. **passed** — “The session is refused with an explanation”: the exact security-level explanation above appeared.
2. **passed** — “The current session stays Aggressive”: `/security` showed Aggressive active after the refusal.

Checkpoint times: `AGG29-resume-list` 2026-10-13T17:35:09Z; `AGG29-resume-outcome` 17:35:16Z; `AGG29-security-verify` 17:35:24Z.

## AGG-03

### Fresh-state setup

- Stopped the AGG-29 product before this case.
- Moved the prior product profile directory aside and created a fresh `$AGMS_RUN/product-home` with the supplied feature configuration (no prior security state).
- Reset the fixture project to its tracked baseline; `git clean -fdx` removed the ignored `.env`, then it was restored byte-for-byte to the supplied synthetic fixture content. The outside file remained `outside-the-project: synthetic file`.
- `agms-tui fixture AGG03-baseline-restored` confirmed `.env` contained only the supplied synthetic comment/token and notes remained `buy coffee` and `review budget`.
- Started a fresh product profile in `$AGMS_RUN/fixture/workspace` at 17:37:32Z; screen `AGG03-start` showed the correct abbreviated workspace directory.
- Pressed Enter at the visible trust prompt (option 1), screen `AGG03-main`.
- Opened `/security`, selected Aggressive, pressed Enter. Review screen `AGG03-review` displayed `Preflight passed: no known raw-secret route stays open to agents or the model.` and `Controls ready: 6 of 6`.

### Requested actions versus actual actions

1. `Enter`
   - Actual: pressed Enter at 17:37:55Z on the open Aggressive review.
2. `Press r`
   - Actual: pressed `r` at 17:38:00Z.
3. `Open /security`
   - Actual: typed `/security` and pressed Enter at 17:38:06–07Z.
4. `/quit, relaunch in the same folder, open /security`
   - Actual: pressed Escape to return to the main prompt, typed `/quit`, and pressed Enter at 17:38:15Z; the tmux session ended. The requested `AGG03-quit` screen returned `tmux error: no server running...`, confirming the product had exited. Called `agms-tui stop` at 17:38:25Z, then relaunched with default `zai`/`glm-5.3-flash` at 17:38:32Z. The main screen (`AGG03-relaunch-trust`) again showed the same abbreviated fixture directory. Typed `/security` and pressed Enter at 17:38:38Z.
5. `Press i`
   - Actual: pressed `i` at 17:38:45Z; screen `AGG03-inspector` at 17:38:46Z.

### Positive checkpoints

- `AGG03-saved` (17:37:57Z) showed `Saved: Aggressive (nested agents: refuse)` and explicitly offered `r restart now`.
- `AGG03-after-restart` (17:38:00Z) showed a fresh Corbanu Terminal screen in `~/runs/…/fixture/workspace`.
- `AGG03-security-after-restart` (17:38:08Z) showed `Active in this session: Aggressive`, `Core's level: Aggressive`, and `> Aggressive (active)`.
- `AGG03-security-after-relaunch` (17:38:39Z) again showed `Active in this session: Aggressive`, `Core's level: Aggressive`, and `> Aggressive (active)`.
- `AGG03-inspector` (17:38:46Z) showed:
  - `Saved: Aggressive [configured: /security]`
  - `[on] Live policy: Aggressive in force; next start Aggressive; kill switch off; generation 0.1 [observed: Core policy]`
- Fixture after case (`agms-tui fixture AGG03-post`, 17:39:05Z) remained unchanged; notes, outside file and synthetic `.env` were all original.
- No product-model refusal or decline occurred.
- The product was stopped at 17:39:05Z after observation.

### Ambiguity / incidental observation

- The inspector also displayed `Core config: Permissive [configured: config layers]` and `Aggressive set; off or not observed: Credential broker`, while another inspector row said `Model keys: held by the isolated credential broker`. This was not in the case's expected list and I did not debug it; all explicitly expected inspector strings were present.

### Expected-result outcomes

1. **passed** — “‘Saved: Aggressive’ is shown and a restart is offered”: both appeared on `AGG03-saved`.
2. **passed** — “The app restarts in the same folder”: after `r`, the fresh main screen showed the same abbreviated fixture directory; the explicit relaunch also showed the same directory.
3. **passed** — “Picker shows Aggressive (active) after the restart and after the relaunch”: both `/security` screens showed `> Aggressive (active)`.
4. **passed** — “Inspector shows Saved Aggressive, live policy in force, next start Aggressive”: the inspector text contained all three requested states.

## Verdicts

VERDICT AGG-29: passed
VERDICT AGG-03: passed
