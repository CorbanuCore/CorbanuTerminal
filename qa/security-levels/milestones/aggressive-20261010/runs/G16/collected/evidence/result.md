# G16 functional execution report

Executor role: independent functional executor. This is execution evidence only; no human acceptance is claimed.

## Preflight and identity

- Executor preflight command: `agms-probe all > $AGMS_RUN/evidence/preflight-executor.json; echo rc=$?` (requested and executed). Actual observed final output: `rc=0`.
- `preflight-executor.json`: `"ok": true`, all listed probes ok.
- Coordinator `preflight-summary.json`: `"ok": true`, `launcher_ok: true`, `tmux_child_ok: true`, `pty_input_echoed: true`, `failed: []`, `probe_count: 85`.
- `preflight-launcher.json` and `preflight-tmux-child.json`: both `"ok": true`.
- Candidate SHA-256: `/opt/agms/pkg/bin/corbanu` = `181ad0299d238accbf936512531c90988e8d15cb02a50ea8241f8566b2372a4a` (expected and observed).
- Frozen cases SHA-256: `/opt/agms/packet/design-cases.md` = `cebdc554c916cd96e3b46f4893d2c0bc46db6f69bb5b337aa5098e55e566cda4` (expected and observed).
- Candidate version observed by coordinator preflight: `corbanu 0.1.48`.
- Run root: `/Users/agmx3/runs/agms-G16-agmx3-20261011T030344Z`.
- Fixture listing recorded before product interaction, label `pre-start`, at 2026-10-11T03:06:23Z. Notable synthetic fixture contents: `workspace/.env` contains `NOTES_SYNC_TOKEN=agms-canary-7f3a9c-synthetic-token`; `outside-notes.txt` contains `outside-the-project: synthetic file`; workspace files README.md, src/notes.py, notes.txt, .gitignore and git metadata were listed.

## Case AGG-26

### Requested actions (from frozen case)

1. Starting conditions: Aggressive active; Permissive inspector “This session” rows recorded as the baseline.
2. `/security`, Up Up to Permissive, Enter, read the review.
3. Esc, then repeat the selection and press Enter.
4. Restart.
5. `/security`, then inspect.

### Expected results under test

- The review lists every Aggressive protection, including nested agents and protected credential, home and startup paths, and says it applies at the next start.
- Esc changes nothing.
- After restart every row matches the baseline.
- `cat .env` and `curl` behave as in Permissive.

### Execution log

Times are UTC as observed at execution.
- 2026-10-11T03:07:34Z: Requested `agms-tui start`; actual launch showed the folder trust prompt and directory `.../fixture/workspace`.
- 2026-10-11T03:07:41Z: Pressed `Enter` at displayed trust prompt option 1. Actual startup screen label `A01-startup` showed Corbanu Terminal v0.1.48, model glm-5.3-flash, and route `zai default`.
- 2026-10-11T03:07:50-51Z: Sent `/security` as text and then `Enter` separately. Actual picker label `A02-security-picker-permissive` showed “Active in this session: Permissive”; Permissive was highlighted and active.
- 2026-10-11T03:08:02-03Z: Pressed `i` at Permissive. Actual inspector label `A03-inspector-permissive-top` showed Saved/Core/Active/Live Permissive and the following “This session” baseline rows: Sandbox (current folder, /private/tmp, run tmp; approved or allow-listed outside); Approvals on-request; Network off inside sandbox, on outside, web search cached; Vault readable to agent commands and secret-like env variables passed through; Model keys read by Corbanu (broker_model_auth off); Child agents get this session's values; Sandbox backend macOS Seatbelt; Egress proxy destination policy on.
- 2026-10-11T03:08:11-13Z: Pressed `Escape`, then `Down` twice. Picker label `A04-picker-aggressive-highlight` showed Permissive active and Aggressive highlighted.
- 2026-10-11T03:08:18-33Z: Pressed `Enter`, then scrolled with Down. Review labels `A05-aggressive-review-top` and `A05b-aggressive-review-bottom` listed Aggressive sandbox, approvals, network, vault, model keys, child agents, nested agents refuse, controls ready 6/6, credential files denied after restart (workspace/.env), Corbanu history/state denied after restart (sessions, 4 databases), and the statement that the rows take effect when Corbanu Terminal restarts.
- 2026-10-11T03:08:38-39Z: Pressed `Enter` to confirm. Actual label `A07-aggressive-saved` showed “Saved: Aggressive (nested agents: refuse)” and “Core's level is Aggressive now in this session, and from every start.”
- 2026-10-11T03:08:47-50Z: Pressed displayed `r restart now`. Actual label `A09-restarted-aggressive` showed restart message and a fresh Corbanu Terminal v0.1.48 startup in the same workspace.

### Case actions in Aggressive starting state
- 2026-10-11T03:09:16-17Z: Requested `/security`; sent `/security` as text and `Enter` separately. Actual picker label `B01-security-aggressive-active`: “Active in this session: Aggressive” and “Core's level: Aggressive”; Aggressive highlighted and active.
- 2026-10-11T03:09:22-25Z: Pressed `Up` twice and captured picker `B02-permissive-highlight` (Permissive highlighted; Aggressive active). Pressed `Enter`. Actual review label `B03-downgrade-review` showed “Return to Permissive?” and “Protections removed at the next start:” with bullets for Sandbox, Approvals, Network, Vault, Model keys, and Child agents. It also said “Core's level stays Aggressive in this session and becomes Permissive from the next start,” “The Aggressive vault rule file is removed at the next start,” and “Takes effect when you restart Corbanu Terminal; this session stays Aggressive until then.” Downward scrolling produced `B03b-downgrade-review-scrolled`; it did not reveal additional review rows. The visible review did **not** list nested agents, protected credential files, protected Corbanu home/history/startup paths, the Aggressive preflight/controls status, or the workspace `.env` path. Thus the first expected result was not met.
- 2026-10-11T03:09:43-45Z: Pressed `Escape`. Actual picker `B04-picker-after-esc` still showed Permissive highlighted, Aggressive active, and explicitly said “Cancelled. Nothing changed.” Positive checkpoint: Esc changed nothing.
- 2026-10-11T03:09:48-50Z: Pressed `Enter` to repeat the highlighted Permissive selection. Actual review `B05-downgrade-review-repeat` displayed the same visible content as B03.
- 2026-10-11T03:09:57-58Z: Pressed `Enter` to confirm. Actual saved screen `B06-permissive-saved`: “Saved: Permissive”; “Restart Corbanu Terminal to activate Permissive. Until then this session stays Aggressive”; grants and for-session approvals ended. The screen offered `r restart now`.
- 2026-10-11T03:10:02-06Z: Pressed the displayed `r restart now`. Actual screen `B07-after-restart` showed a Corbanu Terminal credential/API key prompt instead of the expected restarted Permissive UI. Its visible text included “Welcome to Corbanu Terminal”, “Use your Z.AI API key”, “Paste your ZAI_API_KEY API key. Corbanu stores it in the encrypted vault and never adds it to chat. An existing environment variable takes precedence; unset it outside Corbanu to use the saved key.”, an “API key” input box, “Press enter to save”, and “Press esc to go back”. This is a credential prompt, so execution was stopped per the run instructions; no text or key was entered.
- 2026-10-11T03:10:20Z: Ran `agms-tui stop` and ended the product terminal. Fixture label `after-credential-block` showed the same synthetic `.env` content as before; no workspace fixture change was observed.

### Expected-result outcomes

- Review lists every Aggressive protection, including nested agents and protected credential, home and startup paths, and says it applies at the next start: **failed** — screens B03/B03b listed only six broad protections and did not visibly list nested agents, protected credential files, protected home/history/startup paths, controls readiness, or `.env`; the wording said protections were removed at next start rather than that Aggressive applies at the next start.
- Esc changes nothing: **passed** — B04 explicitly showed “Cancelled. Nothing changed” and retained Aggressive active.
- After restart every row matches the Permissive baseline: **blocked** — restart displayed a ZAI_API_KEY credential prompt, so the post-restart inspector could not be opened.
- `cat .env` and `curl` behave as in Permissive: **not exercised** — blocked before reaching the restarted product and before any model prompts or commands were sent.

### Final case observation

AGG-26 was stopped on a credential/API key prompt at restart. No credential was supplied. One observed expected result failed before the stop; the remaining restart-inspection and behavior checks were blocked/not exercised. This is execution evidence, not human acceptance.

VERDICT AGG-26: failed
