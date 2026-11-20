## Upstream Codex rust-v0.167.0

- Corbanu Terminal is now built on OpenAI Codex `rust-v0.167.0` (upstream
  commit `086ccc833b`), including upstream's changes since Corbanu's previous
  upstream base. Corbanu-specific behavior is kept; the other sections of these
  notes describe where it changed.
- New commands from upstream: `corbanu agents` opens the agent command center
  to browse and switch sessions; `corbanu queue` queues a message for an
  existing session; `corbanu migrate-rollouts` inspects or migrates legacy local
  sessions to paginated thread history; and the experimental
  `corbanu migrate-nfs-db` switches the configured SQLite databases to DELETE
  journal mode in place.
- Removed: `corbanu mcp-server`, which ran Corbanu Terminal as a stdio MCP
  server. Upstream removed the command (D275). Configuring the MCP servers
  Corbanu uses (`corbanu mcp …`) is unchanged.
