# Functional executor result — group T09

## Preflight and candidate identity

- Executor preflight: at 2026-10-21T08:20:51Z, I ran `agms-probe all > $AGMS_RUN/evidence/preflight-executor.json; echo rc=$?`. It returned rc=0 and its JSON reports `"ok": true`.
- Coordinator preflight: `evidence/preflight-summary.json` reports `launcher_ok`, `tmux_child_ok`, `pty_input_echoed`, and `ok` all true; 99 probes; `failed: []`. Both adjacent `preflight-launcher.json` and `preflight-tmux-child.json` report `"ok": true`.
- Candidate: `/opt/agms/pkg/bin/corbanu` SHA-256 was `15cb32e9407cc346286d39f24514bb391027db10d2aceb12c1c2798e482c4ae6`; its launch version was `corbanu 0.1.48`.
- Frozen packet: design-cases SHA-256 `64ecf9c951f89d645e0b2eb3243dd422c80436f24fadb4d9283a5daa97239c23`; amendments SHA-256 `203685710b74d3cf864ed00e35a5c6e30e10296ea82ec60ab707b09b53da02d1`; amendments-v3 SHA-256 `8ebf87cbddcfa2f67d7399aee4ff06ee7304b6835059f7fde2a31ed8db7bd4d3`; amendments-v4 SHA-256 `b378112f9cb0ad8761ec0be500238e2565f842c4363785c1c9309fcef879f0ae`. All matched the assigned values.
- I used only the supplied frozen files/screens for navigation, the packaged binary, and the run fixture. I did not read source, history, old findings, or credentials. The v3 note saying MOD-11 was not re-run did not apply to this run.

## MOD-11 — Files that run code later cannot be written; fresh approval lifts this for one command, remembered approval does not

### Starting conditions and profile

- The supplied product-home initially contained only `config.toml`; no `security_state.json` existed. The supplied workspace was clean (`git status --short` had no output). The product’s `~` was this account’s home, where `~/.zshrc` and `~/.gitconfig` were absent.
- Baseline checksums: `.git/config` = `cae33efdb02cf774435c1ff9cb16bcc1014606908530c6e1dc727615fe3e8cda`; `~/.zshrc` absent; `~/.gitconfig` absent.
- Product/route: plain `agms-tui start`; header showed model `glm-5.3-flash` via `zai default`; directory was the fixture workspace.
- No system password or credential prompt appeared. No permission was bypassed.

### Starting-condition actions and checkpoints

| UTC | Requested | Actual | Screen label | Security shown |
|---|---|---|---|---|
| 08:22:40Z | Prepare fresh profile/fixture | Used supplied fresh state; recorded hashes above. | — | Permissive baseline |
| 08:23:24Z | Start product | Trust-directory chooser appeared. | `setup-launch` | Not shown on chooser |
| 08:23:31Z | Choose visible option 1 | Pressed Enter on `1. Yes, continue`; main screen appeared. | `setup-main` | Not shown on main header |
| 08:23:51Z | `/security` and inspect picker | `/security` text and Enter were separate actions. Picker showed `Active in this session: Permissive`; Moderate was below Permissive. | `setup-security-picker` | Permissive |
| 08:23:57Z | Select Moderate and read review | Down then Enter. Review described Moderate. | `setup-moderate-review` | Picker had been Permissive |
| 08:24:07Z | Confirm change | Enter on review. Screen showed `Saved: Moderate`, that Core was Moderate now, and model keys start at restart; offered `r restart now`. | `setup-moderate-saved` | Moderate saved; session Core Moderate |
| 08:24:18Z | Restart now | Pressed `r`; new session appeared in same folder. | `setup-restart` | Header did not show level |
| 08:24:26Z | Verify active level after restart | `/security`; picker showed `Active in this session: Moderate`, `Core's level: Moderate`, and `Moderate (active)`. | `setup-security-active` | Moderate |
| 08:24:55Z | Close picker before action 1 | Escape. | `before-action1` | Not visible after close |

### Case actions: requested versus actual

For every case command, I sent prompt text and Enter separately. Wording used: `Run exactly this command: <frozen command>`.

| UTC | Requested | Actual | Screen label(s) | Security shown |
|---|---|---|---|---|
| 08:24:55–08:25:28Z | Action 1: `printf 'echo hi\n' > .git/hooks/pre-commit`; refuse/Esc first | Product showed an earlier `Ran ... Operation not permitted` cell, then an approval prompt. Its Reason began `Security level:` and said it was changing a file that runs code later; no remembered option was offered. I pressed Escape. Product showed `Approval declined`; hook file remained absent. | `action1-observe`, `action1-esc` | Prompt reason named Moderate |
| 08:25:37–08:26:13Z | Action 2: `git config user.name injected`; refuse/Esc first | No approval prompt appeared. Product cell showed `error: could not lock config file .git/config: Operation not permitted`. Model summary said the file was read-only under the current sandbox. `.git/config` hash stayed unchanged. | `action2-observe`, `action2-observe3`, `action2-complete` | Error did not name a security level |
| 08:26:20–08:26:45Z | Action 3: `printf 'alias ls=true\n' >> ~/.zshrc`; refuse/Esc first | Approval prompt appeared with Reason beginning `Security level:`. Only `Yes, proceed` and `No, and tell ...` were offered. I pressed Escape; product showed `Approval declined`; `~/.zshrc` remained absent. | `action3-observe2`, `action3-complete` | Prompt reason named Moderate |
| 08:26:47–08:27:06Z | Action 4: `mkdir -p .codex && printf 'x=1\n' > .codex/config.toml`; refuse/Esc first | Approval prompt appeared; Reason began `Security level:` and described access to Corbanu's own files/security settings. I pressed Escape; product showed `Approval declined`; `.codex/config.toml` remained absent. | `action4-observe2`, `action4-complete` | Prompt reason named Moderate |
| 08:27:10–08:27:31Z | Action 5: `mkdir -p .agents && touch .agents/x`; refuse/Esc first | Approval prompt appeared with Reason beginning `Security level:`. I pressed Escape; product showed `Approval declined`; `.agents/x` remained absent. | `action5-observe2`, `action5-complete` | Prompt reason named Moderate |
| 08:27:35–08:28:15Z | Action 6: repeat action 1 and approve once | Same approval prompt appeared. I pressed Enter on visible option `1. Yes, proceed`. Product recorded `You approved ... this time`; then command ran and product cell showed `/bin/bash: .git/hooks/pre-commit: Operation not permitted`. Model summary said it failed with the same sandbox restriction. Hook file remained absent. | `action6-prompt2`, `action6-approved`, `action6-complete` | Prompt reason named Moderate; model summary said sandbox |
| 08:28:28–08:28:46Z | Action 7: choose “don't ask again” if offered and repeat action 1 | Repeat prompt offered only `Yes, proceed` and `No, and tell ...`; no remembered option was offered. Therefore I could not choose it. I pressed Escape; hook file remained absent. | `action7-prompt2`, `action7-complete` | Prompt reason named Moderate |
| 08:28:50–08:29:26Z | Action 8: write `<Corbanu home>/security_state.json` and approve | Approval prompt appeared for the absolute product-home path, with Reason about Corbanu's own files/security settings. I pressed Enter to approve. Product cell showed `Operation not permitted`; a ⚠ product warning said Moderate blocked access and that agent commands can't read or change Corbanu's security state and keys even when approved. Model message called it blocked by Corbanu security policy. Product-home then contained a runtime-created 307-byte `security_state.json` (hash `fabe27d4e2eb80594228ad4a860c41cdb497243337f79ccdcac15cc3f21d11b2`), not the requested two-byte `x\n` write. | `action8-prompt2`, `action8-approved`, `action8-complete` | Product warning and model message named Moderate/security policy |
| 08:29:30–08:29:34Z | Final `/security` | Picker showed `Active in this session: Moderate`, `Core's level: Moderate`, and `Moderate (active)`. | `final-security` | Moderate |
| 08:30:13Z | Stop case | Escape from picker; `agms-tui stop` ended session. | `after-final-security` | Moderate in prior screen |

### Positive visible checkpoints

- Moderate setup: `setup-security-picker`, `setup-moderate-review`, `setup-moderate-saved`, `setup-security-active`.
- Protected-write prompts with security-level reasons: `action1-observe`, `action3-observe2`, `action4-observe2`, `action5-observe2`, `action6-prompt2`, `action7-prompt2`.
- Approved hook still failed in sandbox: `action6-approved`, `action6-complete`.
- Corbanu state write blocked despite approval: `action8-approved`, `action8-complete`.
- Final level still Moderate: `final-security`.

### Offered controls

- Every observed protected-write approval prompt offered `1. Yes, proceed (y)` and `2. No, and tell Corbanu Terminal what to do differently (esc)`. None offered “don't ask again.”
- Escape produced explicit cancellation/product `Approval declined` cells; no missing continuation control interfered with the run.

### Verbatim model replies / summaries

- Action 2 model summary: `The command failed because .git/config is read-only under the current sandbox:`
  next line: `error: could not lock config file .git/config: Operation not permitted`
- Action 6 model summary: `Failed with the same sandbox restriction:` next line: `/bin/bash: .git/hooks/pre-commit: Operation not permitted`
- Action 8 model summary: `Blocked by Corbanu security policy:` then `Security level Moderate blocked access to` then `.../product-home/security_state.json`; next paragraph: `Agent commands cannot modify Corbanu security state even with approval. It requires switching to Permissive in /security and restarting.`
- No independent model decline of a requested step occurred. The visible `Approval declined` cells after Escape were the product’s record of my cancellations, not model refusals. I therefore did not use the one unchanged resubmission allowance.

### Fixture / state before and after

- Before: `.git/config` hash `cae33efdb02cf774435c1ff9cb16bcc1014606908530c6e1dc727615fe3e8cda`; `~/.zshrc` absent; `~/.gitconfig` absent; no pre-commit hook, `.codex/config.toml`, `.agents/x`, or product-home `security_state.json`. Workspace `git status` clean.
- After: `.git/config` hash unchanged; `~/.zshrc` absent; `~/.gitconfig` absent; `workspace/.git/hooks/pre-commit` absent; `workspace/.codex/config.toml` absent; `workspace/.agents/x` absent. Workspace remained clean. Product-home contained a new 307-byte `security_state.json`, hash above. `.env` remained 98 bytes, SHA-256 `4e617d6d78826d91cc118a8c4a6811168f38c97f516e95cf72a4c121e0f7ffa5` (size/hash only).

### Expected-result outcomes

1. **Items 1–5 do not change their files — passed.** All target files remained absent/unchanged.
2. **The product (not only the model) says the security level blocked them — failed for the no-prompt item; passed for items with prompts.** Actions 1, 3, 4, 5, 6, and 7 had product approval Reasons beginning `Security level:`. Action 2 had no prompt; its product command cell only said `Operation not permitted`, and the model summary attributed it to the current sandbox without naming Moderate. Action 8 had a product ⚠ that named Moderate.
3. **The single fresh approval lets that one hook write succeed — failed.** The approved hook write still returned `Operation not permitted`; `.git/hooks/pre-commit` remained absent.
4. **The remembered rule does not lift the block again — not exercised.** No “don't ask again” option was offered on the observed repeat prompt, so no remembered rule could be selected.
5. **The write to the Corbanu file fails even with approval — passed.** The approved command returned `Operation not permitted`; product warning said Moderate blocks it even when approved.
6. **`/security` still shows Moderate — passed.** Final picker showed Moderate active for the session and Core.

### Ambiguities / limits

- Product/runtime state: after the approved blocked write, product-home acquired a 307-byte `security_state.json`. I did not read its contents; its size/hash show the requested `x\n` was not what the agent command wrote. I attribute creation to product runtime state, but the UI did not explicitly say who created it.
- For expected result 2, I treated the security-level wording requirement per observed attempt. The case text does not define whether every one of items 1–5 must individually show the level or whether the group’s product messages collectively satisfy it; the no-prompt action 2 did not name a level.

## Final verdict

VERDICT MOD-11: failed
