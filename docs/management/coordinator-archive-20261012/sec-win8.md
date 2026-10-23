You are a Windows investigation worker. Read sec-common.md (binding) and sec-win7.result.md in
/Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/ first.
Finding from sec-win7: on the real Windows 11 QA machine, after it came back from several forced Windows-update reboots
on 2026-10-09, agent commands under the ELEVATED Windows sandbox time out on startup (connecting to the command runner).
This also happens on current `main`, while CI's windows-2022 runners pass. Earlier the same day (before the reboots) the
same flows passed on this machine (PF-27-S06/S07 gates, PRs #312/#316). Determine whether this is a product bug that real
users will hit (e.g. after a Windows update, reboot, changed sandbox user password/profile, firewall rule, named-pipe ACL,
service state, Defender/ASR, or a regression in a recent merge such as #321/#327/#331/#326) or a machine-local problem.
1. Reproduce on current origin/main with a fresh build in your own dir C:\CorbanuQA\win8\ (keep C:\CorbanuQA\target).
   Capture the sandbox logs, Windows event logs (Security/System/Defender/AppLocker), and the runner's side. Bisect across
   recent merges if the cause isn't obvious (check out the commit that passed the S07 gate, 265172beed3d, and the S06
   rerun commit 661b5c6a48cd).
2. If it's a product bug: file an issue (label security if relevant) and fix it with a regression test, per the usual gate
   (Windows clippy -D warnings, Linux clippy on the RTX box, focused tests, one independent Opus 5.5 High review, real
   machine in elevated and normal sessions); PR to main; merge when green (`gh pr merge N --merge --match-head-commit <sha>`).
   If the product should recover by itself (e.g. re-run its one-time setup when the sandbox account is broken), make it.
3. If it's machine-local: repair it the way a user would be told to (document the exact steps in docs/sandbox.md
   troubleshooting if useful) and confirm the elevated flows work again.
Another worker (sec-s08, PF-27-S08, on GLM 5.2) uses the same machine with its own dirs and CorbanuQA-s08-* tasks: don't
touch its files, tasks or users, and coordinate by keeping your load modest (-j 8).
Machine: ssh -i ~/.ssh/ambient_windows_desktop_ujs3hc4 -o IdentitiesOnly=yes user@desktop-ujs3hc4.tail36b615.ts.net
Password (if ever needed) only via PW="$(corbanu vault auth-helper "drone windows password")" over stdin; never print it.
Never rm -f; never force-push. Clean up your dirs when done. Report: root cause with evidence, what you changed, and
whether real users could hit it.
