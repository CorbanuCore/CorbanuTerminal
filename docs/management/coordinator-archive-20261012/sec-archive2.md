You are the P1 security integration owner for PF-27-S08. Read
/Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md (binding), sec-archive1.result.md (how S06/S07
were archived) and sec-s08c.result.md in that folder, and the corbanu-terminal-development skill
(/Volumes/CorbanuDrive/Corbanu/.codex-work/corbanu-terminal/home/skills/corbanu-terminal-development/SKILL.md) for the
archive procedure.
Travis ACCEPTED PF-27-S08 on 2026-10-09 ("accept s08", in chat with the coordinator) with its documented known limits,
after key-path decision (c) was recorded (PR #360). Archive it exactly as S06/S07 were: move the record to
docs/sprints/archive/p1-security-hardening/, status completed, a "Closure — 2026-10-09" section stating Travis's acceptance
with the known limits (link the evidence README's Known limits list and summarise the eight items sec-s08c listed,
including the correction that overwrite/delete of existing Credential Manager items was not probed), tick the remaining
items, and keep follow-ups linked as follow-ups (PF-27-S09 in progress with PR #363; the vault-lock access S09 needs).
Update the plan/index lines and anything the checkers require. Run every plan/sprint checker; they must pass; if a checker
refuses, report why and don't force it. Another worker (sec-s09) is editing the PF-27-S09 record concurrently — touch only
S08's record and S08's plan/index lines. Docs-only PR to main; merge only when ALL checks are green, never with checks
pending (`gh pr merge N --merge --match-head-commit <sha>`; never force-push; never rm -f). Report: PR number, checker output.
