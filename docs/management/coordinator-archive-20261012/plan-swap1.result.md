P0 is closed and P1 security hardening is now the active plan in slot 1. It went in as one docs-only PR, #279, merged at `cf2625bd8d` with `--match-head-commit`. Both checkers and both unit-test suites pass locally and on main.

**P0 closed:** the plan is now `docs/plans/completed/main-2026-10-08-p0-security-levels.md`. The filename rule expects a release number, but no release contains this work (0.1.48 came out before it). So I used `main-2026-10-08` and noted this exception in `completed/index.md`.

The closing note says:
- **Shipped:** the work of all 20 core sprints plus PF-27-S05 is on main behind default-off flags, and 20 of the 21 sprint records are archived.
- **Not done:** none of the three milestones has run (Aggressive ships, Moderate ships, flag removal). There has been no code-blind VM run, human sign-off or Permissive-compatibility rerun, and no release. All of these are listed as open items in P1.
- **Links:** the Task Node rewards (P0SEC-TN-01..09, all Rewarded, 27.5 PFT) through the task map, and the demo reel.

**One difference from "all sprints archived":** PF-28-S02, PF-29-S01/S02, PF-33-S02 and PF-41-S01 were merged behind flags but their records were still open. I archived them as completed so P0 could close cleanly. Their unfinished items (milestone checks, known limits) are now in a "Carried forward from P0" table in P1; none are marked done.

**P1 active in slot 1:**
- **Lanes:** broker, windows-host and qualification, with worktree coordinates recorded for PF-13-S07, PF-27-S06 and PF-27-S07.
- **PF-13-S07:** moved into P1 and still `ready`; its final archive waits on the Aggressive milestone. I also corrected two claims in its record to match the evidence: 11 routes blocked (it said 10), and `.ssh` reads denied in 3 of 4 runs (it said all).
- **PF-27-S06:** set to `blocked` until a real Windows machine runs its remaining gate. To let sec-win2's PF-27-S07 run at the same time, I put PF-27-S06 in its own windows-host lane and limited what it can write to evidence files. I also recorded it as merged behind its flags, so S07 can depend on it.
- **PF-27-S07..S09:** sec-win2's PR #280 merged first. I merged main into my branch and changed only their plan path; their execution orders 42–45 are unchanged.
- **Links:** plan paths are updated across the sprint records, indexes, site navigation, the product-spec link and the P0 Task Node task map. I left the Task Node evidence records unchanged because they have already been rewarded.

I left a coordination note for sec-win2 in `plan-swap1.result.md`. My worktree is removed.

**Decisions for Travis:**
1. **Deadline and target release for P1.** Both are "TBD" because none were given.
2. **Where PF-76-S01 and PF-77-S01 go.** These two non-security drafts are still parked under the closed P0 plan. I suggest PF-77-S01 to the Task Node plan and PF-76-S01 to the provider-auth plan.
3. **`scripts/initiative_control/coordinator.py`** still points the "security" workstream at the old P0 plan path. I didn't change it because it's code, not docs. It needs a small follow-up, which I can do if you want.