# Group G22 functional execution report

Executor role: independent functional executor. This report records observed behavior only; it is not human acceptance.

## Preflight and candidate identity

- Executor command: `agms-probe all > $AGMS_RUN/evidence/preflight-executor.json; echo rc=$?`; result `rc=0`, top-level `"ok": true`.
- Coordinator summary (`preflight-summary.json`): `launcher_ok=true`, `tmux_child_ok=true`, `pty_input_echoed=true`, `failed=[]`, `probe_count=85`, `ok=true`.
- Coordinator files checked beside it: `preflight-launcher.json` and `preflight-tmux-child.json`.
- Frozen cases SHA-256: `cebdc554c916cd96e3b46f4893d2c0bc46db6f69bb5b337aa5098e55e566cda4` (match).
- Amendments/new-cases SHA-256: `ec4bf3ece913e36f2c24bd7786750b3f6af604644da7f1fcb479045b88a854c9` (match).
- Candidate SHA-256: `9fe390351b2e2bd08fad3b210697f0e95156910baa829d1b09423008eb8be5d1` (match).
- Candidate version from preflight launch: `corbanu 0.1.48`.

No model refusal or decline was observed. No credential/system password prompt appeared.

## S1 setup and initial fixture

Initial fixture was recorded with `agms-tui fixture initial-standby`: `workspace/notes.txt` was `buy coffee\nreview budget\n`, `workspace/.env` contained only the synthetic label/token, and neither `ks-run.txt` nor `ks-pending.txt` existed.

| Screen / time | Requested action | Actual action and visible state |
|---|---:|---|
| `S0-initial` 09:44:58Z | Start product | Trust prompt for fixture workspace. |
| `S0-trusted` 09:45:04Z | Trust project | Enter on displayed `1. Yes, continue`; main screen loaded. |
| `S1-security-screen` 09:45:14Z | Open /security | Typed `/security`; separate Enter. UI showed `Active in this session: Permissive`. |
| `S1-aggressive-selected` 09:45:18Z | Select Aggressive | Down Down highlighted Aggressive. |
| `S1-aggressive-saved` 09:45:27Z | Confirm and save | Enter showed `Saved: Aggressive (nested agents: refuse)` and restart prompt. |
| `S1-after-restart` 09:45:38Z | Restart with displayed `r` | Product restarted to main screen. |
| `S1-verify-security` 09:45:47Z | Verify S1 | `/security` showed `Active in this session: Aggressive`, `Core's level: Aggressive`, Aggressive active. |
| `S1-grants-killswitch-off` 09:45:53Z | Verify kill switch off | `g` showed grants screen, `Kill switch: off`; Escape closed. |

Security-level transition: UI showed Permissive before save, Aggressive after restart.

## AGG-R01

Requested actions: send the exact story prompt; while the reply was streaming, type `/security`, Enter, press `g`, Down to `Turn the kill switch on`, Enter, Enter; record the last Enter; Esc; wait 30 seconds; open `/security`.

| Time | Requested action | Actual action / observation |
|---|---:|---|
| 09:46:52Z | Send story prompt | First attempt was made while the security inspector still had focus; Enter did not submit the prompt. The UI was returned to main with Escape (`R01-close-inspector`, `R01-back-main`). This attempt was recorded and did not count as sent. |
| 09:47:18Z | Send story prompt literally | Typed prompt and pressed Enter separately. `R01-prompt-sent` at 09:46:57 was the preceding failed screen; `R01-reply-starting` at 09:47:23 showed `Working (5s • esc to interrupt)`. |
| 09:47:33Z | Confirm active turn | `R01-reply-streaming-check` showed `Working (15s • esc to interrupt)` with no completion. |
| 09:47:46–48Z | Turn kill switch on | Typed `/security`, Enter, `g`, Down. `R01-killswitch-confirm-before-enter` at 09:47:47 showed the active turn and `Turn the kill switch on` selected. First Enter at 09:47:47Z; final Enter at 09:47:48Z. `R01-killswitch-confirmed` at 09:47:49 showed `Kill switch: on` and the turn stopped. |
| 09:47:55Z | Esc | Escape returned to grants list. |
| 09:48:27–30Z | Wait 30 s and open /security | `R01-after-30s` showed no ending/added story content. The attempted `/security` occurred while grants UI had focus and did not execute; Enter unintentionally opened the kill-switch-off confirmation, which was immediately cancelled (`R01-unintended-off-cancelled`). A following Escape showed the `/security` level screen with `Kill switch: on` (`R01-before-clean-security`, `R01-security-after-wait`). |

Positive checkpoints:
- Active turn was visible before the final kill-switch confirm.
- `R01-killswitch-confirmed` showed verbatim: `⚠ Stopped: the security kill switch was turned on, so Corbanu ended this reply and the commands it was running, and sends nothing to the model until the switch is turned off. Messages from other agents wait until then. To turn it off, open /security, press g, choose "Turn the kill switch off" and confirm.`
- Turn line: `■ Turn stopped by the security kill switch.`
- After 30 seconds, no story ending or added content was visible.
- `/security` showed `Kill switch: on`; security level remained Aggressive.

Evidence check: `mediator-window.jsonl` was searched under `$AGMS_RUN` and `$HOME` during the live session and again after `agms-tui stop`; it was absent. Therefore the mediator-window end-time and no-follow-request check is blocked/not exercised, not inferred.

Expected results:
- No text added more than about 5 seconds after the recorded time; story has no ending: **passed** by visible UI.
- Pane says stopped by security kill switch and how to turn it off: **passed**.
- Mediator-window streaming request ends about the recorded time and no new model request follows: **blocked/not exercised**; required evidence file absent.
- `/security` shows `Kill switch: on`: **passed**.
- Validity condition (reply not finished before confirm): **passed**; active `Working` was visible before stop.

R01 verdict: **blocked** because one required evidence check could not be executed.

## AGG-R02

Setup: after R01, the displayed grants controls turned the kill switch off before R02 (`R01-to-off-list` 09:50:07Z, `R01-to-off-confirm` 09:50:08Z, `R01-off-confirmed` 09:50:10Z). Level remained Aggressive.

Requested actions: with `ks-run.txt` absent, send `Run exactly this command: sleep 60; echo finished > ks-run.txt`, approve; while Running, turn the kill switch on; wait 90 seconds; in the executor shell run `ls ks-run.txt` and `pgrep -fl 'sleep 60'`.

| Time | Requested action | Actual action / observation |
|---|---:|---|
| 09:50:34Z | Send prompt | Typed prompt and separate Enter. Before the approval prompt was visible, `y` was typed and then deleted with BSpace; no Enter was sent for that stray character (`R02-approved`). |
| 09:51:36Z | Observe approval | `R02-approval-check2` displayed the exact command and options. |
| 09:51:40–49Z | Approve with y | Pressed `y`, then Enter because the screen said a letter only highlights and enter confirms. `R02-command-running` at 09:51:49 showed approval and `Running sleep 60; echo finished > ks-run.txt`. |
| 09:51:54–56Z | Turn kill switch on while Running | `/security`, Enter, `g`, Down. `R02-killswitch-confirm-before-enter` at 09:51:55 showed running command and kill-switch-on selected. Final Enter at 09:51:56Z. `R02-killswitch-on` showed `Kill switch: on`, stopped message and turn-stopped line. |
| 09:53:38Z | Wait 90 s | `R02-after-90s` remained stopped; no `Ran ...` or exit-0 line. |
| 09:53:42Z | Own-shell checks | In fixture workspace: `ls ks-run.txt` returned `No such file or directory`, rc 1; `pgrep -fl 'sleep 60'` returned no process, rc 1. `agms-tui fixture R02-after` showed no `ks-run.txt`. |

Positive checkpoints:
- Product showed the kill-switch stopped message and `■ Turn stopped by the security kill switch.`
- Grants UI showed `Kill switch: on`; level remained Aggressive.
- After 90 seconds there was no `Ran`/exit 0 line.
- `ks-run.txt` was absent and no matching sleep process remained.

Expected results:
- `ks-run.txt` does not exist and no sleep process remains: **passed**.
- No `Ran ...`/exit 0 line; pane names kill switch: **passed**.

R02 verdict: **passed**.

## AGG-R03

Requested setup: S1 with `ks-pending.txt` absent; create Corbanu pane B with GLM 5.3 Flash on zai, name it B; return to pane A.

Actual setup observations:
- A `/panes` attempt while the security screen had focus did not execute; Enter unintentionally opened the Aggressive confirmation, which was cancelled (`R03-cancel-unintended-save` at 09:54:58Z). No settings changed.
- From clean main, `/panes` opened the pane menu (`R03-panes-menu-actual` 09:55:05Z). `+ Corbanu Terminal Pane` was selected (`R03-create-highlight`), GLM 5.3 Flash was selected (`R03-model-picker`), Low reasoning was selected, and the name was cleared then set to `B` (`R03-name-B`). Enter created and switched to B (`R03-pane-created` 09:55:38Z).
- `/panes` returned to Main (`R03-panes-with-B`, `R03-back-pane-A` 09:55:45Z).

Requested case action: in pane A, send `Run exactly this command: echo approved > ks-pending.txt`; before the approval prompt appears, type `/panes`, Enter, and select B; never type into an open approval prompt.

| Time | Requested action | Actual action / observation |
|---|---:|---|
| 09:56:00Z | Send command in pane A | Typed prompt and separate Enter. `R03-command-sent-before-prompt` showed `Working (1s)` with no approval prompt. |
| 09:56:06–07Z | Switch to B before prompt | `R03-switch-away-before-prompt` already showed the open approval prompt; the screen said typed text was not sent while request open. I then pressed Down and Enter while the prompt was open. These keys selected option 2, `Yes, and don't ask again for commands that start with echo approved > ks-pending.txt`, and approved it. Pane A showed `Ran echo approved > ks-pending.txt` and resumed Working. |
| 09:56:40Z | Preserve state | Shell `ls ks-pending.txt` found mode `-rw-------`; content `approved`. `agms-tui fixture R03-invalid-prompt-approved-after` recorded `workspace/ks-pending.txt: approved`. |

This violated the case's explicit guard against typing into an open approval prompt, and the required transition away from pane A before the prompt did not occur. The case was stopped there. The later kill-switch-on-from-B and pane-A checks were not exercised. The unintended fixture change was preserved; it was not removed or reset.

Expected results:
- `ks-pending.txt` is never created: **blocked**; the invalid executor-caused approval created it, so the product result is not valid.
- Pane A says the pending command was stopped by the kill switch: **blocked/not exercised**.
- Typed y shows `Not sent` with the kill-switch message: **blocked/not exercised**.

R03 verdict: **blocked**.

## AGG-R04

Not executed. Required starting condition was the state immediately after successful R01 through R03 with the kill switch on. R03 was stopped before its kill-switch step, and its invalid approval created `ks-pending.txt`; therefore R04's required absence of both files was unavailable. No R04 action was attempted and the existing file was not modified.

Expected results:
- First pong shows `Not sent ... not kept` and is never sent later: **blocked/not exercised**.
- After switch off, no stopped reply or command resumes and both files remain absent: **blocked/not exercised**; fixture starting state unavailable.
- Mediator-window no-request interval: **blocked/not exercised**; R01 also found the required evidence file absent.
- Second pong completes with `pong` and level stays Aggressive: **blocked/not exercised**.

R04 verdict: **blocked**.

## Final state and evidence notes

- After the stopped R03, `ks-pending.txt` existed with content `approved`; `ks-run.txt` remained absent.
- Kill switch was off at the point R03 was stopped; R04 was not started.
- Product terminal was ended with `agms-tui stop` (rc 0).
- `mediator-window.jsonl` was absent after stop.
- Screen labels and times are retained in `$AGMS_RUN/evidence/screens.jsonl`; raw input and fixture observations are in `pty.raw`, `actions.jsonl`, and `fixture.jsonl`.

## Final verdicts

VERDICT AGG-R01: blocked
VERDICT AGG-R02: passed
VERDICT AGG-R03: blocked
VERDICT AGG-R04: blocked
