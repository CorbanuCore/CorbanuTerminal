You are the convergence-lane worker (round 5). Read /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md
(binding: keychain rule, disk hygiene, per-sprint gate) and sec-qual3.result.md. Issue #239 is already fixed (PR #244, merged).
Re-run the PF-13-S07 route matrix with a SAVED security level, once under Aggressive and once under Moderate (all
protection flags on, disposable homes, fake canaries only), on macOS and on the RTX box. Unsandboxed routes (MCP, hooks,
`!` commands) are allowed and must be reported as "not contained", not as leaks. File and fix/hand off any real leaks.
Add the results to qa/security-levels/sprints/PF-13-S07/, get one independent read-only review of the new evidence
(installed `corbanu exec` on claude-plan, Opus 5.5 high), open a PR, merge when green
(`gh pr merge N --merge --match-head-commit <sha>`), and archive PF-13-S07 if its gate is complete.
Report as in sec-common.md.
