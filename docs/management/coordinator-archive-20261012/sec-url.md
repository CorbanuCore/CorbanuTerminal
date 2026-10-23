You are a security sprint worker on the "first free" lane. Read
/Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md first (binding).

Sprint: PF-33-S01 "URL DNS and redirect policy"
(docs/sprints/current/p0-security-levels/pf-33-s01-url-dns-and-redirect-policy.md). Dependencies are met: PF-33-S03 (pure
destination-policy contract) is archived/completed, and PF-27-S02 is merged (#191, flag `secretless_agent_launch`).
Reuse the PF-33-S03 contract; don't redo it. Next in line after this is PF-33-S02 (connection pinning and alternate-
egress denial).

1. Allocate the sprint (worktree, branch, base, write_scope in the sprint file and plan front matter); both checkers
   must pass. Avoid overlap with open lanes: broker (PF-28-S01, PR #208), untrusted content (PF-30-S03 / PF-23-S01).
   Merge origin/main often; keep hunks in shared files (core/src/config/mod.rs, feature registry, config schema)
   minimal.
2. Implement it behind a default-off feature flag, under the per-sprint gate (focused tests, GLM 5.2 tmux run, one
   independent Opus 5.5 High review, demo videos via scripts/demo_video.py).
3. PR to main; merge when green (stale CodeQL checks on PRs can be ignored). Then PF-33-S02 if time allows.
Report as in sec-common.md.
