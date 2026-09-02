# PF-56 Kimi external review controller

- User override date: 2026-09-02
- Required reviewer/model: Kimi 3.0 (`moonshotai/kimi-k3`)
- Required provider: Vercel (`vercel`)
- Required effort: high
- Authorized invocation count: exactly one
- Fallback/substitution: forbidden
- TMUX socket/session: `pf56-kimi-vercel-review`
- Working directory: `/home/pfrpc/repos/worktrees/corbanu-main-f7356a94e0`
- Mode: Corbanu Terminal TUI, read-only sandbox, approvals never
- Brief: `qa/provider-auth/pf-56/review/kimi-brief.md`
- Output: `qa/provider-auth/pf-56/review/kimi-output.md`
- Exit status: `qa/provider-auth/pf-56/review/kimi-exit-status.txt`
- Runtime evidence: `qa/provider-auth/pf-56/review/kimi-runtime.md`
- Disposition: `qa/provider-auth/pf-56/review/kimi-disposition.md`

The earlier Claude Fable 5 process exited before inference because its OAuth
session could not be refreshed. Its original brief, controller, output, and
exit status are retained alongside these files as superseded evidence; it is
not counted as the chosen completed external review.
