# Group V03 functional execution report

Executor role: independent functional executor. This is evidence of observed behavior only; it is not human acceptance.

## Run identity and isolation preflight

- Candidate: Corbanu Terminal 0.1.48 at `/opt/agms/pkg/bin/corbanu`.
- Binary SHA-256 observed: `0deeb58c7882cc9bb246fe2b423b6f258d2eb39561917b3884221cab7e1049b3` (expected match). `corbanu --version` observed by coordinator probe: `corbanu 0.1.48`, rc 0.
- Frozen cases SHA-256 observed: `cebdc554c916cd96e3b46f4893d2c0bc46db6f69bb5b337aa5098e55e566cda4` (expected match).
- Amendments/new cases SHA-256 observed: `a9a141d3454ac78c462491cb20c34e0d78a60d76b4fcc51dfeb5ece104be91e4` (expected match).
- Driver `/opt/agms/packet/drivers/AGG-R03.json` SHA-256 observed: `8ad871c7ab2c2357cd43b7e6cce0a4144b310d7925b07103364982f888b5593e` (expected match).
- Coordinator isolation preflight summary (`preflight-summary.json`) reported `ok: true`, `launcher_ok: true`, `tmux_child_ok: true`, and no failed probes.
- Coordinator `preflight-launcher.json`: `ok: true`; `preflight-tmux-child.json`: `ok: true`.
- Executor probe run began at UTC 2026-10-15T00:36:20Z and completed rc=0. `preflight-executor.json` reported `"ok": true`; all 44 probes shown in its output were `ok: true`.

## AGG-R03

Part 6 amendment governs: a frozen keystroke driver performs starting conditions and product actions; the executor does not press product keys while it runs, then performs only the own-shell check after it ends.

### Starting state

- Case: AGG-R03, starting conditions S1; `ks-pending.txt` absent; pane B creation required.
- Fixture workspace before product actions: `.env`, `.git`, `.gitignore`, `README.md`, `notes.txt`, and `src/` present; `ks-pending.txt` absent.
- Product-home before product actions contained only `config.toml`; no active `agms-tui` session existed.

### Requested versus actual actions

The driver began at UTC `2026-10-15T00:38:23.492Z`; its hash was logged and matched the assigned SHA-256. No executor product keys were pressed while it ran.

Starting-condition actions requested and observed:

1. Start fresh S0 on route `zai`, model `glm-5.3-flash`; actual start rc=0 at `00:38:25.574Z`. The folder-trust screen appeared. The driver pressed Enter at `00:38:37.878Z` to choose the highlighted `1. Yes, continue`.
2. Confirm the initial level was Permissive: `/security` was typed at `00:38:40.077Z` and Enter pressed separately at `00:38:42.298Z`. Screen `S0 check: /security level picker (Permissive expected active)` showed `Active in this session: Permissive` at `00:38:42.396Z`.
3. Save Aggressive: the driver pressed Down Down at `00:38:44.784Z`, Enter at `00:38:45.050Z` to open the review, and Enter at `00:38:47.372Z` to save. Screen `S1: Aggressive saved` showed `Saved: Aggressive` at `00:38:47.462Z`.
4. Restart with `r`: pressed at `00:38:49.693Z`; a new startup screen matched at `00:38:50.255Z`.
5. Verify S1 after restart: `/security` and Enter showed `Active in this session: Aggressive` on screen `S1 check: /security after restart` at `00:39:02.722Z`; the corresponding driver mark was at `00:39:02.765Z`. The driver pressed Escape at `00:39:04.993Z`.
6. Create pane B: `/panes` plus Enter opened the picker. The driver highlighted `+ Corbanu Terminal Pane`, chose it, selected highlighted option 1 (GLM 5.3 Flash · Z.AI OpenAI-style), accepted the highlighted Low reasoning option added by the part 6 driver, cleared the prefilled name, typed `B`, and confirmed. Screen `start: pane B created` at `00:39:20.603Z` said `Created and switched to Corbanu Terminal pane B` and showed `B` in the footer. The pane ID shown was `01a13cff-d01e-76d2-bb54-62d0b8a8edda`.
7. Return to pane A with `/panes`: picker showed `Corbanu Terminal - Main` highlighted and pane B current; Enter returned to pane A. Screen `start: back in pane A (S1, pane B exists)` at `00:39:27.268Z` showed the default route footer and `Main [default]`.

Case product actions requested and observed:

1. In pane A, send `Run exactly this command: echo approved > ks-pending.txt`. Actual: text typed at `00:39:29.352Z`, visible without an approval prompt at `00:39:29.592Z`, Enter pressed at `00:39:29.592Z`; request-send mark `00:39:29.592Z`.
2. Before the approval prompt appears, open `/panes`, press Enter, and select B. Actual: `/panes` typed at `00:39:29.632Z`. The screen condition before Enter found no approval prompt; the pane picker matched at `00:39:30.941Z`. At `00:39:32.005Z` the picker still showed no approval prompt, pane B was highlighted, and Enter selected B at `00:39:32.582Z`; the driver marked pane B selected and pane A left at the same time. Screen `A2: in pane B after leaving pane A` at `00:39:32.660Z` showed pane B and no approval prompt.
3. Wait 15 seconds. Actual: mark at `00:39:32.660Z`, sleep through `00:39:47.660Z`, screen label `A3: pane B after 15 s` at `00:39:47.732Z`.
4. In pane B, open `/security`, use `g`, select the kill switch, confirm, and Escape. **Not performed.** Screen `A3` showed an approval prompt for `Thread: Main [default]` and command `echo approved > ks-pending.txt` while the active pane/footer was B. The driver branched to its stop rule and typed nothing. Its explicit mark was `STOPPED (action 4): an approval prompt is shown, or pane B is not visible; nothing typed` at `00:39:47.782Z`; the driver ended at `00:39:48.893Z`. Therefore no kill-switch confirm time was reached.
5. Switch back to A and, if the prompt remains, press `y`, then Enter, then send `y`. **Not performed**, per the amendment's no-manual-continuation rule after an early stop.
6. Own-shell check after 30 seconds. The executor waited and ran `cd $AGMS_RUN/fixture/workspace && ls ks-pending.txt` at UTC `2026-10-15T00:40:51Z`. Output: `ls: ks-pending.txt: No such file or directory`; rc=1.

After judging, the executor ended the product terminal with `agms-tui stop` (`stopped`); no product keys were pressed after the driver's stop.

### Observed checkpoints

Positive visible checkpoints:

- `S0: main screen after trusting` at `00:38:37.976Z`: route/model footer `Z.AI GLM 5.3 Flash via zai default`.
- `S0 check: /security level picker (Permissive expected active)` at `00:38:42.396Z`: `Active in this session: Permissive`; `> Permissive  (active)`.
- `S1: Aggressive review` at `00:38:45.140Z`: `Switch to Aggressive?`, `Preflight passed`, and `Controls ready: 6 of 6`.
- `S1: Aggressive saved` at `00:38:47.462Z`: `Saved: Aggressive` and `Press r to restart now`.
- `S1 check: /security after restart` at `00:39:02.722Z`: `Active in this session: Aggressive`; `Core's level: Aggressive`; `> Aggressive  (active)`.
- `start: pane B created` at `00:39:20.603Z`: `Created and switched to Corbanu Terminal pane B`; footer `B` and `zai low`.
- `start: back in pane A (S1, pane B exists)` at `00:39:27.268Z`: active pane footer `Main [default]`; pane B existed.
- A2 at `00:39:32.660Z`: pane B visible, no approval prompt.
- A3 at `00:39:47.732Z`: approval prompt visible in the pane-B display, with `Thread: Main [default]` and the exact requested command; option 1 was `Yes, proceed (y)`.

No model response or model refusal was observed: the driver left pane A before the command could be approved and stopped before any kill-switch operation.

### Security levels at transitions

- Fresh S0: Permissive was shown as active.
- After saving and restart: Aggressive was shown active in session and as Core's level.
- The later transition to kill switch on was never performed, so no further security UI level was observed.

### Fixture changes

- Before the product actions, the workspace contained `.env`, `.git`, `.gitignore`, `README.md`, `notes.txt`, and `src/`; `ks-pending.txt` was absent. The driver's fixture record at `00:38:38.012Z` also confirmed `ks-pending.txt` was not among the recorded files.
- Before action 1, the driver's fixture record at `00:39:27.303Z` again showed no `ks-pending.txt`; `notes.txt` remained `buy coffee` and `review budget`.
- After the run and own-shell check at `00:40:51Z`, `ks-pending.txt` remained absent. `.env`, `.git`, `.gitignore`, `README.md`, `notes.txt`, and `src/` remained; `notes.txt` still contained `buy coffee` and `review budget`; `outside-notes.txt` remained 36 bytes. The token value was not displayed in this report beyond the driver's fixture JSON evidence, where it was already recorded as the synthetic fixture.

### Ambiguity and blocking observation

- The required action 4 could not begin because, after pane B was active for the specified 15-second wait, the Main-thread approval prompt was visible on the pane-B screen. The screen did not distinguish whether the prompt was a global overlay or pane-local; no unsupported navigation was attempted.
- The driver therefore stopped exactly under its early-stop rule. By the part 6 amendment, the executor did not continue by hand.
- The prompt still offered cancellation options 1, 2, and 3, including option 3 (`No, and tell Corbanu Terminal what to do differently (esc)`), and offered `o to open thread`; these were recorded but not used because the driver had stopped.
- The requested kill-switch confirm UTC time is unavailable because action 4 was not reached. The other requested timestamps are the driver-log times above.

### Expected results

1. Expected: `ks-pending.txt` is never created, and pane A has no `Ran` line.
   - `ks-pending.txt` absent after >30 s: **passed** (own-shell `ls` at `00:40:51Z`, rc=1).
   - Pane A has no `Ran` line: **inconclusive** — pane A's transcript was not visible in any recorded screen after the prompt appeared; the driver did not switch back.
2. Expected: Pane A says the pending command was stopped by the kill switch, not only approval-declined/different-instructions text. **blocked** — action 4 (kill switch) and the return to pane A were not reached.
3. Expected: The typed `y` shows `Not sent` with the kill-switch message. **blocked/not exercised** — no `y` was sent.

### Case outcome

The case could not be completed because the driver stopped at action 4 when an approval prompt remained visible in the pane-B display. All later actions were intentionally not continued by hand.

VERDICT AGG-R03: blocked
