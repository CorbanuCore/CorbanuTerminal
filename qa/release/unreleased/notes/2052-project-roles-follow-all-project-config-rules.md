## Project agent roles follow every project-config rule (D525, #2052)

- **Settings in an agent role file a project declares now follow the same
  rules as the same settings in the project's `.codex/config.toml`.** Before,
  a child agent spawned with such a role treated them as your own launch
  settings. Now:
  - hooks in a project role are listed as project hooks, keyed by the role
    file, and need your review before they run; a `[hooks.state]` table in the
    role file is ignored, as in project config;
  - an MCP server a project role defines or changes counts as a project's
    server: no Aggressive "always allow" record, and an approval isn't saved
    to your config;
  - a project role can't change a security level's feature defaults (for
    example `features.broker_model_auth`), skill settings (`[[skills.config]]`)
    or `project_root_markers`.

  A project role's `[security]` table already could only raise the security
  level, as in project config; that is unchanged.
- **`[agents] provider_allowlist` can be narrowed but not widened by a
  project.** A list in a project's config or in a project role only removes
  providers from the list your own, managed or `-c` config sets; it can't add
  any.
- Not changed: roles you declare yourself (in `CODEX_HOME/agents/` or your user
  config), roles from managed config and built-in roles.
