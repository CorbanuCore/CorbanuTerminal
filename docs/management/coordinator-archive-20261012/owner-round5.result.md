The loop is on and has processed real work. The owner is armed at generation 5 (`tmux-workers`, `manager_enabled: true`) with no holds. The manager prepared `owner-first-check-01-r3`, and the owner ran it through ACK, START and RETURN. The result is now `returned` in the coordinator.

**I had to adapt the documented promotion recipe.** It assumes the owner starts armed in fixture-only mode. Here it was already running real workers with a held action, and the hold had to be cleared before anything could be restaged. So I disarmed generation 3 and uninstalled the schedule first. Without that, the old code would have put the hold straight back. The preflight's items 1–4 ran unchanged. Item 5 is my own variant, written for an owner that is off with its schedule uninstalled (`round5/live/restage_preflight.py`), and the restage script is likewise adapted (`restage-recipe.py`).

**Hold resolution**
- The old check couldn't stop the held worker (`w-ylg59fqs`). It refused with `host_changed` because the Mac's recorded boot time had shifted by 62 ms. I stopped it through its own socket only: `/quit` (exit status 0), then `kill-server`. Both processes are gone and the socket is dead.
- The new `owner_daemon.py --resolve-hold` cleared both holds (`wrong_ack` and `operation_held`):
  - `resolution_evidence_digest` is `5c7bbe06…9da7a` and `resolved_at` is 1791032927.80.
  - The action went from `dispatch_uncertain` to `failed`.
  - Its four operations went from `held` to `resolved`; no rows were deleted.
- The command only accepts the exact open hold reasons, the recorded claim and allocation, and a worker proven stopped. It fails the claim and never resumes or relaunches it.

**ACK fix and tests**
- The exact ACK line is now accepted with surrounding spaces or newlines. It is also accepted when the final message is empty and an earlier message in the same turn carried it. Bridge ACKs are still byte-exact.
- Still refused: a wrong action id, digest, model or effort, extra content, other messages, unusual padding, and ACKs from outside the turn.
- New tests:
  - `AckTokenTests`, including the exact message shape that caused the round-4 hold.
  - Two tmux tests: a newline ACK reaches START, and a newline ACK with the wrong effort holds.
  - About a dozen refusal cases for `--resolve-hold`.
- The full suite passed in a clean worktree: 906 tests OK.
- Code review (`claude-opus-5-5-plan`) first asked for changes: the stopped-worker check passed if the worker's record files were deleted. After the fix it approved.

**Qualification and review:** I reran round 4 run 3 on the PF-83 VM with the new package `03aefd15…2a46`.
- Fence, hosts, credentials and share handling were the same, and the VM was restored exactly (digests, inodes and the file-metadata list all match).
- There were zero direct public connections.
- The positive control returned, and the broker-down control failed safely.
- The main run had 13 ticks, no holds, and all 23 model responses traced through the broker.
- In that run, the model sent its ACK exactly the way round 4 broke on (trailing newline, empty final). The new check accepted it.
- The independent review said **QUALIFIED**, with counts 1/1/1. Its limits are recorded in `qual/review-notes.md`; none of the VM hardening follow-ups were applied.

**New generation:** 5, after preflight items 1–5 all passed. The schedule is ticking on its interval with no errors.

**Live receipts (UTC):** claim at 13:12:55 → **ACK** 13:14:27 (exact line) → **START** 13:14:34 → **RETURN** 13:15:29, returned to the coordinator at 13:15:32. Every tick showed `ACTIVE` with no holds. The task was a read-only docs check in a disposable worktree (`owner-action-r5-20261003`). All four commands exited 0 and the worktree is still clean. I then closed the idle worker cleanly.

**Commits** (fast-forward from `3c8736914f`, all pushed to `integrate/management-workstreams-20260911`): `54e6a90229`, `841548849e` and `491046fe4d` (code and tests), and `e477859183` (the record). The four uncommitted files were never staged.

**Dashboard: skipped.** `sync-source.sh` publishes from this worktree's working copy, which still has the four uncommitted files.

**Still open**
1. **Boot-time drift:** any worker running when the recorded boot time shifts will hold with `host_changed`, and the normal close refuses it. The workaround is the manual socket stop plus `--resolve-hold`. Fixing it changes the package, so it would need requalifying.
2. **Manager formatting:** 1 of 2 manager cycles failed (`invalid_json`) because it wrapped its JSON in a code fence. I reconciled it with `fail_manager` and the retry succeeded.
3. **Routing:** new actions still default to hand, so each owner action needs an explicit handoff. The owner also doesn't close workers after RETURN.
4. **r3 verification:** r3 is returned but the manager hasn't accepted or rejected it yet.
5. **Leftovers:** VM hardening, `slack-receiver-02`, and the reviewer's non-blocking notes.

Evidence is in `.codex-work/owner-manager-enable-20261002/round5/`.