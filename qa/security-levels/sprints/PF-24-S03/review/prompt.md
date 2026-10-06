You are an independent senior security and Rust reviewer (Opus 5.5, high effort). Review the PF-24-S03 change on branch `codex/pf-24-s03-security-picker` in this worktree against `origin/main` (base 24242c1b0e). Do not edit files; read the code and report.

Mandate (sprint docs/sprints/current/p0-security-levels/pf-24-s03-flagged-security-picker.md):
- Behind the `security_levels` flag, `/security` lets a human choose Permissive or Aggressive; Moderate shows "not available yet". Flag off: everything unchanged.
- Differences shown before confirmation; Esc changes nothing. Level persists; restart restores it; unknown stored value fails visibly, never Permissive.
- Return to Permissive restores prior settings exactly. No agent tool, prompt, config overlay or project file can change the level. Child agents inherit.
- Aggressive is built ONLY from existing controls per the mapping table (sandbox cwd-only no tmp, approvals untrusted, network off + web search off, vault `corbanu vault` forbidden via execpolicy + secret env stripped, children inherit). The UI must never show a protection as active when it is not.

Implementation choices to scrutinize (deliberate deviations; say whether you agree):
1. Activation happens at the next start (launch overlay of `-c` overrides + harness overrides, then verification of the loaded Config); the picker writes `$CODEX_HOME/security_level.toml` and a rules file. Rationale: web_search, shell_environment_policy and execpolicy are session-static, so they cannot be applied live through the PF-83 thread-settings path.
2. The sandbox row uses a named permission profile `corbanu-aggressive` (extends `:workspace`, tmp read-only, network off, CODEX_HOME read-only, CODEX_HOME/secrets deny-read) instead of a legacy SandboxPolicy, because under `untrusted` an approved command that the legacy sandbox blocks is silently retried unsandboxed (core/src/tools/orchestrator.rs + sandboxing.rs `should_bypass_approval`). A denied-read entry disables unsandboxed execution.
3. `/permissions` changes are refused while Aggressive is stored or active (guards in tui/src/app/permission_confirmation.rs and config_persistence.rs).

Look for: correctness bugs; any path where Aggressive is shown active but a row is not enforced (resume, fork, /new, panes, sub-agents, remote/daemon app server, project config layers, profiles, CLI flags, requirements); ways an agent could change or downgrade the level; flag-off behaviour changes; persistence/atomicity issues; test gaps; style issues against codex-rs/AGENTS.md and codex-rs/tui/AGENTS.md.

Start with `git diff origin/main...HEAD -- codex-rs` and read the new files under codex-rs/tui/src/security/ and codex-rs/tui/src/bottom_pane/security_level_picker.rs.

Output Markdown: a verdict (approve / approve with fixes / request changes), then numbered findings each with severity (critical/high/medium/low/nit), file:line, the problem, and a concrete fix. Be concise.
