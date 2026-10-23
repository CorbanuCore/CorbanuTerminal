RECOVERY NOTE (coordinator, 2026-10-12T11:07:16Z): your previous run of this brief hung at ~11:01Z when a transient macOS
access error broke the claude-plan auth helper (not your fault). Your full transcript is /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/throttle-fp.log.died-20261012T1107 — read its last
~400 lines (and grep it for PR numbers, worktrees and glitch dirs) to recover your state before doing anything else.
Then FINISH economically — Claude credits are low this week and the coordinator is doing an orderly pause:
- get your open PR(s) green and merged per the rules below (re-run a failed job once with `gh run rerun <id> --failed` if it
  is the known intermittent Linux core-security test `pf_23_s01_typing_a_protected_command_into_a_running_shell_needs_the_human`);
- push any unpushed finished work as a PR; do not start new scope beyond this brief; skip optional extras (videos only if
  the brief requires them for a user-visible change already made);
- leave nothing uncommitted: every worktree you own must end merged or pushed as an open PR (say which in the report);
- clean up targets/worktrees of merged work; write your report as in sec-common.md.

You are a PF-84 follow-up worker. Read /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md (binding; Linux work on glitch, `ssh glitch`, max -j10), macct-fix4.result.md and db-contention.result.md (if present) in that folder, then issues #509, #510 and merged PRs #485, #486 and the db-contention PR(s).

Owner decision (Travis, 2026-10-11): named accounts are throttled by KEY FINGERPRINT, not by account name — two names pointing at the same key share one cooldown/lease state, because providers limit by key. Implement:
1. Throttle key = salted fingerprint of the actual credential (reuse the S05 per-home salt and fingerprint code; never store or log raw keys; the request path must never open or create the vault — keep #474's "no vault rewrite" rule; if a fingerprint is unavailable for an account that predates S05, fall back to the account name and say so in docs). Migration: carry active cooldowns/leases over from name-keyed rows; idle rows may be dropped. Test two names with the same key sharing a cooldown, and two keys under one provider not sharing.
2. #509: compaction, the turn-completion check and memory extraction go through the same throttle check/record path (respect an active cooldown for their account/key, record their 429s). A test per path.
3. #510: Z.AI's transport retry on HTTP 429 must respect `Retry-After` (or stop transport retries on 429 and let the throttle own the wait). Mock-server test.
Coordinate with the db-contention change (shared state DB access patterns): base on origin/main after it merges; keep every throttle DB access short and non-blocking under contention.
Gate: focused tests with and without developer-accounting; Linux clippy -D warnings on glitch and macOS clippy; one independent Opus 5.5 High read-only review per PR via installed `corbanu exec` (iterate to APPROVE); a tmux check with a mock 429 server in a disposable home (CORBANU_TEST_NO_NATIVE_KEYRING=1): two named accounts on the same key share the cooldown; a third on a different key keeps answering. Do heavy builds on glitch; the Mac is CPU-saturated. New worktrees off origin/main under /Volumes/CorbanuDrive/Corbanu/worktrees/. Merge only when ALL checks are green, never with checks pending (`gh pr merge N --merge --match-head-commit <sha>`; never force-push; never rm -f; never pkill broadly). Close issues with evidence. Remove build targets when done. Report as in sec-common.md.
