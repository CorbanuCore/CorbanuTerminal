## Security

With secretless agent launch on, an executor that Corbanu starts over stdio from `environments.toml` now gets only the allowed environment (plus its own configured `env`), not Corbanu's full environment, so MCP servers and commands it starts can't read your keys and tokens from it. An executor started before secretless launch was turned on during a run is refused until you restart Corbanu. An `ssh`-based executor can't reach your SSH agent (`SSH_AUTH_SOCK`) while it's on (#2200).
