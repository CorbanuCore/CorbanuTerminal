I merged both PRs with every check green, and #428 is closed.

**What shipped**

| PR | What | State |
|---|---|---|
| #432 | #428, Travis's option 1: the approval for a model-chosen spawn `account` switch now also applies whenever the TUI shows Aggressive, even when Core's own level stays Permissive. That includes "Aggressive enforced, boundary unverified" and "Active in this session: Aggressive". With approvals off, the switch is refused. | merged `fd0c8c4579` (28 passed, 9 skipped) |
| #433 | Archives S01, S02 and S03 and updates the plan, indexes and navigation | merged `b78c0111a4` (24 passed, 12 skipped) |

**How #432 works**
- Every Aggressive launch selects the `corbanu-aggressive` permission profile and sets the `CORBANU_SECURITY_ORIGIN` marker. Core now treats either one as Aggressive for this approval. Either signal alone can only make the check stricter.
- **One behaviour change beyond the brief:** if no answer comes back (a session that can't ask, such as `corbanu exec`, or a dismissed question), the agent is told that no answer was given. It is no longer told "the user declined". This came from the review.
- I stayed out of macct-fix2's files (#431) and macct-lane3's S04 files (#430). Both PRs touch two of the same files as #432: one in core's config and one test file.

**Gate evidence for #432** (in `qa/provider-auth/pf-84/d3-428-20261011/README.md`)
- **Tests:** the new tests fail with the fix removed and pass with it. They cover approvals off, Allow once, Cancel, no answer, and each TUI state from #428.
  - Focused `just test` on core, TUI and security-level: 221 passed; with developer-accounting: 202 passed.
  - The existing security-prompt tests: 130 passed.
- **Linux clippy:** `-D warnings` on glitch is clean, with and without developer-accounting.
- **tmux runs:** GLM 5.3 Flash, disposable homes, real default Z.AI key, plus a named account `work` holding a fake key.
  - I reproduced two of #428's states: Aggressive saved through `/security` without the preflight and then restarted, and the "boundary unverified" state.
  - Before (same build with only the new check removed): the child spawned on `work` without asking and got a 401.
  - After: the question appeared and Cancel ran nothing. In every run, Core's recorded level stayed Permissive.
  - A scan of the run folders found no real key, and the folders are deleted.
- **Video:** [pf84-d3-tui-aggressive](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-84-s03-pf84-d3-tui-aggressive-147b6d91318b-2026-10-10.mp4) (43 s, leak scan passed).
- **Review:** one Opus 5.5 High read-only pass returned "approve with changes". I fixed three of its four findings. The fourth was a process gap: #428 wasn't in any sprint record, which #433 now covers.

**What #433 changed**
- S01, S02 and S03 are in the archive with status `completed`. Each has a "Closure — 2026-10-11" section with your acceptance and links to its gate file, both acceptance READMEs and reviews, and PRs #407, #408, #409, #411, #420, #421, #422, #429, #431 and #432.
- **Limits recorded:** S02 and S03 stay behind the off-by-default flag. #425 and #426 (fix in #431, still open) and #428 (now fixed by #432) must be resolved before the flag is removed.
- **Follow-ups kept:** S02-15, #427, and S03's known limits.
- **Limited-testing agreement:** both acceptance runs were source-blind, not the full isolated code-blind run, so I recorded your acceptance as the limited-testing agreement. Please confirm that matches what you meant.
- S03's last open item, Linux clippy, is now ticked using the glitch run from #432.
- The plan's sprint table, its D3 decision row (now noting option 1 and #432), the PF-84 sprint index, the sprints index and `mkdocs.yml` are updated.
- `python3 docs/plans/check.py && python3 docs/sprints/check.py` passes on the branch and on main after the merge.

**Cleanup:** my two worktrees, branches, build target and copied binaries are removed, and so is everything I created on glitch.

**Still open**
- #431 (macct-fix2, for #425, #426 and #427) is not merged. #425 and #426 still block removing the flag.
- S02-15 stays open.