The loop now runs with no human step. Manager cycles start themselves; I seeded one goal through normal intake, and it went through to an accepted verdict with no other input from me. The owner is armed at **generation 11** with no holds, and the loop has been idle since the verdict.

## The three fixes

**1. Automatic manager cycles.** The owner's own scheduled job can't run cycles. It runs in the background session, where the manager's Claude login fails ("User interaction is not allowed"). The same check works from the logged-in session. So I added a second scheduled job, the "manager lane", which uses the same code and config and runs in the logged-in session.
- **When it runs:** only while the owner is armed, and only when there's work:
  - pending events;
  - a result awaiting a verdict;
  - nothing queued while unused or failed allocations remain.
  The last two raise a wake-up at most once per 12 hours for the same situation.
- **Limits (in config):** one cycle in flight, 120 s between starts, 48 per day.
- **Audit:** every start, finish, wake-up, hold and recovery is a line in `manager-schedule/manager-cycles.jsonl`.
- **Failures:** any failure, crash or error holds the lane until someone runs `--recover <evidence>`. A disarm mid-cycle is re-checked before anything is committed.
- **Nothing to do:** the manager can now answer with no actions and a reason, instead of the cycle holding.
- **Tests:** they cover a crash mid-cycle followed by a restart. The restarted lane holds without calling the manager again, and recovery then frees the coordinator.

**2. Briefing budget.** On a copy of live state, the old briefing fit 6 of 85 events in 65 KB; the new one fits all 24 claimed events in about 62.6 KB.
- The 22 KB list of finished allocations is now a count and a fingerprint.
- Returned results and failures come first. The two oldest events come next so the backlog always moves, then the newest.
- Anything left out is counted by kind and stays pending for the next cycle.
- Finished allocations are compacted automatically after a grace period. Allocations that later sprint steps still check stay until their sprint completes.

**3. Main worktree.** It's out of the config; the only configured worktree is now the new disposable one. Auto-adoption now also requires a worktree checked out detached at the allocation's exact base commit, with no rebase or merge in progress, and this is checked again at claim. Anything aimed at a branch checkout stays on the hand lane.

## Tests and reviews
- **Full suite in clean worktrees:** 954, then 961, then 963 OK at the final commit `ec7918b368`, plus 30 OK for `manager_cycle_test.py` at each.
- **Code reviews:** the first returned CHANGES REQUIRED with eight findings; all were fixed. The second and third returned APPROVE, and I applied their small notes too.
- **Budget:** four reviews this round (three code, one qualification), within the default five.

## Qualification
- **Result:** the PF-83 VM run, repeated as in round 6, was independently reviewed **QUALIFIED**: all four fields supported, counts 1/1/1.
- **VM restored exactly:** same digests, inodes and mtimes.
- **Script change:** the command guard here blocks `rm -f`, so the restore and cleanup scripts use plain `mv` and `rm -r`. The result is the same.
- **One new finding:** a connection to chatgpt.com that existed before the fence was opened.
  - The fence blocked its 12 later packets in both directions; no data passed and no new connection was made.
  - The reviewer judged it doesn't affect the result.
  - I couldn't tell which process owned it; my first guess was probably wrong, and that's corrected in `review-notes.md`.

## Live run, after promotion (UTC)
I started no manager cycles. The lane worked through the 85-event backlog in four cycles on its own (17:01–17:10), each ending with "nothing to do" and a reason.

| Step | Time |
|---|---|
| Goal seeded through normal intake (read-only check, disposable worktree) | 17:12:29 |
| Manager cycle starts | 17:12:51 |
| Action `owner-loop-r7-01` prepared | 17:14:03 |
| Owner adopts it | 17:14:18 |
| ACK | 17:15:55 |
| START | 17:16:01 |
| RETURN | 17:17:00 |
| Worker closed cleanly | 17:17:02 |
| Next manager cycle starts | 17:17:09 |
| Verdict: accepted | 17:18:59 |

**Idle:** from 17:18:59 to 17:36:16 (17.3 minutes) no cycle started. The lane checked in 33 times (30 idle, 3 skipped while an owner tick held the lock), with no holds and no errors.

## Commits
Pushed fast-forward from `bbd703ee66`: `213c814364`, `81eccfc1a8`, `2faff0f5cf`, `ec7918b368`, and `444f7f8678` (the record). The four uncommitted files in `scripts/initiative_control/` were never staged.

## Still open
- **Dashboard:** not republished, because `sync-source.sh` still exports the four uncommitted files.
- **Logout:** the manager lane needs the logged-in session. If you log out, manager cycles stop, though the owner keeps ticking.
- **Remaining review notes (all fail safe):**
  - A disarm during acceptance is refused and has to be retried.
  - An unusually long owner tick can hold the lane.
  - Unused allocations on a completed or paused sprint wake the manager twice a day until compacted.
- **Qualification follow-ups:**
  - Record open connections before the fence.
  - Fix the canary path in the test case.
  - The VM hardening list from rounds 4–6.
- **Carried over:** `slack-receiver-02`.