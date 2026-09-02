# PF-56 external review controller

- Status: superseded by the user's 2026-09-02 Kimi 3.0/Vercel override
- Required reviewer: Claude Fable 5
- Required effort: high
- Invocation count: one
- TMUX socket: pf56-fable-review
- TMUX session: pf56-fable-review
- Working directory: /home/pfrpc/repos/worktrees/corbanu-main-f7356a94e0
- Mode: noninteractive print, plan permissions, no fallback model, no session persistence
- Allowed tools: Read, Grep, Glob, read-only git/rg/sed/wc Bash commands
- Disallowed tools: Edit, Write
- Brief: qa/provider-auth/pf-56/review/brief.md
- Output: qa/provider-auth/pf-56/review/output.md
- Exit status: qa/provider-auth/pf-56/review/exit-status.txt
- Outcome: authentication failed before model inference (`exit 1`); this is not
  a completed formal review and is preserved without retry or fallback.
