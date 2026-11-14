## MCP servers that opt in to 2026-07-28: `server/discover` wait (#1950)

- **`discover_timeout_sec` changes no longer share a startup that is still running.** If two configurations of the same MCP server differ only in `discover_timeout_sec`, each waits for its own discovery budget instead of reusing the other's unfinished startup. A server that has already started is still reused when only that setting changes.
