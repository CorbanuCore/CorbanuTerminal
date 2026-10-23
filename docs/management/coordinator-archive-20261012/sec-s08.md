You are the broker-lane worker for sprint PF-27-S08 (P1 security hardening). Read sec-common.md (binding: keychain
rule, disk hygiene, per-sprint gate), sec-win3.result.md, sec-win4.result.md and sec-win6.result.md in
/Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/, the corbanu-terminal-development skill
(/Volumes/CorbanuDrive/Corbanu/.codex-work/corbanu-terminal/home/skills/corbanu-terminal-development/SKILL.md), and the
sprint record docs/sprints/current/p1-security-hardening/pf-27-s08-windows-broker-restricted-token.md on origin/main.
Its dependency PF-27-S07 is archived (PR #329). Allocate the sprint (worktree/branch/base per the procedure), then
deliver its execution mandate: the Windows broker runs under a token (restricted token or AppContainer — pick per the
record's analysis, justify it) that cannot write outside its own runtime state, open the user's other processes, or start
work via WMI / Task Scheduler / out-of-process COM, while it still serves its pipes, reaches upstreams and gets stored
keys. Behind the existing default-off flags; Permissive, macOS and Linux unchanged.
Note from sec-win6 (#327): the broker now starts with a different parent process, which EDR may flag as PPID spoofing;
consider this when choosing the launch mechanism and document it.
Coordinate: sec-win7 is changing windows-sandbox / exec code for #300 (PR #331) and then #301/#304 (PR #326). Merge
origin/main often; avoid editing their files except at conflict resolution.
Gate (per sec-common and the sprint record): slices as PRs, each with Windows clippy -D warnings, Linux clippy on the RTX
box, focused tests, one independent Opus 5.5 High review; windows-security-probes CI coverage where runners allow; then a
real-Windows gate run from a normal (medium-integrity) session driving GLM 5.2 via the repo tmux SOP, with short SOP demo
videos (leak-scanned, uploaded to the `demos` release, indexed in qa/demos/index/PF-27-S08.md).
Real Windows 11 machine (toolchain under C:\CorbanuQA; keep C:\CorbanuQA\target; use your own dir C:\CorbanuQA\s08\):
  ssh -i ~/.ssh/ambient_windows_desktop_ujs3hc4 -o IdentitiesOnly=yes user@desktop-ujs3hc4.tail36b615.ts.net
Password (if ever needed) only via PW="$(corbanu vault auth-helper "drone windows password")" over stdin; Z.AI key the same
way into a user-only file; never print either. Merge each PR when green (`gh pr merge N --merge --match-head-commit <sha>`;
never force-push; never rm -f). Set the sprint to `ready` when the gate passes; do NOT archive (Travis accepts).
Clean up Windows dirs/tasks/users when done. Report as in sec-common.md, including known limits for Travis.
