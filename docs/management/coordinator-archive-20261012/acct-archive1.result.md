PF-60-S03 is archived. The docs-only PR #335 is merged into main as merge commit `0c3ad37a10`, and both checkers pass on main after the merge.

**What was archived**
- The record moved to `docs/sprints/archive/portfolio-agent-cost-accounting/` with status `completed`.
- It has a new "Closure — 2026-10-09" section saying Travis accepted the sprint on 2026-10-09, in chat with the coordinator. It links the evidence:
  - the first acceptance run (#292) and the three re-runs (#311, #315, #328), each with its own code-blind Opus review;
  - the fix PRs: #299, #303, #291/#305, #306/#318 and #322;
  - the demo index, now including #332's re-recorded videos.
- Follow-ups are listed as follow-ups, not blockers:
  - #287: the clock-behind warning wording.
  - #325.
  - The body-review findings, now owned by PF-60-S05 (#330).
- One thing to know about the body-review findings: S05's mandate covers only the Blocker, Majors 2–5 and Minor 10. Its own record excludes Minors 6, 7 and 9 and Nits 11–14, so I recorded those as open with no sprint yet.
- Every Remaining, Verification and Exit evidence item now says what happened to it, because the checker won't accept unticked items in a completed sprint. Open limits are marked "carried, not discharged", not closed:
  - the acct-sweep-64 items;
  - the items inherited from PF-60-S02;
  - the work-budget limit that counts rows across the whole store.
- I also wrote that the code-blind runs used scratch homes rather than the sealed-VM isolated-execution gate, and that Travis accepted them as the qualification.

**Other files updated, all docs**
- **Plan:**
  - The current-state text and the S03 row in the sprint map now point to the archived record.
  - The S03 lane worktree is marked released, and I removed its front-matter entry now that S05 has its own.
- **Index:** the accounting row in `docs/sprints/index.md` now shows 2 current sprints and 3 archived.
- **Navigation:** the S03 line is removed from `mkdocs.yml`.
- **Not touched:** `qa/demos/index/PF-60-S03.md` and S05's record.

**Conflicts and waits**
- I waited for #332, which merged at 23:28, before opening the PR.
- acct-s05's #334 (S05 moved to `ready` with its own worktree) merged while my PR was open. I merged main into my branch and resolved the plan and index conflicts by keeping S05's text as it is on main and my S03 changes.

**Checker output** (on the final branch and again on main after the merge)
- `python3 docs/plans/check.py`: `plans: active 3/3; available slots 0`
- `python3 docs/sprints/check.py`: `sprints: current 103; archived 151` (it was 104 and 150 before), no errors
- The plans unit tests (6) and sprints unit tests (24) pass, and `git diff --check` is clean.
- No checker refused anything.
- PR checks: 21 passed, 12 skipped, 0 failed. I merged with `--match-head-commit 2cb6edbeda`.

**Left alone**
- `mkdocs.yml` still has a navigation line for PF-60-S02 pointing to its old `current/` path, which no longer exists. This was there before my PR, and it's outside S03's lines.
- `docs/research/tasknode-integration/workstream-record-pf60-accounting-20260914.md` and some old qa manifests still name S03's `current/` path. I kept them as historical evidence, as the S06/S07 archive did.

My worktree is removed, and there was no build output to clean up. The local branch `docs/archive-pf60-s03-20261009` is still there, because `git branch -d` didn't delete it; it's already merged, so nothing is lost if it's removed.