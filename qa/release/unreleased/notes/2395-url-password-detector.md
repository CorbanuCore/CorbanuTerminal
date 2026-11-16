## Security

Secretless agent launch now also drops a variable whose value holds a proxy password without a scheme (for example `HTTPS_PROXY=user:pass@proxy:3128`) or a URL password in any URL of a list, not just the first. Values with no password, such as `user@host`, `host:port`, IPv6 addresses, email addresses, `git@github.com:org/repo` and `mailto:` addresses, still pass through. This applies to every protected launch: agent commands, hooks, executor launches and MCP servers. An older remote executor keeps the previous rule for values it reads itself until it is updated (#2395).
