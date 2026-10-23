# Owner and manager jobs: return to standby (2026-10-08)

Scope: bring both existing scheduled jobs back to healthy idle (option A, standby).
Not done: new allocations, mode or config changes, seeded work.

## Starting state (00:30Z)

- Owner armed, generation 11, `tmux-workers`, `manager_enabled: true`, no unresolved owner holds.
  Config sha256 `9a8d854a…` (canonical config digest `9cfde5f7…`), package `38055df7…`.
- Owner job `com.corbanu.initiative-owner` (user/501): HOLD `missing_worktree` since 2026-10-06 23:45Z.
  `worktrees/owner-loop-r7-20261003`, the only configured worktree, was removed by the 2026-10-06 disk cleanup.
- Manager job `com.corbanu.initiative-owner.manager` (gui/501): HOLD `TimeoutExpired` since 2026-10-06 06:48:08Z.

## Owner job

1. Recreated the worktree as a detached linked worktree at its original base `ec7918b368`
   (from the shared CorbanuTerminal repo). Clean, HEAD detached. Config file and its digest unchanged and still
   equal to both installation pins; `--activation-status` shows the stored config digest matches.
2. `--recover` with evidence. Result: RECOVERED.
3. The first tick afterwards refused with `unsafe_file`. Cause: the recovery worker had run `owner_daemon.py --help`
   with plain `python3` (no `-B`), which wrote `runtime/__pycache__`. `schedule_pins` rejects any non-regular entry.
   Removed that directory, confirmed `schedule_pins` matches both pins, then ran `--recover` again.
4. Owner ticks `ACTIVE`, no holds, no workers started.

Lesson: never import or run anything from the live runtime dir without `-B` (or outside a copy).

## Manager job: diagnosis

The cause was a short subprocess timeout during a host stall, not a long tick.

- The failing tick took 8.3 s and wrote no `cycle_started`. So it failed before any cycle, where the only
  subprocess timeouts are 2–5 s (`python --version` pin check, `ps`, `launchctl print`).
- The owner job hit `TimeoutExpired` at the same moment (tick at 06:48:06Z, 17.9 s). It counted that as error 32
  and recovered on its next tick. The manager lane latches any error until `--recover`.
- Ticks on both lanes took 20–190 s around then (normally under 1 s), which points to a host-wide stall. The
  drive filled up later that day.
- No manager claim in the coordinator (`manager: null`), no launcher processes. Coordinator readiness was
  `empty`, with no returned actions and no idle allocations, so a recovered tick would be IDLE.

Ran `--recover` on `manager-schedule` with evidence. Result: RECOVERED. The reconciliation found no claim to release.

## Watch (00:41Z–00:57Z)

- Owner: about 32 ticks, all `ACTIVE`, no holds, no new errors (still 32 in total). Median 3.1 s, max 4.3 s.
- Manager: 31 ticks, all `IDLE` (one `BUSY`: the owner held the admission lock, which is normal).
  No holds, no cycles started. Median 0.7 s, max 2.1 s.
- Runtime dir: 58 pinned files only.

## Still open (needs Travis)

- Option A (standby) is the state now. B (real allocations) or C (uninstall) is still Travis's call.
- The manager job runs only while Travis is logged in (gui domain).
- The coordinator still says "security paused by owner", and `slack-receiver-02` is still a stalled hand claim.
  Neither was changed here.
- Possible follow-up: treat a single `TimeoutExpired` in the manager pre-cycle phase as a counted error rather than
  a latched hold (as the owner lane already does). This needs a code change and requalification, since the pinned
  package would change.
