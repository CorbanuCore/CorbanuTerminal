You are the Task Node worker for the P0 security-levels work. Read the tasknode-usage skill
(/Volumes/CorbanuDrive/Corbanu/.codex-work/corbanu-terminal/home/skills/.system/tasknode-usage/), tasknode-round3.result.md
and tn-privinf.result.md in /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/, and sec-common.md (keychain and
secret rules are binding: use Task Node credentials only via the existing configured flow; never print secrets).
Travis's standing rules: Task Node itself moves tasks through stages; our job is to submit evidence of completion. Do not
submit against tasks already closed/rewarded. If a task's scope does not match our real work, create or request new,
right-sized tasks that reflect the actual breakdown, and keep a mapping from each task to the sprint, PRs and gate records.
Work to cover (all merged to main in CorbanuCore/CorbanuTerminal; use `gh` and docs/sprints + qa/security-levels for
evidence): PF-23-S01/S02/S03, PF-24-S02/S03, PF-25-S01 (and PF-25-S02 if #261 has merged), PF-27-S02/S04/S05, PF-28-S01/S02,
PF-29-S01/S02, PF-30-S01/S02/S03, PF-33-S01/S02, PF-41-S01, PF-13-S07 saved-level matrix (#255).
Do: 1) read-only inventory of current Task Node tasks and states; 2) map existing open/accepted tasks to this work;
3) request/create right-sized tasks for unmapped work (group sensibly, e.g. one per sprint or per track); 4) submit
evidence (merged PRs, merge commits, gate records, test counts, demo video links) for every accepted task whose work is
done; idempotent, no duplicates; 5) write the mapping to qa/initiative-control/ in a docs-only PR and merge it when green
(`gh pr merge N --merge --match-head-commit <sha>`). Report what was submitted, rewards, and exactly which new tasks
need Travis to accept in Task Node.
