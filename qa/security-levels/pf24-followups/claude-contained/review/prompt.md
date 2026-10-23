You are an independent security reviewer. Review the change on this checkout: `git diff origin/main...HEAD` (branch sec-tui5-218-contained-panes, CorbanuCore/CorbanuTerminal issue #218 part 1). Read the code; do not modify files.

Context: Claude panes (`codex-rs/tui/src/claude_panes/`) run Claude Code headless per turn. Until now they ran `claude --permission-mode bypassPermissions` directly with Corbanu's whole environment, no sandbox, full network, and direct providers got the raw key via `apiKeyHelper = "corbanu vault auth-helper <label>"`. Under the `/security` level Aggressive they are refused (PR #220). This change, behind the default-off feature `contained_external_agents`, launches every turn under the PF-27-S02 secretless launch contract (`codex-rs/core/src/security/launch_contract.rs`, armed by `secretless_agent_launch`): clean allowlisted environment, every provider behind the per-turn loopback bridge, Claude Code wrapped in Seatbelt/Linux sandbox with writes only to the pane cwd and a per-pane state dir outside CODEX_HOME, network only to the bridge (as HTTP_PROXY; Seatbelt allows one loopback port; the Linux sandbox's managed-proxy routing carries it over a Unix socket). The refusal under protected levels stays until part 2 (tool approvals). See qa/security-levels/pf24-followups/claude-contained/README.md.

Check:
1. Secret exposure: can the contained Claude process, its tools or its environment/argv/files obtain a provider key, Claude OAuth token, vault store, or Corbanu config? Is the bridge token handling sound? Does the absolute-form request handling in bridge.rs open anything?
2. Sandbox correctness: the profile built in containment.rs + protect_external_agent_launch; writable roots; the state dir location; Linux arg0/linux sandbox path; network policy (only the bridge port); env allowlist (proxies removed).
3. Fail-closed behaviour: contract not armed, unsupported platform, transform errors, bridge missing; ordering (sandbox decided before the bridge starts).
4. Regressions for the feature-off path (must be byte-for-byte today's behaviour) and for Claude Plan / Ambient / Vercel profiles.
5. Tests: meaningful? gaps?

Output: a verdict (approve / approve with fixes / request changes) and numbered findings with file:line, severity and a concrete fix. Be concise.
