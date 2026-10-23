# AGG-T07 and AGG-T08 — independent functional execution

Executor run: G32. Candidate and packet digests were verified before execution; `agms-probe all` reported `"ok": true` at 2026-10-14T00:17:21Z. Product version observed: `corbanu 0.1.48`.

Fixture state before cases (own shell observation): `$AGMS_RUN/fixture/workspace` is the fresh `ledger-notes` project; `../outside-notes.txt` contains `outside-the-project: synthetic file`. Product profile is the supplied fresh product home with security feature flags enabled and starting Permissive. No case in T07/T08 requests fixture mutation; no fixture change was needed.

## Setup: S0 → S1

- Requested: fresh profile/project; trust project; save Aggressive and restart with `r`; `/security` shows `Active in this session: Aggressive`.
- Actual: started with `agms-tui start --route zai --model glm-5.3-flash` at 2026-10-14T00:18:51Z. Trust screen offered options 1/2; pressed Enter on option 1.
- Checkpoint SETUP-S0-MAIN (2026-10-14T00:19:57Z after a 60-second screen wait): v0.1.48 main UI, footer `Z.AI GLM 5.3 Flash via zai default`, directory in the workspace.
- 00:20:01Z: opened `/security`; SETUP-SECURITY showed `Active in this session: Permissive`, with Permissive active.
- Navigated Down, Down to Aggressive (SETUP-SECURITY-AGG); Enter showed the switch confirmation.
- SETUP-SECURITY-CONFIRM showed Aggressive settings, no credential/system password prompt, and `enter confirm and save`. Pressed Enter.
- SETUP-SECURITY-SAVED showed `Saved: Aggressive (nested agents: refuse)` and `Press r to restart now`.
- Pressed `r` at 00:20:12Z. Restarted to main UI; SETUP-RESTART showed v0.1.48, footer `Z.AI GLM 5.3 Flash via zai default`, same workspace.
- Opened `/security`; SETUP-S1-SECURITY showed `Active in this session: Aggressive` and `Core's level: Aggressive`, with Aggressive active. S1 established.
- Fixture before cases: unchanged fresh test project; no T07/T08 setup mutation requested. Security level at S1 transition: Permissive before save; Aggressive active after restart.
## AGG-T07 (blocker)

Starting condition: S1 on GLM 5.3 Flash · Z.AI (OpenAI-style) (`zai`), established above.

### Requested vs actual

1. Requested: Type `/model`, Enter. Actual: sent `/security` Escape first to close setup inspection, then typed `/model` and pressed Enter at 00:21:31Z.
2. Requested: Down to Claude-style, Enter; Enter on default effort if asked. Actual: highlighted entry 2 and pressed Enter; the product showed an effort menu with `More reasoning…` highlighted, then an `Advanced Reasoning` menu with `Max` highlighted; pressed Enter on each displayed default/selected control.
3. Requested: send `Reply with exactly: pong`; record time. Actual: typed prompt at 00:21:53Z and pressed Enter at 00:21:54Z.
4. Requested: Type `/model`, note current, Esc. Actual: performed; no selection.
5. Requested: Type `/security`, note active, Esc. Actual: performed; no selection.
6. Requested: Type `/model`, choose OpenAI-style, Enter; send same prompt; record time. Actual: opened `/model`, moved Up from Claude-style to OpenAI-style, Enter. Product again offered effort; selected its displayed default/Max path. Typed prompt at 00:22:53Z and pressed Enter at 00:22:53Z.
- Fixture check after T07 (`agms-tui fixture T07-FIXTURE`; `git status --short` empty): no case mutation observed.

### Positive visible checkpoints

- T07-STEP1-MODEL at 00:21:31Z: GLM 5.3 Flash listed once for OpenAI-style (entry 1) and once for Claude-style (entry 2). Entry 1 was OpenAI-style and marked `(current)`. Security transition/level remained Aggressive.
- T07-STEP2-FOOTER at 00:21:51Z: `Model changed to GLM 5.3 Flash · Z.AI (Claude-style) max`; footer named `zai-anthropic`. No restart, onboarding, or API-key prompt appeared.
- T07-CLAUDE-PONG at 00:22:03Z: Claude-style turn completed with visible `• pong`. No broker or refusal error visible.
- T07-STEP4-MODEL-CURRENT: Claude-style entry was marked `(current)`.
- T07-STEP5-SECURITY: `Active in this session: Aggressive`; `Core's level: Aggressive`.
- T07-STEP6-OPENAI-FOOTER: `Model changed to GLM 5.3 Flash · Z.AI (OpenAI-style) max`; footer named `zai`.
- T07-OPENAI-SEND at 00:22:53Z onward: OpenAI-style turn completed with visible `• pong`. No broker or refusal error visible.

### Expected-result outcomes

1. Step 1 listing/order/current marking: **passed**.
2. Footer names `zai-anthropic`; no restart/onboarding/API-key prompt: **passed**.
3. Both pongs complete; no broker/refusal error: **passed**.
4. Claude-style marked current; Aggressive remains active: **passed**.
5. Relay paths at recorded request times: **not exercised by the executor** (relay is explicitly invisible). Recorded request turn times for independent evidence: Claude-style prompt text/Enter 00:21:53Z/00:21:54Z; OpenAI-style prompt text/Enter 00:22:53Z/00:22:53Z. No visible failure was observed.

No product-model refusal occurred in AGG-T07.

## AGG-T08 (advisory)

Starting condition: S1. To establish `pane A switched in /model to Claude-style`, from T07's ending OpenAI state at 00:23:49Z: opened `/model`, highlighted Claude-style entry 2, Enter, accepted the displayed default/Max effort path, and observed footer `zai-anthropic` (`T08-SETUP-CLAUDE-FOOTER`). No security-level transition was made; the level remained Aggressive.

### Requested vs actual

1. Requested: Type `/model`; note order and marking; Esc. Actual: opened `/model` at 00:23:49–00:23:59Z; screen `T08-STEP1-MODEL`; Esc.
2. Requested: Type `/panes`, choose `+ Corbanu Terminal Pane`; at model choice note the first/highlighted GLM 5.3 Flash connection; accept it without changing connection; name pane B. Actual: opened `/panes` at 00:23:59Z, selected `+ Corbanu Terminal Pane`; the model choice highlighted OpenAI-style entry 1 and marked it `(current)`; pressed Enter to accept it without changing connection; accepted the displayed default/Max effort path; cleared the suggested display name with Backspace, typed `B`, pressed Enter. Product created and switched to pane B.
3. Requested: In pane B send `Reply with exactly: pong`; record wall-clock time. Actual: typed prompt at 00:24:36Z and pressed Enter at 00:24:36Z.
- Fixture check after T08 (`agms-tui fixture T08-FIXTURE`; `git status --short` empty): no case mutation observed.

### Positive visible checkpoints

- T08-STEP1-MODEL: all three model levels were grouped OpenAI-style before Claude-style (`1/2`, `3/4`, `5/6`). Claude-style GLM 5.3 Flash was marked `(current)` for pane A.
- T08-STEP2-PANES: `/panes` listed `+ Corbanu Terminal Pane`; no missing required control.
- T08-STEP2-MODEL-CHOICE: for pane creation, GLM 5.3 Flash OpenAI-style was both first and highlighted, and was marked `(current)`.
- T08-STEP2-PANE-B-CREATED: `Created and switched to Corbanu Terminal pane B`, `Session renamed to B`, footer `Z.AI GLM 5.3 Flash via zai max`, trailing pane label `B`.
- T08-PANE-B-SEND at 00:24:36Z onward: pane B turn completed with visible `• pong`; no broker or refusal error visible.
- Security level observed as Aggressive in S1 and unchanged throughout T08; no new security transition requested.

### Expected-result outcomes

1. `/model` lists every model with OpenAI-style first: **passed**.
2. New pane offers/highlights OpenAI-style (`zai`) as the default for GLM 5.3 Flash: **passed**. It did not preselect inherited Claude-style, so no advisory finding applies.
3. Pane B pong completes: **passed**.
4. Relay path at recorded time: **not exercised by the executor** (relay is explicitly invisible). Recorded pane B prompt text/Enter time: 00:24:36Z; independent evidence should show `/chat/completions` (or path matching the displayed `zai` connection). No visible failure was observed.

No product-model refusal occurred in AGG-T08.

## Final outcomes

- AGG-T07: all directly observable requested checkpoints passed; relay-path evidence left for the independent post-run check with the timestamps above.
- AGG-T08: all directly observable requested checkpoints passed; relay-path evidence left for the independent post-run check with the timestamp above.
- No credential/system password prompt, unexpected profile access, missing required control, lost observation, or budget exhaustion occurred.
- This is the independent functional executor's outcome, not a claim of human acceptance.

VERDICT AGG-T07: passed
VERDICT AGG-T08: passed

