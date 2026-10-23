You are the convergence-lane worker for PF-41-S01. Read /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md
first (binding: keychain rule, disk hygiene, per-sprint gate, Linux clippy on the RTX box).
All dependencies are merged: PF-23-S03 (archived), PF-29-S02 (#235), PF-24-S02 (#253). Implement PF-41-S01 "Effective
security inspector and degradation state" (docs/sprints/current/p0-security-levels/pf-41-s01-effective-security-inspector.md).
Binding decision: unsandboxed routes (MCP, hooks, `!` commands) are allowed but must be shown as "not contained" in the
inspector. Nested exec is configurable refuse|pass, default refuse; show which one is active.
Another worker (sec-tui7) is on PF-25-S01/S02 in the TUI picker files; coordinate by not editing
bottom_pane/security_level_picker* and keep TUI edits to new inspector files where possible.
Gate per sec-common.md, including a GLM 5.2 (provider zai) tmux run per the repo SOP and short demo videos of the
inspector end to end. Open a PR to main, merge when green (`gh pr merge N --merge --match-head-commit <sha>`); if the
sprint index conflicts, merge origin/main into your branch (never force-push). Report as in sec-common.md.
