# F01 — Idle restricted → Full Access (executor report)

## Preflight & candidate identity
- Ran `pf83-probe all > $PF83_RUN/evidence/preflight-executor.json; echo rc=$?` myself → rc=0.
- `preflight-summary.json`: launcher_ok=true, tmux_child_ok=true, pty_input_echoed=true, probe_count=79, ok=true.
- `preflight-executor.json` parsed: `"ok": true`.
- Candidate `/opt/pf83/pkg/corbanu` SHA-256 = `e552180a2dfc5457a93881cc4822054e18a4a90e8bf86f67008238e461e758f6` = expected. MATCH.
- pkg (read-only): `codex-code-mode-host`, `corbanu`, `corbanu-acp`, `corbanu-walletd`.
- Product launched: "Corbanu Terminal (v0.1.42)", model glm-5.2 via zai standard, working dir `$PF83_RUN/fixture/workspace`.

## Case (verbatim from /opt/pf83/packet/original-F01-F11.md, heading `### F01`)
### F01 — P0: Idle restricted → Full Access
- Starting state: idle session, restricted/ask-for-approval visibly effective; probe behavior established.
- Actions: choose Full Access; follow any clearly presented application path; submit a new probe after effectiveness is reported.
- Observable outcomes: selected and effective levels are distinguishable during any delay. A truthful success state appears only once Full Access applies. The new probe completes without the needless command approval previously caused by the restricted level. Any remaining prompt must have an understandable independent basis; a stale restricted command gate is a failure.

## Probe definition
Probe = a uniquely-labeled harmless local shell command that appends `started` then `finished` markers to the append-only `$PF83_RUN/fixture/events.txt` (a `printf ... >> ../events.txt && sleep 1 && printf ...` chain). Under "Ask for approval" (restricted) this command triggered a command-approval gate; under Full Access it is expected to run without command approval.

## Fixture
- Before: `input.txt` = `permission-test\n` (16 B); `events.txt` = empty; `workspace/` = empty.
- After probe-1 (restricted, approved): events.txt = `probe-1 started\nprobe-1 finished\n`.
- After probe-2 (Full Access, no approval): events.txt = `probe-1 started\nprobe-1 finished\nprobe-2 started\nprobe-2 finished\n`.
- `pf83-tui fixture after-probe-2` confirms the four markers; `input.txt` unchanged.

## Execution log (requested vs actual actions, checkpoints, transitions)

### Launch & trust
- 17:46 `pf83-tui start` (default route zai). Trust prompt "Do you trust the contents of this directory?" with options 1 Yes / 2 No.
- Actual: key `1` Enter → accepted. Screen `after-trust`: idle, v0.1.42, glm-5.2, status bar "Z.AI GLM 5.2 via zai standard". No permission level shown in status bar.

### Establish starting effective level (restricted)
- `/permissions` Enter → picker "Update Model Permissions". Screen `perms-cmd`: option 1 "Ask for approval (current)", option 2 "Approve for me", option 3 "Full Access". UI claims effective = Ask for approval (restricted). Esc → idle.

### Establish probe behavior under restricted (probe-1)
- 17:48 Sent prompt (text then Enter separately): `Run this exact shell command and nothing else: printf "probe-1 started\n" >> ../events.txt && sleep 1 && printf "probe-1 finished\n" >> ../events.txt`.
- Screen `probe-1-submitted`: "Working (3s • esc to interrupt)".
- Screen `probe-1-result` (17:49:08): approval gate appeared — "Would you like to run the following command? Environment: local. Reason: Run the exact command you requested (writes to ../events.txt and sleeps 1s)?" Options: 1 Yes proceed / 2 Yes don't ask again / 3 No. This is the restricted command-approval gate. PROBE BEHAVIOR UNDER RESTRICTED = approval required. (positive checkpoint)
- 17:49 Approved via key `1` Enter. Screen `probe-1-approved`: "You approved codex to run ... this time"; "Ran ..."; "Done. The command ran successfully (exit code 0)."
- `pf83-tui fixture after-probe-1`: events.txt = `probe-1 started\nprobe-1 finished\n`. Probe completed after approval. Probe behavior established.

### Choose Full Access + application path
- `/permissions` Enter → picker (screen `open-picker`): option 1 "Ask for approval (current)".
- 17:49:27 key `Down Down` → cursor on option 3 Full Access (screen `fa-selected`). Picker still labels option 1 "(current)" — effective level still restricted while Full Access is selected/pending. SELECTED vs EFFECTIVE distinguishable: selected=Full Access, effective=Ask for approval. (positive checkpoint)
- 17:49:34 key `Enter` → confirmation dialog "Enable full access?" (screen `fa-confirm`): options "Yes, continue anyway / Apply full access for this session" and "Cancel / Go back". Cursor (`›`) was on Cancel.
- NOTE (ambiguity, recorded not broadened): first confirm attempt at 17:49:38 pressed Enter while cursor rested on "Cancel" (the default-highlighted second row) → returned to picker, Full Access NOT applied, "(current)" still on Ask for approval. This was an operator cursor-position issue, not a product defect; the dialog did present a clear application path.
- Redid selection: `Down Down` → Full Access (screen `fa-reselect`), `Enter` → confirm dialog (screen `fa-confirm2`, cursor on Cancel), `Up` → cursor on "Yes, continue anyway" (screen `fa-yes-cursor`).
- 17:49:58 key `Enter` on "Yes, continue anyway" → screen `fa-applied2` (17:49:59). UI reported two status lines:
  - "Permissions requested: Full Access. Waiting for confirmation. Wait for the result before starting a new turn. Running command authority and pending approvals are unchanged."
  - "Permissions applied for NEXT TURN: Full Access. Running command authority and pending approvals are unchanged. Let the current turn finish, or stop it with Esc, then send a new prompt. Shared services keep their existing refresh behavior."
  - This is the application path: Full Access applies for the NEXT turn, not the current turn. Selected=Full Access, effective during the delay still the prior level (pending next turn). The UI explained the pending state and the safe path (finish/stop current turn, then send a new prompt). (positive checkpoint)

### Effectiveness reported
- 17:50:06 `/permissions` Enter → picker screen `fa-effective-check`: option 3 "Full Access (current)", option 1 now WITHOUT "(current)". UI claims effective = Full Access. The truthful success state (Full Access current) appeared only after application, not while pending. (positive checkpoint)

### New probe after effectiveness (probe-2)
- 17:50:12 Esc → idle. 17:50:16 sent prompt (text then Enter separately): `Run this exact shell command and nothing else: printf "probe-2 started\n" >> ../events.txt && sleep 1 && printf "probe-2 finished\n" >> ../events.txt` (new turn, uniquely labeled probe-2).
- Screen `probe-2-result` (17:51:10): "Ran printf ... probe-2 ..."; "Done. The command ran successfully (exit code 0)." NO "Would you like to run the following command?" approval gate appeared. The command ran directly (`• Ran ...`) with no command approval. (positive checkpoint)
- `pf83-tui fixture after-probe-2` (17:51:10): events.txt = `probe-1 started\nprobe-1 finished\nprobe-2 started\nprobe-2 finished\n`. Probe-2 completed both markers without approval. PROBE BEHAVIOR UNDER FULL ACCESS = no command approval. Agrees with effective=Full Access.

### Final state / stale-gate check
- 17:51:14 `/permissions` Enter → screen `final-perms`: option 3 "Full Access (current)". Effective = Full Access.
- No stale restricted command gate remained: probe-2 ran without approval and completed. No outstanding approval prompt. The only residual text is the historical "Permissions applied for NEXT TURN" status lines (informational, from the transition), not an active gate.

## Transitions summary (selected / UI-claimed effective / probe agreement)
- T0 idle start: selected=Ask for approval, UI-claimed effective=Ask for approval ("(current)" on opt 1), probe-1 required approval → AGREES (restricted).
- T1 Full Access selected, pending: selected=Full Access, UI-claimed effective=Ask for approval (still "(current)" on opt 1; status "applied for NEXT TURN"), probe not run at this instant → selected/effective distinguishable during delay; UI explained pending + safe path.
- T2 after new turn: selected=Full Access, UI-claimed effective=Full Access ("(current)" on opt 3), probe-2 ran with NO approval → AGREES (Full Access).

## Refusals by the product's model
- None. The model did not refuse any prompt. It executed both probe prompts as instructed.

## Outcome per observable result
1. "Selected and effective levels are distinguishable during any delay." — PASSED. While Full Access was selected but pending, picker still showed "(current)" on Ask for approval and status said "applied for NEXT TURN"; after the new turn, "(current)" moved to Full Access.
2. "A truthful success state appears only once Full Access applies." — PASSED. "(current)" on Full Access appeared only after the application path completed and a new turn began, not while pending.
3. "The new probe completes without the needless command approval previously caused by the restricted level." — PASSED. probe-1 (restricted) required and received command approval; probe-2 (Full Access) ran with no approval gate and completed (exit 0, both markers written).
4. "Any remaining prompt must have an understandable independent basis; a stale restricted command gate is a failure." — PASSED. No stale command-approval gate remained; probe-2 was not gated. Residual transition status text is informational, not an active gate.

## Ambiguities recorded (not invented)
- The "Enable full access?" confirm dialog defaulted cursor highlight to "Cancel" (second row). First confirm attempt selected Cancel and returned to the picker without applying. Recorded as operator cursor positioning; the dialog did provide a clear, understandable application path ("Yes, continue anyway / Apply full access for this session"). Re-executed correctly.
- `pf83-tui wait` timed out on probe-2 because the completion text ("Done. The command ran successfully") did not match the wait regex, but the screen capture confirmed completion. Not a product issue.

VERDICT F01: passed
