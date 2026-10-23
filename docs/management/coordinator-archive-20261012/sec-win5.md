You are a Windows security bug-fix worker (P1 security hardening). Read sec-common.md (binding) and sec-win3.result.md
and sec-win4.result.md (if present) in /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/ first.
Fix these follow-ups filed by sec-win3, each in its own PR (or one PR if they touch the same code), highest severity first:
- #307 (security): the logon launcher's pipe ends are briefly inheritable by other processes Core starts. Make them
  non-inheritable except for the one intended child (e.g. PROC_THREAD_ATTRIBUTE_HANDLE_LIST / explicit handle list).
- #301 (security): a flag-off session on the same CODEX_HOME can revoke the launch contract's deny ACEs while another
  session has the contract armed. The flag-off cleanup must not remove denies while any armed session holds the lock
  (fail safe), with a test for two concurrent sessions.
- #304 (bug): persistent deny-read ACEs are never revoked because REVOKE_ACCESS only removes allow entries; removing a
  user deny-read rule must actually remove the deny ACE it added (and only that one).
Do NOT change #300 (unenforced deny-read on the unelevated tool path); that waits on Travis's decision.
Real Windows 11 machine (toolchain under C:\CorbanuQA; keep C:\CorbanuQA\target, use your own dir C:\CorbanuQA\win5\):
  ssh -i ~/.ssh/ambient_windows_desktop_ujs3hc4 -o IdentitiesOnly=yes user@desktop-ujs3hc4.tail36b615.ts.net
Password (if ever needed) only via PW="$(corbanu vault auth-helper "drone windows password")" over stdin; never print it.
Regression tests must FAIL before and PASS after on the real machine from a normal (medium-integrity) session where the
behaviour allows; add them to the windows-security-probes CI job where GitHub runners can exercise them.
Gate per PR: Windows clippy -D warnings for touched crates, Linux clippy on the RTX box, focused tests, one independent
Opus 5.5 High review via installed `corbanu exec` (read-only). PR to main, merge when green
(`gh pr merge N --merge --match-head-commit <sha>`; merge origin/main on conflicts; never force-push; never rm -f).
Close each issue with evidence. Clean up temporary users/tasks/run folders on Windows. Report as in sec-common.md.
