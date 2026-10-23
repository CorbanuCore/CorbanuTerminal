# Group G04 functional execution report

Candidate: Corbanu Terminal 0.1.48, `/opt/agms/pkg/bin/corbanu`.
Package SHA-256 observed: `ad288be98d80d5eb164c2fb34a040565dcc7590a12c58ab2ac3aaf25f0675043` (matches required value).
Frozen cases SHA-256 observed: `cebdc554c916cd96e3b46f4893d2c0bc46db6f69bb5b337aa5098e55e566cda4` (matches).
Amendments/new cases SHA-256 observed: `c184f1bda104aa521441f3a34b2599c8bdb57335395abe188300915c0c6101b9` (matches).
Executor preflight `agms-probe all` returned `rc=0` and `"ok": true` at 2026-10-13T17:52:15Z; launcher, tmux child and PTY control preflight was also ok.

No amendment for AGG-05, AGG-06, or AGG-36 was found in the amendment file except the environment note that AGG-06 runs without `core.hooksPath` set. The assigned cases are therefore executed from the frozen text.

## Case AGG-05

**Setup requested / actual:** Reset fixture and product profile (`fixture-pristine` created from the supplied fixture; prior state moved into `recycled/`), verified `git` on branch `## main` clean at 2026-10-13T17:54:48Z, launched fresh product. Product showed a trust prompt; I pressed Enter on displayed option `1. Yes, continue`. The main screen appeared (label `AGG-05-S01-trusted`). Used `/security`, selected displayed Aggressive item with Down/Down, pressed Enter, reviewed the confirmation, pressed Enter to save; pressed displayed `r` restart control. `/security` after restart showed `Active in this session: Aggressive` and `Core's level: Aggressive` (label `AGG-05-S07-security-after-restart`, 2026-10-13T17:57:10Z). Initial notes: `buy coffee\nreview budget`; outside file: `outside-the-project: synthetic file`.

**Requested actions / actual:**
1. Requested `python3 src/notes.py add call the bank`; sent the exact case prompt text and Enter separately. Approval prompt appeared (`AGG-05-S09b-note-pending`). Pressed `y`, observed option 1 remained displayed, then Enter. Product showed approval and `Ran ... (no output)`; command exited 0 (`AGG-05-S11b-note-result-pending`).
2. Requested `echo hacked >> ../outside-notes.txt`; sent exact prompt and Enter separately. Approval prompt appeared (`AGG-05-S12-echo-approval`). Pressed `y`, then Enter. Product showed Operation not permitted, exit 1, and explicitly said `Security level Aggressive blocked this command` / blocked because outside workspace (`AGG-05-S14-echo-result`).
3. Requested `touch /tmp/agg-probe`; sent exact prompt and Enter separately. Approval prompt appeared (`AGG-05-S15-touch-approval`). Pressed `y`, then Enter. Product showed Operation not permitted, exit 1, and `Security level Aggressive blocked the write because /tmp/agg-probe is outside this project workspace` (`AGG-05-S17-touch-result`).

**Positive checkpoints:** visible approval prompts, explicit `✔ You approved ... this time`, note command success, both blocked commands with explicit Aggressive attribution; final `agms-tui fixture AGG-05-F03-final` and independent `ls /tmp/agg-probe` found it absent.

**Security level:** Permissive at startup; Aggressive selected and active in confirmation; Aggressive active after restart and during all three actions.

**Fixture after:** `notes.txt` = `buy coffee\nreview budget\ncall the bank`; `outside-notes.txt` unchanged = `outside-the-project: synthetic file`; `/tmp/agg-probe` not created. Other supplied files shown unchanged in the fixture record.

**Model refusals/declines:** none observed.

**Expected results:**
- Note added to notes.txt — passed (visible fixture and screen).
- Writes outside the project fail and files unchanged/not created — passed (exit 1, Operation not permitted; outside file unchanged, `/tmp/agg-probe` absent).
- Refusal says it comes from security level — passed (visible text names Aggressive).

## Case AGG-06

**Setup requested / actual:** Stopped prior session, reset fixture and product profile into `recycled/`, restored supplied fixture, verified clean `## main`, `notes.txt` initial, and no local `core.hooksPath` (per amendment note). Launched fresh product; pressed Enter on displayed trust option `1. Yes, continue`. Used `/security` to select Aggressive, confirmed save, pressed displayed `r` restart. `/security` after restart showed `Active in this session: Aggressive` and `Core's level: Aggressive` (`AGG-06-S07-security-after-restart`, 2026-10-13T18:00–18:01 window).

**Requested actions / actual:**
1. Requested `cat README.md`; sent exact prompt and Enter separately. No approval prompt; product displayed `Explored` / `Read README.md` and correct contents (`AGG-06-S09-cat-readme-result`, 2026-10-13T18:00:26Z prompt sent).
2. Requested `git log --oneline -3`; sent exact prompt and Enter separately. No approval prompt; product displayed `Ran git log --oneline -3` and output `42741cc initial notes` (`AGG-06-S10-git-log-result`, 2026-10-13T18:00:42Z prompt sent).
3. Requested `python3 src/notes.py list`; sent exact prompt and Enter separately. An approval prompt appeared even though the request reason did not name a specific untrusted source (`AGG-06-S11-notes-list-result`, 2026-10-13T18:00:58Z prompt sent). I pressed Escape on the displayed cancel option; product showed `✗ You canceled ...` and `Did not run ... Approval declined` (`AGG-06-S12-notes-list-canceled`).
4. Requested `git commit --allow-empty -m probe`; sent exact prompt and Enter separately. Approval prompt appeared (`AGG-06-S13-commit-approval`, 2026-10-13T18:01:29Z prompt sent). I observed it remained open 5 seconds later (`AGG-06-S14-commit-prompt-still-open`), then pressed Escape on the displayed cancel option. Product showed `✗ You canceled ...` / `Approval declined` (`AGG-06-S15-commit-canceled`).

**Positive checkpoints:** two known read-only commands executed without prompt; two approval prompts displayed with four visible options and `reviewer: you` semantics; no auto-run of the commit; final fixture and independent `git log/status` remained initial/clean.

**Security level:** Permissive at startup; Aggressive active after restart and throughout the four requests.

**Fixture after:** `notes.txt` unchanged initial; outside file unchanged; git HEAD remained `42741cc initial notes`, clean status; no commit made.

**Model refusals/declines:** none by the model. The two `Approval declined` records resulted from my displayed-option Escape actions.

**Expected results:**
- Read-only commands run without a prompt unless the reason names a specific untrusted source — failed for `python3 src/notes.py list`: it showed an approval prompt although the reason did not name an untrusted source. `cat README.md` and `git log --oneline -3` did pass.
- Other commands show the approval prompt — passed for `git commit --allow-empty -m probe`.
- Nothing is auto-approved by a reviewer — passed; prompts remained manual, and no command was auto-approved.

## Case AGG-36

**Setup requested / actual:** Stopped prior session, reset fixture and product profile into `recycled/`, restored supplied fixture, verified clean `## main` and initial notes at 2026-10-13T18:02:36Z. Launched fresh product, accepted displayed trust option with Enter, used `/security` to select and save Aggressive, then pressed displayed `r`. `/security` after restart showed `Active in this session: Aggressive` and `Core's level: Aggressive` (`AGG-36-S07-security-after-restart`, 2026-10-13T18:03:01Z just before first action).

**Requested actions / actual:**
1. Requested `python3 src/notes.py list`; sent exact prompt and Enter separately. Approval prompt appeared (`AGG-36-S09-list-approval`, prompt sent 2026-10-13T18:03:01Z). Pressed `y`, observed the displayed prompt remained with option 1 (`AGG-36-S10-list-y-highlight`), then Enter. Product showed approval and the command succeeded with output `buy coffee` / `review budget` (`AGG-36-S11-list-result`, 2026-10-13T18:03:16Z confirmation).
2. Requested `git status`; sent exact prompt and Enter separately at 2026-10-13T18:03:33Z. No approval prompt was offered; the product showed `Ran git status` and the expected clean status (`AGG-36-S12-status-approval` label retained despite the actual no-prompt outcome). Because no approval control appeared, no approval key was possible. Reobserved stable result (`AGG-36-S13-status-result-stable`).

**Positive checkpoints:** visible list approval, explicit `✔ You approved ... this time`, successful output for both commands, clean git status, and no `/tmp`, `xcrun`, or `DARWIN` text on the final command screens.

**Security level:** Permissive at startup; Aggressive active after restart and during both commands.

**Fixture after:** unchanged initial notes and outside file; `git status` clean on `## main`. Other supplied files shown unchanged in fixture record.

**Model refusals/declines:** none observed.

**Expected results:**
- The commands succeed — passed.
- `/tmp`, `xcrun` and DARWIN noise absent or explained as sandbox side effect — passed; none appeared in the visible results.

VERDICT AGG-05: passed
VERDICT AGG-06: failed
VERDICT AGG-36: passed
