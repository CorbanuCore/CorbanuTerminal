## Security

With protected launch enabled, MCP servers started on an executor now apply the same inherited environment allowlist as local MCP servers. Host and remote `env_vars` cannot forward provider keys, Corbanu credentials or credential helpers. Servers that need a credential can still receive it through their own explicit `env` configuration.
