You are the untrusted-content lane worker (round 2). Read /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md
first (binding), then the previous worker's report:
/Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-taint.result.md (PF-30-S01 merged as PR #178, flag
`source_envelopes`, merge commit b96b23344b).

1. Housekeeping (small docs PR, or fold into your sprint PR): record the #178 merge commit in the PF-30-S01 QA file,
   mark PF-30-S01 completed and archive it per the repo's archive rules, and allocate PF-30-S02 (worktree, branch,
   base, write_scope in the sprint file and the plan front matter). The plan rewrite (PR #173) raised the global
   concurrent-sprint cap to 5 pending Travis's confirmation; if the checker still blocks you, adjust within that
   documented rule rather than leaving the sprint as draft.
2. Implement PF-30-S02 "Persistent taint across summaries and memory"
   (docs/sprints/current/p0-security-levels/pf-30-s02-persistent-taint-and-memory.md). Start with the slice the S01
   worker proposed: persist each message's origin in the session (rollout) file so resumed history keeps its correct
   standing, then summaries/compaction and memory. Keep old rollouts readable (treat missing origin as untrusted).
3. Small fix in passing: with the flag off, Moderate stops with "source admission registry is full or poisoned";
   make the message say what's actually happening (fail closed because the feature is off). Separate commit.
4. Per-sprint gate per sec-common.md (focused tests, GLM 5.2 tmux run, one independent Opus 5.5 High review, demo
   videos via scripts/demo_video.py). PR to main; merge when green. Update sprint files and index rows.
Report as in sec-common.md.
