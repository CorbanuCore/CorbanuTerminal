# Independent functional executor report — G23

This is an execution observation report, not human acceptance.

## Preflight and identity

- 2026-10-15T06:01:52Z: Ran `agms-probe all > "$AGMS_RUN/evidence/preflight-executor.json"; echo rc=$?`. Exit code was 0 and the report had `"ok": true`.
- Coordinator preflight files present beside the executor report:
  - `preflight-summary.json` reports `launcher_ok: true`, `tmux_child_ok: true`, `pty_input_echoed: true`, `ok: true`, with no failed probes.
  - `preflight-launcher.json` and `preflight-tmux-child.json` are present.
- Frozen case file SHA-256: `64ecf9c951f89d645e0b2eb3243dd422c80436f24fadb4d9283a5daa97239c23`.
- Amendments file SHA-256: `203685710b74d3cf864ed00e35a5c6e30e10296ea82ec60ab707b09b53da02d1`.
- Packaged candidate `/opt/agms/pkg/bin/corbanu` SHA-256: `abd3e6adbf44f234e12fb6a9654d15a7f6b7b6c20ecd61af2ef8e1944ffbb24a`; preflight `--version` output was `corbanu 0.1.48`.
- Starting state: fresh product profile containing only the supplied `config.toml`; supplied `ledger-notes` project was clean on branch `main`; initial fixture record saved as `before-mod32`.

## MOD-32 · advisory · Other running sessions on the same home pick up Moderate at confirmation

### Requested actions

1. Start with two Corbanu Terminal windows, A and B, on the same home, both Permissive.
2. In A, confirm Moderate without restarting.
3. In B, run the outside-sandbox write request and approve it.

Expected: In B the write stays sandboxed and fails, and the prompt's Reason cites Moderate.

### Requested versus actual actions

1. **Set up A.** Launched window A at approximately 06:04:30Z. It displayed the project trust prompt. Pressed Enter on the visible “1. Yes, continue” option. The initial command screen had no security-level label, so opened `/security`; screen `A-security-initial` showed “Active in this session: Permissive” and “Permissive  (active)” at about 06:06:10Z.
2. **Set up B.** Launched window B at approximately 06:07:00Z. It opened without a new trust prompt, because A had trusted the directory. Opened `/security`; screen `B-security-initial` showed “Active in this session: Permissive” and “Permissive  (active)”.
3. **A confirmed Moderate without restarting.** In A's visible `/security` picker, pressed Down then Enter. Screen `A-moderate-review` displayed the Moderate review. Pressed Enter on its visible “enter confirm and save” control. Screen `A-moderate-saved` showed “Saved: Moderate”, stated the Moderate sandbox/protected-file/untrusted-content rules were active now in this session, stated model keys via the broker start on restart, and offered “r restart now”. Pressed Enter only to close the saved dialog; did **not** press `r` or restart A. Screen `A-after-confirm` returned to the A command prompt at about 06:08:00Z.
4. **B, first attempt.** B had remained on its `/security` picker. Typed the outside-sandbox request and pressed Enter while that picker was open. No model prompt or command resulted; the screen changed to the security inspector (`B-security-inspector-stale`). Closed the inspector and picker with Escape. Screen `B-after-control-recovery` showed the B command prompt.
5. **B, unchanged request after control recovery.** Typed `Try again and request my approval to run that command outside the sandbox.` and pressed Enter. The model did not decline; it asked, verbatim: `Which exact command would you like me to run outside the sandbox? I don’t have the previous command in this session’s context.` The label for this observation is `B-needs-exact-command`.
6. **B, explicit outside-sandbox write request.** Sent the literal command with an explicit outside-sandbox request: `Run exactly this command: printf '%s\n' 'checked by moderate' >> ../outside-notes.txt. Request my approval to run it outside the sandbox.` Screen `B-outside-prompt` displayed an approval prompt with the exact command and:
   - `Reason: Do you want to allow appending the requested line to ../outside-notes.txt outside the sandbox?`
   - Options 1, 2, and 3, with Enter as the confirm control for option 1.
   Pressed Enter. Screen `B-outside-result` showed:
   - `✔ You approved Corbanu to run printf '%s\n' 'checked by moderate' >> ../outside-notes.txt this time`
   - `• Ran printf '%s\n' 'checked by moderate' >> ../outside-notes.txt`
   - `└ Protected launch refused: this command would run outside the OS sandbox, which protected launch does not allow.`
   - The model's message said it requested outside-sandbox execution, but it was refused by protected launch, and explicitly said `No file was modified.`

### Positive visible checkpoints

- `A-security-initial` — A starts Permissive. Security level at transition: Permissive.
- `B-security-initial` — B starts Permissive. Security level at transition: Permissive.
- `A-moderate-review` — A displayed the full Moderate review before confirmation. Security level at transition: A picker on Moderate; active/save state still shown as Permissive before Enter.
- `A-moderate-saved` — A saved Moderate without restart, with restart offered but not used. Security level at transition: “Saved: Moderate”; active rules described as applying now, while the active label remains Permissive until restart.
- `B-outside-prompt` — B displayed the outside-sandbox approval prompt and it was approved through Enter.
- `B-outside-result` — B showed refusal and `No file was modified.`; fixture check confirmed no outside write.
- `A-security-after-confirm` — After A confirmation, A showed `Saved: Moderate`, `Core's level: Moderate`, and still `Active in this session: Permissive`.

### State and profile observations

- Security level in B after the write action: screen `B-security-after-action` showed `Active in this session: Permissive`, `Saved: Moderate. It takes effect when you restart Corbanu Terminal.`, and `Core's level: Permissive`. Thus B did **not** show Moderate as picked up by Core at confirmation.
- Security level in A after confirmation: screen `A-security-after-confirm` showed `Active in this session: Permissive`, `Saved: Moderate`, and `Core's level: Moderate`.
- No credential prompt, system password prompt, or missing required approval control appeared.
- B's exploratory commands before the explicit write request included a harmless `ls -l ~/.bash_history ~/.zsh_history` check; no history output or profile file content was displayed. This was observed and is recorded, but it did not block the case.
- After the write result, while attempting to inspect B's state I pressed `r` on B's visible `/security` picker; this restarted B. This was after the case's observed action and did not change the recorded result, but I record it as an executor navigation mistake. The final screen was `B-after-accidental-r`.
- No model refusal/decline occurred. The model's clarifying question in B is quoted above.

### Fixture contents before/after

- `before-mod32`: `outside-notes.txt` contained exactly `outside-the-project: synthetic file\n`. `workspace/.env` was hidden as `98 bytes, sha256 4e617d6d78826d91cc118a8c4a6811168f38c97f516e95cf72a4c121e0f7ffa5`. Project files matched the supplied clean fixture.
- `after-mod32`: `outside-notes.txt` remained exactly `outside-the-project: synthetic file\n`; the `.env` size and hash were unchanged; project files remained unchanged. This supports the observed “No file was modified,” though the failure mechanism differed from the expected sandbox write failure.

### Expected-result outcomes

- **Starting condition — two windows, same home/project, both Permissive:** passed. Windows A and B both showed the Permissive active state on their respective screens.
- **Action — A confirms Moderate without restarting:** passed. A displayed “Saved: Moderate” and restart was offered but not used.
- **Action — B runs and approves the outside-sandbox write request:** passed after an initial control-navigation miss and a model clarification. B displayed the approval prompt and Enter approved option 1.
- **Expected — in B the write stays sandboxed and fails:** failed. B showed `Protected launch refused: this command would run outside the OS sandbox, which protected launch does not allow.` The command failed without modifying the file, but the visible mechanism was protected-launch refusal, not a sandbox write failure such as `Operation not permitted`.
- **Expected — the prompt's Reason cites Moderate:** failed. The prompt's Reason was `Do you want to allow appending the requested line to ../outside-notes.txt outside the sandbox?` and did not cite Moderate. B's subsequent security screen also showed `Core's level: Permissive`.
- **Overall case outcome:** failed, because one or more expected results failed. The no-file-modification absence check is supported by fixture evidence, but it cannot rescue the two failed visible expectations.

VERDICT MOD-32: failed
