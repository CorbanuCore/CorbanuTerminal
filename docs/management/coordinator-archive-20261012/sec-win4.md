You are the Windows qualification worker for PF-27-S07. Read sec-common.md (binding) and win-gate1.md,
win-gate1.result.md and sec-win3.result.md in /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/ first, then
qa/security-levels/sprints/PF-27-S07/README.md on origin/main ("Still not done": the GLM 5.2 tmux run and the SOP videos
need a real Windows machine, as for PF-27-S06).
Real Windows 11 machine, toolchain already provisioned under C:\CorbanuQA (a main build is kept in C:\CorbanuQA\target):
  ssh -i ~/.ssh/ambient_windows_desktop_ujs3hc4 -o IdentitiesOnly=yes user@desktop-ujs3hc4.tail36b615.ts.net
If a Windows password is required it is vault label "drone windows password", only via
PW="$(corbanu vault auth-helper "drone windows password")" piped over stdin; never print it. Z.AI key the same way
(stdin into a user-only file, never on a command line or in logs).

1. Build current origin/main on Windows (record the commit). Run the PF-27-S07 functional checks per the repo tmux SOP
   driving GLM 5.2 (provider zai) on a disposable CODEX_HOME, from a NORMAL (medium-integrity) session as sec-win3 did,
   with the PF-27-S07 flags on (isolated_credential_broker, secretless_agent_launch): new threads protected from
   creation, the broker never openable by a same-user unprivileged process or the sandbox token, and the CODEX_HOME
   deny removed when the flag is off (SDDL before/after). Also confirm agent commands still work in a normal session and
   the vault stays denied under workspace-write (regression of #294/#295).
2. Record short SOP demo videos of those behaviours, leak-scan each (recorder scan + a second scan on the Mac), upload to
   the `demos` release, index them in qa/demos/index/PF-27-S07.md, and add a "Real Windows gate run" section to the
   PF-27-S07 sprint record (measured results, what CI covers and what it doesn't).
3. If a check fails: file an issue (label security where relevant) with evidence, don't fix code here; record it in the
   gate section and report it.
4. Docs-only PR, merge when green (`gh pr merge N --merge --match-head-commit <sha>`; never force-push). Set the sprint
   status to `ready` only if every check passed; do NOT archive unless the plan/sprint checkers allow it.
Clean up temporary users, scheduled tasks, key files and run folders on Windows; keep C:\CorbanuQA\target. Report as in
sec-common.md: commit tested, what passed/failed, video links, anything for Travis.
