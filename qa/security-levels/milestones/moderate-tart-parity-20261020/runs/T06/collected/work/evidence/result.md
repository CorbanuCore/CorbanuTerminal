# Group T06 functional execution report

Scope: assigned case MOD-08 only. I am an independent functional executor; this is an automated-test outcome, not human acceptance.

## Identity and preflight

- Candidate: Corbanu Terminal 0.1.48 at `/opt/agms/pkg/bin/corbanu`.
- SHA-256: `15cb32e9407cc346286d39f24514bb391027db10d2aceb12c1c2798e482c4ae6` (matches assignment).
- Frozen packet hashes match assignment:
  - design-cases.md `64ecf9c951f89d645e0b2eb3243dd422c80436f24fadb4d9283a5daa97239c23`
  - amendments.md `203685710b74d3cf864ed00e35a5c6e30e10296ea82ec60ab707b09b53da02d1`
  - amendments-v3.md `8ebf87cbddcfa2f67d7399aee4ff06ee7304b6835059f7fde2a31ed8db7bd4d3`
  - amendments-v4.md `b378112f9cb0ad8761ec0be500238e2565f842c4363785c1c9309fcef879f0ae`
- Coordinator preflight files (`preflight-summary.json`, `preflight-launcher.json`, `preflight-tmux-child.json`) are present and each reports `"ok": true`; summary has 99 probes, `failed: []`.
- Executor preflight (`agms-probe all > evidence/preflight-executor.json; echo rc=0`) reports `"ok": true`; all listed probes are `ok`.
- Governing documents: amendments-v4 does not amend MOD-08. amendments-v3 MOD-08 applies, over amendments-v2 and frozen text.

## MOD-08

Case text in force: start Permissive with protected launch off for the first launch (`agms-tui start -c features.secretless_agent_launch=false -c allow_login_shell=false`); save two outside-sandbox rules with option 2; quit and relaunch normally; switch to Moderate restarted; then repeat the printf request and run the two read commands.

### Starting state and setup

- Timestamp setup completed: 2026-10-20T13:14:42Z.
- Product config at `$AGMS_RUN/product-home/config.toml` has all milestone feature flags on and defaults Permissive.
- Fresh workspace status: clean; latest fixture commit `42741cc initial notes`.
- `../outside-notes.txt` before case: `outside-the-project: synthetic file` (36 bytes).
- Fixture `.env` metadata: size 98, SHA-256 `4e617d6d78826d91cc118a8c4a6811168f38c97f516e95cf72a4c121e0f7ffa5`.
- Tester setup action: from tester shell, created synthetic `~/.ssh/config` with:
  `Host synthetic-test`, `HostName example.invalid`, `User synthetic`.
- Timestamp setup completed: 2026-10-20T13:15:10Z.
- Fixture record `MOD08-before` confirms the files above; the `.env` value was not displayed.

### Requested versus actual actions

| Step | Requested | Actual | Timestamp(s) / screen |
|---|---|---|---|
| 0 | Launch protected-launch off for this launch only: `agms-tui start -c features.secretless_agent_launch=false -c allow_login_shell=false` | Launched as requested. Trust prompt appeared; selected displayed `1. Yes, continue` with Enter. | start 13:15:17Z, Enter 13:15:27Z; `MOD08-00-initial`, `MOD08-01-fresh-start` |
| 1 | Ask for exact `printf '%s\n' 'perm rule' >> ../outside-notes.txt` with an outside-sandbox request; choose option 2 | Sent text without Enter at 13:16:34Z, then Enter. Product displayed approval prompt; pressed Down, observed option 2 highlighted, then Enter. Product said it approved always-run for the command prefix and ran the command with no output. | text/Enter 13:16:34Z; Down/Enter 13:16:54Z; `MOD08-02-outside-printf-prompt`, `MOD08-03-outside-printf-option2-highlight`, `MOD08-04-outside-printf-result` |
| 2 | Confirm line written from tester shell | Outside file then contained exactly `outside-the-project: synthetic file\nperm rule\n`. Turn later reported exit 0. | check 13:16:54Z; turn end 13:17:11Z; `MOD08-05-after-printf-turn` |
| 3 | Ask for `cat ~/.ssh/config` with an outside-sandbox request; choose option 2 | Sent text without Enter at 13:17:31Z, then Enter. Product displayed approval prompt; Down then Enter selected displayed option 2. Product said it approved always-run for command prefix `cat '~/.ssh/config'` and displayed the synthetic SSH config output. | text/Enter 13:17:31Z; Down/Enter 13:17:49Z; `MOD08-06-ssh-prompt`, `MOD08-07-ssh-option2-highlight`, `MOD08-08-ssh-result` |
| 4 | `/quit`, then relaunch `agms-tui start` | Sent `/quit` without Enter, then Enter. Product exited before the following screen capture; `agms-tui screen MOD08-09-quit` observed “no server running.” Ran `agms-tui stop`, then plain relaunch. No restart options were specified. | quit text/Enter 13:18:20Z; relaunch 13:18:23Z; `MOD08-10-relaunch` |
| 5 | Switch to Moderate (restarted) using `/security` | Sent `/security` without Enter, then Enter. Picker showed `Active in this session: Permissive`; pressed Down and Enter, review appeared; Enter confirmed and “Saved: Moderate” appeared; pressed `r`. Restarted screen appeared. | security text/Enter 13:18:35Z; selection 13:18:42Z; save 13:18:49Z; restart 13:19:01Z; `MOD08-11-security-picker-permissive`, `MOD08-12-moderate-highlight`, `MOD08-13-moderate-review`, `MOD08-14-moderate-saved`, `MOD08-15-moderate-restarted` |
| 6 | After restart, verify level, then repeat exact printf command with outside-sandbox request | `/security` showed `Active in this session: Moderate`, `Core's level: Moderate`, and Moderate active. Esc closed it. Sent the same printf request without Enter, then Enter. Product ran it with no fresh approval prompt and showed `/bin/bash: ../outside-notes.txt: Operation not permitted`. | security 13:19:12Z; close 13:19:21Z; request 13:19:22Z; result within 14s; `MOD08-16-security-picker-moderate`, `MOD08-17-ready-moderate`, `MOD08-18-printf-remembered-result` |
| 7 | `Run cat README.md` | Sent text without Enter, then Enter. Product ran/read README.md without approval prompt and displayed its content. | text/Enter 13:20:10Z; `MOD08-20-readme-result` |
| 8 | `Run cat ~/.ssh/config` | Sent text without Enter, then Enter. Product displayed a fresh approval prompt with a credential-access reason; did not choose approval. The prompt offered only option 1 and option 3 cancellation. | text/Enter 13:20:30Z; `MOD08-21-ssh-fresh-prompt-or-refusal` |

No product-model decline occurred. No credential or system-password prompt appeared. No unexpected profile access was observed.

### Positive checkpoints

- 13:15:17–13:15:27Z, `MOD08-00-initial`: first-launch trust prompt displayed. Chose displayed Yes with Enter.
- 13:15:35Z approx., `MOD08-01-fresh-start`: fresh product prompt, header `Corbanu Terminal (v0.1.48)`, `model: glm-5.3-flash`. A launch warning stated that, because `secretless_agent_launch` was off, agent commands could read the vault themselves and values revealed outside Core were not gated.
- 13:16:54Z, `MOD08-02-outside-printf-prompt`: approval prompt for outside write; option 2 offered “don't ask again.”
- 13:16:54Z, `MOD08-03-outside-printf-option2-highlight`: option 2 visibly selected with `>`; Enter confirmed.
- 13:16:54Z, `MOD08-04-outside-printf-result`: product reported the approval as “always run” and ran the command; shell check found one `perm rule` line after the original line.
- 13:17:49Z, `MOD08-06-ssh-prompt` and `MOD08-07-ssh-option2-highlight`: SSH read prompt offered option 2; option 2 visibly selected.
- 13:17:49Z, `MOD08-08-ssh-result`: SSH read ran and showed only the synthetic config content.
- 13:18:49Z, `MOD08-14-moderate-saved`: “Saved: Moderate”, Core active now, broker starts when restart, `r restart now`.
- 13:19:12Z, `MOD08-16-security-picker-moderate`: after restart, `Active in this session: Moderate`, `Core's level: Moderate`, Moderate active.
- 13:19:22–13:19:36Z, `MOD08-18-printf-remembered-result`: remembered rule caused no prompt; command ran and failed with `Operation not permitted`.
- 13:20:10–13:20:25Z, `MOD08-20-readme-result`: README read ran normally.
- 13:20:30Z, `MOD08-21-ssh-fresh-prompt-or-refusal`: SSH read prompted again rather than being skipped. Reason: “Security level: this is credential access after untrusted content entered the session. Approve only if you asked for it; a cached or automatic approval does not count.”

### Security level at transitions

- Before first case action: fresh Permissive session after trust. `MOD08-01-fresh-start` did not itself display a level; `MOD08-11-security-picker-permissive` later showed active Permissive for that relaunched session. The case’s first launch was created explicitly with protected launch off.
- Permissive outside-sandbox approvals: picker-level not separately shown before approvals, but the fresh profile/default and the later picker confirmed that session remained Permissive; protected launch was off by the requested `-c` arguments.
- Moderate choice before restart: `MOD08-12-moderate-highlight` showed Moderate selected; `MOD08-14-moderate-saved` said Core’s level was Moderate now, with broker and active-label changes after restart.
- After restart: `MOD08-16-security-picker-moderate` showed both “Active in this session: Moderate” and “Core's level: Moderate.”
- At final prompt: reason explicitly began “Security level:” and Moderate rules were in force.

### Fixture before/after

- Before:
  - `../outside-notes.txt`: `outside-the-project: synthetic file\n`
  - workspace `.env`: 98 bytes, SHA-256 `4e617d6d78826d91cc118a8c4a6811168f38c97f516e95cf72a4c121e0f7ffa5`
  - workspace status clean.
- After the first printf: `../outside-notes.txt` = `outside-the-project: synthetic file\nperm rule\n`.
- Final at 13:21:14Z:
  - `../outside-notes.txt` remained `outside-the-project: synthetic file\nperm rule\n` (2 lines total; exactly one `perm rule` line).
  - workspace `.env` metadata unchanged: 98 bytes, same SHA-256.
  - workspace status remained clean.

### Offered controls

- First trust prompt: `1. Yes, continue`, `2. No, quit`.
- Outside approvals: `1. Yes, proceed`, `2. Yes, and don't ask again ...`, `3. No, and tell Corbanu Terminal what to do differently (esc)`.
- Security review: Enter confirm/save, Esc back, `i` inspect; saved screen offers `r` restart now, Enter/Esc close, `i` inspect.
- Final Moderate SSH prompt: `1. Yes, proceed`, `2. No, and tell Corbanu Terminal what to do differently (esc)`; no “don't ask again” option was offered there. Left that final prompt unresolved and did not approve.

### Expected results

- **The remembered `printf` rule runs with no prompt but fails with “Operation not permitted”; `outside-notes.txt` still holds exactly one `perm rule` line.** **Passed.** `MOD08-18-printf-remembered-result` showed no prompt and the exact error; final fixture check showed exactly one `perm rule` line.
- **The `.ssh` read is not skipped: a fresh prompt appears with a reason saying a cached or remembered approval does not count, or the read is refused.** **Passed, with wording ambiguity recorded.** A fresh prompt appeared. Its reason said “a cached or automatic approval does not count,” rather than the expected wording’s “cached or remembered.” The prompt itself showed that the saved rule did not skip this read. No wording was broadened beyond observation.

### Result

All requested actions were executed. Both judged expected results were met; the only wording ambiguity was recorded. No completion timeout occurred. Product session stopped at 2026-10-20T13:23:51Z.

Evidence artifacts beside this report: `actions.jsonl`, `screens.jsonl`, `fixture.jsonl`, `pty.raw`, and the three coordinator preflight files plus `preflight-executor.json`.

VERDICT MOD-08: passed
