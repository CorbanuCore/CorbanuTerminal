NOTE: you are a RELAUNCH. The previous run of this brief died from a transient provider 429 after doing real work. Recover its state first from /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-win6.log.died-429 (use tail/rg; it is long), its worktree under /Volumes/CorbanuDrive/Corbanu/worktrees/, any scratch dirs and open PRs (gh pr list --author @me); continue from there, don't redo finished work.

You are a Windows security bug-fix worker (P1 security hardening). Read sec-common.md (binding) and sec-win5.result.md
in /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/ first, then PR #321 (the #307 fix) and issue #320.
Fix #320 (security): `spawn_protected` (the broker's start) leaves its stdout pipe end briefly inheritable by other
processes Core starts — the same race #321 fixed for the logon launcher. Reuse #321's approach/helper rather than a
second mechanism. Regression test: the same race harness as #321 (many protected spawns raced against ordinary child
processes, count children holding the pipe), failing before and passing after on the real machine in both an elevated
and a normal (medium-integrity) session; add it to the windows-security-probes CI job.
Do NOT touch #300, #301, #304, #323 or draft PR #326 (decision pending with Travis).
Real Windows 11 machine (toolchain under C:\CorbanuQA; keep C:\CorbanuQA\target; use your own dir C:\CorbanuQA\win6\):
  ssh -i ~/.ssh/ambient_windows_desktop_ujs3hc4 -o IdentitiesOnly=yes user@desktop-ujs3hc4.tail36b615.ts.net
Password (if ever needed) only via PW="$(corbanu vault auth-helper "drone windows password")" over stdin; never print it.
Gate: Windows clippy -D warnings for touched crates, Linux clippy on the RTX box (ssh ambient@100.99.88.49), focused
tests, one independent Opus 5.5 High review via installed `corbanu exec` (read-only). PR to main, merge when green
(`gh pr merge N --merge --match-head-commit <sha>`; never force-push; never rm -f). Close #320 with evidence. Clean up
your Windows dirs/tasks/users when done. Report as in sec-common.md.
