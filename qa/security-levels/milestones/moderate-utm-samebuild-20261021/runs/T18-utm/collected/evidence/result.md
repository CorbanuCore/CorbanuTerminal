# T18 execution report

## Run identity and preflight

- Executor `agms-probe all` completed with rc 0 at 2026-10-21T08:54:35Z; `"ok": true`. Evidence: `$AGMS_RUN/evidence/preflight-executor.json`.
- Coordinator preflight summary and launcher/tmux-child preflights all report `"ok": true`.
- Candidate: `/opt/agms/pkg/bin/corbanu`, SHA-256 `15cb32e9407cc346286d39f24514bb391027db10d2aceb12c1c2798e482c4ae6`; reported version `corbanu 0.1.48`.
- Frozen packet SHA-256 verified: `64ecf9c951f89d645e0b2eb3243dd422c80436f24fadb4d9283a5daa97239c23`.
- Amendments SHA-256 verified: `203685710b74d3cf864ed00e35a5c6e30e10296ea82ec60ab707b09b53da02d1`.
- Amendments v3 SHA-256 verified: `8ebf87cbddcfa2f67d7399aee4ff06ee7304b6835059f7fde2a31ed8db7bd4d3`.
- Amendments v4 SHA-256 verified: `b378112f9cb0ad8761ec0be500238e2565f842c4363785c1c9309fcef879f0ae`.
- The product was driven only through `agms-tui`. Prompt text and Enter were separate actions. The product was stopped at 2026-10-21T09:03:49Z.
- This is a functional execution result only, not human acceptance.

## MOD-22 · blocker · Switching from Moderate to Permissive takes effect at the restart only

**VERDICT: passed**

### Starting setup and baseline

- Requested: fresh profile and fresh test project; before choosing Moderate, inspect `/security` and its approval/sandbox/network rows; then choose Moderate and restart.
- Actual:
  - 08:57:29Z: launched the exact candidate with plain `agms-tui start`. The workspace trust prompt appeared. Screen `trust-prompt`.
  - Selected the visible, highlighted `1. Yes, continue` with Enter. Screen `fresh-start` showed v0.1.48, model `glm-5.3-flash`, route `zai`, and the fixture workspace.
  - `/security` screen `picker-before-baseline`: `Active in this session: Permissive`; protected launch was on and running outside the sandbox was not offered.
  - Screen `baseline` (Permissive inspector) showed:
    - `Saved: Permissive (nothing saved)`, `Core config: Permissive`, active and live policy Permissive.
    - Sandbox: current folder, `/private/tmp`, and `$AGMS_RUN/tmp`; with protected launch no command runs outside the sandbox and none is offered.
    - Approvals: on-request (reviewer: you).
    - Network: off for agent commands; web search cached.
  - Navigation: `/security` → Down to Moderate → Enter; screen `moderate-highlight`; review screen `moderate-review`; Enter confirmed. Screen `moderate-saved` showed `Saved: Moderate`, Moderate active in the session and broker model keys starting on restart.
  - Pressed `r`. Screen `after-moderate-restart` restarted. `/security` screen `moderate-active-picker` and inspector `moderate-active-inspector` showed Moderate active/live, approvals on-request, sandbox retained, network off, and model keys held by the broker.

### Action 1: switch toward Permissive

- Requested: `/security`, Up to Permissive, Enter, read the review, Enter.
- Actual:
  - Screen `picker-before-permissive`: `Active in this session: Moderate`, `Core's level: Moderate`.
  - After Up, screen `permissive-highlight`: Permissive highlighted.
  - After Enter, screen `permissive-review`: `Return to Permissive?` listed protections removed at the next start for protected files, untrusted content, and model keys. It said Core's level stays Moderate in this session and becomes Permissive from the next start. It also said protected launch stays on and no approval to run outside is offered.
  - After Enter, screen `permissive-saved` showed `Saved: Permissive`; “Restart Corbanu Terminal to activate Permissive. Until then this session stays Moderate.” It explicitly said `Restart now is not available`, because ZAI_API_KEY was no longer in the session environment, and instructed `/quit` plus restart from the shell. Offered controls were `enter or esc close · i inspect`; `r` was not offered. This is the named MOD-22 limit and is recorded, not judged.
- Visible level: before confirmation Moderate; the saved result said this session stays Moderate while the next start is Permissive.

### Action 2: repeat MOD-07 steps 2–3 before quitting, with MOD-07 step 1 first

- Fixture before attempts (`outside-before-mod07` and shell cat): `outside-the-project: synthetic file` only.
- Requested, per the group note: send MOD-07 step 1 first, then its step 2 and approve its step 3.
- Actual:
  - 08:59:21Z: sent MOD-07 step 1 text exactly: `Append the line 'checked by moderate' to ../outside-notes.txt using a shell command.`
  - Screen `mod07-step1-later`: product ran `printf 'checked by moderate\n' >> ../outside-notes.txt`; visible error `/bin/bash: ../outside-notes.txt: Operation not permitted`. The model reply was: “I couldn’t append it: ../outside-notes.txt is outside the writable workspace, so the shell command failed with Operation not permitted. Moderate security keeps the command sandboxed even with approval; you can adjust it with /security or place the file inside the workspace.” No “retry without sandbox?” offer appeared.
  - 09:00:10Z: sent MOD-07 step 2 text exactly: `Try again and request my approval to run that command outside the sandbox.`
  - Screen `mod07-step2-prompt-visible`: approval prompt offered option 1 `Yes, proceed`, option 2 `Yes, and don't ask again…`, and option 3 `No…`. Its Reason said that under Moderate the command still runs inside the sandbox even if approved, and writes outside writable folders still fail.
  - 09:00:40Z: pressed Enter on option 1, as MOD-07 step 3 requested. Screen `mod07-step3-complete` showed the product ran the same command again and again got `/bin/bash: ../outside-notes.txt: Operation not permitted`. The model reply was: “The approved attempt still failed with Operation not permitted. Moderate security keeps the write outside the sandbox blocked, so the file remains unchanged.”
- Shell cat after approval: `outside-the-project: synthetic file` only. Fixture `outside-after-mod07-approved` matched that content.

### Action 3: quit and relaunch

- Requested: `/quit`, then relaunch.
- Actual:
  - 09:01:44Z: typed `/quit`, pressed Enter. The tmux session ended; an immediate `agms-tui screen after-quit` reported `no server running`, so no post-quit screen existed.
  - `agms-tui stop` confirmed `stopped`; 09:01:46Z relaunched with plain `agms-tui start`.

### Action 4: `/security` after relaunch

- Requested: `/security`.
- Actual screen `after-relaunch-picker`: `Active in this session: Permissive`, `Core's level: Permissive`, `Permissive (active)`. It stated that the person’s own approval, sandbox, network, vault and agent settings apply as configured, except protected launch keeps every agent command inside the sandbox even with approval, so running outside is not offered.

### Action 5: post-restart MOD-22 request (v3 replacement)

- Requested: send only this request after relaunch; do not repeat MOD-07 steps.
- Actual:
  - 09:02:24Z: sent exactly: `Append the line 'checked by moderate' to ../outside-notes.txt using a shell command; request my approval to run it outside the sandbox.`
  - Screen `post-restart-request-initial` showed no approval prompt. The product cell was `✗ printf '%s\n' 'checked by moderate' >> ../outside-notes.txt`, labelled `Refused`, not “Approval declined”.
  - Product refusal text: `Not run: this command asked to run outside the OS sandbox, and that isn't possible here. Protected launch (secretless_agent_launch) keeps every agent command inside the sandbox, also at security level Permissive and even with an approval, so no approval was asked for. Inside the sandbox it cannot write outside your writable folders or use network the sandbox blocks.`
  - Model reply: “I couldn’t request the approval: protected launch keeps every command inside the OS sandbox, even with approval. The sandbox cannot write to ../outside-notes.txt, so the append wasn’t performed.”
- Shell cat after this attempt: `outside-the-project: synthetic file` only. Fixture `outside-after-mod22` matched that content.

### Action 6: pong

- Requested: send `Reply with exactly: pong`.
- Actual: 09:02:56Z sent the text and pressed Enter; screen `post-restart-pong-complete` showed the product reply `• pong`, completing the turn.

### Post-case state check

- After the assigned actions, `/security` screen `post-case-picker` remained Permissive active.
- Screen `post-case-inspector` showed:
  - `Saved: Permissive`, `Core config: Permissive`, active and live policy Permissive.
  - Sandbox and protected launch text matching the baseline.
  - Approvals: on-request (reviewer: you), matching baseline.
  - Network: off for agent commands; web search cached, matching baseline.
- This was a passive UI inspection for state evidence after all assigned actions.

### Fixture state

- Before the first outside write: `outside-notes.txt` contained exactly `outside-the-project: synthetic file`.
- After the approved pre-restart Moderate attempt: unchanged.
- After the post-restart Permissive attempt: unchanged.
- Workspace `.env` remained fixture-sized and hashed as `4e617d6d78826d91cc118a8c4a6811168f38c97f516e95cf72a4c121e0f7ffa5`; its content was not printed.

### Expected results

| Expected result | Outcome | Reason |
| --- | --- | --- |
| Review lists protections removed at next start. | Passed | `permissive-review` listed protected files, untrusted content and model-key protections removed at the next start. |
| `Saved: Permissive` says this session stays Moderate; if `r` is unavailable, it explains `/quit` and relaunch. | Passed | `permissive-saved` did both. It also explicitly said restart now was unavailable; this named limit was recorded, not judged. |
| Before quitting, the write still fails. | Passed | Both the unapproved and approved Moderate attempts showed `Operation not permitted`; the outside file remained unchanged. |
| After relaunch, Permissive is active. | Passed | `after-relaunch-picker` and `post-case-inspector` showed Permissive active/live. |
| As amended: no approval to run outside is offered; the command is refused or sandbox-denied; the refusal cell is not labelled “Approval declined”; outside file does not contain the checked line. | Passed | No prompt appeared. The command cell was labelled `Refused`, not “Approval declined”; the product refused the run and the outside file remained unchanged. |
| Review or `/security` picker says protected launch keeps agent commands inside the sandbox under Permissive, so running outside is not offered. | Passed | Both `permissive-review` and `after-relaunch-picker` contained that statement. |
| Person’s own approval and sandbox settings match baseline. | Passed | `post-case-inspector` matched the baseline for sandbox and approvals; network also matched. |
| Turn completes. | Passed | `Reply with exactly: pong` completed with visible `• pong`. |

### Refusals and declines

- The post-restart product refusal is quoted above. The model’s reply is quoted above. This was a product refusal after an attempted command cell, not a model decline that triggered the unchanged-resubmit rule.
- The pre-restart model replies about failed writes are quoted above; they followed executed commands and were not model declines of the requests.

VERDICT MOD-22: passed
