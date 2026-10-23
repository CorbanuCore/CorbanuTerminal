You are a Windows security bug-fix worker (P1 security hardening). Read sec-common.md (binding), sec-win7.result.md
and sec-win8.result.md in /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/ first, then issues #323 and #345
(`gh issue view N`) and the merged PRs #326 and #343 they follow up on.
Fix both, each in its own PR, highest severity first:
- #345 (security): over SSH, the window-station/desktop grant is made to the sandbox user's SID, so the sandboxed commands
  themselves hold it, it lasts for the whole SSH session, and other Windows users' sandboxed commands could open the
  station. Scope it to the runner's own logon SID (as the secondary logon service does on WinSta0) and to the runner's
  lifetime (remove it when the runner exits), and make sure sandboxed child commands don't get it. Also handle
  `sandbox_private_desktop = false` over SSH (currently fails; noted on #345) or document why not.
- #323 (security): an armed contract's deny-read entries on paths outside CODEX_HOME aren't protected from another
  CODEX_HOME's sync. Implement the issue's proposed fix (each armed contract also holds a shared lock that every sync
  honours before removing any entry it didn't add for that home) or a better one with the same guarantee; test two homes
  concurrently.
Regression tests must FAIL before and PASS after on the real machine, elevated and in a normal (medium-integrity) session,
over SSH and in the console session where relevant; add them to windows-security-probes CI where runners allow.
Real Windows 11 machine (toolchain under C:\CorbanuQA; keep C:\CorbanuQA\target; use your own dir C:\CorbanuQA\win9\):
  ssh -i ~/.ssh/ambient_windows_desktop_ujs3hc4 -o IdentitiesOnly=yes user@desktop-ujs3hc4.tail36b615.ts.net
Password (if ever needed) only via PW="$(corbanu vault auth-helper "drone windows password")" over stdin; never print it.
Gate per PR: Windows clippy -D warnings for touched crates; Linux clippy on the RTX box (ssh ambient@100.99.88.49) — the
box is currently OFFLINE, so if it's still unreachable rely on PR CI's Linux clippy and say so; focused tests; one
independent Opus 5.5 High review via installed `corbanu exec` (read-only). Merge only when ALL checks are green — never
with checks pending (`gh pr merge N --merge --match-head-commit <sha>`; never force-push; never rm -f). Close each issue
with evidence. Clean up your Windows dirs/tasks/users. Report as in sec-common.md.
