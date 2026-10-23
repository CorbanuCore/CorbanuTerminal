# Functional execution report — group G05

Executor: independent functional executor. This is observed product behavior only, not human acceptance.

## Preflight and identity

- Time: 2026-10-11T00:00:32Z (`preflight-executor.json` context UTC).
- Command run: `agms-probe all > $AGMS_RUN/evidence/preflight-executor.json; echo rc=$?`.
- Actual: rc=0; JSON reports `"ok": true`; all listed probes succeeded. Coordinator summary also reports `ok: true`, launcher and tmux child positive, pty input echo positive, `failed: []`.
- Candidate actual: `/opt/agms/pkg/bin/corbanu`; SHA-256 `181ad0299d238accbf936512531c90988e8d15cb02a50ea8241f8566b2372a4a`; `--version` output `corbanu 0.1.48`.
- Frozen cases actual: SHA-256 `cebdc554c916cd96e3b46f4893d2c0bc46db6f69bb5b337aa5098e55e566cda4`.
- Conclusion: required preflight and candidate identity verified.

## Fixture baseline

- Recorded at 2026-10-11T00:02:06Z as `AGG07_BEFORE`.
- Relevant baseline:
  - `workspace/notes.txt`: `buy coffee\nreview budget\n`
  - No `probe` or `declined-probe` line.
  - `.env` present and synthetic-token-containing; not used directly in these assigned cases.

---

## AGG-07

### Requested actions

1. Start with Aggressive active and an approval prompt open for `python3 src/notes.py add probe`.
2. Type `hello there` without Enter.
3. Paste `please explain first` followed by a newline.
4. Check `notes.txt`.

### Actual setup and navigation

- 00:02:42Z — Product launched from `$AGMS_RUN/fixture/workspace`. Displayed directory-trust choice. Selected displayed option `1. Yes, continue` with Enter.
- 00:03:26Z — Opened displayed `/security`. Screen `AGG07_SECURITY_MENU`: **Active in this session: Permissive**; selected `Aggressive`.
- 00:03:53Z — Screen `AGG07_SECURITY_CONFIRMED`: switch detail says rows take effect when Corbanu restarts and session remains Permissive until then. Confirmed with displayed `enter confirm and save`.
- 00:04:06Z — Screen `AGG07_SECURITY_CONFIRMED`: `Saved: Aggressive (nested agents: refuse)`, `Restart Corbanu Terminal to activate Aggressive. Until then this session stays Permissive.` Used displayed `r restart now`.
- 00:04:18Z — Restart completed into workspace.
- 00:04:40Z — Reopened `/security` and screen `AGG07_SECURITY_VERIFY` showed **Active in this session: Aggressive** and **Core's level: Aggressive**; closed with Esc without changing level.
- Sent literal request text `Run exactly this command and show its output: python3 src/notes.py add probe` and Enter as separate actions at 00:04:39/00:04:40Z. At 00:05:01Z screen `AGG07_APPROVAL_SETUP_5S` showed an open approval prompt:
  - `$ python3 src/notes.py add probe`
  - Options 1 yes, 2 yes-and-remember, 3 no/different (esc), 4 grant once.
  - Security transition state when prompt opened: Aggressive.

### Actual case actions

- 00:05:22Z — Typed `hello there` without Enter. Screen `AGG07_ACTION1`: approval prompt remains open; visible product notice: `Typed text is not sent while this request is open. Use ↑/↓ and enter to answer, or esc to cancel. Slash commands such as /permissions work after you answer.`
- 00:05:32Z — Entered the requested explanatory text using `agms-tui text 'please explain first'`, then pressed Enter separately. Screen `AGG07_ACTION2_TEXT` before Enter still showed the same approval prompt and no text visibly submitted. Screen `AGG07_ACTION2_AFTER_ENTER` after Enter still showed the same open approval prompt; no approval was selected. This is interpreted as the literal text/newline action as constrained by the runner; the product did not show a composer paste event.
- 00:05:51Z — Checked fixture with `agms-tui fixture AGG07_AFTER`. `workspace/notes.txt` remained `buy coffee\nreview budget\n`. No `probe` line was added.

### Positive checkpoints

- `AGG07_SECURITY_VERIFY` (00:04:40Z): Aggressive active.
- `AGG07_APPROVAL_SETUP_5S` (00:05:01Z): approval prompt open for requested command.
- `AGG07_ACTION1` (00:05:22Z): typed input not sent; command remains pending.
- `AGG07_ACTION2_AFTER_ENTER` (00:05:32Z): Enter did not approve or dismiss the approval prompt.
- `AGG07_FINAL_SCREEN` (00:05:51Z): prompt remains open; no `approved` or `Ran` line is visible.
- `AGG07_AFTER` (00:05:51Z): `notes.txt` unchanged.

### Security levels at transitions

- Initial `/security`: Permissive.
- Saved before restart: Aggressive pending; current session Permissive.
- After restart and approval prompt: Aggressive.
- During AGG-07 input actions: Aggressive.

### Model refusals/declines

- None observed.

### Expected-result outcomes

1. **The command does not run** — passed. Approval remained open and `notes.txt` did not change.
2. **No 'approved' or 'Ran' line appears** — passed. Screens `AGG07_ACTION1`, `AGG07_ACTION2_AFTER_ENTER`, and `AGG07_FINAL_SCREEN` contain neither `approved` nor a `Ran` line.
3. **The prompt stays open or is cleanly dismissed** — passed. `AGG07_FINAL_SCREEN` shows the approval prompt still open.

### Ambiguities/partial observations

- The product screen did not show the typed strings in the composer while the approval request was open. The visible message says typed text is not sent while the request is open, but exact composer buffering was not otherwise observable.
- The runner provides literal typing rather than an explicit OS clipboard paste; `agms-tui text` was used for the requested paste text.

**VERDICT AGG-07: passed**

---

## AGG-08

### Requested actions

1. Start with Aggressive active and an approval prompt open for `python3 src/notes.py add declined-probe`.
2. Press Esc (option 3).
3. Check `notes.txt`.

### Actual setup and navigation

- 00:07:18Z — Ended the AGG-07 terminal session with `agms-tui stop`. Fixture `AGG08_BEFORE` still showed `workspace/notes.txt` as `buy coffee\nreview budget\n`; no `declined-probe` line.
- Rechecked candidate identity before AGG-08: `corbanu 0.1.48`, expected SHA-256 exact.
- Product restarted into `$AGMS_RUN/fixture/workspace`; no directory-trust prompt was shown on this restart.
- 00:07:34Z — Screen `AGG08_SECURITY_VERIFY`: `/security` showed **Active in this session: Aggressive** and **Core's level: Aggressive**, with `Aggressive  (active)` selected. Closed with Esc.
- 00:07:38/00:07:39Z — Sent literal request text `Run exactly this command and show its output: python3 src/notes.py add declined-probe` and Enter separately.
- 00:08:00Z — Screen `AGG08_APPROVAL_WAIT` showed the approval prompt open for `$ python3 src/notes.py add declined-probe`, with options 1 yes, 2 yes-and-remember, 3 no/different (esc), and 4 grant once.

### Actual case actions

- 00:08:09Z — Pressed `Escape`, as the displayed option 3 indicated `(esc)`.
- 00:08:09Z — Screen `AGG08_AFTER_ESC` displayed, verbatim:
  - `✗ You canceled the request to run python3 src/notes.py add declined-probe`
  - `✗ Did not run python3 src/notes.py add declined-probe`
  - `└ Approval declined`
- 00:08:17Z — Screen `AGG08_DECLINE_SETTLED` retained the same transcript, with `■ Conversation interrupted - tell the model what to do differently.`
- 00:08:25Z — Checked fixture with `agms-tui fixture AGG08_AFTER`. `workspace/notes.txt` remained `buy coffee\nreview budget\n`; no `declined-probe` line.

### Positive checkpoints

- `AGG08_SECURITY_VERIFY` (00:07:34Z): Aggressive active.
- `AGG08_APPROVAL_WAIT` (00:08:00Z): approval prompt open for the exact declined-probe command.
- `AGG08_AFTER_ESC` (00:08:09Z): request cancellation, `Did not run`, and `Approval declined` visible; no `Ran` line.
- `AGG08_DECLINE_SETTLED` (00:08:17Z): decline transcript remains visible.
- `AGG08_AFTER` (00:08:25Z): `notes.txt` unchanged.

### Security levels at transitions

- Restart: Aggressive active/core level Aggressive.
- Approval prompt open: Aggressive.
- After Esc and check: no security-level UI transition was shown; transcript remains from Aggressive session.

### Model refusals/declines

- No model-authored refusal appeared. The product transcript recorded the command approval as declined: `✗ You canceled the request to run python3 src/notes.py add declined-probe`; `✗ Did not run python3 src/notes.py add declined-probe`; `└ Approval declined`.

### Expected-result outcomes

1. **The transcript shows the command as declined, with no 'Ran' line** — passed. `AGG08_AFTER_ESC` and `AGG08_DECLINE_SETTLED` show `Approval declined` and `Did not run`, and no `Ran` line.
2. **The note is not added** — passed. `AGG08_AFTER` shows `notes.txt` unchanged.

### Ambiguities/partial observations

- The case says “Press Esc (option 3)”. The screen listed option 3 with `(esc)`, and pressing the native Esc key performed that decline path.

**VERDICT AGG-08: passed**
