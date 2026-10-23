# G02 functional execution report

Scope: assigned cases AGG-29, AGG-03 only, in the requested order. This is automated functional observation, not human acceptance.

## Preflight and identity

- Requested: run `agms-probe all > $AGMS_RUN/evidence/preflight-executor.json; echo rc=$?`; verify `"ok": true`.
- Actual: command returned `rc=0`; executor preflight JSON reports `"ok": true`. Coordinator files `preflight-launcher.json`, `preflight-tmux-child.json`, and `preflight-summary.json` were present beside it; all reported ok and summary had `failed: []`.
- Candidate identity: `/opt/agms/pkg/bin/corbanu` SHA-256 `181ad0299d238accbf936512531c90988e8d15cb02a50ea8241f8566b2372a4a`; package version probe reported `corbanu 0.1.48`.
- Frozen case file SHA-256: `cebdc554c916cd96e3b46f4893d2c0bc46db6f69bb5b337aa5098e55e566cda4`.
- Initial fixture baseline (`agms-tui fixture BASELINE`): notes.txt contained `buy coffee` and `review budget`; outside-notes.txt contained `outside-the-project: synthetic file`; .env contained only the synthetic token line documented by the fixture.
- Product home initial `config.toml` had Aggressive-related feature flags but no saved security-level selection.
- 2026-10-10T23:22Z: preflight passed and cases proceeded.

---

## AGG-29 — resume across security levels

Status: complete

### Setup performed to establish stated starting conditions

- Requested starting conditions: one Permissive conversation recorded, then Aggressive active after restart.
- Requested setup: launch fresh, trust workspace, record one harmless Permissive conversation, save Aggressive, press `r` to restart now.
- Actual setup:
  - 23:24:07Z launched with default route/model. Trust screen (`PRESTART`) showed the workspace path and `1. Yes, continue`.
  - 23:24:17Z pressed Enter. Fresh screen appeared at 23:24:48Z (`FRESH_PERMISSIVE`); the earlier `Use /skills` wait timed out, but the same fresh-start layout was visibly present.
  - 23:25:01Z typed the prompt and then pressed Enter separately. By 23:25:57Z (`RECORDED_REPLY_LATER`) the model had replied exactly `recorded`; security was Permissive because no transition had been made yet.
  - 23:26:06Z opened `/security` (`PICKER_PERMISSIVE_AG29_SETUP`): `Active in this session: Permissive`, with Permissive marked active. Pressed Down twice (`PICKER_HIGHLIGHT_AGGRESSIVE_AG29_SETUP`): Aggressive selected, still Permissive active. Enter opened the review (`AGGRESSIVE_REVIEW_AG29_SETUP`); it stated `Preflight passed`.
  - 23:26:15Z pressed Enter. `SAVED_AGGRESSIVE_AG29_SETUP` showed `Saved: Aggressive (nested agents: refuse)` and restart offer. Security level until restart was Permissive; Core's level became Aggressive now. Pressed `r`; by 23:26:24Z `AFTER_RESTART_AG29_SETUP` showed the product restarted in the same fixture workspace.
  - 23:26:31Z reopened `/security` (`PICKER_AGGRESSIVE_ACTIVE_AG29_SETUP`): `Active in this session: Aggressive`, `Core's level: Aggressive`, and `> Aggressive  (active)`.
- Checkpoints: the one Permissive-era session was visible in `/resume` as `Please reply with exactly: recorded` (`RESUME_LIST_AG29`, 23:26:32Z). This confirms the setup produced one recorded conversation.
- Fixture state: unchanged through setup; `AFTER_AG29_SETUP` still showed notes.txt with the two original notes and outside-notes.txt unchanged.

### Case action 1

- Requested: `/resume`, choose the Permissive-era session, Enter.
- Actual: 23:26:32Z `/resume` showed exactly one session (`RESUME_LIST_AG29`). 23:26:42Z pressed Enter on the highlighted session. By 23:26:50Z, `RESUME_ATTEMPT_AG29` showed verbatim: `■ Security level Aggressive: this conversation was recorded before protected mode was activated and may contain secrets. Start a new session instead.`
- Security level shown after the refusal: 23:26:57Z `PICKER_AFTER_RESUME_REFUSAL_AG29` showed `Active in this session: Aggressive`, `Core's level: Aggressive`, and `> Aggressive  (active)`.
- Checkpoints: refusal explanation visible; current-session Aggressive visible after refusal.

### AGG-29 expected results

- [x] Passed — The session is refused with an explanation. The refusal text above explicitly explains the pre-Aggressive recording and possible secrets.
- [x] Passed — The current session stays Aggressive. `/security` after the refusal showed both active and Core as Aggressive.

### AGG-29 observations and ambiguities

- The fresh-start wait for the exact design-copy phrase `Use /skills` timed out because the live placeholder used another suggested prompt. This is a non-app-specific observation issue, not an expected-result failure.
- No fixture files changed during AGG-29 setup or the case.
- 23:26:57Z session remained open after recording the security state.

---

## AGG-03 — save Aggressive and inspect after restart/relaunch

Status: complete

### Starting condition setup / state

- Requested: fresh profile; Aggressive review open with preflight passed.
- Actual setup: after AGG-29, the product session was stopped at 23:28:05Z. The used product-home directory was renamed aside inside `$AGMS_RUN` (not read), a new empty product-home was created, and only the supplied initial feature-flag `config.toml` was restored. This produced a fresh profile state. The fixture workspace files remained unchanged.
- 23:28:15Z launched. Trust screen (`AG03_TRUST`) appeared. Pressed Enter at 23:28:17Z. By 23:28:19Z `AG03_FRESH_START` showed the main screen and same fixture workspace.
- 23:28:19–20Z opened `/security` (`AG03_PICKER_PERMISSIVE`): `Active in this session: Permissive` with Permissive active. Pressed Down twice (`AG03_PICKER_HIGHLIGHT_AGGRESSIVE`): Aggressive selected, Permissive still active. Enter opened the review (`AG03_AGGRESSIVE_REVIEW_START`), which showed `Preflight passed: no known raw-secret route stays open to agents or the model.`

### Actions and checkpoints

- Requested action 1: Enter. Actual 23:28:29Z confirmed/saved. `AG03_SAVED_AGGRESSIVE` showed verbatim `Saved: Aggressive (nested agents: refuse)`, followed by `Restart Corbanu Terminal to activate Aggressive. Until then this session stays Permissive.` and `Press r to restart now…`. Passed.
- Requested action 2: Press r. Actual pressed at 23:28:30Z. By 23:28:38Z `AG03_AFTER_RESTART` showed the restarted product in the same fixture workspace. Passed.
- Requested action 3: Open /security. Actual 23:28:49Z. `AG03_PICKER_AFTER_RESTART` showed `Active in this session: Aggressive`, `Core's level: Aggressive`, and `> Aggressive  (active)`. Passed. Then Escape was used only to close the picker before the next slash command.
- Requested action 4: /quit, relaunch in the same folder, open /security. Actual 23:28:50Z typed `/quit` (`AG03_QUIT_TEXT`) and pressed Enter; after 23:28:55Z `agms-tui screen` returned `tmux error: no server running…`, confirming the session ended. 23:29:04Z relaunched with default options; by 23:29:06Z `AG03_RELAUNCH_START` showed the same fixture workspace. No trust prompt appeared on this relaunch. At 23:29:08–09Z opened `/security`; `AG03_PICKER_AFTER_RELAUNCH` showed `Active in this session: Aggressive`, `Core's level: Aggressive`, and `> Aggressive  (active)`. Passed.
- Requested action 5: Press i. Actual 23:29:22Z from the picker. `AG03_INSPECTOR_TOP` showed:
  - `Saved: Aggressive [configured: /security]`
  - `Core config: Permissive [configured: config layers]`
  - `[set] Active this run: Aggressive, every row verified at launch [resolved: launch check]`
  - `[on] Live policy: Aggressive in force; next start Aggressive; kill switch off; generation 0.1 [observed: Core policy]`
  It also showed a warning-style top line: `◐ Aggressive set; off or not observed: Credential broker`. Scrolled view `AG03_INSPECTOR_SCROLLED` showed `[set] Protected boundary: preflight clean at launch; earlier conversations are not resumed`. Passed.

### AGG-03 expected results

- [x] Passed — 'Saved: Aggressive' is shown and a restart is offered (`AG03_SAVED_AGGRESSIVE`).
- [x] Passed — The app restarts in the same folder after `r` (`AG03_AFTER_RESTART`), and the later relaunch also showed the same folder (`AG03_RELAUNCH_START`).
- [x] Passed — Picker shows Aggressive (active) after both the `r` restart and the `/quit` relaunch (`AG03_PICKER_AFTER_RESTART`, `AG03_PICKER_AFTER_RELAUNCH`).
- [x] Passed — Inspector shows Saved Aggressive and live policy in force, next start Aggressive (`AG03_INSPECTOR_TOP`). The inspector also honestly reports Core config from config layers as Permissive while live policy is Aggressive.

### AGG-03 observations and ambiguities

- The relaunch did not display the folder-trust prompt; the first screen captured was already the main screen. This did not affect any named expected result and is recorded as an observed difference.
- The inspector's top line says `off or not observed: Credential broker`, while its session rows say `Model keys: held by the isolated credential broker (broker_model_auth on)` and the scrolled runtime says `Credential broker: on in config; its health is not observed here`. This is an ambiguity in wording, not a contradiction evaluated by AGG-03.
- Fixture state after AGG-03 (`AFTER_AGG03`) was unchanged from baseline: notes.txt still had the two original notes and outside-notes.txt remained unchanged.

---

## Fixture changes

- Across both assigned cases, no fixture file content changed. The synthetic token and .env contents were not displayed in the live product screens.
- For AGG-03 setup only, the prior product-home was set aside by renaming it inside `$AGMS_RUN`; a fresh product-home and the supplied initial feature-flag config were created. This was necessary to establish the case's fresh-profile starting condition.

## Final verdicts

VERDICT AGG-29: passed
VERDICT AGG-03: passed
