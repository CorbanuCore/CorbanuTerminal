PF-27-S09 is archived. **PR #392** is merged into main as merge commit `070cb4d391`. I merged it pinned to head `742d611bc9` once all 21 checks had passed, 12 were skipped and none were pending or failing. Main hadn't moved since I branched, so no catch-up merge was needed.

**Checker output** (run on the branch, on a local merge with main, and on main after the merge; all three runs matched):
- `python3 docs/plans/check.py` → `plans: active 3/3; available slots 0`
- `python3 docs/sprints/check.py` → `sprints: current 100; archived 154`
- Plans unit tests (6) and sprints unit tests (24) pass.
- No checker refused anything.

**Issues filed** (none of these existed before; all labelled `security`):
- **#389:** an acceptance run in a console session, where the vault key is in Credential Manager, on the real Windows machine.
- **#390:** a probe for the exclusive pipe-squat case, where the broker can't create its pipe at all.
- **#391:** a product decision from you on when to turn `broker_model_auth` on by default.

**What changed:**
- **S09 record:**
  - Moved to `docs/sprints/archive/p1-security-hardening/` with status `completed`; its `write_scope` points to the new location and its `updated` date is now 2026-10-10.
  - New "Closure — 2026-10-10" section: your acceptance on 2026-10-10 ("Pass it"), after the independent run returned ACCEPT WITH LIMITS.
  - It lists the sprint's six known limits and the acceptance run's four, and links both evidence READMEs and PRs #363, #379 and #384.
  - It states that `broker_model_auth` is still off by default. I checked the code: `default_enabled: false`.
  - All remaining items are ticked, and Remaining now reads "None. Follow-ups are listed under Closure."
  - #389, #390 and #391 are listed as follow-ups, not blockers.
- **P1 plan:**
  - Removed S09's worktree entry and released its owner row; the broker lane is now free.
  - The S09 row in the sprint table points to the archived record and cites PRs #363, #379 and #384.
  - Added an S09 follow-up row.
  - Updated the lane and implementation-sequence lines.
- **S08 follow-up row:** this was the one line outside S09's own lines that I changed. It said S09 was "in progress" and still had to sort out the broker's write access to `secrets/.vault.lock`. It now says S09 is archived and that this was resolved in #363: Core creates the lock file and the broker locks it read-only.
- **Indexes and navigation:** the P1 sprint index now shows 32 current and 4 archived. I also updated the P1 line in the main sprint list and removed S09 from the MkDocs navigation.
- **Link fixes:** only the S09 link paths changed, in the archived S06, S07 and S08 records and the PF-27-S07 README, as was done for S08.

**Left alone:**
- **Task Node task map:** S09 has no row there, so I didn't touch it. Its "Not covered yet" line still lists PF-27-S09, which remains true.
- **Research files:** two evidence files in `docs/research/tasknode-integration/` still quote S09's old path, so those links are now broken. S08's archive left the same case alone.
- **Git cleanup:** my worktree is removed. Git refused to delete the local branch `pf-27-s09-archive` with `git branch -d`, so it is still there; I didn't force it. There was no build output to clean up.