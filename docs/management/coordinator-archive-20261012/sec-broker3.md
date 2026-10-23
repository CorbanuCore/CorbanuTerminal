You are the broker lane worker (round 3). Read /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md first
(binding), then /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-broker2.result.md (PF-27-S02 merged as #191,
flag `secretless_agent_launch`; PF-27-S04 archived; PF-27-S05 and PF-27-S06 created).

1. Close out what PF-27-S02 can close without Travis:
   - Linux end-to-end GLM 5.2 agent run on a host with Landlock: use the RTX box (`ssh ambient@100.99.88.49`, Ubuntu,
     build with `~/corbanu-rtx/build.sh` pattern but on your own branch/checkout under ~/corbanu-rtx/; disk is ~96% full,
     clean up your own build output afterwards). Don't disturb `~/corbanu-rtx/integration` or `~/corbanu-rtx/bin`.
   - Tick the CI checkbox and record the evidence in the same docs commit as the next PR.
   - The two open decisions (unsandboxed routes: MCP servers/hooks/`!`/app-server command/exec; Claude panes) are
     Travis's. Record the recommendation in the sprint; leave them open. Do not block on them.
2. Then implement PF-28-S01 "Central secret and protected-output gate"
   (docs/sprints/current/p0-security-levels/pf-28-s01-central-secret-output-gate.md), under the per-sprint gate.
   Coordinate shared files (core/src/config/mod.rs, feature registry, config schema) by merging origin/main often
   and keeping hunks minimal.
Per-sprint gate per sec-common.md (focused tests, GLM 5.2 tmux run, one independent Opus 5.5 High review, demo videos
via scripts/demo_video.py). Fresh worktree off origin/main; own CARGO_TARGET_DIR. PR to main; merge when green (stale
CodeQL checks on PRs can be ignored). Update sprint files and index rows. Report as in sec-common.md.
