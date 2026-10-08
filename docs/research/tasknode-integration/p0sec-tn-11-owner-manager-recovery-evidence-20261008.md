# Task Node evidence record: P0SEC-TN-11

Compile the P0SEC-TN-11 Owner/Manager Recovery Evidence Record. Task `task_1a565564946c12669f4a151f721f84c8`, request `req_a83b37ccbbbe3e514fc4c7c8e79e81db8195d88ac8d2e803d9d3291b5ec8141f`. Every PR below is merged to `main` in CorbanuCore/CorbanuTerminal; each merge commit was checked with `git merge-base --is-ancestor <sha> origin/main` at `ad96c55cb5` (2026-10-08). Quoted blocks are copied verbatim from the records named above them (relative links re-pointed to this file's location).

Initiative-control operations work done in support of the P0 security program; it is not a security sprint. Records: [qa/initiative-control/owner-manager-standby-recovery-20261008.md](../../../qa/initiative-control/owner-manager-standby-recovery-20261008.md), [qa/initiative-control/coordinator-stale-state-20261008.md](../../../qa/initiative-control/coordinator-stale-state-20261008.md), [qa/initiative-control/management-bootstrap/manager-lane-timeout-and-logout-20261008.md](../../../qa/initiative-control/management-bootstrap/manager-lane-timeout-and-logout-20261008.md), [qa/initiative-control/management-bootstrap/owner-manager-reboot-safe-20261008.md](../../../qa/initiative-control/management-bootstrap/owner-manager-reboot-safe-20261008.md). Raw evidence for #268/#276 lives on the operator Mac under `.codex-work/owner-manager-enable-20261002/round8/` and `round9/` (not in the repo).

| PR | Merge commit on main | What |
| --- | --- | --- |
| #263 | `e7a744d7f9` | Owner/manager jobs recovered to standby |
| #265 | `003298a509` | Coordinator stale-state cleanup |
| #271 | `f6fc0c83e7` | Coordinator stale-state cleanup follow-up |
| #268 | `362007413c` | Manager lane counts a pre-cycle timeout as an error |
| #275 | `8d107f4c1d` | Reboot-safe schedule install (`--launch-agent`) |
| #276 | `1d618c2fa7` | Owner/manager requalified and reboot-safe |

PR links:

- #263: https://github.com/CorbanuCore/CorbanuTerminal/pull/263
- #265: https://github.com/CorbanuCore/CorbanuTerminal/pull/265
- #271: https://github.com/CorbanuCore/CorbanuTerminal/pull/271
- #268: https://github.com/CorbanuCore/CorbanuTerminal/pull/268
- #275: https://github.com/CorbanuCore/CorbanuTerminal/pull/275
- #276: https://github.com/CorbanuCore/CorbanuTerminal/pull/276

## #263: recovery to standby

Verbatim, `qa/initiative-control/owner-manager-standby-recovery-20261008.md` lines 6-12:

> ## Starting state (00:30Z)
>
> - Owner armed, generation 11, `tmux-workers`, `manager_enabled: true`, no unresolved owner holds.
>   Config sha256 `9a8d854a…` (canonical config digest `9cfde5f7…`), package `38055df7…`.
> - Owner job `com.corbanu.initiative-owner` (user/501): HOLD `missing_worktree` since 2026-10-06 23:45Z.
>   `worktrees/owner-loop-r7-20261003`, the only configured worktree, was removed by the 2026-10-06 disk cleanup.
> - Manager job `com.corbanu.initiative-owner.manager` (gui/501): HOLD `TimeoutExpired` since 2026-10-06 06:48:08Z.

Verbatim, `qa/initiative-control/owner-manager-standby-recovery-20261008.md` lines 14-25:

> ## Owner job
>
> 1. Recreated the worktree as a detached linked worktree at its original base `ec7918b368`
>    (from the shared CorbanuTerminal repo). Clean, HEAD detached. Config file and its digest unchanged and still
>    equal to both installation pins; `--activation-status` shows the stored config digest matches.
> 2. `--recover` with evidence. Result: RECOVERED.
> 3. The first tick afterwards refused with `unsafe_file`. Cause: the recovery worker had run `owner_daemon.py --help`
>    with plain `python3` (no `-B`), which wrote `runtime/__pycache__`. `schedule_pins` rejects any non-regular entry.
>    Removed that directory, confirmed `schedule_pins` matches both pins, then ran `--recover` again.
> 4. Owner ticks `ACTIVE`, no holds, no workers started.
>
> Lesson: never import or run anything from the live runtime dir without `-B` (or outside a copy).

Verbatim, `qa/initiative-control/owner-manager-standby-recovery-20261008.md` lines 42-47:

> ## Watch (00:41Z–00:57Z)
>
> - Owner: about 32 ticks, all `ACTIVE`, no holds, no new errors (still 32 in total). Median 3.1 s, max 4.3 s.
> - Manager: 31 ticks, all `IDLE` (one `BUSY`: the owner held the admission lock, which is normal).
>   No holds, no cycles started. Median 0.7 s, max 2.1 s.
> - Runtime dir: 58 pinned files only.

## #265, #271: coordinator stale state

Verbatim, `qa/initiative-control/coordinator-stale-state-20261008.md` lines 10-20:

> ## 1. Security resumed (revision 2981 → 2982)
>
> There is no CLI operation for the workstream's text fields. `set_stream_mode` doesn't apply: the security
> stream was already `mode: enabled`. The stale "paused" came from the 2026-09-13 `pause` text, the
> "Owner paused" history line, and the manager's carried-over reasoning. So the change is recorded the way
> earlier owner grants were (`authority:security-ownership-fable-20260914`): a meaningful `event`.
>
> - `authority:security-resumed-hand-20261006`, kind `human_authority`, actor Travis. It says security has been
>   active and hand-coordinated since the 2026-10-06 program decisions, supersedes the stored pause text, and grants
>   the manager no allocation or dispatch. Release, flag removal and milestone sign-offs stay with Travis.
> - The raw SQLite text was not edited.

Verbatim, `qa/initiative-control/coordinator-stale-state-20261008.md` lines 22-31:

> ## 2. `slack-receiver-02` settled (revision 2982 → 2983)
>
> - Native inspection first, at 02:15Z: `/private/tmp/crecv.5GJbiB` (packet, home, tmux socket) is gone.
>   No process matches session `01a09e29` or `crecv`. No Slack listener or poller is running.
> - `reconcile_dispatch` (dispatcher `hand`, no agent) moved it from `running` to `failed`, with
>   `owner_failure` evidence kind `owner_hand_claim_settlement`. Reason: a stale ACK-only receiver from
>   2026-09-14, deadline long passed, stall already reported, session ended. No duplicate launch, no
>   replacement and no Slack message.
> - It has been archived to `action_history`. No action is in flight now, and the `slack-receiver` resource is free.
>   Its allocation was compacted afterwards (see Follow-up).

Verbatim, `qa/initiative-control/coordinator-stale-state-20261008.md` lines 33-56:

> ## Manager reaction (standby, as designed)
>
> The two meaningful events triggered one manager cycle, `0528f17a…`, at 02:16Z. It was **accepted with no
> action**, prepared nothing and gave no verdicts. Its reason now reads: security is resumed but hand-coordinated
> with no manager allocation, and the receiver claim is settled. It no longer says "security stays paused".
>
> ### Follow-up (02:18–02:25Z)
>
> - **Second cycle.** Settling the claim left the `slack-receiver-02` allocation idle (its only action had failed,
>   none accepted). The owner sent `owner-wake:no_prepared_work`, which started cycle `609a4fce…` at 02:18Z.
>   That cycle was also no-action. It said "security and product continuation remain paused" again: the
>   authority event had already been consumed, and every briefing still carries the workstream's stored 2026-09-13
>   `pause` text.
> - **Allocation compacted (2988 → 2989).** `compact_allocation` turned `slack-receiver-02` into the consumed stub.
>   This is the same audited call routine compaction makes. The event is not meaningful, the original allocation
>   stays in the audit table, and evidence is recorded. Without it, routine compaction would only run after the
>   7-day failed-work grace. Until then the owner would send this wake every 12 hours.
>   After compaction: no further wakes, both jobs idle with no hold or errors, owner armed at generation 11.
>
> **Still open:** the stored security workstream text (`pause`, "Owner paused" history) cannot be changed through
> any CLI operation. A raw SQLite edit was not made. So until that text is corrected, a manager briefing can
> still conclude that security is paused. Fixing it needs either a small owner operation, for example to update a
> workstream's text fields with evidence, or Travis's approval for a one-off audited state edit. The fix doesn't
> affect dispatch: the stream is `mode: enabled`, and the manager has no security allocations.

## #268: pre-cycle timeout

Verbatim, `qa/initiative-control/management-bootstrap/manager-lane-timeout-and-logout-20261008.md` lines 6-22:

> ## Pre-cycle timeout is an error, not a hold
> On Oct 6 06:48Z a host-wide slowdown made one 2-5 s check time out before any cycle started, and the
> manager lane latched `TimeoutExpired` until a manual `--recover` a day later (the owner lane counted the same
> timeout as an error and kept ticking). Now, in `owner_daemon.scheduled_tick`:
> - a `subprocess.TimeoutExpired` with the manager cycle counter unchanged (read before the pins check, which
>   runs the pinned interpreter with a 5 s timeout, and again afterwards) is counted as an error and logged as
>   `transient_error` in `manager-cycles.jsonl`;
> - the third consecutive one latches `manager_pre_cycle_timeouts` (`refusal: TimeoutExpired x3`); BUSY does not
>   reset the streak, a normal pass does, `--recover` does;
> - a timeout after a cycle started, an unreadable counter and every other error still latch at once;
> - `main()` reports a `--recover` that times out instead of crashing.
>
> Reviews: Opus 5.5 High, round 1 CHANGES REQUIRED (the fix missed the only reachable pre-cycle timeout, the pins
> check; tests injected where production never times out), round 2 APPROVE with three P3s, all applied.
>
> Not live: the pinned owner package changes, so it needs the PF-83 requalification before a repin. That waits
> for the logout decision below so one requalification covers both.

Verbatim, PR #268 description (https://github.com/CorbanuCore/CorbanuTerminal/pull/268), test line (the repo record has no counts):

> Tests: `ManagerLaneTests` 25/25; full initiative_control discovery 971 run, 2 failures that reproduce on untouched `origin/main` (`test_attention…pf76…`, `test_preparation…receiving_tree…`, both from current docs state); `manager_cycle_test` 30/30.

## #275, #276: reboot-safe install, requalification, live generation 13

Verbatim, `qa/initiative-control/management-bootstrap/owner-manager-reboot-safe-20261008.md` lines 1-13:

> # Owner and manager jobs: requalified and reboot-safe (2026-10-08)
>
> Travis, 2026-10-08:
> - The login dependency stays: the manager runs in his GUI session and stops when he logs out.
> - Yes to a reboot-safe install.
> - The manager stays on standby: no allocations and no mode change.
>
> **Result:** the owner is armed at **generation 13**, `tmux-workers`, with `unresolved_holds: []`.
> - Package `e4d91473883fd43a8879b679fdc4dc1a7b3582b8661a3fa5fe9a95691f556ce4`. It contains #268 (a manager-lane pre-cycle timeout counts as an error) and #275 (the reboot-safe installer).
> - Config digest `361d4059…04cd`. It is unchanged apart from the package digest; worktrees and `manager_cycle` are the same.
> - Both jobs tick idle and are loaded from plists that launchd reloads at login.
>
> Evidence is in `.codex-work/owner-manager-enable-20261002/round9/`: `qual/`, `live/`, `code-review-*/` and `suite-*.txt`.

Verbatim, `qa/initiative-control/management-bootstrap/owner-manager-reboot-safe-20261008.md` lines 15-34:

> ## Installer (#275, merged `8d107f4c1d`)
>
> `activate.py --owner install --launch-agent` makes a schedule reboot-safe.
> - **GUI-domain job (the manager lane):** its plist goes to `~/Library/LaunchAgents/<label>.plist` with session type Aqua. launchd reloads it into `gui/501` at each login.
> - **User-domain job (the owner lane):** launchd does not reload `~/Library/LaunchAgents` into the Background `user/501` domain.
>   - Evidence on this Mac: Homebrew's postgresql plist there lists Background among its session types, yet after boot and login it is loaded only in `gui/501`.
>   - So the owner job keeps `<root>/owner.plist`.
>   - A login agent, `~/Library/LaunchAgents/com.corbanu.initiative-owner-login.plist` (Aqua, `RunAtLoad`), bootstraps the job into `user/501` at login unless it is already loaded in either domain.
>   - The agent retries every 30 s until the bootstrap succeeds and logs to `~/Library/Logs/com.corbanu.initiative-owner-login.log`.
> - **Receipts:** they record the plist path, plus the login agent path and its digest. Old receipts still mean `<root>/owner.plist`.
> - **Uninstall:** it removes the LaunchAgents files before the bootout, so nothing reloads an uninstalled job.
>
> Tests: `test_owner_daemon` 185 OK.
> - Full `scripts/initiative_control` discovery: 979 tests, with 2 failures. The same 2 fail on untouched main.
> - `manager_cycle_test`: 30 OK.
>
> Reviews (Opus 5.5 High):
> - Round 1: CHANGES REQUIRED, 2 P1s on write and uninstall ordering.
> - Round 2: CHANGES REQUIRED, a P2 on the login agent's log location.
> - Round 3: APPROVE. Its P3s were applied.

Verbatim, `qa/initiative-control/management-bootstrap/owner-manager-reboot-safe-20261008.md` lines 36-51:

> ## Requalification (PF-83 VM, run `7f1bb0`)
>
> This repeats round 7 step for step. The changes are listed in `qual/script-diff-vs-round7-*.txt`:
> - names;
> - a clean candidate worktree;
> - two round-7 follow-ups: a root `lsof`/`netstat` before the fence, and the case's canary path now names this round.
>
> What the run showed:
> - **Fence:** the fence and the credential hold were restored byte-identical (digests, inodes, mtimes), and the manifest was identical.
> - **Direct connections:** none.
>   - en0 has zero guest-originated SYNs and zero guest packets to public addresses.
>   - The pre-fence connection on guest port 52519 is now attributed by the root `lsof`: it was the console user's Chrome. pf dropped all of its packets.
> - **Positive control:** returned with 3/3 broker joins, and the owner closed the worker.
> - **Broker-down control:** failed closed (`runtime_failure`).
> - **Lifecycle:** 9 ticks, no holds, 24/24 broker joins, ACK, START, RETURN, owner close. Counts: real_ack 1, real_start 1, real_return 1.
> - **Independent review:** session `01a119d7-de4a-7683-be3c-62f435e3e582`, **VERDICT: QUALIFIED**, all four fields supported. Its wording corrections are in `qual/review-notes.md`.

Verbatim, `qa/initiative-control/management-bootstrap/owner-manager-reboot-safe-20261008.md` lines 68-84:

> ## Verification
>
> - **Plists** (`live/plist-verification.txt`, `live/launchctl-print.txt`):
>   - `plutil -lint` OK for all three.
>   - `launchctl print` shows `user/501/com.corbanu.initiative-owner` with path `<root>/owner.plist`, Background session.
>   - `gui/501/com.corbanu.initiative-owner.manager` loads from `~/Library/LaunchAgents/…manager.plist` (Aqua).
>   - `gui/501/com.corbanu.initiative-owner-login` is loaded, last exit 0.
>   - No `print-disabled` override in either domain.
> - **Login agent, live** (`live/login-agent-check.txt`):
>   - Right after a completed tick I ran `launchctl bootout user/501/com.corbanu.initiative-owner` and then `launchctl kickstart gui/501/com.corbanu.initiative-owner-login`.
>   - The job came back in `user/501` from `<root>/owner.plist`, and the agent exited 0.
>   - The next ticks were normal, with no `interrupted_tick`.
> - **Idle watch** (`live/watch-ticks.jsonl`, 05:22:30Z-05:37:47Z):
>   - Owner: 28 ticks, all `ACTIVE` with `unresolved_holds: []`, hold null, 0 errors, `firing: interval`. Median 3.3 s, max 3.9 s.
>   - Manager: 30 ticks, hold null, 0 errors. All were `IDLE` except 2 `BUSY` skips, when the owner held the admission lock (normal). Median 0.9 s.
>   - No manager cycle started, and no worker was launched. Both stderr logs are empty.
> - **Not done:** I did not reboot the Mac. `sfltool dumpbtm` needs admin rights; one attempt hung and was stopped after 25 s, so Login Items approval is unverified.

## Open, not done

- The Mac was not rebooted; the post-login reload was simulated (bootout + login-agent kickstart).
- Login Items approval is unverified (`sfltool dumpbtm` needs admin rights).
- The stored security workstream text (`pause`, "Owner paused" history) is uncorrected; a manager briefing can still conclude security is paused.
- Live dashboard publication is blocked (detached source checkout owned by another session); the server still shows the 2026-10-06 snapshot and a sync-failed marker.
- The manager stays on standby with no allocations, and runs only in Travis's GUI session (login dependency kept, his decision).

Verbatim open items as recorded:

Verbatim, `qa/initiative-control/coordinator-stale-state-20261008.md` lines 52-56:

> **Still open:** the stored security workstream text (`pause`, "Owner paused" history) cannot be changed through
> any CLI operation. A raw SQLite edit was not made. So until that text is corrected, a manager briefing can
> still conclude that security is paused. Fixing it needs either a small owner operation, for example to update a
> workstream's text fields with evidence, or Travis's approval for a one-off audited state edit. The fix doesn't
> affect dispatch: the stream is `mode: enabled`, and the manager has no security allocations.

Verbatim, `qa/initiative-control/management-bootstrap/owner-manager-reboot-safe-20261008.md` lines 84-84:

> - **Not done:** I did not reboot the Mac. `sfltool dumpbtm` needs admin rights; one attempt hung and was stopped after 25 s, so Login Items approval is unverified.
