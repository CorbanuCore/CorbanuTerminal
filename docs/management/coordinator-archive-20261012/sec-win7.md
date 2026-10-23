You are a Windows security worker (P1 security hardening). Read sec-common.md (binding), sec-win3.result.md and
sec-win5.result.md in /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/ first, then issue #300, issues #301 and
#304, and draft PR #326 (its description, review reports under sec-win5-review-* in that folder, and the S1 finding).
Travis decided both on 2026-10-08. Do them in this order, each as its own PR:

A. #300 — fail closed. On the unelevated Windows sandbox tool path (`SandboxAttempt::env_for` route), refuse any agent
   command whose profile has deny-read entries (user deny paths or PF-23 protected-path rules) that the unelevated backend
   cannot enforce, exactly as `process_exec_tool_call` already does. The refusal message must be plain and actionable:
   say the command was not run because this Windows sandbox mode can't block reading the protected files, and how to
   switch to `[windows] sandbox = "elevated"`. No change for elevated, macOS or Linux. Regression tests (fail before, pass
   after on the real machine in a normal session; CI where runners allow). Update the user docs for the Windows sandbox.

B. #301/#304 — finish draft PR #326 with option 1 (rule-driven removal): pass the configured deny-read RULES alongside
   their expanded paths through the sandboxing crate, core exec, the windows-sandbox backends and the elevated setup
   payload; record which rule created each deny entry; remove an entry only when its rule is gone from config — never
   because a scan didn't see a path (S1). Keep #326's safe parts (exact ACE matching incl. split directory ACEs, ownership
   record in `.sandbox-secrets`, no link following, no removal while an armed contract holds the lock). Add a test for
   S1 (a held-open folder hides a secret from the scan; its deny must stay). Reuse #326's branch (merge origin/main; never
   force-push) and take it out of draft when the gate passes. Close #301 and #304 with evidence. #323 is out of scope
   unless the fix naturally covers it (then say so and close it with evidence).

Another worker (sec-win6) is fixing #320 (`spawn_protected` pipe race, PR #327) in nearby code; merge origin/main often.
Real Windows 11 machine (toolchain under C:\CorbanuQA; keep C:\CorbanuQA\target; use your own dir C:\CorbanuQA\win7\):
  ssh -i ~/.ssh/ambient_windows_desktop_ujs3hc4 -o IdentitiesOnly=yes user@desktop-ujs3hc4.tail36b615.ts.net
Password (if ever needed) only via PW="$(corbanu vault auth-helper "drone windows password")" over stdin; never print it.
Gate per PR: Windows clippy -D warnings for touched crates, Linux clippy on the RTX box (ssh ambient@100.99.88.49),
focused tests, one independent Opus 5.5 High review via installed `corbanu exec` (read-only), real-machine tests in both
an elevated and a normal session. Merge when green (`gh pr merge N --merge --match-head-commit <sha>`; never rm -f).
Clean up your Windows dirs/tasks/users when done. Report as in sec-common.md.
