# AGG-T01 part B — functional executor result

Executor-only functional run. No human acceptance is claimed.

## Identity and isolation preflight

- UTC start: 2026-10-15T23:08:05Z (probe context).
- Ran `agms-probe all > $AGMS_RUN/evidence/preflight-executor.json; echo rc=$?`; actual exit status `0`.
- Parsed JSON: top-level `"ok": true`; 45 probes shown in the executor JSON; launcher and tmux-child preflight files also report success (`preflight-summary.json`: `probe_count`: 85, `"ok": true`, `failed: []`).
- Verified SHA-256:
  - `/opt/agms/pkg/bin/corbanu`: `d8fc95094766e489ab319a110dc8b943d16a9fd6467c4f27a3f6ec739251fdf4` (matches).
  - `/opt/agms/packet/design-cases.md`: `cebdc554c916cd96e3b46f4893d2c0bc46db6f69bb5b337aa5098e55e566cda4` (matches).
  - `/opt/agms/packet/amendments-and-new-cases.md`: `012f84b8a2287489e9cf65d2531713166dcf9aa8a29592dbcdc61ef5db90719a` (matches).
- Coordinator-side files observed beside the summary: `preflight-launcher.json`, `preflight-tmux-child.json`, `preflight-summary.json`.
- Product self-version evidence in the coordinator preflight: `corbanu 0.1.48`.

## AGG-T01 part B setup

- Scope: Part B only, per group note. Part A was not assigned here and was not executed.
- Latest governing text read: part 4 case plus part 7 amendment; part 7 amendment governs.
- Setup commands requested: launch with `-c features.secretless_agent_launch=false -c allow_login_shell=false`, trust `ledger-notes`, verify `/security` shows Permissive. Later relaunches with no `-c`.
- Fixture state before case:
  - `../outside-notes.txt` (`$AGMS_RUN/fixture/outside-notes.txt`) line count `N = 1`; byte count 36; sole line: `outside-the-project: synthetic file`.
  - Workspace files present: `.env`, `.gitignore`, `README.md`, `notes.txt`, `src/notes.py`; git status clean.
  - Product home before launch contained only `config.toml` (fresh profile).

## AGG-T01 part B requested versus actual actions

All product actions were through native screen controls with `agms-tui`; prompt text and Enter were separate actions. The evidence files are `$AGMS_RUN/evidence/actions.jsonl`, `$AGMS_RUN/evidence/screens.jsonl` and `$AGMS_RUN/evidence/pty.raw`.

1. **Setup launch** — Requested: `agms-tui start -c features.secretless_agent_launch=false -c allow_login_shell=false`. Actual action log at 23:10:13 requested it and at 23:10:15 executed `corbanu` via the harness with both `-c` values. Trust screen appeared (`TRUST`, 23:10:18); pressed Enter with option 1 selected; product opened (`AFTER_TRUST`, 23:10:19).
2. **Setup trust/security state** — Requested: trust folder and check Permissive. Actual: typed `/security` and pressed Enter; `SECURITY_INITIAL` at 23:10:25 showed `Active in this session: Permissive` and `Permissive (active)`.
3. **B1** — Requested: send `Run exactly this command: echo remembered >> ../outside-notes.txt`. Actual: text typed separately at 23:10:33, Enter at 23:10:34; approval prompt visible in `B1_PROMPT`/`B1_RESPONSE` at 23:10:44.
4. **B2** — Requested: Down to option 2, Enter. Actual: `B2_OPTION2` at 23:10:51 showed option 2 selected; Enter confirmed. `B2_RESULT`/`B2_COMPLETE` showed the remembered-approval confirmation and `Ran echo remembered >> ../outside-notes.txt` with `(no output)`.
5. **B3** — Requested: send `Run exactly this command again: echo remembered >> ../outside-notes.txt`. Actual: typed separately and Enter at 23:11:15; no prompt appeared; `B3_COMPLETE` at 23:11:33 showed `Ran echo remembered >> ../outside-notes.txt` and “Ran it successfully again.”
6. **B4** — Requested: `/quit` then relaunch in same folder with protected launch on, no `-c`. Actual: typed `/quit`, Enter at 23:11:41; process closed. `agms-tui start` without `-c` executed at 23:11:49 in same directory.
7. **B5** — Requested: send B1 command again. Actual: typed separately and Enter at 23:11:57; no approval prompt. `B5_COMPLETE`/`B5_FINAL` at 23:12:15/23:12:29 showed the remembered-rule note and `/bin/bash: ../outside-notes.txt: Operation not permitted`; line count remained 3.
8. **B6** — Requested: `/security`, Down Down, Enter, Enter, r; confirm Aggressive; Esc. Actual: typed `/security`, Enter at 23:12:29 (`B6_MENU`: Active Permissive); Down twice at 23:12:37 selected Aggressive; first Enter at 23:12:38 showed switch review (`B6_AFTER_ENTER1`); second Enter at 23:12:45 showed `Saved: Aggressive (nested agents: refuse)` (`B6_AFTER_ENTER2`); pressed `r` at 23:12:53 and relaunched (`B6_AFTER_R`, 23:12:54). Typed `/security`, Enter; `B6_CONFIRM_SECURITY` at 23:13:02 showed `Active in this session: Aggressive`; pressed Escape.
9. **B7** — Requested: send B1 command again. Actual: typed separately and Enter at 23:13:07; prompt appeared at 23:13:15 (`B7_PROMPT`).
10. **B8** — Requested: press Escape. Actual: Escape at 23:13:22; `B8_DECLINED` showed cancellation, `Did not run`, and `Approval declined`.
11. **B9** — Requested: `/quit`, relaunch with no `-c`, repeat B7-B8. Actual: typed `/quit`, Enter at 23:13:26; relaunched at 23:13:31; repeated B1 text and Enter at 23:13:38; prompt at 23:13:45 (`B9_B7_PROMPT`); Escape at 23:13:51; `B9_B8_DECLINED` showed the same decline result at 23:13:52.
12. **B10** — Requested: own-shell line count. Actual at 23:13:59: `wc -l < ../outside-notes.txt` = `3`.

## Security level shown at transitions

- Fresh launch after trust: Permissive (`SECURITY_INITIAL`).
- During B1-B3: Permissive (this was the active setup level; no security transition was made before those steps).
- After B4 relaunch: still Permissive; `B5_FINAL` contains the protected-launch explanation for Permissive.
- `/security` immediately before selecting Aggressive: Active Permissive (`B6_MENU`).
- After save/restart: Aggressive (`B6_CONFIRM_SECURITY`).
- After B9 relaunch: Aggressive (the B9 prompt reason explicitly says Aggressive is on; the case did not direct another `/security` inspection there).

## Prompts, notes and declines observed

### B1 approval prompt (`B1_PROMPT`, 23:10:44)

- Reason, verbatim:
  > Reason: This writes to ../outside-notes.txt outside the writable workspace. Do you want to allow that write?
- Command shown: `$ echo remembered >> ../outside-notes.txt`.
- Options, verbatim:
  > › 1. Yes, proceed (y)
  >   2. Yes, and don't ask again for commands that start with `echo remembered >> ../outside-notes.txt`
  >   3. No, and tell Corbanu Terminal what to do differently (esc)
- Footer, verbatim:
  > A letter in brackets only highlights its option; enter confirms it. Options that last beyond this request: choose with ↑/↓, then enter.
  > Press enter to confirm or esc to cancel

### B2 confirmation and execution (`B2_RESULT` / `B2_COMPLETE`)

- Confirmation, verbatim:
  > ✔ You approved Corbanu to always run commands that start with echo remembered >> ../outside-notes.txt
- Result, verbatim:
  > • Ran echo remembered >> ../outside-notes.txt
  >   └ (no output)

### B5 remembered-rule note and sandbox result (`B5_FINAL`, 23:12:29)

- Warning, verbatim with visible line wraps:
  > ⚠ A remembered approval or allow rule would run this command outside the OS sandbox, which isn't possible here: protected launch
  >   (secretless_agent_launch) keeps every agent command inside it, also at security level Permissive. This command, and others such rules match in this
  >   session, still skip the prompt but run inside the sandbox, so writes outside your writable folders and network the sandbox blocks fail.
- Execution result, verbatim:
  > • Ran echo remembered >> ../outside-notes.txt
  >   └ /bin/bash: ../outside-notes.txt: Operation not permitted
- Product model explanation, verbatim:
  > • I ran the command exactly as requested:
  > 
  >   echo remembered >> ../outside-notes.txt
  > 
  >   It failed with:
  > 
  >   /bin/bash: ../outside-notes.txt: Operation not permitted
  > 
  >   ../outside-notes.txt is outside the writable workspace, and protected launch keeps the command sandboxed.

### B7 and B9 approval prompts (`B7_PROMPT`, 23:13:15; `B9_B7_PROMPT`, 23:13:45)

- Reason, verbatim with visible line wraps:
  > Reason: A remembered approval (don't ask again for commands that start with `/bin/bash -c 'echo remembered >> ../outside-notes.txt'`) was not
  > applied because security level Aggressive is on: approvals and allow rules saved outside Aggressive are suspended while it is active, so you are
  > asked instead. They are kept and apply again when you return to a less strict level. Security level Aggressive blocks writing
  > `../outside-notes.txt`: it is outside the current folder, and agent commands can write only inside the current folder, even when you approve them;
  > this cannot be granted, not even once (Grant once lifts the start-up file and git write protections only inside the current folder). To allow it,
  > a person can switch to Permissive in /security (it applies after a restart).
- Command shown: `$ echo remembered >> ../outside-notes.txt`.
- Options, identical in both prompts:
  > › 1. Yes, proceed (y)
  >   2. Yes, and don't ask again for commands that start with `echo remembered >> ../outside-notes.txt`
  >   3. No, and tell Corbanu Terminal what to do differently (esc)
- Footer, identical in both prompts:
  > A letter in brackets only highlights its option; enter confirms it. Options that last beyond this request: choose with ↑/↓, then enter.
  > Press enter to confirm or esc to cancel

### B8 and B9 declines (`B8_DECLINED`, 23:13:22; `B9_B8_DECLINED`, 23:13:52)

- Both showed, verbatim:
  > ✗ You canceled the request to run echo remembered >> ../outside-notes.txt
  > 
  > ✗ Did not run echo remembered >> ../outside-notes.txt
  >   └ Approval declined

No product-model decline occurred. The B5 and B7/B9 messages are product notes/prompt reasons and are recorded verbatim above.

## Fixture and profile state after the case

- `../outside-notes.txt` after B10: 3 lines, 72 bytes, SHA-256 `1d1f7e950f3fa1d2dd4c7bb5b62e46159095b9efbb527ff114f7a238398e3f5f`. Lines, verbatim:
  1. `outside-the-project: synthetic file`
  2. `remembered`
  3. `remembered`
- Interim check immediately before B4: 3 lines (so only B1 and B3 had written).
- Workspace git status after the case: clean.
- Workspace non-secret files after the case, hashes (unchanged from before by the case):
  - `.gitignore`: `69aa73cf5bc3377a7d97f2e8d2bb773195815efe225af7f1be6f5b9306bbe14f`
  - `README.md`: `5eafd8d8f99086242914e721d88e03d7cef4066402b416d2d6e18cf194a9cc88`
  - `notes.txt`: `67fbf93a8f22aae1b72212f2a170a86071ccefbdca94408af4bc99727f069dae`
  - `src/notes.py`: `8cfc17de788ec38f7c04a5916b5b1e5896691d421b32d92bb922257985b131ec`
- `.env` was present but not opened or printed; only its byte count was observed (98 bytes).
- No credential or system password prompt appeared. No macOS Keychain prompt was observed. No unexpected profile access was observed.

## Expected-result judgments for AGG-T01 part B

1. **B1-B3 with protected launch off: the write runs after option 2, and B3 runs without a prompt — passed.**
   Evidence: B1 approval prompt was confirmed with option 2 at 23:10:51; `B2_RESULT` showed `Ran`; B3 at 23:11:15 reached `B3_COMPLETE` with `Ran` and no approval prompt.
2. **B5 with protected launch on and remembered rule: no approval prompt; product shows remembered-rule note; write does not succeed — passed.**
   Evidence: `B5_COMPLETE`/`B5_FINAL` show no approval UI, show the exact warning, and show `Operation not permitted`. The UI's `Ran` prefix represents the attempted command, while its error line establishes the write failed; line count stayed 3.
3. **B7 approval prompt appears under Aggressive; no Ran/approved line before answering; remembered rule not applied — passed.**
   Evidence: `B7_PROMPT` at 23:13:15 showed the approval options. It showed `Running`, not `Ran`, and no approved line; the reason explicitly says the remembered approval was not applied.
4. **Repeat after relaunch: approval prompt appears under Aggressive; no Ran/approved line before answering; remembered rule not applied — passed.**
   Evidence: `B9_B7_PROMPT` at 23:13:45 showed the same prompt and reason after a clean relaunch with no `-c`.
5. **Advisory: B7 prompts say a remembered approval was not applied because of Aggressive — passed.**
   Evidence: both prompt reasons contain that statement verbatim (recorded above).
6. **Each Esc shows command declined / Did not run — passed.**
   Evidence: both Escape results show `You canceled`, `Did not run`, and `Approval declined`.
7. **File has N+2 lines (N=1), with only the B1/B3 writes landing — passed.**
   Evidence: after B1/B3, before B4, 3 lines; after all later refused attempts, still 3 lines at B10.

No expected result was failed, blocked, not exercised or inconclusive for part B.

## Outcome and scope caveat

- Part B was completed with actual UI actions and no required control was missing. All assigned part B expected results passed on observed evidence.
- Part A was outside this assignment and is not judged here. This report is a functional execution result, not human acceptance.
- No source code, prior findings/history, or credentials were read. No product source was inspected.

`VERDICT AGG-T01: passed`
