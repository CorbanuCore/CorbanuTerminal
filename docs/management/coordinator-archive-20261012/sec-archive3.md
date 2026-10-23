You are the P1 security integration owner for PF-27-S09. Read
/Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md (binding), sec-archive2.result.md (how S08 was
archived) and sec-s09.result.md in that folder, and the corbanu-terminal-development skill
(/Volumes/CorbanuDrive/Corbanu/.codex-work/corbanu-terminal/home/skills/corbanu-terminal-development/SKILL.md) for the
archive procedure.
Travis ACCEPTED PF-27-S09 on 2026-10-10 (in chat with the coordinator: "Pass it"), after the independent code-blind
acceptance on the real Windows machine returned ACCEPT WITH LIMITS (PR #384,
qa/security/pf-27-s09/independent-acceptance-2026-10-09/README.md). Archive it exactly as S08 was: move the record to
docs/sprints/archive/p1-security-hardening/, status completed, a "Closure — 2026-10-10" section stating Travis's acceptance
with the known limits — the sprint's own list (best-effort key overwrite; scan covers hand-over not start-up; no Credential
Manager over SSH; decision (c) broker token can read/write the user's other generic credentials; ChatGPT refresh unit-only;
missing tracing headers/debug log on brokered Windows requests) plus the acceptance run's four limits (SSH-only runs, no
console-session Credential Manager run; post-hand-over scan; ChatGPT refresh not tested; exclusive pipe-squat scenario not
tested) — link both evidence READMEs and PRs #363/#379/#384, tick the remaining items, keep follow-ups linked as follow-ups
(open issues for: console-session Credential Manager acceptance run, exclusive pipe-squat probe, broker_model_auth flag
default-on decision — file them if missing). Note the flag is still default-off. Update the plan/index lines and anything
the checkers require. Run every plan/sprint checker; they must pass; if a checker refuses, report why and don't force it.
Touch only S09's record and S09's plan/index lines (other workers edit PF-60-S04 and AGENTS.md concurrently).
Docs-only PR to main; merge only when ALL checks are green, never with checks pending (`gh pr merge N --merge
--match-head-commit <sha>`; never force-push; never rm -f). Report: PR number, checker output, issues filed.
