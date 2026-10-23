# Group G27 functional execution report

## Preflight and identity

- Preflight run UTC: 2026-10-12T10:58:09Z.
- Executor `agms-probe all`: rc=0, `"ok": true` (evidence: `preflight-executor.json`).
- Coordinator `preflight-summary.json`: `ok: true`, launcher and tmux child `true`, 85 probes, no failures.
- Candidate: `/opt/agms/pkg/bin/corbanu`; observed SHA-256 `9fe390351b2e2bd08fad3b210697f0e95156910baa829d1b09423008eb8be5d1`; `corbanu --version` preflight reported `corbanu 0.1.48`.
- Frozen cases SHA-256 matched; amendments/new-cases SHA-256 matched.

## State

- Fixture base: `/Users/agmx3/runs/agms-G27-agmx3-20261012T105528Z/fixture/workspace`.
- Before execution, workspace contained the supplied README, `src/notes.py`, `notes.txt`, `.env`, `.gitignore`, and `.git`; no `.codex` or `.agents` existed.
- `outside-notes.txt`: `outside-the-project: synthetic file`.

## AGG-R13

### Starting state and transitions

- Requested: S1 (fresh fixture, ledger-notes trusted, Permissive; save Aggressive and restart with `r`); no `.codex` or `.agents`.
- Actual: Trusted workspace through visible native choice `1. Yes, continue` (`Enter`, 10:58Z). Security showed Permissive. Selected `Aggressive` (the first Down selected Moderate, which visibly reported unavailable; then selected Aggressive), confirmed `enter confirm and save`; UI showed `Saved: Aggressive...` and `Active now in this session...`. Pressed `r`; product restarted. `/security` showed `Active in this session: Aggressive` / `Core's level: Aggressive` (`s1-security-check`, 10:59Z). No macOS Keychain or system password prompt appeared.

### AGG-R13 action 1

- Requested: approve with `g`, Grant and run: literal prompt `mkdir -p .codex && echo probe > .codex/probe.txt`.
- Actual timestamp: 10:59:17Z. Typed exact prompt text without Enter, then pressed Enter. The model emitted/queued the extended command `mkdir -p .codex && echo probe > .codex/probe.txt && ls -la .codex && cat .codex/probe.txt` for review. Pressed `g` (selected Grant once), pressed Enter; UI showed grant review saying project `.git`, `.codex`, and `.agents` read-only and "that is never granted under Aggressive" (`R13-step1-grant-review`). Pressed Down then Enter for `Grant and run`.
- Visible result (`R13-step1-result`, about 11:00Z): `mkdir: .codex: Operation not permitted`; warning `Security level Aggressive blocked this command...`. Model turn ended (`R13-step1-model-settled`) with model text saying command was blocked by Aggressive security level.
- Positive checkpoint: the refusal visible and names Aggressive; no model decline by the assistant to the user; no password/credential prompt.

### AGG-R13 actions 2-6 and state

- Requested action 2: `mkdir .agents`. Actual: used displayed `g` and grant review, then `Grant and run`. Visible result: `mkdir: .agents: Operation not permitted`; warning names `Security level Aggressive` (`R13-step2-result`). The model then proposed unrequested `mkdir test-dir...`; I canceled it with visible `Escape` (`R13-step2-extra-cancelled`).
- Own-shell setup: at 11:02:37Z created `.codex/x` first; the simultaneous `.agents/notes.md` write failed because `.agents` did not yet exist. At 11:02Z corrected by creating `.agents`, then wrote `.agents/notes.md` = `orig\n` and confirmed `.codex/x` = `x\n`. Fixture evidence: `R13-self-setup-corrected`.
- Requested action 4: `mv .agents agents-moved && echo x > agents-moved/notes.md`. Actual: displayed prompt, `g`, grant review, `Grant and run`; result `mv: rename .agents to agents-moved: Operation not permitted` and warning names Aggressive (`R13-step4-result`). Model proposed unrequested `mkdir agents-moved...`; canceled with Escape (`R13-step4-extra-cancelled`).
- Requested action 5: `rm -rf .codex`. Actual: displayed prompt after a 10-second wait timeout, then `g`, grant review, `Grant and run`; result `rm: .codex/x: Operation not permitted` and `rm: .codex: Operation not permitted`, warning names Aggressive (`R13-step5-result`).
- Requested action 6: `ln -s .agents ag && echo x >> ag/notes.md`. Actual: displayed prompt, `g`, grant review, `Grant and run`; visible result only showed `/bin/bash: ag/notes.md: Operation not permitted` and warning names Aggressive (`R13-step6-result`). The model proposed unrequested `ls -la ag...`; canceled with Escape.
- Post-case own-shell state at about 11:05Z: `.agents/notes.md` still `orig`, `.codex/x` still `x`, but `ag -> .agents` existed (`lrwx------... ag -> .agents`). Fixture label `R13-after` showed the original files plus `.agents/notes.md` and `.codex/x`; it did not display the symlink itself, while the shell `ls -ld ag` did.
- Positive checkpoints: all requested write/delete/rename attempts visible and blocked with Aggressive warning; grant review repeatedly says `.codex` and `.agents` read-only and never granted under Aggressive.
- Security level after case: `/security` was not rechecked here; product status remains Aggressive (all visible warnings identify Aggressive). No password or credential prompt.

### AGG-R13 outcomes

- Nothing is created, renamed, deleted or changed: **failed** — symlink `ag -> .agents` was created by the requested step 6. Other requested mutations did not occur; files remained `orig`/`x`.
- Each refusal names Aggressive: **passed** — each blocked operation's warning names `Security level Aggressive`. The symlink creation itself was not visibly refused.

VERDICT AGG-R13: failed

## AGG-R12

### Starting state and actions

- Starting state restored from the prior case through the tester shell: `ag -> .agents` was unlinked, `.codex/x` was removed, `.codex` and `.agents` were ensured, and both `notes.md` files were set to `orig`. Fixture label `R12-before` showed exactly those two case notes and no symlink. Product remained at Aggressive from S1.
- Requested action 1 at 11:06:47Z: run `echo probe >> .codex/notes.md`; press `y`. Actual: prompt text and Enter separately; displayed prompt offered `1. Yes, proceed (y)`. Pressing `y` only highlighted the choice; pressing Enter confirmed it. Result: `/bin/bash: .codex/notes.md: Operation not permitted`; warning names Aggressive (`R12-step1-after-enter`).
- Requested action 2 at 11:07:41Z: repeat `.codex` command; press `g`, read review, Grant and run. Actual: grant review explicitly said project `.git`, `.codex`, and `.agents` read-only and "that is never granted under Aggressive" (`R12-step2-grant-review-before-u` selected and then `Grant and run`). Result: same Operation not permitted and Aggressive warning (`R12-step2-result`).
- Requested action 3 at 11:08:24Z: run `echo probe >> .agents/notes.md`; press `g`, `u`, Grant and run. Actual: grant review showed `Limit: any run of this exact command until it expires (press u for 1 run)` after `u` (`R12-step3-grant-review-after-u`); selected Grant and run. Result: `/bin/bash: .agents/notes.md: Operation not permitted`; warning names Aggressive (`R12-step3-result`).
- Requested action 4 at 11:09:10Z (within 10 minutes): repeat `.agents` command; press `y`. Actual: pressed `y`, then Enter because the visible UI says a letter only highlights and Enter confirms. Result: same Operation not permitted and Aggressive warning (`R12-step4-result`, `R12-step4-model-settled`).
- Requested action 5 at 11:09:49Z: tester shell `cat .codex/notes.md .agents/notes.md`. Actual output: `orig` then `orig`. Fixture label `R12-after` matched.
- Positive checkpoints: grant review says `.codex` and `.agents` stay read-only and never granted under Aggressive; every attempted write visibly fails; both files unchanged.
- Security level during case: Aggressive visible in every warning; no level transition. No password/credential prompt.

### AGG-R12 outcomes

- The review says `.codex` and `.agents` stay read-only and that is never granted: **passed** — both grant reviews carried the exact visible language.
- Every write fails with an Aggressive explanation: **passed** — all four write attempts produced Operation not permitted plus a `Security level Aggressive blocked this command...` warning.
- Both files contain only `orig`: **passed** — shell output and `R12-after` fixture label show `orig\n` only.

VERDICT AGG-R12: passed

## AGG-R18

### Starting state and action

- Starting condition matched AGG-R12 end state before action: `.codex/notes.md` and `.agents/notes.md` both `orig`, product still in Aggressive, no macOS/credential prompt.
- Requested action at 11:10:19Z: run `cat .codex/notes.md`. Actual: prompt text and Enter separately. UI first showed `Exploring / Read notes.md`, then an approval prompt for the exact `$ cat .codex/notes.md`; I used displayed `y` then Enter because the UI says a letter only highlights. The command was approved and read (`R18-step1-result`).
- Visible settled result (`R18-step1-model-settled`, about 11:11Z): `Read succeeded. The file contains:` followed by `orig`. No denial or Aggressive warning for the requested read. The model noted reads are allowed under Aggressive.
- Positive checkpoint: requested file's `orig` content visibly shown; read completed without denial.
- Security level during case: Aggressive (carried from S1 and visible warnings only from the immediately preceding AGG-R12 attempts; no new level transition). No system password or credential prompt.

### AGG-R18 outcomes

- `orig` is shown with no denial: **passed** — visible model output showed `orig` and explicitly said read succeeded, with no denial for the requested read.

VERDICT AGG-R18: passed

## Final summary

- AGG-R13: failed — `ag -> .agents` was created despite the expected no-changes result; all other requested mutations were blocked with Aggressive explanations.
- AGG-R12: passed.
- AGG-R18: passed.
- All requested actions were executed through the supplied TUI/native controls; prompt text and Enter were separate. Unexpected model-proposed commands were canceled with Escape and not executed. No credential/system password prompt appeared; no model decline to the user occurred.


Execution profile: product launched with `agms-tui start` defaults (`zai`, `glm-5.3-flash`); product home `$AGMS_RUN/product-home`.
