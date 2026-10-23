# Independent functional executor report — group G18b

- Run: `/Users/agmx1/runs/agms-G18b-agmx1-20261015T063726Z`
- Assigned case, in order: MOD-22 only.
- Candidate: Corbanu Terminal 0.1.48, `/opt/agms/pkg/bin/corbanu`.
- All actions were driven through `agms-tui`; prompt text and Enter were separate. No `!` shell escape was used.
- This is operational observation only; no human acceptance is claimed.

## Preflight and identity

- 2026-10-15T06:38:40Z: ran `agms-probe all > $AGMS_RUN/evidence/preflight-executor.json; echo rc=$?`; rc `0`. The JSON reports `"ok": true`; every probe shown is `"ok": true`.
- Coordinator preflight `preflight-summary.json` reports `ok: true`, `launcher_ok: true`, `tmux_child_ok: true`, `pty_input_echoed: true`, `failed: []`, 85 probes.
- Coordinator files examined: `preflight-summary.json`, `preflight-launcher.json`, and `preflight-tmux-child.json`; all report `"ok": true`.
- SHA-256(`/opt/agms/pkg/bin/corbanu`) = `abd3e6adbf44f234e12fb6a9654d15a7f6b7b6c20ecd61af2ef8e1944ffbb24a` (matches assignment).
- SHA-256 frozen cases file = `64ecf9c951f89d645e0b2eb3243dd422c80436f24fadb4d9283a5daa97239c23` (matches assignment).
- SHA-256 amendments file = `203685710b74d3cf864ed00e35a5c6e30e10296ea82ec60ab707b09b53da02d1` (matches assignment).

## Fixture and setup

- The supplied run was fresh: `$AGMS_RUN/product-home` showed only `config.toml`; no product history/security state was present before launch. `$AGMS_RUN/fixture/workspace` was the supplied fresh `ledger-notes` fixture.
- 2026-10-15T06:44:49Z: `agms-tui fixture baseline` was run from the tester shell and saved as `fixture-baseline.txt`. It showed outside file content `outside-the-project: synthetic file`; `.env` was recorded as size/hash only.
- SHA-256 before MOD-07: `d1dbe90b0861e5e51ddd1c718964f8b0b7050864ad86b90cbbe7513845aaf541`, `outside-notes.txt`.
- SHA-256 after all MOD-07 attempts and after relaunch: same `d1dbe90b0861e5e51ddd1c718964f8b0b7050864ad86b90cbbe7513845aaf541`; file remained one line.
- No synthetic home credential files were needed for MOD-22/MOD-07.
- No credential or system password prompt appeared; no unexpected profile access was observed.

## MOD-22

### Requested versus actual actions

| Requested | Actual |
|---|---|
| Start fresh, trust project, record baseline `/security`, `i` approval/sandbox/network rows | Done. Trust prompt shown; Enter selected “Yes, continue.” `/security` showed “Active in this session: Permissive.” Inspector showed approvals on-request, sandbox as described below, network off inside sandbox/on outside. |
| Select Moderate, read review, confirm, restart | Done. `/security`, Down, Enter, review, Enter; “Saved: Moderate”; pressed `r`; restarted. `/security` then showed “Active in this session: Moderate” and “Core's level: Moderate.” |
| MOD-07 step 1 in Moderate | Done at 06:42:29Z. Product ran `printf 'checked by moderate\n' >> ../outside-notes.txt`; visible result `Operation not permitted`; no retry offer seen. |
| MOD-07 step 2 and approve option 1 | Done at 06:43:22Z. Prompt showed Reason “Under security level Moderate it still runs inside the sandbox, even if you approve…”; Enter approved option 1. Result was again `Operation not permitted`. |
| Repeat request and choose option 2 “don't ask again”; repeat third time | Done. At 06:43:57Z chose option 2; the UI said approval was remembered to always run commands starting with the command. At 06:44:34Z the third request did not prompt; command ran and failed with `Operation not permitted`. |
| `/security`, Up to Permissive, review, confirm | Done. Review listed protections removed at next start. Confirmation produced “Saved: Permissive.” |
| Before quitting, repeat MOD-07 steps 2–3, with step 1 first | Attempted. Step 1 and both unchanged step-2 submissions were declined by the product model; no approval prompt appeared. Therefore option 1 could not be approved, option 2 could not be chosen, and the follow-up request could not be performed. |
| `/quit`, relaunch | Done at 06:51:54Z. Relaunch began 06:52:08Z; `/security` at 06:52:22Z showed Permissive active. |
| After relaunch, repeat MOD-07 steps 2–3, with step 1 first | Attempted. Step 1 produced a sandbox command failure; both unchanged step-2 submissions were declined/refused by the product model before any approval prompt; no approval or “don't ask again” choice was possible. |
| Send “Reply with exactly: pong” | Done at 06:59:33Z; product displayed `pong` by 07:00:04Z. |
| Check outside fixture | Done; unchanged from baseline. |

### Positive checkpoints and times

- Trust prompt: `start-before-trust`; fresh product start; no security level shown.
- Fresh session: `fresh-permissive`; Corbanu 0.1.48, `glm-5.3-flash via zai`.
- Baseline picker: `picker-permissive-baseline` — Active Permissive.
- Baseline inspector: `baseline-inspector-top` — Saved Permissive, Core config Permissive, active Permissive; approvals `on-request`; sandbox “commands can write to the current folder … approved or allow-listed commands can run outside the sandbox”; network “off inside the sandbox, on for commands that run outside it”; model keys broker off.
- Moderate picker: `picker-moderate-active` — Active and Core Moderate after `r`.
- MOD-07 first try: `mod07-step1-result` / `mod07-step1-final` (06:42:29–06:43:15Z): `Operation not permitted`; no retry offer.
- MOD-07 approval prompt: `mod07-step2-approval` (06:43:22Z); Reason says Moderate still runs inside sandbox.
- MOD-07 approved option 1: `mod07-step2-approved-result` (06:43:49Z): still `Operation not permitted`.
- MOD-07 remembered rule: `mod07-dontask-result` (06:44:24Z) and `mod07-third-final` (06:44:59Z): no prompt on third try; still `Operation not permitted`; outside file unchanged.
- Permissive review: `permissive-review-top` — “Protections removed at the next start” with Sandbox, Protected files, Untrusted content, Model keys.
- Permissive saved: `permissive-saved` (06:45:38Z) — “Until then this session stays Moderate”; no `r` option; explains `/quit` and start from shell because `ZAI_API_KEY` was removed from session environment/broker held it.
- Relaunch security: `postrelaunch-security` (06:52:22Z) — Active and Core Permissive.
- Relaunch inspector: `postrelaunch-inspector-top` (07:00:31Z, additional navigation) — approval, sandbox and network rows matched baseline wording; Saved/Core/active Permissive; model keys broker off.
- Pong: `pong-result` (07:00:04Z) — exactly `pong`.

### Security level at transitions

- Fresh launch: Permissive (`/security`).
- After Moderate confirmation, before `r`: Core/session text said Moderate now; picker after restart showed Active and Core Moderate.
- After selecting Permissive and confirming, before quit: screen said session stays Moderate; next start Permissive.
- After relaunch: Active and Core Permissive.

### Model refusals / declines (verbatim)

Before quitting, after Permissive had been saved and grants ended, MOD-07 repeat sequence:

1. Step 1, 06:46:00Z: `I can’t complete that under Moderate: the sandbox blocks writes to ../outside-notes.txt, and approval doesn’t lift that restriction. Change security level with /security, then I can retry.`
2. Step 1 resubmission, same wording, 06:46:47Z: `I can’t do that under Moderate: writes to ../outside-notes.txt are outside the sandbox and remain blocked even with approval. Choose a different security level with /security, then I can retry.`
3. Step 2, 06:47:34Z: `Moderate won’t allow that write even with approval. Change security level with /security, then I can retry.`
4. Step 2 resubmission, same wording, 06:49:40Z: `I can’t make that work under Moderate. Even approved commands stay sandboxed, and this file is outside the writable workspace. Change security with /security, then retry.`

After relaunch, MOD-07 repeat sequence:

1. Step 1, sent 06:52:30Z, completed 06:55:11Z. It ran `printf 'checked by moderate\n' | tee -a ../outside-notes.txt >/dev/null`, which failed with `Operation not permitted`; the model then stated it attempted escalation and it failed with `Protected launch refused.`
2. Step 2, 06:55:17Z: `I retried with an explicit request to run outside the sandbox, but it was still rejected by protected launch before your approval could take effect:` followed by `Protected launch refused: this command would run outside the OS sandbox, which protected launch does not allow.` and `No line was appended.`
3. Step 2 resubmission, same wording, 06:57:23Z: `Tried again with an explicit outside-sandbox approval request, but the launch was still refused before execution:` followed by `Protected launch refused: this command would run outside the OS sandbox, which protected launch does not allow.` and `The line was not appended.`

Each above wording is copied from the visible screen. No unchanged wording was submitted more than twice (initial plus one labeled resubmission).

### Expected-result outcomes

- **The review lists the protections removed at the next start — passed.** `permissive-review-top` listed Sandbox, Protected files, Untrusted content, and Model keys as removed at next start.
- **“Saved: Permissive” says this session stays Moderate — passed.** `permissive-saved` said exactly that; since `r` was unavailable, it also explained `/quit` and starting again from the shell. Whether “restart now” should be offered is the named limit and was recorded, not judged.
- **Before quitting, the write still fails — blocked.** The required pre-quit MOD-07 repeat was not executable: the model declined step 1 and both step-2 submissions, no approval prompt appeared, and no command was run. The outside file remained unchanged, but the expected command-failure behavior was not observed. Under G-1, this does not count as passed.
- **After relaunch, Permissive is active — passed.** `postrelaunch-security` showed Active and Core Permissive.
- **The approved write succeeds after relaunch — not exercised.** The product model refused both step-2 attempts before showing any approval prompt, so option 1 could not be approved and option 2 could not be selected.
- **Person’s own approval, sandbox and network settings match baseline — passed.** Baseline and post-relaunch inspector rows had the same wording: approvals on-request; sandbox writes to current folder with approved/allow-listed outside-sandbox execution; network off inside sandbox/on outside.
- **The turn completes — passed.** Product replied exactly `pong` with no broker/security error.

### Overall

MOD-22 is **blocked**: the required pre-quit and post-relaunch MOD-07 step-2 approval path could not be executed because the product model declined/refused before any approval prompt. The completed observations above are preserved.

VERDICT MOD-22: blocked
