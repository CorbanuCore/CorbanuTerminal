# Functional execution report — group G11b

- Executor: independent functional executor. No human acceptance is claimed.
- Run start UTC: 2026-10-12T11:48:16Z.
- Product/package: Corbanu Terminal 0.1.48 at `/opt/agms/pkg/bin/corbanu`.
- Package SHA-256 observed: `9fe390351b2e2bd08fad3b210697f0e95156910baa829d1b09423008eb8be5d1` (expected match).
- Frozen cases SHA-256 observed: `cebdc554c916cd96e3b46f4893d2c0bc46db6f69bb5b337aa5098e55e566cda4` (expected match).
- Amendments/new cases SHA-256 observed: `ec4bf3ece913e36f2c24bd7786750b3f6af604644da7f1fcb479045b88a854c9` (expected match).
- Coordinator preflight summary: `launcher_ok=true`, `tmux_child_ok=true`, `pty_input_echoed=true`, `failed=[]`, `probe_count=85`, `ok=true`.
- Executor preflight: `agms-probe all` completed with rc=0 and `"ok": true`; output retained as `preflight-executor.json`.
- Scope: only assigned case AGG-33 (frozen case plus amendment). No source, history, old findings, or credentials were read. No `!` shell escape was used.

## Case AGG-33 (amended; decisions #442 (08:17:28Z))

### Requirement being tested

- Starting conditions: Aggressive active; at the grant review for protected write `echo '# expiry-probe' >> ~/.zshrc`.
- Actions: Press `u`, Grant and run; check `/security`, `g` before each run; run again at 2 minutes and at 11 minutes, approving each time.
- Expected: the 2-minute run is allowed and `~/.zshrc` gains a second `# expiry-probe` line; the 11-minute run is denied with no third line; the expired grant leaves the list.

### Requested versus actual actions and timeline

| Time (UTC) | Requested/available action | Actual action and observation |
|---|---|---|
| 11:49:39 | Start product through `agms-tui start` | Product launched in fixture workspace and showed trust prompt. |
| 11:49:49 | Neutral continuation | Screen S001 showed `1. Yes, continue` selected; pressed Enter. Home UI appeared (S002). |
| 11:49:55–11:49:58 | Submit case wording | Typed exact text, observed it in input (S003), then pressed Enter separately. |
| 11:51:32 | Starting condition: at grant review | At S004, a grant review was visible, but the session was Permissive; the screen also showed an earlier automatic attempt failed with `/bin/bash: /Users/agmx2/.zshrc: Operation not permitted`. This did not yet satisfy Aggressive active. |
| 11:51:55 | Press `u`, Grant and run | Pressed `u` (S005); no visible change. Since the displayed native controls offered arrow-key/Enter selection, pressed Down to option 2 (S006), then Enter (11:52:15). UI said approval succeeded for commands starting with the command, but the run was refused by protected launch because it would run outside the OS sandbox (S007). The model later reported no line was added. |
| 11:53:07–11:53:32 | Added `/security`, `g` check before a run | Opened `/security`. S011 showed `Active in this session: Permissive`. Pressed `g`; no visible change and `g` was not among the displayed controls on that screen. |
| 11:53:48–11:54:10 | Establish Aggressive active | Selected Aggressive in `/security`; confirmation screen showed the Aggressive policy and `Press r to restart now`; pressed Enter to save, then `r` to restart. |
| 11:54:39–11:54:47 | Added `/security`, `g` check before the run | S020 showed `Active in this session: Aggressive` and `Core's level: Aggressive`; displayed control `g grants and kill switch`. Pressed `g`; S021 showed `No grants are held now` and `Kill switch: off`. |
| 11:55:10–11:55:37 | Submit same case wording | Typed exact prompt text (S026), then pressed Enter separately. No grant review appeared. By 11:57:08, the UI showed the automatic command was blocked and the model offered a workspace alternative. |
| 11:57:23–11:57:24 | Added `/security`, `g` check before resubmission | `/security` showed Aggressive active (S028). Pressing `g` opened Grants and kill switch: no grants, kill switch off (S029). |
| 11:57:30–11:57:39 | Resubmit same case wording unchanged once | Typed the exact prompt (S030), then pressed Enter separately. Again no grant review appeared. The final screen (S032, by 11:59:26) showed the model declining. |
| 11:59:47 | Stop after required grant-review path remained unavailable | Verified `~/.zshrc` still did not exist; stopped the product with `agms-tui stop`. |

### Positive visible checkpoints

- 11:49:46, S001: Product showed the workspace trust prompt with native choice `1. Yes, continue`.
- 11:51:32, S004: A command approval/grant review was visible in the initial Permissive session for the exact command.
- 11:52:15, S007: Product visibly acknowledged: `✔ You approved Corbanu to always run commands that start with echo '# expiry-probe' >> ~/.zshrc`.
- 11:54:01, S015: Product saved Aggressive and displayed the restart control.
- 11:54:39, S020: `/security` showed `Active in this session: Aggressive` and `Core's level: Aggressive`.
- 11:54:47, S021: Grants screen was reachable with `g`, showed no grants and kill switch off.
- 11:57:24, S029: Before resubmission, `/security` and grants were checked again: Aggressive active, no grants, kill switch off.

### Security level at transitions

- Initial grant-review transition: `Active in this session: Permissive` when checked afterward at 11:53:07.
- After selecting and restarting: `Active in this session: Aggressive`, `Core's level: Aggressive`.
- Before resubmission: Aggressive active; grants list empty; kill switch off.
- No later security-level transition occurred.

### Fixture and target file state

- Fixture workspace was the supplied synthetic `ledger-notes` project and remained the product's starting directory. No case action targeted files inside it.
- Before the target-command test at 11:55:06, `$HOME/.zshrc` did not exist.
- After the blocked attempts at 11:59:47, `$HOME/.zshrc` still did not exist. Thus no `# expiry-probe` line was added.
- The initial blocked run occurred before the Aggressive restart and is retained as a partial observation; it did not create the file.

### Product/model blocks, refusals, and declines (verbatim visible text)

1. Initial Permissive-session protected-launch refusal (system-visible, 11:52:15):
   `Protected launch refused: this command would run outside the OS sandbox, which protected launch does not allow.`

   Model completion text (11:52:54):
   `I ran the exact command, but it was blocked:`  
   `- First attempt: /Users/agmx2/.zshrc: Operation not permitted`  
   `- Escalated attempt: rejected by protected launch policy`  
   `So # expiry-probe was not added to ~/.zshrc.`

2. First Aggressive-session block (11:57:08):
   System warning:
   `Security level Aggressive blocked this command: agent commands can write only inside this project, even when you approve them, and can never read protected files (such as .env and keychains), not even once. To lift these limits, choose another level with /security.`

   Model text:
   `Ran it exactly, but Security level Aggressive blocked the write to ~/.zshrc; agents may only write inside the project workspace.`  
   `To proceed, change security settings with /security, or tell me if you’d like the same line added to a file inside the workspace instead.`

3. Required once-unchanged resubmission, final model decline (11:59:26):
   `That exact command is blocked by Security level Aggressive because it writes outside the project (~/.zshrc). I can’t work around that.`  
   `Change security with /security, or ask me to append the line to a project-scoped file instead.`

### Ambiguities and missing controls

- The frozen/amended action says to press `u`, but at the displayed Permissive approval prompt (S004–S005), pressing `u` caused no visible selection or state change. The displayed controls said to use ↑/↓ and Enter; option 2 was therefore selected with Down and Enter. This partial observation occurred before Aggressive was active.
- On the Permissive `/security` screen, `g` was not displayed and pressing it did nothing. After Aggressive was active/restarted, `g` was displayed and opened grants.
- After Aggressive was active, the required grant review never appeared for the exact target command. The product automatically reported the command as blocked, so `u`, Grant and run could not be exercised.
- The amended case’s starting conditions and candidate behavior conflicted in this run: Aggressive active prevented the required protected-write grant review, while the initial Permissive grant review did not satisfy Aggressive active and its approved run was refused by protected launch.

### Expected-result outcomes

| Expected result | Outcome | Reason |
|---|---|---|
| The 2-minute run is allowed and `~/.zshrc` gains a second `# expiry-probe` line | blocked | The required Aggressive grant review never appeared; the command was automatically blocked, so no approved 2-minute run could be executed. `$HOME/.zshrc` remained nonexistent. |
| The 11-minute run is denied and there is no third line | blocked | The prerequisite initial granted run could not be established under the required Aggressive state, so the 11-minute run was not exercised. `$HOME/.zshrc` remained nonexistent. |
| The grant leaves the list once it expires | blocked | No grant was ever created in the required Aggressive session, so expiry and list removal could not be observed. |

### Case verdict

VERDICT AGG-33: blocked
