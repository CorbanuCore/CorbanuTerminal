## Chat Completions and Anthropic: earlier MCP calls keep their own name

- **An MCP tool's earlier calls no longer replay as a built-in's** (#688). On
  providers without tool namespaces, a call recorded for an MCP tool that the
  request no longer offers (its server was removed, or the 128-tool limit left
  it out) used to replay under its bare name, such as `shell`, which a
  built-in tool can share. It now replays under the MCP tool's flat name
  (`mcp__server__tool`), or a hashed form if another tool has that name.
  MCP servers configured without the `mcp__` prefix get the same treatment
  (`server__tool`); only built-in tool namespaces keep the bare name.
- **MCP servers without the prefix can't take a built-in's namespace** (#1655).
  With `features.non_prefixed_mcp_tool_names`, a server that would be exposed
  without `mcp__` under a built-in tool namespace (`web`, `clock`, `skills`,
  `collaboration`, `image_gen`, `memories`, `multi_agent_v1`, or the configured
  multi-agent namespace) is now rejected when the config loads and by
  `corbanu mcp add`, with a message saying how to rename it. Skill dependency
  install skips such a server, and plugin servers keep the `mcp__` prefix.
  This includes servers from a trusted project's `.codex/config.toml`: with the
  feature on, such a project's config doesn't load until the server is renamed
  or left out of `server_names`. Disabled servers are skipped unless they are
  listed in `server_names`.
- **Nor can they take a namespace the Responses API reserves** (#1666). The
  same check now also refuses `api_tool`, `browser`, `computer`, `container`,
  `file_search`, `functions`, `multi_tool_use`, `python`,
  `python_user_visible`, `submodel_delegator`, `terminal` and `tool_search`,
  the names `features.multi_agent_v2.tool_namespace` already can't use.
  App-server config writes and MCP imports from other agents apply it too.
  `corbanu mcp add` now suggests another name to add (#1679) instead of
  `corbanu mcp remove` for a server that isn't configured. The warning for a
  skipped skill dependency now says which kind of reserved namespace it would
  take and no longer suggests "limiting" `server_names`.
  An MCP import from another agent that skips such a server now says it was
  not imported and suggests adding it under another name, instead of
  `corbanu mcp remove` (#1681, #1738, thanks to @citadelculture).
