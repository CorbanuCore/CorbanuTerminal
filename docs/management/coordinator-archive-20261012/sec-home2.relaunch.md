RECOVERY NOTE (coordinator, 2026-10-12T11:07:16Z): your previous run of this brief hung at ~11:01Z when a transient macOS
access error broke the claude-plan auth helper (not your fault). Your full transcript is /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-home2.log.died-20261012T1107 — read its last
~400 lines (and grep it for PR numbers, worktrees and glitch dirs) to recover your state before doing anything else.
Then FINISH economically — Claude credits are low this week and the coordinator is doing an orderly pause:
- get your open PR(s) green and merged per the rules below (re-run a failed job once with `gh run rerun <id> --failed` if it
  is the known intermittent Linux core-security test `pf_23_s01_typing_a_protected_command_into_a_running_shell_needs_the_human`);
- push any unpushed finished work as a PR; do not start new scope beyond this brief; skip optional extras (videos only if
  the brief requires them for a user-visible change already made);
- leave nothing uncommitted: every worktree you own must end merged or pushed as an open PR (say which in the report);
- clean up targets/worktrees of merged work; write your report as in sec-common.md.

You are a security bug-fix worker. Read /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md (binding; Linux work on glitch, `ssh glitch`, max -j10; note the rule on merging main), sec-home.result.md in that folder, then issues #594, #598, #607 and #603 (`gh issue view N`) and merged PRs #592, #596, #599. One PR per issue or small related group, off origin/main.

- #594 (Linux): a Corbanu home reached through a symlink can still be replaced. Extend #592's mount-point protection to the real home path and each folder above it inside a writable root (resolve the link; protect both spellings), with a real-bwrap test reproducing the #587 attack through a linked home before/after.
- #598 (macOS + Linux): commands approved under #540 (a person's one-command approval at Moderate) get no protection against renaming folders above the home. Keep the runtime entries and the home itself non-renameable for those commands too (owner rule: runtime entries are never writable, not even with a grant), real-sandbox tests both platforms.
- #607: #582's remaining placeholder races (a host rename over a live placeholder; another run unlinking one) — fix or document precisely with tests.
- #603: implement only the non-decision edge cases listed there. The two owner decisions in it are NOT yours: leave them as they are now and keep the issue open for them.
Gate: regression tests (fail before / pass after) with real `sandbox-exec` on macOS and real bwrap on glitch; focused tests incl. the core security CI selection with TMPDIR=/tmp and default; macOS clippy and Linux clippy -D warnings on glitch; one independent Opus 5.5 High read-only review per PR via installed `corbanu exec` (iterate to APPROVE). Permissive unchanged (test); if a change alters Permissive, do NOT merge: title it "— needs security sign-off" and stop. Concurrent sandbox workers: agg-dec2, sbx-links2, main-ci2 — note overlaps. Do heavy builds on glitch where possible.
New worktrees under /Volumes/CorbanuDrive/Corbanu/worktrees/. Merge only when ALL checks are green, never with checks pending (`gh pr merge N --merge --match-head-commit <sha>`; never force-push; never rm -f; never pkill broadly). Close issues with evidence. Remove build targets when done. Report as in sec-common.md.
