# Corbanu Terminal: the Aggressive security level

## Who and why

Corbanu Terminal is an AI terminal: a person chats with a coding/trading agent that can read files and run
commands on their computer. `/security` lets the person choose how locked down that agent is. Three levels:

- **Permissive**: today's behaviour, nothing added.
- **Moderate**: not available yet (shown as such).
- **Aggressive**: the lockdown posture. "Minimize sensitive access and require the user to open every
  important door." Sensitive access is denied by default; the person opens narrow, expiring exceptions.

The intended user is someone who handles secrets or money on this machine (API keys, wallets, private notes)
and wants the agent to be unable to reach them, or the network, without the person explicitly saying so.

## What a person who picks Aggressive should experience

1. **Choosing the level.** `/security` shows the level active now and the choices. Picking Aggressive shows
   the differences first, setting by setting ("now" versus "Aggressive"), plus a preflight check of the
   machine. Esc at any point changes nothing. Enter confirms and saves. Only the person, in the terminal
   UI, can change the level: the agent, a prompt, a project's files or config cannot raise, lower or switch it.
2. **When it applies.** Some protection starts the moment the person confirms; the full set starts at the
   next start of Corbanu Terminal (there is a "restart now" key). The level survives quitting and starting
   again. The screens say which part applies when.
3. **Protections that should be on** under Aggressive:
   - Agent commands run in an OS sandbox that can write only inside the current project folder.
   - The person approves every command that is not known to be read-only; nothing is auto-approved by a
     reviewer.
   - Agent commands have no network access; web search is off.
   - Secrets: the vault store and sign-in file are unreadable to agent commands; agent attempts to use the
     `corbanu vault` command are refused; secret-like environment variables are removed from commands;
     credential files (for example a project's `.env`, keychains) and Corbanu's own home, history and
     settings are unreadable to agent commands. Startup files that could plant persistence (shell startup
     files, git hooks/config) are protected.
   - Model keys: provider API keys are held by a separate credential broker that sends the model requests.
     If the broker cannot start, model requests are refused and never sent directly, and the message says
     which setting is involved and how to change it.
   - Agents started by the agent (nested `corbanu exec` and similar) are refused by default; child agents
     get the same restrictions and cannot weaken them.
   - Output shown to the person or the model should not contain managed secrets.
4. **Approvals and exceptions.** Approving a command runs it once, still inside the sandbox; an approval
   never opens the protected files. A separate "grant once" review lets exactly that one command run
   without the protected-file rules: it shows who/what/where, is limited (one run, or any run until it
   expires after 10 minutes) and Esc goes back with nothing granted. Typing ordinary text while an approval
   prompt is open must never approve it. Declining a command must show it as declined, not as having run.
5. **Grants and kill switch.** From `/security` (key `g`) the person can see grants held now, revoke all
   authority, and turn on a kill switch that keeps everything closed, including across restarts, until
   they turn it off there.
6. **Inspector.** From `/security` (key `i`) a read-only inspector shows what is actually enforced: the
   saved level, what was checked at launch, what is observed live, grants, recent denials, and anything
   degraded. It should never show a protection as active when it is not. Controls that do not exist yet
   read "not available"; things that run outside the sandbox read "not contained".
7. **Refusals are explained.** When something is refused, the person should be able to tell that the
   security level caused it, and what they can do (approve, grant once, or change the level).
8. **Errors and recovery.** If the preflight finds a problem, Aggressive cannot be saved and the screen says
   what to fix. If the protected boundary is not clean at start, a warning says so. The person can always
   return to Permissive: the review lists every protection that will be removed, it applies at the next
   start, and their own previous settings come back exactly.
9. **What does not change.** Model and provider choice, MCP servers, apps and hooks are unchanged (and run
   outside the sandbox). Every model route the product offers should keep working, including the two Z.AI
   routes (`zai` and `zai-anthropic`): a normal turn on either should complete.
10. **Outside Aggressive (existing behaviour that must still hold).** In Permissive, `/permissions` lets the
    person pick a permission level; choosing Full Access asks "Enable full access?" first, and cancelling
    that says so, so the person knows nothing changed.

## Constraints for this test run

- Platform: macOS on Apple silicon, the packaged Corbanu Terminal 0.1.48 candidate. Windows and Linux are
  not part of this run.
- The profile has the security features switched on (they are behind feature flags); the security level
  itself starts at Permissive and is chosen in `/security`. Moderate is not available.
- Testers drive the real terminal UI with real keys, in an isolated macOS virtual machine, as an ordinary
  unprivileged account with no real credentials. Each test group starts from a fresh profile.
- Model: GLM 5.3 Flash through the Z.AI `zai` route; the `zai-anthropic` route is also provisioned (GLM 5.3
  Flash, and GLM 5.2 where a case needs it). The test machine routes model traffic through a local relay;
  that relay's address (127.0.0.1:18443) can appear in command errors and is not part of the product.
- The product's own model may decline a request by itself. That is the model, not a protection; testers
  record the exact words.
- Test data: a small git project `ledger-notes` (README.md, `src/notes.py` with `list` / `add` commands,
  `notes.txt`, a `.env` holding a synthetic token `NOTES_SYNC_TOKEN=...`, a `.gitignore`), and a file
  `outside-notes.txt` just outside the project folder. All values are synthetic; there are no real accounts,
  wallets or money, and nothing may be bought, signed or sent.
- A macOS Keychain or other system password prompt must never appear.
- There is no target number of tests: prioritize what an ordinary Aggressive user does and the failures that
  would hurt them most.

## Screens from this candidate (`screens/`, PNG with a text copy)

| File | What it shows |
| --- | --- |
| 00-trust-folder | First start in the project: trust prompt |
| 01-fresh-start | Fresh session, Permissive |
| 02-security-picker-permissive | `/security` picker while Permissive |
| 03-inspector-permissive, 03b-…-scrolled | Inspector (`i`) while Permissive, top and scrolled |
| 04-picker-aggressive-highlight | Aggressive highlighted |
| 05-aggressive-review, 05b-…-bottom | Review of changes before confirming Aggressive (top, bottom) |
| 07-aggressive-saved | After Enter: saved, restart offered |
| 09-after-restart | After pressing `r` (restart now) |
| 10-picker-aggressive-active | `/security` after restart |
| 11-inspector-aggressive, 11b-…-scrolled | Inspector under Aggressive |
| 12-grants-killswitch | `g`: grants and kill switch |
| 13-killswitch-review, 13b-killswitch-esc | Kill-switch review, then Esc |
| 14-approval-prompt-aggressive | Command approval under Aggressive |
| 15-grant-review | "Grant once" review (option 4) |
| 16-approved-ran | After approving with Enter |
| 17-protected-read | Asking for `cat .env`: the model's answer |
| 18a-network-approval, 18b-…-denied | A network command: approval, then result |
| 19-vault-command | `corbanu vault list` requested |
| 20a-approval-after-untrusted, 20b-…-denied | Reading Corbanu's own settings: approval with a reason, then result |
| 21-downgrade-review | Choosing Permissive while Aggressive |
| 22-relaunch-aggressive | Quit and start again |
| 23-resume-list, 24-resume-attempt | `/resume` after the restart |
| E1-preflight-blocked | Error example: review with the preflight blocked (a profile launched with extra settings) |
| E2-startup-boundary-warning | Error example: startup warning that the protected boundary is not clean |
