You are the TUI lane worker (round 4). Read /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md first
(binding), then sec-tui3.result.md, sec-picker.result.md and sec-broker2.result.md in the same folder.

Travis's decisions (2026-10-06), each as its own PR under the per-fix gate:
1. Nested agent launches under Aggressive (`corbanu exec`, `resume`, `app-server`, `mcp-server`, bare prompt started by
   an agent command): make the behaviour configurable between `refuse` and `pass` (pass = the child runs with
   Aggressive enforced, via a marker the CLI honours and the child can't drop), default `refuse`. The setting must
   not be changeable by agents, project files or `-c` overlays from agent commands (same protection as the level
   file). Tests for both modes, including that `pass` really enforces Aggressive in the child (vault, network,
   outside-write probes).
2. Claude panes and other external provider tools (CLI agents Corbanu launches in panes) are not covered by
   PF-27-S02's secretless launch and sandbox contract. Investigate how they are launched today, then either bring
   them under the contract (clean environment, sandbox, no raw secrets) or block them under protected levels with a
   clear message. Prefer bringing them under the contract if it is feasible in this round; if not, block under
   Aggressive/protected levels now and file an issue for the full fix. Say which and why.
Do not change the Aggressive sandbox design itself (`corbanu-aggressive` profile); that is still with Travis.
Gate per fix: focused `just test`, GLM 5.2 tmux run, one independent Opus 5.5 High review, short demo video. Check any
Linux-only code with clippy in Docker or on the RTX box. PR to main; merge when green (stale CodeQL checks can be
ignored). Report as in sec-common.md.
