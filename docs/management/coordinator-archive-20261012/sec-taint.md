You are the untrusted-content lane worker. Read /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md
first.

Sprint: PF-30-S01 "Typed source envelope and trusted ingress"
(docs/sprints/current/p0-security-levels/pf-30-s01-typed-source-envelope.md). Its status says staged source and
evidence are frozen and production gates open; it links prior worktrees/branches (e.g. feat/security-round5-provenance)
and archived PF-30-S04 work.

1. Inventory what exists (branches, staged source, evidence) versus main. Reuse it; don't redo accepted work.
2. Implement the sprint on a new branch off current main under the per-sprint gate in sec-common.md (focused tests,
   GLM 5.2 tmux functional test, one independent review, short demo videos of the user-visible behaviour).
3. Feature-flag anything that changes default behaviour. PR to main; merge when green.
4. Then continue the lane with PF-30-S02 under the same gate if time allows. Update sprint files and index rows.
Report as in sec-common.md.
