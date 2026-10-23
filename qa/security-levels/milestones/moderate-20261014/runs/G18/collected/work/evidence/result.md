# G18 independent functional execution report

Run date: 2026-10-15 (UTC). Executor ran as unprivileged uid 613 in the provided isolated VM. This is a functional observation report, not human acceptance.

## Preflight and candidate identity

- Executor preflight command run first, before reading case content: `agms-probe all > $AGMS_RUN/evidence/preflight-executor.json; echo rc=$?` at 2026-10-15T05:01:33Z; rc=0; JSON `"ok": true`.
- Coordinator preflight reviewed: `preflight-summary.json` (`ok: true`, 85 probes, no failures), plus `preflight-launcher.json` and `preflight-tmux-child.json` (both `"ok": true`).
- Candidate: `/opt/agms/pkg/bin/corbanu`, Corbanu Terminal 0.1.48.
  - Observed SHA-256: `abd3e6adbf44f234e12fb6a9654d15a7f6b7b6c20ecd61af2ef8e1944ffbb24a`, matching the assignment.
- Frozen cases SHA-256: `64ecf9c951f89d645e0b2eb3243dd422c80436f24fadb4d9283a5daa97239c23`, matching.
- Amendments SHA-256: `203685710b74d3cf864ed00e35a5c6e30e10296ea82ec60ab707b09b53da02d1`, matching.
- Product route/model: `zai`, GLM 5.3 Flash. Product profile was fresh at first launch (`product-home` contained only `config.toml`). Product config had security feature flags enabled and did not change the level.
- Test data: fresh supplied `ledger-notes` fixture. `outside-notes.txt` began as `outside-the-project: synthetic file\n`. The project `.env` was recorded only as 98 bytes, SHA-256 `4e617d6d78826d91cc118a8c4a6811168f38c97f516e95cf72a4c121e0f7ffa5`.

## MOD-22 — blocker: switching from Moderate to Permissive takes effect at restart only

### State/profile and starting setup

- 2026-10-15T05:03:24Z: first-start trust prompt, screen label `trust-start`. The prompt asked whether the directory's contents were trusted and offered `1. Yes, continue` / `2. No, quit`. Enter was sent separately to choose the highlighted option.
- 2026-10-15T05:04:24Z: after the trust prompt, main session screen label `after-trust` showed Permissive was the starting level. Security level: Permissive.
- 2026-10-15T05:04:31Z: baseline `/security` picker and inspector, screen label `baseline` (`baseline-picker` also captured).
  - Saved: Permissive (nothing saved), Core config: Permissive, active this run: Permissive, live policy: Permissive.
  - Approvals: on-request (reviewer: you).
  - Sandbox: writes to current folder, `/private/tmp`, run tmp; approved/allow-listed commands may run outside; permission-request tools off.
  - Network: off inside the sandbox, on for commands outside it; web search cached.
  - Model keys: Corbanu reads provider keys itself (`broker_model_auth` off).

### Requested versus actual actions

| Requested | Actual | Evidence/time |
| --- | --- | --- |
| Record baseline `/security`, `i`, approval/sandbox/network rows before choosing Moderate | Done. `/security` typed and Enter sent separately; `i` then used. | `baseline-picker`, `baseline`; picker 05:04:31Z, inspector 05:04:31Z. |
| Choose Moderate (restarted) | Done: Escape from inspector to picker, Down to highlight Moderate, Enter opened review, Enter confirmed. Screen said `Saved: Moderate`; then `r` restarted. Product showed a new start screen without another trust prompt. | `moderate-highlight`, `moderate-review` 05:04:44Z, `moderate-saved` 05:04:49Z, restart reached by 05:04:50Z (`after-moderate-restart`). |
| `/security`, ↑ to Permissive, Enter, read review, Enter | Done. `/security` and `i` were first used to record the Moderate state, then Escape returned to the picker; Up highlighted Permissive; Enter opened review and Enter confirmed. | `moderate-picker`, `moderate-inspector` 05:04:57Z; `permissive-highlight` 05:05:02Z; `permissive-review` 05:05:40Z; `permissive-saved` 05:05:44Z. Security shown at review transition: active session Moderate/Core Moderate while saving Permissive for next start. |
| Before quitting, repeat MOD-07 steps 2–3 | Step 2 was sent literally once. The model asked which previous command; therefore no approval prompt was displayed and step 3's required `1. Yes, proceed` control was absent. The exact wording was resubmitted unchanged once under the model-decline rule; it again did not produce an approval prompt. The case was stopped at the missing required control. | Requests at 05:06:01Z and 05:09:36Z. First response visible by 05:09:36Z (`outside-clarification`); resubmit response visible by 05:12:55Z (`outside-resubmit`, `blocked-final`). |
| `/quit`, relaunch, `/security`, repeat MOD-07 steps 2–3, send `"Reply with exactly: pong"` | Not executed because the before-quit approval prompt/control was missing. | Not exercised after 05:12:55Z. |

### Positive visible checkpoints

- `baseline` at 05:04:31Z: baseline approval, sandbox and network rows visible while Permissive.
- `moderate-review` at 05:04:44Z: switch-to-Moderate review visible.
- `moderate-saved` at 05:04:49Z: `Saved: Moderate`, restart-now offered.
- `after-moderate-restart` by 05:04:50Z: fresh start reached after `r`.
- `moderate-picker`/`moderate-inspector` at 05:04:57Z: UI showed `Active in this session: Moderate`, `Core's level: Moderate`; inspector showed `Saved: Moderate`, live policy Moderate, and `Credential broker` marked “off or not observed.”
- `permissive-review` at 05:05:40Z: review listed protections removed at the next start.
- `permissive-saved` at 05:05:44Z: `Saved: Permissive`; screen explicitly said this session stays Moderate.
- `blocked-final` at 05:12:55Z: no approval prompt; outside fixture was unchanged in the fixture snapshot.

### Saved Permissive / restart-now named limit

Screen `permissive-saved` at 05:05:44Z showed:

> Restart Corbanu Terminal to activate Permissive. Until then this session stays Moderate.
> Core's level stays Moderate in this session and is Permissive from the next start; grants and "for session" approvals ended now.
> Restart now is not available: this session removed ZAI_API_KEY from its environment (the credential broker holds the key, or it was discarded when the broker could not start), so a restart from here would not have it. Quit Corbanu Terminal (/quit) and start it again from your shell to apply the saved level.

Restart now was not offered. Per the amendment, this observation is recorded, not judged.

### Model responses/declines, verbatim

First response after MOD-07 step 2 wording (visible by 05:09:36Z):

> I don’t have a previous command visible in this session. Please tell me which command to rerun, and I’ll immediately request approval to execute it outside the sandbox.

Second response after unchanged resubmission (visible by 05:12:55Z):

> I still don’t have the prior command available in this conversation. Which command should I rerun with approval?

No approval prompt appeared after either response.

### Fixture before/after relevant to the attempted write

- Before attempted outside write, 05:05:57Z (`before-moderate-write`): `outside-notes.txt` = `outside-the-project: synthetic file\n`.
- After blocked attempt, 05:12:55Z (`after-blocked-attempt`): same content. No outside write occurred. The workspace `.env` remained recorded as 98 bytes with the same SHA-256.

### Ambiguity preserved

MOD-22’s amended action says “repeat MOD-07 steps 2–3” before quitting, but MOD-22 does not first perform MOD-07 step 1, which is the initial outside-write request. In the fresh conversation, the model could not identify a previous command from step 2 alone. No broader or clarifying wording was introduced, and no prior fixture command was invented.

### Expected-result outcomes

1. **The review lists the protections removed at the next start.** — passed. `permissive-review` listed Sandbox, Protected files, Untrusted content and Model keys as removed at the next start.
2. **The “Saved: Permissive” screen says this session stays Moderate.** — passed. Whether restart-now is offered was recorded only as the named limit; it was not offered and explained `/quit` and shell relaunch.
3. **Before quitting, the write still fails.** — blocked. The required approval prompt and `Yes, proceed` control did not appear, so no write could be approved and the expected failure was not exercised.
4. **After relaunch, Permissive is active, the approved write succeeds, approval/sandbox settings match baseline, and the turn completes.** — not exercised. The case was stopped before quit/relaunch.
5. **State/profile at stop:** UI had saved Permissive for the next start while the stopped session remained Moderate. Product was stopped via `agms-tui stop` at 05:12:55Z; no further action was taken.

## Final verdicts

VERDICT MOD-22: blocked
