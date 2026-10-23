You are the broker lane worker (round 4). Read /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md first
(binding), then sec-broker3b.result.md and sec-url.result.md in the same folder.

PF-28-S01 is merged (#208, flag `secret_output_gate`). Next: PF-28-S02 "Reflected-secret response scrubbing"
(docs/sprints/current/p0-security-levels/pf-28-s02-reflected-secret-response-scrubbing.md). Its scope now also includes,
per the coordinator (2026-10-06):
- the gaps PF-28-S01 handed over: encodings wrapped across lines (base64 at 76 columns, `xxd -p`), decode-and-rescan,
  secrets echoed back in model/provider responses, MCP OAuth tokens refreshed after startup, seed phrases written
  comma-separated or numbered;
- credential method/path binding for legacy brokered credentials (`credential_broker.rs`), handed over from PF-33-S01.
If the sprint checker requires PF-28-S01 archived first, allocate S02 the same way PF-28-S01 and PF-33-S02 did (start
with the dependency merged behind its flag, recorded in both sprint files).

Per-sprint gate per sec-common.md (focused tests, GLM 5.2 tmux run, one independent Opus 5.5 High review, demo videos
via scripts/demo_video.py). Fresh worktree off origin/main; own CARGO_TARGET_DIR. Check Linux-only code with clippy in
Docker or on the RTX box before merging (PR CI doesn't run Linux clippy). PR to main; merge when green (stale CodeQL
checks can be ignored). Update sprint files and index rows. Report as in sec-common.md.
