You are the P1 security integration owner for two sprints. Read
/Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md (binding) and the corbanu-terminal-development
skill (/Volumes/CorbanuDrive/Corbanu/.codex-work/corbanu-terminal/home/skills/corbanu-terminal-development/SKILL.md) for
the plan/sprint/archive procedure and checkers.
Travis ACCEPTED PF-27-S06 and PF-27-S07 on 2026-10-08 (in chat with the coordinator), including their documented known
limits. Both sprint records are `ready` with real-Windows gate runs passed (PRs #312 and #316; evidence under
qa/security-levels/sprints/PF-27-S06/ and PF-27-S07/).
1. Receive and archive both sprints exactly per the repo's plan/sprint procedure in docs/sprints/current/p1-security-hardening/
   (and any initiative-control ledger/indexes it requires). Record Travis's acceptance (date, "accepted with known limits",
   the limits list by reference) in each sprint record. Keep open follow-ups linked as follow-ups, not blockers:
   #300 (decided: fail closed, being implemented), #301/#304 (draft PR #326, decided: rule-driven removal), #320 (PR #327),
   #323, and PF-27-S08/S09 (planned).
2. Run every plan/sprint checker the repo has; they must pass. If a checker refuses to archive, report exactly why and
   don't force it.
3. Docs-only PR to main; merge when green (`gh pr merge N --merge --match-head-commit <sha>`; never force-push; never
   rm -f). Other workers are concurrently changing Windows code and accounting docs; touch only the sprint/plan files.
Report: what was archived, checker output, PR number.
