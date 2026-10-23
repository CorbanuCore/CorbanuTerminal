PF-27-S08 is archived and **PR #364** is merged into main as merge commit `23158e8d14`. I merged it pinned to head `05ab22557c`, once all 21 checks had passed, 12 were skipped and none were pending or failing.

**Checker output** (run on the branch after merging in the latest main, then again on main after the merge):
- `python3 docs/plans/check.py` → `plans: active 3/3; available slots 0`
- `python3 docs/sprints/check.py` → `sprints: current 102; archived 152`
- Plans unit tests (6) and sprints unit tests (24) pass.
- No checker refused anything.

**What changed:**
- **S08 record:**
  - Moved to `docs/sprints/archive/p1-security-hardening/` with status `completed`; its `write_scope` now points to the new location.
  - New "Closure — 2026-10-09" section: Travis accepted S08 with its known limits after decision (c) in PR #360. It links the evidence README's Known limits list and summarises the eight items, including that overwriting or deleting existing Credential Manager items was not probed.
  - Every remaining item is ticked, and Remaining now reads "None. Follow-ups are listed under Closure."
  - Two follow-ups are listed, not as blockers: PF-27-S09 (in progress, PR #363) and the write access to `secrets/.vault.lock` that S09 has to give the broker.
- **P1 plan:**
  - Removed S08's worktree entry and released its owner row.
  - The S08 row in the sprint table points to the archived record and cites PRs #333 and #360.
  - Added an S08 follow-up row to the carried-forward table.
  - Updated the lane and implementation-sequence lines.
- **Indexes and navigation:** the P1 sprint index now shows 33 current and 3 archived. I also updated S08's line in the main sprint list and removed it from the MkDocs navigation.
- **Task Node task map:** the S08 row now points to the archived record and SECACCT-TN-03 shows "accepted, archived".
- **Link fixes** for the new path: the archived S06 and S07 records and the PF-27-S07 evidence README.

**Left alone:**
- **S09 record:** not touched, since sec-s09 is editing it.
- **Research files:** two evidence files in `docs/research/tasknode-integration/` still quote S08's old path, so those links are now broken. S06/S07 had the same case and I handled it the same way.
- **Git cleanup:** my worktree is removed. Git refused to delete the local branch `pf-27-s08-archive` with `git branch -d`, so it is still there; I didn't force it. There was no build output to clean up.