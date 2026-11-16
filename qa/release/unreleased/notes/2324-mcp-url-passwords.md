## Security

With secretless agent launch on, MCP servers no longer inherit an environment variable whose value embeds a URL password (for example `HTTPS_PROXY=http://user:pass@proxy`), even when the name is allowed; other URLs, including ones with only a user name, pass through unchanged. This covers local stdio servers and servers started through an executor. Values in a server's own `env` setting are still passed as written. An older remote executor that cannot filter these values is refused for MCP servers while secretless launch is on; update it to the current version (#2324).
