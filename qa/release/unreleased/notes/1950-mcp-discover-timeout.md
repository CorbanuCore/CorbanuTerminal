## MCP servers that opt in to 2026-07-28: `server/discover` wait (#1950)

- **A stdio MCP server that opted in to MCP 2026-07-28
  (`CODEX_MCP_PROTOCOL_VERSION=2026-07-28`) and falls back to legacy is now
  started twice.** When it rejects `server/discover` as legacy, or doesn't
  answer it in time, Corbanu stops that process and starts the server again for
  the legacy `initialize`, so a late discover reply can no longer fail startup.
  The second start gets the startup timeout of its own, so a server that never
  answers `server/discover` can take up to about twice `startup_timeout_sec`.
  That timeout also covers stopping the first process and starting the second,
  so a remote executor that never finishes either can no longer hang startup
  (#2019). On a remote executor Corbanu doesn't wait for the first process to
  exit, so the old and new server processes can briefly overlap.
- **New `discover_timeout_sec` setting** for these servers: how long Corbanu
  waits for `server/discover` before it falls back. It defaults to, and is
  capped at, `startup_timeout_sec`, and is ignored for servers that haven't
  opted in.
