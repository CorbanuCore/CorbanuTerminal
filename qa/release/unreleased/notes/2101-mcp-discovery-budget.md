## Fixes

- **MCP `discover_timeout_sec` changes no longer share a startup that is still running** (follow-up to #1950's `server/discover` wait). If two configurations of the same MCP server differ only in `discover_timeout_sec`, each waits for its own discovery budget instead of reusing the other's unfinished startup. A server that has already started is still reused when only that setting changes.
