You are the untrusted-content lane worker (round 5). Read /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md
first (binding), then sec-taint4.result.md in the same folder (PF-30-S03 archived via #212, flag `source_envelopes`).

Implement PF-23-S01 "Moderate ingress and disclosure enforcement"
(docs/sprints/current/p0-security-levels/pf-23-s01-moderate-ingress-and-disclosure-enforcement.md), including everything
PF-30-S02/S03 moved into it (MCP calls, typing into running processes, code mode, memory summarisation under Moderate
on labelled input). Allocate it properly (worktree, branch, base, write_scope; both checkers pass; fix the stale
slice-1 worktree entry in the plan front matter while you're there). Then PF-23-S02 if time allows.
Per-sprint gate per sec-common.md (focused tests, GLM 5.2 tmux run, one independent Opus 5.5 High review, demo videos
via scripts/demo_video.py). PR CI does not run codex-core Rust tests or Linux clippy: before merging, run the affected
crates' tests locally and check Linux-only code with clippy in Docker or on the RTX box. Fresh worktree off
origin/main; own CARGO_TARGET_DIR. PR to main; merge when green (stale CodeQL checks can be ignored). Report as in
sec-common.md.
