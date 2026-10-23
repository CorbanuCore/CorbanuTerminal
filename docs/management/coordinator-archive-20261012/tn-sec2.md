You are the Task Node worker, round 2 for P0 security. Read tn-sec1.result.md and the tasknode-usage skill
(/Volumes/CorbanuDrive/Corbanu/.codex-work/corbanu-terminal/home/skills/.system/tasknode-usage/) and sec-common.md (binding).
Travis accepted the nine P0SEC-TN tasks and the coordinator ran submit.sh: all nine evidence submissions returned ok.
Now: poll each task's state; answer every verification request truthfully from the merged PRs, gate records and the
map at qa/initiative-control/p0-security-levels/tasknode-task-map.md (idempotent, one answer per request, keep
receipts next to the existing ones in .codex-work/workers-20261007/tn-sec1/); fix the PF-13-S07 route-count mismatch
(10 vs 11) in the evidence record if it is ours, via a docs PR. Keep polling until each task is Rewarded or plainly
waiting on a human reviewer, up to ~3 hours. Report states, rewards and anything needing Travis.
