You are a security sprint worker on the "first free" lane (the network lane finished PF-33-S01/S02). Read
/Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md first (binding).

Sprint: PF-29-S01 "Protected-mode inventory and activation preflight"
(docs/sprints/current/p0-security-levels/pf-29-s01-protected-mode-inventory.md), then PF-29-S02 "Human-reviewed
credential migration and recovery". PF-29-S01 depends on PF-28-S02 (PR #216, in CI now; start against it as soon as
it merges, or build on main now if your scope doesn't touch its files) and PF-20-S02 (check it's archived).
Follow the precedent used for PF-28-S01/PF-33-S02: start with dependencies merged behind their flags, recorded in
both sprint files. Note PF-27-S02's known limit: shell profile files and credential files outside the deny list
(e.g. ~/.ssh) stay readable; PF-29's migration is the intended fix, so cover it.

Per-sprint gate per sec-common.md (focused tests, GLM 5.2 tmux run, one independent Opus 5.5 High review, demo
videos via scripts/demo_video.py). PR CI does not run codex-core Rust tests or Linux clippy: run affected crates'
tests locally and check Linux-only code with clippy in Docker or on the RTX box before merging. Default-off feature
flag for behaviour changes. Never touch the user's real credentials or vault: use disposable homes and fake values.
Fresh worktree off origin/main; own CARGO_TARGET_DIR. PR to main; merge when green (stale CodeQL checks can be
ignored). Update sprint files and index rows. Report as in sec-common.md.
