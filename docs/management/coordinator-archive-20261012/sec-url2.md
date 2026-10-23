You are a security sprint worker on the network lane (round 2). Read
/Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md first (binding), then
/Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-url.result.md (PF-33-S01 merged as #210/#213, flag
`url_destination_policy`).

Coordinator decisions (2026-10-06, following the S01 worker's recommendation and the PF-28-S01 precedent):
- PF-33-S02 may start now with PF-33-S01 merged but not yet archived. Record this in both sprint files. If the
  sprint checker blocks the allocation, adjust the checker minimally (allow a dependency that is merged-behind-flag
  with its gate evidence recorded) with a test, rather than leaving the sprint as draft.
- Credential method/path binding moves to PF-28-S02 (broker lane). Record it there as an explicit Remaining item and
  remove it from S01's open list.

Then implement PF-33-S02 "Connection pinning and alternate-egress denial"
(docs/sprints/current/p0-security-levels/pf-33-s02-connection-pinning-and-bypass.md) behind a default-off flag (reuse
`url_destination_policy` if the sprint scope fits, otherwise a new flag), under the per-sprint gate (focused tests,
GLM 5.2 tmux run, one independent Opus 5.5 High review, demo videos via scripts/demo_video.py). Avoid overlapping the
broker lane's files (credential_broker.rs, secret-broker crates) and the untrusted-content lane's. Fresh worktree off
origin/main; own CARGO_TARGET_DIR. PR to main; merge when green (stale CodeQL checks on PRs can be ignored). Note:
PR CI doesn't run Linux clippy; run `just fix -p <crate>` and, for Linux-only code, check it compiles cleanly in
Docker or on the RTX box (`ssh ambient@100.99.88.49`, use your own checkout under ~/corbanu-rtx/, clean up after).
Report as in sec-common.md.
