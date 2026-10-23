You are the broker-lane worker for sprint PF-27-S09 (P1 security hardening): Windows model-client auth through the
broker. Read sec-common.md (binding: keychain rule, disk hygiene, per-sprint gate), sec-s08.result.md, sec-win6.result.md
and sec-win8.result.md in /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/, the corbanu-terminal-development
skill (/Volumes/CorbanuDrive/Corbanu/.codex-work/corbanu-terminal/home/skills/corbanu-terminal-development/SKILL.md), the
sprint record docs/sprints/current/p1-security-hardening/pf-27-s09-windows-model-client-auth.md and the archived
PF-27-S05 record it ports.
Key path is DECIDED (Travis, 2026-10-09): option (c) — the Windows broker reads stored keys from Credential Manager itself,
like the macOS/Linux brokers read the OS keyring. A docs PR (#360, by sec-s08c) is recording it; merge origin/main once
it lands. Allocate the sprint per the procedure, then deliver its mandate: with `broker_model_auth` on Windows, Core's model
requests are authorized by the broker over named pipes (with the PF-27-S06 server process-id check); Core holds
references, not raw keys. Behind the existing default-off flags; Permissive, macOS and Linux unchanged.
Also: #345/#343 made the elevated sandbox work over SSH; make sure brokered model auth works over SSH and in the console
session.
Gate: slices as PRs, each with Windows clippy -D warnings, Linux clippy on the RTX box (ssh ambient@100.99.88.49), focused
tests, one independent Opus 5.5 High review; windows-security-probes CI coverage where runners allow; then a real-Windows
gate from a normal (medium-integrity) session driving GLM 5.2 via the repo tmux SOP, with short SOP demo videos
(leak-scanned, uploaded to `demos`, indexed in qa/demos/index/PF-27-S09.md). Merge ONLY when ALL checks are green — never
with checks pending, whatever the reason (`gh pr merge N --merge --match-head-commit <sha>`; never force-push; never rm -f).
Real Windows 11 machine (toolchain under C:\CorbanuQA; keep C:\CorbanuQA\target; use C:\CorbanuQA\s09\):
  ssh -i ~/.ssh/ambient_windows_desktop_ujs3hc4 -o IdentitiesOnly=yes user@desktop-ujs3hc4.tail36b615.ts.net
Password (if ever needed) only via PW="$(corbanu vault auth-helper "drone windows password")" over stdin; Z.AI key the same
way into a user-only file; never print either. If the provider's content filter stops you while writing about attack
techniques, keep such text factual and brief. Set the sprint to `ready` when the gate passes; do NOT archive. Clean up
Windows dirs/tasks/users. Report as in sec-common.md, including known limits for Travis.
