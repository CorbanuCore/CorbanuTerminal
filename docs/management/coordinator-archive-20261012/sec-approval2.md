You are a TUI security follow-up worker. Read /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md first
(working rules and per-fix gate apply), then /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-tui.result.md
(the #174 fix, merged as PR #187).

Travis's decisions (2026-10-06) on the #187 follow-ups:
1. Keep two-key approval (choice, then Enter) and arrow-only list movement. No change.
2. YES: persistent options ("don't ask again", session-scoped approvals, any option that grants beyond this one
   request) must be chosen with the arrow keys. A letter/digit shortcut must not select them; it may only select
   one-shot options (approve once / decline). Show the hint in the prompt.
3. Leave Enter on a first prompt that appears while idle as "Yes, proceed" (unchanged).

Implement 2 with regression tests (including: shortcut key for a persistent option does nothing; a paste of a
shortcut letter + newline can't select a persistent option; arrows + Enter still can). Check every approval prompt
type (exec, patch, network host, MCP elicitation if it has persistent choices, permissions requests).
Gate: focused `just test -p codex-tui`, snapshots, a GLM 5.2 tmux run, one independent Opus 5.5 High review, and a
short demo video via scripts/demo_video.py. PR to main; merge when green; reference #174 in the PR.
Report as in sec-common.md.
