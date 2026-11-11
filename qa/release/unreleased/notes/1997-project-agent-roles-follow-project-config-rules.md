## Project agent roles follow the project-config denylist (D512, #1997)

- **An agent role a project declares can no longer set the settings a
  project's own config can't set (the project-config denylist).** This
  covers role files in a project's `.codex/agents/` folder and files named
  by `[agents.<role>] config_file` in a project's config. Before, a child agent spawned with such a role could get a different model
  provider or provider URL (where its requests and credentials go), `notify`,
  `otel`, `sqlite_home`, `log_dir` or `[debug.config_lockfile]`, even though a
  project's `.codex/config.toml` can't set them. Corbanu now ignores those
  settings in project roles, the child keeps its parent's values, and a startup
  warning names the ignored settings. A project role file also can't declare
  roles of its own for the child to spawn (`[agents.<role>]` entries), which
  would otherwise have reached a grandchild agent with those settings.
- Not changed: roles you declare yourself (in `CODEX_HOME/agents/` or your user
  config), roles from managed config and built-in roles. If one of your
  projects' roles chose a provider, declare that role in your user config.
- Not covered yet: other rules that apply only to project config (feature
  defaults, project MCP servers, hooks) don't apply to project roles yet; see
  #2052.
