You are an independent security reviewer. Review issue #218 part 2 on this checkout: the commits not yet on main, i.e. `git diff origin/main...HEAD` (CorbanuCore/CorbanuTerminal). Part 1 (contained launch, already reviewed) is the base. Read the code; do not modify files.

Context: contained Claude panes (feature `contained_external_agents`, with the PF-27-S02 contract armed by `secretless_agent_launch`) now run Claude Code with `--permission-mode default --permission-prompt-tool stdio --input-format stream-json` instead of `bypassPermissions`. Each `can_use_tool` control request becomes a TUI approval popup (codex-rs/tui/src/claude_panes/approval.rs; execution.rs `answer_control_request`); the answer goes back on stdin. The prompt goes on stdin. Contained panes load no setting sources and no MCP servers. Under the `/security` level Aggressive, contained panes are no longer refused (codex-rs/tui/src/security/level.rs, containment.rs `contained_launch_ready`). See qa/security-levels/pf24-followups/claude-contained/README.md (Part 2).

Check:
1. Can Claude Code or anything it runs get a tool executed without a person's approval (other control subtypes, malformed requests, request_id confusion, permission suggestions/updatedPermissions, settings/hook/MCP/skills paths, session resume, subagents, plan mode, `acceptEdits`, the Bash tool's own sandbox flags)?
2. Is the decision path sound: popup defaults, Esc, digits, stale popups, dropped receivers, smoke (no UI) denies, concurrent requests and panes?
3. Is lifting the refusal correct and only when containment is really in force (feature on, contract armed, platform sandbox)? Any entry point that still launches uncontained under Aggressive?
4. Stdin/stdout protocol robustness (closing stdin, early exit, interrupts, large inputs), redaction of what the popup shows.
5. Tests: meaningful? gaps?

Output: a verdict (approve / approve with fixes / request changes) and numbered findings with file:line, severity and a concrete fix. Be concise.
