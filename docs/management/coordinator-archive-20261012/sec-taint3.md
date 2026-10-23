You are the untrusted-content lane worker (round 3). Read /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md
first (binding), then /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-taint2.result.md (PF-30-S02 slice 1
merged as PR #190, flag `source_envelopes`; gate record qa/security-levels/sprints/PF-30-S02/persistent-origins-gate.md).

1. Finish PF-30-S02's open items (see "Still open" in that report): sub-agent lineage, export/import, binding memory
   summarisation to the session's security level, capacity tests and digest-on-read checks, taint kept after a
   one-off approval, a real paginated-thread / referenced-fork resume test, and moving the token-budget MCP thread
   hint out of trusted host context. If an item is genuinely out of scope for this sprint per its sprint file, move it
   explicitly to the right later sprint record instead of leaving it dangling. Then close and archive PF-30-S02.
2. Then start PF-30-S03 "Post-taint authority checks"
   (docs/sprints/current/p0-security-levels/pf-30-s03-post-taint-authority-checks.md) under the same gate.
Per-sprint gate per sec-common.md (focused tests, GLM 5.2 tmux run, one independent Opus 5.5 High review, demo videos
via scripts/demo_video.py). Use a fresh worktree off origin/main and your own CARGO_TARGET_DIR. PR to main; merge when
green (stale CodeQL checks on PRs can be ignored). Update sprint files and index rows.
Report as in sec-common.md.
