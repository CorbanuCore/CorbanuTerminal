You are the accounting integration owner for sprint PF-60-S03. Read
/Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md (binding), the corbanu-terminal-development
skill (/Volumes/CorbanuDrive/Corbanu/.codex-work/corbanu-terminal/home/skills/corbanu-terminal-development/SKILL.md) for
the plan/sprint/archive procedure, and sec-archive1.result.md in that folder (how PF-27-S06/S07 were archived).
Travis ACCEPTED PF-60-S03 on 2026-10-09 (in chat with the coordinator) after the independent code-blind acceptance and
its three targeted re-runs (PRs #292, #311, #315, #328; evidence under
qa/portfolio/agent-cost-accounting/pf-60-s03/independent-acceptance-20261008/), with these tracked as follow-ups, not
blockers: #287 (clock-behind WARN wording vs recorded admission time), #325 (rejected attempt counted as billed; bucket
past the last request reads Partial), and the body-review findings now owned by PF-60-S05 (PR #330).
1. Archive PF-60-S03 exactly per the repo procedure (sprint record + plan current-state + indexes + mkdocs nav as the
   checkers require). Record the acceptance (date, by whom, evidence links, follow-ups) in the record. Demo index:
   qa/demos/index/PF-60-S03.md (PR #332 updates it; if #332 isn't merged yet, wait for it, polling every 5 min up to 2 h,
   and don't edit that file).
2. Run every plan/sprint checker; they must pass. If a checker refuses, report exactly why; don't force it.
3. Docs-only PR to main; merge when green (`gh pr merge N --merge --match-head-commit <sha>`; never force-push; never
   rm -f). Another worker (acct-s05) is concurrently updating PF-60-S05's record; touch only S03 and the plan/index lines
   for S03.
Report: what was archived, checker output, PR number.
