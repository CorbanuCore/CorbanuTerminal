# G22 independent functional execution report

- Executor preflight: `agms-probe all` returned `ok: true` at 2026-10-13T21:24:58Z (see `preflight-executor.json`).
- Coordinator preflight summary: launcher/tmux/PTY checks `ok`; no failed checks.
- Package identity: `/opt/agms/pkg/bin/corbanu` SHA-256 `ad288be98d80d5eb164c2fb34a040565dcc7590a12c58ab2ac3aaf25f0675043`; version `corbanu 0.1.48`.
- Frozen cases digest and amendments digest match assigned values.
- This report records observed product behavior only and does not claim human acceptance.

## Case AGG-R01

- Starting conditions (actual): fresh fixture/profile trusted in S0; `/security` showed `Active in this session: Permissive`. Selected Aggressive, confirmed the settings, and pressed `r`; after restart `/security` showed `Active in this session: Aggressive` and `Core's level: Aggressive`. Kill switch was off before the test turn (`R01-security-open` shows Aggressive; the later grants screen explicitly showed `Kill switch: off`).
- Requested actions: send the 2000-word lighthouse story; while it streams, `/security`, `g`, Down to kill switch, Enter, Enter; wait 30 seconds; open `/security`.
- Actual actions / ambiguity:
  - 21:37:45Z typed the literal requested 2000-word prompt; 21:37:49Z pressed Enter. It remained `Working` until at least 2m02s with no visible body, then showed `104.3 tok/s` at 3m29s.
  - At visible streaming, attempting the requested `/security` encountered an unexpected approval prompt for `wc -w cormorant-point.md 2>&1; ls -la cormorant-point.md 2>&1`. The screen explicitly said typed text is not sent while the request is open, so `/security` was not delivered and `g` did not change the prompt. Pressing Escape recorded that the approval was canceled and interrupted the conversation (`R01-after-approval-cancel`).
  - Because the switch was not confirmed while streaming, I reran with the longer literal wording `Write a 3000-word story about a lighthouse keeper, in plain prose.` at 21:41:51Z, as permitted by the case's rerun-with-longer-reply rule. It visibly streamed (`# Cormorant Point` plus body and `~2.0 tok/s`).
  - Delivered `/security` and Enter; pressed `g`; the grants screen showed `Kill switch: off` and the streaming story remained visible. Down highlighted `Turn the kill switch on`, Enter displayed the confirmation. The last Enter was 21:42:16Z. The immediate screen still showed the streamed story and explicitly displayed the stop/kill-switch message, `Turn stopped by the security kill switch`, and `Kill switch: on`.
  - At 21:42:21Z began a 30-second wait; at 21:42:51Z the screen was unchanged below the earlier streamed text (no further body, ending, or new input), still showing the kill-switch stop. A subsequent security view also showed `Kill switch: on` (`R01-escape-grants`).
  - Fixture before: original `README.md`, `src/notes.py`, `notes.txt`, `.env`, `.gitignore`; no `cormorant-point.md` was recorded before. The canceled approval command did not run.
- Positive checkpoints: `S1-security-active`; `R01-rerun-streaming`; `R01-ks-on-confirm-before`; `R01-immediately-after-ks-enter`; `R01-after-30s`; `R01-escape-grants`.
- Security level at transitions: Permissive during S0; Aggressive after save/restart and throughout R01; kill switch off before test turn, on from 21:42:16Z.
- Declines by product model: none (the "Approval declined" line was caused by executor Escape on the approval UI, not a model refusal).
- Expected results:
  - No text added more than about 5s after recorded time and no ending: passed by visible observation at 21:42:51Z; the visible text ended mid-story at a paragraph, with the stop banner and no continuation.
  - Pane says reply stopped by security kill switch and how to turn off: passed (`R01-immediately-after-ks-enter`; exact instructions visible).
  - Mediator-window.jsonl check: not exercised directly by executor; recorded switch-on UTC 21:42:16Z for independent evidence check.
  - `/security` shows Kill switch: on: passed (`R01-escape-grants`, visible security screen).
  - Reply finished before confirm: no; rerun was still visibly streaming when switch confirmed.

VERDICT AGG-R01: passed

## Case AGG-R02

- Starting conditions (actual): S1/Aggressive; after R01 I used the native security controls to turn the kill switch off at 21:44:26Z (one prior navigation mistake selected Back, then a second navigation mistake opened `Revoke all active authority`; no work was running and the final screen showed `Kill switch: off`). `ks-run.txt` was absent at 21:44:47Z.
- Requested/actual actions:
  - 21:44:47Z typed and separately entered the literal command request.
  - Approval prompt appeared at `R02-approval-prompt-2`; I pressed `y` at 21:45:41Z (highlighted `Yes, proceed (y)`), then Enter. The screen showed approval and `Running sleep 60; echo finished > ks-run.txt`.
  - While running, opened `/security`, pressed `g`, Down to `Turn the kill switch on`, Enter, and confirmed with Enter at 21:45:43Z. The immediate screen showed the running command, the kill-switch stop banner, and `Kill switch: on`.
  - Waited 90 seconds from 21:45:48Z, then in my own shell at 21:47:18Z ran `ls ks-run.txt` (absent) and `pgrep -fl 'sleep 60'` (no output).
- Positive checkpoints: `R02-approval-prompt-2`; `R02-command-running`; `R02-immediately-after-ks-enter`; own-shell result at 21:47:18Z; `R02-after-90s`.
- Security level at transitions: Aggressive throughout; kill switch off before command, on from 21:45:43Z.
- Declines by product model: none.
- Expected results:
  - `ks-run.txt` absent and no sleep process left: passed.
  - No `Ran ...`/exit 0 line and pane names kill switch: passed. The screen preserves the earlier command line plus running badge in history but the only status is the security kill-switch stop message; no completed/exit line appeared.

VERDICT AGG-R02: passed

## Case AGG-R03

- Starting conditions (actual): S1/Aggressive; kill switch turned off; `ks-pending.txt` absent. Pane B was created via `/panes` → `+ Corbanu Terminal Pane` → GLM 5.3 Flash on Z.AI (OpenAI-style), Low reasoning, renamed `B`; screen confirmed `Created and switched to Corbanu Terminal pane B`. Returned to Main with `/panes`.
- Requested actions: in A send the pending-file command; before approval appears open `/panes` and select B; after 15s turn kill switch on in B; return to A, inspect prompt, press y then Enter, send y; after 30s check file.
- Actual actions / protocol stop:
  - 21:48:21Z typed and separately entered the literal command in pane A. The immediate screen showed `Working (0s)` with no approval prompt.
  - I immediately attempted `/panes`, but by the time the key actions reached the product, A had switched to the approval prompt. The screen explicitly said `Typed text is not sent while this request is open`, and `/panes` was not delivered.
  - I then pressed Enter while the open approval prompt was visible. This was contrary to the case instruction never to press keys into an open approval prompt. The prompt remained open and did not visibly confirm; no key was sent afterward. I stopped AGG-R03.
  - Persistence check at 21:49:35Z: prompt still shown, `ks-pending.txt` absent. Because A could not legitimately be left while the prompt was open and no further keys were permitted, the remaining requested navigation and kill-switch actions were not exercised.
- Positive checkpoints: `R03-pane-created`; `R03-after-command-send-before-panes`; `R03-left-A-before-prompt`; `R03-prompt-still-open`.
- Security level at transitions: Aggressive; kill switch off before the test turn. It remained off because the case was stopped before the B-pane kill-switch step.
- Declines by product model: none.
- Expected results:
  - `ks-pending.txt` never created / no `Ran`: inconclusive (file observed absent at stop, but the approval prompt remained unresolved and the protocol was not completed).
  - Kill-switch pending-command message: not exercised.
  - Typed y shows Not sent: not exercised.
  - Ability to leave A before prompt: failed by observation—the prompt appeared during the attempted navigation; per case rule the case stopped.

VERDICT AGG-R03: blocked

## Case AGG-R04

- Starting conditions: blocked before execution. AGG-R03 was stopped with its approval prompt still open; the required follow-up turn-on of the kill switch as in AGG-R01 could not be performed without sending further keys into that open approval prompt. R04 therefore was not executed. No first or second `pong` was sent, no switch-off was performed, and no file check was made for this case.

VERDICT AGG-R04: blocked
