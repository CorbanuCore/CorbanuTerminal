PF-27-S06 and PF-27-S07 are archived with Travis's acceptance recorded, and the docs-only PR #329 is merged into main as merge commit `8165c1375f`. Every check on the PR passed, and both checkers pass again on main after the merge.

**What was archived**
- Both records moved to `docs/sprints/archive/p1-security-hardening/` with status `completed`. This is the first P1 archive folder, so I created it.
- Each record now has a "Closure — 2026-10-08" section saying Travis accepted the sprint with known limits on 2026-10-08. The limits are referenced through a link to the "Known limits" list in each sprint's evidence README. The gate evidence is cited as PR #312 for S06 and PR #316 for S07.
- In both records, every item under Remaining, Verification and Exit evidence is now ticked.
- Follow-ups are listed in each record as follow-ups, not blockers:
  - #300: decided fail closed, being implemented.
  - #301 and #304: decided rule-driven removal, draft PR #326.
  - #323: open.
  - PF-27-S08 and S09: planned.
- Two status changes since your brief:
  - #320 is now closed, fixed by PR #327 (merged).
  - #307 is closed, fixed by PR #321. It was still listed as open in the S06 record, so I recorded it as fixed.
- The two S06 follow-ups with no home yet are still in the plan's carried-forward table: file tools other than patches under the Windows launch contract, and the elevated sandbox's reads of profile files.

**Other files updated, all docs**
- **P1 plan:**
  - The map rows for S06 and S07 now point to the archived records and show completion.
  - Their worktree entries are released.
  - A follow-up row is added.
  - The plan's Windows-machine item now covers only S08/S09.
- **Indexes and navigation:** the P1 sprint index and the main sprint list (now 34 P1 sprints current, 2 archived), and two lines removed from the MkDocs navigation in `mkdocs.yml`.
- **Task Node task map** (`qa/initiative-control/p0-security-levels/tasknode-task-map.md`): the PF-27-S06 row now points to the archived record and the 7 videos. I noted that S07 isn't covered by any Task Node task; I didn't create one.
- **Link fixes:** links to S06 in the closed P0 plan and the archived PF-27-S02 and S04 records now point to its archived location.

**Checker output**
- `python3 docs/plans/check.py`: `plans: active 3/3; available slots 0`
- `python3 docs/sprints/check.py`: `sprints: current 103; archived 150` (it was 105 and 148 before), no errors
- The plans unit tests (6) and sprints unit tests (24) pass.
- No checker refused to archive.

**Left alone**
- Two research evidence files in `docs/research/tasknode-integration/` quote the old S06 path word for word, so I didn't edit them. Those links are now broken.
- A link in the archived PF-27-S04 record to PF-23-S03 was already broken before this PR; I left it.

My worktree is removed, and there was no build output to clean up.