# Independent functional executor report — group T04

- Run: `/Users/agmx1/runs/agms-T04-agmx1-20261020T123629Z`
- Date: 2026-10-20, GMT
- Assigned case: MOD-06
- Candidate: `/opt/agms/pkg/bin/corbanu`
- Candidate SHA-256 observed: `15cb32e9407cc346286d39f24514bb391027db10d2aceb12c1c2798e482c4ae6`
- Frozen case SHA-256 observed: `64ecf9c951f89d645e0b2eb3243dd422c80436f24fadb4d9283a5daa97239c23`
- Amendments SHA-256 observed: `203685710b74d3cf864ed00e35a5c6e30e10296ea82ec60ab707b09b53da02d1`
- Amendments-v3 SHA-256 observed: `8ebf87cbddcfa2f67d7399aee4ff06ee7304b6835059f7fde2a31ed8db7bd4d3`
- Amendments-v4 SHA-256 observed: `b378112f9cb0ad8761ec0be500238e2565f842c4363785c1c9309fcef879f0ae`
- Coordinator preflight `preflight-summary.json`: `ok: true` (launcher, tmux child, PTY echo; 99 probes)
- Executor `agms-probe all` completed 12:38:09Z with rc=0 and `ok: true`; output saved to `preflight-executor.json`.

This report records observed product behavior only. It does not claim human acceptance.

## MOD-06

Status complete.

### Starting conditions and fixture

Recorded before launch from executor shell. `notes.txt`: `buy coffee` / `review budget`. `outside-notes.txt`: `outside-the-project: synthetic file`. Git branch `main`, clean, at `42741cc initial notes`. Product config enables the milestone feature flags and does not choose a level.

### Product launch and baseline (Permissive)

- 12:40:31–12:40:33Z: launched via `agms-tui start`; recorded fixture `pre-baseline`. Screen `t04-06-00-start` showed the directory trust prompt (`1. Yes, continue` highlighted). At 12:40:43Z pressed Enter as displayed.
- 12:40:45Z: screen `t04-06-01-fresh-permissive` showed Corbanu Terminal v0.1.48, model `glm-5.3-flash`, workspace directory. No security level was visible on this screen; the subsequent picker confirmed Permissive.
- Baseline prompt 1 requested at 12:40:51Z: exact text `Run python3 src/notes.py list`; sent as prompt text, then Enter. No approval prompt appeared. At 12:41:14Z, `t04-06-02-baseline-list-result` showed the command ran with output `buy coffee` and `review budget`, and the model summarized both. No security refusal.
- Baseline prompt 2 requested at 12:41:22Z: exact text `Run python3 src/notes.py add 'call bank'`; sent as prompt text, then Enter. No approval prompt appeared. At 12:41:42Z, `t04-06-03-baseline-add-result` showed the command ran with `(no output)` and the model said `Added call bank.` No security refusal.
- Baseline prompt 3 requested at 12:42:53Z: exact amended text `Add a \`count\` command to src/notes.py that prints how many notes exist, then run it`; sent as prompt text, then Enter. No prompt appeared before the edit or before the edited code ran. At 12:43:20Z, `t04-06-05-baseline-edit-intermediate` showed it read `notes.py`, edited `src/notes.py` by adding a count branch, and remained Working. At 12:43:43Z, `t04-06-06-baseline-edit-run-result` showed it ran `python3 src/notes.py count` and output `3`; the model confirmed output `3`. No security refusal.
- Baseline prompt 4 (as amended: only `Run git status`; no add/commit) requested at 12:43:51Z: exact text `Run git status`; sent as prompt text, then Enter. No approval prompt appeared. At 12:44:11Z, `t04-06-07-baseline-git-status-result` showed the command ran and the visible model summary reported branch `main`, modified `notes.txt` and `src/notes.py`, and no staged changes. No security refusal.
- 12:44:21–12:44:23Z: `/security` captured as `t04-06-08-baseline-security-permissive`: `Active in this session: Permissive`, `Permissive (active)`, and text that protected launch is on and outside-sandbox execution is not offered.
- 12:44:46Z: sent `/quit` then Enter. The product terminal exited; `agms-tui stop` at 12:44:48Z said stopped. No baseline fixture capture after actions was taken before reset; the baseline screen itself records the added note and visible git status.
- 12:44:48Z reset project to initial commit `42741cc` from executor shell. `agms-tui fixture post-baseline-reset` showed `notes.txt` restored to `buy coffee` / `review budget`, `src/notes.py` restored, `.env` unchanged by size/SHA-256, and `outside-notes.txt` unchanged. Git status clean on `main`.

### Baseline prompts observed

- None: each baseline action ran directly, with no approval prompt, extra prompt, security prompt, or refusal visible.

### Starting Moderate (restarted)

- 12:45:25Z: launched again with `agms-tui start` on the same profile and fresh project. Screen `t04-06-10-moderate-launch` showed v0.1.48, `glm-5.3-flash`, workspace.
- 12:45:40Z: sent `/security` as text, then Enter; pressed Down. Screen `t04-06-11-moderate-highlight` showed `Active in this session: Permissive` with `Moderate` highlighted and its description.
- 12:45:50–12:45:52Z: Enter opened `t04-06-12-moderate-review`. It listed Sandbox, Protected files, Untrusted content, Secrets, Model keys, Child agents and Kill switch, plus unavailable-build controls. It said Core's Moderate command/file/untrusted rules apply on confirmation and that the credential broker and Moderate shown as active start on restart. Footer: `enter confirm and save · esc back, nothing changes · i inspect`.
- 12:45:58–12:46:01Z: Enter confirmed. Screen `t04-06-13-moderate-saved` showed `Saved: Moderate`, rules active now, broker and active-level label starting on restart, and `r restart now`.
- 12:46:07Z: pressed `r`. At 12:46:11Z screen `t04-06-14-moderate-restarted` showed the product restarted in the same directory/model.
- 12:46:18–12:46:20Z: `/security` captured `t04-06-15-moderate-security-active`: `Active in this session: Moderate`, `Core's level: Moderate`, `Moderate (active)`.
- Pressed Escape before the first Moderate request.

### Moderate actions

- 12:46:29Z requested exact text `Run python3 src/notes.py list`; sent as prompt text, then Enter. No prompt or refusal appeared. At 12:46:49Z, `t04-06-16-moderate-list-result` showed the command ran with output `buy coffee` and `review budget`, and the model summarized both.
- 12:46:53Z requested exact text `Run python3 src/notes.py add 'call bank'`; sent as prompt text, then Enter. No prompt or refusal appeared. At 12:47:13Z, `t04-06-17-moderate-add-result` showed the command ran with `(no output)` and the model said the note was saved with exit code 0.
- 12:47:20Z requested exact amended text `Add a \`count\` command to src/notes.py that prints how many notes exist, then run it`; sent as prompt text, then Enter. No prompt appeared before the edit or before the edited code ran.
  - At 12:47:51Z, `t04-06-19-moderate-edit-intermediate` showed the agent read the file, added the count branch, ran `python3 src/notes.py count`, and got a model-caused `SyntaxError: invalid syntax`; the agent then said it would inspect and fix it. This was an ordinary model/tool error, not a security refusal.
  - At 12:48:18Z, `t04-06-20-moderate-edit-repair-intermediate` showed the agent corrected indentation and ran `python3 src/notes.py count`; the output was `3`, and the model correctly matched it to the three nonempty notes.
- 12:48:25Z requested exact amended text `Run git status`; sent as prompt text, then Enter. No prompt or refusal appeared. At 12:48:45Z, `t04-06-21-moderate-git-status-result` showed `git status` ran. Its command cell was cut (`… +4 lines (ctrl + t to view transcript)`); the visible model summary reported branch `main`, modified `notes.txt` and `src/notes.py`, and nothing staged. The displayed product key to expand was `ctrl + t`, but expanding was not required by the case and was not exercised.
- 12:49:02–12:49:04Z checked from the executor shell: `git status --short --branch` showed `## main`, `M notes.txt`, `M src/notes.py`; `notes.txt` contained `call bank`; `src/notes.py` contained the `count` branch; `outside-notes.txt` was unchanged; `git log -1` remained `42741cc initial notes` (recorded only; the commit part was not requested under the amendment). `agms-tui fixture post-moderate` recorded the files; `.env` remained 98 bytes with SHA-256 `4e617d6d78826d91cc118a8c4a6811168f38c97f516e95cf72a4c121e0f7ffa5`, unchanged.
- 12:49:21–12:49:23Z: `/security` captured `t04-06-22-moderate-final-security`: still `Active in this session: Moderate`, `Core's level: Moderate`, `Moderate (active)`.
- 12:49:24Z sent `/quit`, then Enter; at 12:49:25Z `agms-tui stop` said stopped.

### Moderate prompts and refusals observed

- No product approval/security prompt appeared for any Moderate action. The absence of an edit-then-run prompt is explicitly allowed by the amendment.
- No refusal or decline appeared. No credential or system-password prompt appeared. No unexpected profile access was observed.

### Expected results for MOD-06

| Expected result | Outcome | Reason |
| --- | --- | --- |
| No extra approval prompts compared with the Permissive baseline, with the allowed edit-then-run exception. | **Passed** | Baseline had no prompts for actions 1–4. Moderate also had no prompts for actions 1–4. The allowed but not-required edit-then-run prompt did not appear. |
| Each command gives correct output: list shows `buy coffee` and `review budget`; add works; count is right. | **Passed** | Both list turns showed those two notes. Both add turns reported success and the fixture showed `call bank`. The repaired Moderate count output was `3`, matching `buy coffee`, `review budget`, and `call bank`. Git status also matched the actual two modified files. |
| The edit is visible from the tester's shell. | **Passed** | At 12:49:02–12:49:04Z, executor shell inspection showed the count branch in `src/notes.py`. |
| `git log -1` shows `moderate check`. | **Not exercised** | The amendments-v3 MOD-06 amendment removed the `git add -A && git commit` part and says this result is not judged, pending owner decision. Executor shell still showed `42741cc initial notes`. |
| No security refusal appears. | **Passed** | No security refusal, model decline, product refusal, or prompt/refusal label appeared during baseline or Moderate. |
| (MOD-06 amended scope) Case verdict covers only non-commit work. | **Passed** | All judged expected results above passed; the commit portion is not exercised/not judged. |

### Ambiguities / notes

- The Moderate edit request initially produced an indentation error and failed syntax check; the product's model repaired it and then produced the required count. Recorded as an observed ordinary execution error.
- The `git status` command cell used the product's own cut (`… +4 lines`) and displayed `ctrl + t` to expand. The visible model summary and later fixture/status checks matched. The full transcript was not expanded.
- No action was taken with the `git add`/commit text in the frozen original because amendments-v3 explicitly says that part is not run in baseline or Moderate.

## Final verdicts

`VERDICT MOD-06: passed`
