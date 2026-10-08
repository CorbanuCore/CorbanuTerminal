# Owner and manager jobs: requalified and reboot-safe (2026-10-08)

Travis, 2026-10-08:
- The login dependency stays: the manager runs in his GUI session and stops when he logs out.
- Yes to a reboot-safe install.
- The manager stays on standby: no allocations and no mode change.

**Result:** the owner is armed at **generation 13**, `tmux-workers`, with `unresolved_holds: []`.
- Package `e4d91473883fd43a8879b679fdc4dc1a7b3582b8661a3fa5fe9a95691f556ce4`. It contains #268 (a manager-lane pre-cycle timeout counts as an error) and #275 (the reboot-safe installer).
- Config digest `361d4059…04cd`. It is unchanged apart from the package digest; worktrees and `manager_cycle` are the same.
- Both jobs tick idle and are loaded from plists that launchd reloads at login.

Evidence is in `.codex-work/owner-manager-enable-20261002/round9/`: `qual/`, `live/`, `code-review-*/` and `suite-*.txt`.

## Installer (#275, merged `8d107f4c1d`)

`activate.py --owner install --launch-agent` makes a schedule reboot-safe.
- **GUI-domain job (the manager lane):** its plist goes to `~/Library/LaunchAgents/<label>.plist` with session type Aqua. launchd reloads it into `gui/501` at each login.
- **User-domain job (the owner lane):** launchd does not reload `~/Library/LaunchAgents` into the Background `user/501` domain.
  - Evidence on this Mac: Homebrew's postgresql plist there lists Background among its session types, yet after boot and login it is loaded only in `gui/501`.
  - So the owner job keeps `<root>/owner.plist`.
  - A login agent, `~/Library/LaunchAgents/com.corbanu.initiative-owner-login.plist` (Aqua, `RunAtLoad`), bootstraps the job into `user/501` at login unless it is already loaded in either domain.
  - The agent retries every 30 s until the bootstrap succeeds and logs to `~/Library/Logs/com.corbanu.initiative-owner-login.log`.
- **Receipts:** they record the plist path, plus the login agent path and its digest. Old receipts still mean `<root>/owner.plist`.
- **Uninstall:** it removes the LaunchAgents files before the bootout, so nothing reloads an uninstalled job.

Tests: `test_owner_daemon` 185 OK.
- Full `scripts/initiative_control` discovery: 979 tests, with 2 failures. The same 2 fail on untouched main.
- `manager_cycle_test`: 30 OK.

Reviews (Opus 5.5 High):
- Round 1: CHANGES REQUIRED, 2 P1s on write and uninstall ordering.
- Round 2: CHANGES REQUIRED, a P2 on the login agent's log location.
- Round 3: APPROVE. Its P3s were applied.

## Requalification (PF-83 VM, run `7f1bb0`)

This repeats round 7 step for step. The changes are listed in `qual/script-diff-vs-round7-*.txt`:
- names;
- a clean candidate worktree;
- two round-7 follow-ups: a root `lsof`/`netstat` before the fence, and the case's canary path now names this round.

What the run showed:
- **Fence:** the fence and the credential hold were restored byte-identical (digests, inodes, mtimes), and the manifest was identical.
- **Direct connections:** none.
  - en0 has zero guest-originated SYNs and zero guest packets to public addresses.
  - The pre-fence connection on guest port 52519 is now attributed by the root `lsof`: it was the console user's Chrome. pf dropped all of its packets.
- **Positive control:** returned with 3/3 broker joins, and the owner closed the worker.
- **Broker-down control:** failed closed (`runtime_failure`).
- **Lifecycle:** 9 ticks, no holds, 24/24 broker joins, ACK, START, RETURN, owner close. Counts: real_ack 1, real_start 1, real_return 1.
- **Independent review:** session `01a119d7-de4a-7683-be3c-62f435e3e582`, **VERDICT: QUALIFIED**, all four fields supported. Its wording corrections are in `qual/review-notes.md`.

## Live restage (`live/`)

1. Each job was uninstalled right after a completed tick: first the manager lane, then the owner.
2. Generation 11 was disarmed, leaving the owner OFF at generation 12.
3. The item-5 audit and preflight were round 7's gate. Two disclosed changes:
   - the live config already holds `manager_cycle`;
   - the manager lane must be uninstalled too.

   All five items passed, 1.3 s after the audit, at coordinator revision 2989.
4. The recipe reconfigured the owner to the new package and repinned both schedules with `--launch-agent`.
   - It then stopped at its OFF-tick assertion. Its explicit OFF tick ran into `tick.lock`, which launchd's kickstart tick held, so it printed `owner_run_refused` without running.
   - The next interval tick latched `owner_off` as intended.
5. `restage-continue-r9.py` re-staggered the manager lane by about 15 s while the owner was still OFF. Both jobs had loaded within 1 s of each other, so their intervals were aligned.
6. It then armed **generation 13** and recovered the `owner_off` latch with evidence.

## Verification

- **Plists** (`live/plist-verification.txt`, `live/launchctl-print.txt`):
  - `plutil -lint` OK for all three.
  - `launchctl print` shows `user/501/com.corbanu.initiative-owner` with path `<root>/owner.plist`, Background session.
  - `gui/501/com.corbanu.initiative-owner.manager` loads from `~/Library/LaunchAgents/…manager.plist` (Aqua).
  - `gui/501/com.corbanu.initiative-owner-login` is loaded, last exit 0.
  - No `print-disabled` override in either domain.
- **Login agent, live** (`live/login-agent-check.txt`):
  - Right after a completed tick I ran `launchctl bootout user/501/com.corbanu.initiative-owner` and then `launchctl kickstart gui/501/com.corbanu.initiative-owner-login`.
  - The job came back in `user/501` from `<root>/owner.plist`, and the agent exited 0.
  - The next ticks were normal, with no `interrupted_tick`.
- **Idle watch** (`live/watch-ticks.jsonl`, 05:22:30Z-05:37:47Z):
  - Owner: 28 ticks, all `ACTIVE` with `unresolved_holds: []`, hold null, 0 errors, `firing: interval`. Median 3.3 s, max 3.9 s.
  - Manager: 30 ticks, hold null, 0 errors. All were `IDLE` except 2 `BUSY` skips, when the owner held the admission lock (normal). Median 0.9 s.
  - No manager cycle started, and no worker was launched. Both stderr logs are empty.
- **Not done:** I did not reboot the Mac. `sfltool dumpbtm` needs admin rights; one attempt hung and was stopped after 25 s, so Login Items approval is unverified.

## After the next reboot (Travis)

1. Log in. The manager lane and the login agent load at login, and the owner job is back in `user/501` within a few seconds.
2. Check:
   - `launchctl print user/501/com.corbanu.initiative-owner | grep -E 'path|state|runs'` shows the path `…/corbanu-owner-live/schedule/owner.plist`.
   - `launchctl print gui/501/com.corbanu.initiative-owner.manager | grep -E 'path|runs'` shows the path `~/Library/LaunchAgents/com.corbanu.initiative-owner.manager.plist`.
   - `launchctl print gui/501/com.corbanu.initiative-owner-login | grep 'last exit'` shows 0.
3. After about a minute, both `tick.json` files under `~/Library/Application Support/corbanu-owner-live/{schedule,manager-schedule}/` show `"hold": null` and recent `completed_at` values.
4. If either shows `interrupted_tick`, shutdown killed a tick in flight. Recover that schedule with `owner_daemon.py --schedule <root> --recover "<evidence>"` using the installed runtime and `-B`.
5. If macOS reports "Background Items Added" for python3.14 or `sh`, allow them. Turning them off in Login Items stops the reload.
