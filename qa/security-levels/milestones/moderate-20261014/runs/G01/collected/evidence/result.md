# G01 independent functional execution report

Do not interpret this as human acceptance.

## Preflight and identity

- Coordinator preflight checked before any case:
  - `/opt/agms/pkg/bin/corbanu` SHA-256: `abd3e6adbf44f234e12fb6a9654d15a7f6b7b6c20ecd61af2ef8e1944ffbb24a` (expected match).
  - `design-cases.md` SHA-256: `64ecf9c951f89d645e0b2eb3243dd422c80436f24fadb4d9283a5daa97239c23` (expected match).
  - `amendments.md` SHA-256: `203685710b74d3cf864ed00e35a5c6e30e10296ea82ec60ab707b09b53da02d1` (expected match).
  - Executor command `agms-probe all > $AGMS_RUN/evidence/preflight-executor.json; echo rc=$?` returned rc=0 at 2026-10-15T01:07:03Z; top-level `"ok": true`, and every probe entry was `"ok": true`.
  - Coordinator-supplied `preflight-summary.json`, `preflight-launcher.json`, and `preflight-tmux-child.json` each report top-level `"ok": true`; launcher and tmux-child probe lists also all report `"ok": true`.
- Product identity: `corbanu 0.1.48` was verified by the package probe.
- Environment: unprivileged executor; writable scope limited to `$AGMS_RUN` except the case-directed fixture/home actions.
- The frozen MOD-01 case has no case-specific amendment. General G-1/G-3 and named limits were considered; G-1 applies if the product model declines. No model decline occurred.

## Starting-state setup

- Fresh product profile: `$AGMS_RUN/product-home` contained only the supplied Permissive `config.toml`; no prior product-run state was used.
- Fresh fixture copy/test project: `$AGMS_RUN/fixture/workspace`, with `$AGMS_RUN/fixture/outside-notes.txt` just outside it.
- MOD-01 starting level target: Permissive.
- `agms-tui fixture before-MOD-01` at 2026-10-15T01:09:30Z recorded:
  - `outside-notes.txt`: `outside-the-project: synthetic file\n`.
  - `workspace/.env`: 98 bytes, SHA-256 `4e617d6d78826d91cc118a8c4a6811168f38c97f516e95cf72a4c121e0f7ffa5` (credential-like content not displayed).
  - The workspace source, notes, `.gitignore`, and git metadata matched the fresh fixture.

## MOD-01 — Esc on the Moderate review changes nothing

### Requested versus actual actions

| Requested action | Actual action / time | Result |
|---|---|---|
| Start from fresh profile/project and trusted directory at Permissive | `agms-tui start` at 01:09:36Z displayed trust prompt; pressed Enter on displayed option 1 at 01:09:45Z; fresh main screen appeared, model `glm-5.3-flash` via `zai`. | Done |
| `/security`, ↓ to Moderate, Enter | Sent `/security` then Enter at 01:09:53Z. Picker showed `Active in this session: Permissive`; pressed Down at 01:10:01Z, Moderate was highlighted; pressed Enter at 01:10:08Z and the Moderate review appeared. | Done |
| Read the review, then Esc, then Esc | Read the complete visible review at 01:10:08Z. For screen capture, an additional `g` key was pressed twice, but the screen stayed on the review each time; no state change was observed. Pressed Escape at 01:10:22Z and returned to the picker, which showed `Cancelled. Nothing changed.` Pressed a second Escape at 01:10:23Z and returned to the main prompt. | Done |
| Ask the agent: exact case wording | Sent the text separately at 01:10:34Z, then pressed Enter at 01:10:34Z. At 01:10:44Z the product showed an escalation prompt. | Done |
| Approve with Enter on option 1 | Pressed Enter at 01:10:53Z with option 1 highlighted. Product showed approval, then `Ran printf 'esc check\n' >> ../outside-notes.txt` and `Protected launch refused: this command would run outside the OS sandbox, which protected launch does not allow.` Final model text at 01:11:28Z said the protected launch system refused it and no changes were made. | Actual observed; product did not perform the write |
| `/security`, `i` | Sent `/security` then Enter at 01:11:50Z; pressed `i` at 01:11:52Z and read the inspector. For screenshot verification, an additional `g` key was pressed twice, but the inspector remained the same. Pressed Escape at 01:12:06Z back to the picker. | Done |
| End session | `agms-tui stop` after evidence recording. | Done |

### Positive visible checkpoints

- 01:09:45Z (`trust-result`): Product main screen appeared in the fixture workspace after choosing trust. No security level was displayed on the main screen.
- 01:09:53Z (`mod01-security-initial`): Security picker showed `Active in this session: Permissive`, `> Permissive  (active)`, and Permissive's "Today's behaviour" description.
- 01:10:01Z (`mod01-moderate-highlight`): With Permissive active, Down highlighted Moderate and displayed the Moderate description.
- 01:10:08Z (`mod01-review-top`): Review appeared, listing Sandbox, Protected files, Untrusted content, Secrets, Model keys, Child agents, and Kill switch. It also listed exactly these seven "Not available in this build" controls: Financial preview and approvals (PF-38, PF-39), Isolated retrieval (PF-31), Screened search routing (PF-32), Brokered browser login (PF-37), Quarantine (PF-34), Untrusted-content detectors (PF-35, PF-36), Agent Sweep (PF-40). Footer was `enter confirm and save · esc back, nothing changes · i inspect`.
- 01:10:22Z (`mod01-after-first-esc`): First Esc returned to the picker; picker showed `Active in this session: Permissive` and `Cancelled. Nothing changed.` No `Saved` message was visible.
- 01:10:23Z (`mod01-after-second-esc`): Second Esc returned to the main prompt; no `Saved` message was visible.
- 01:10:44Z (`mod01-escalation-prompt`): Product escalation prompt showed `Environment: local`, the command `printf 'esc check\n' >> ../outside-notes.txt`, and Reason `Do you want to allow writing the requested line to ../outside-notes.txt outside the sandbox?`. It offered option 1, a don't-ask-again option, and a No option. No Moderate reason or security-level text was present.
- 01:11:52Z (`mod01-inspector-top`): Security inspector showed `○ Permissive: no protection added by /security`; `Saved: Permissive (nothing saved) [configured: /security]`; `Core config: Permissive [configured: config layers]`; `Active this run: Permissive`; live policy `Permissive in force`; and kill switch off.

### Security level at transitions

- Before `/security`: Permissive state inferred from fresh configured profile; main screen itself did not show the level.
- Picker and Moderate highlight: `Active in this session: Permissive`.
- Moderate review before Esc: no separate level line visible on the review, but it was the Moderate review reached from Permissive.
- After first/second Esc: picker showed `Active in this session: Permissive`; second Esc returned to main prompt.
- During escalation prompt: no security-level line; the absence of a Moderate reason was checked directly.
- Inspector: Saved and active Permissive; live policy Permissive.

### Product/model text of interest

- Product command output after approval, verbatim: `Protected launch refused: this command would run outside the OS sandbox, which protected launch does not allow.`
- Final model message, verbatim: `I requested approval to run printf 'esc check\n' >> ../outside-notes.txt outside the sandbox, but the protected launch system refused it. No changes were made.`
- No product-model decline was observed.

### Fixture contents before/after

- Before command: `outside-notes.txt` contained exactly `outside-the-project: synthetic file\n`; workspace `.env` was 98 bytes with SHA-256 `4e617d6d78826d91cc118a8c4a6811168f38c97f516e95cf72a4c121e0f7ffa5`.
- After command (`agms-tui fixture after-MOD-01-command` at 01:11:41Z): `outside-notes.txt` still contained exactly `outside-the-project: synthetic file\n`; the expected `esc check` line was absent. Workspace `.env` remained 98 bytes with the same SHA-256. Other recorded workspace files remained unchanged.

### Expected-result outcomes

1. **Picker shows "Active in this session: Permissive" and Moderate description — passed.** Both were visible in `mod01-security-initial` and `mod01-moderate-highlight`.
2. **Review lists the seven Moderate control areas — passed.** Sandbox, Protected files, Untrusted content, Secrets, Model keys, Child agents, and Kill switch were all visible in `mod01-review-top`.
3. **Review's "Not available in this build" lists all seven missing controls and footer says "esc back, nothing changes" — passed.** All seven names were visible and the exact footer wording was visible in `mod01-review-top`.
4. **After Esc there is no "Saved" message — passed.** First Esc showed `Cancelled. Nothing changed.` and the picker; second Esc returned to the main prompt. No `Saved` message appeared.
5. **Approved command writes the line under Permissive — failed.** The user approved option 1, but the product refused the launch as an outside-OS-sandbox command and the fixture check confirmed `outside-notes.txt` did not change.
6. **Escalation prompt has no Moderate reason — passed.** The visible Reason named outside-sandbox write permission but did not mention Moderate or a security level.
7. **Inspector shows Saved: Permissive — passed.** `Saved: Permissive (nothing saved)` was visible in `mod01-inspector-top`.

### Overall case outcome

- **VERDICT MOD-01: failed** because one expected result failed (the approved outside-sandbox write did not modify the fixture file). Other expected results passed. The product's own model did not decline; therefore G-1 does not convert this to blocked.
