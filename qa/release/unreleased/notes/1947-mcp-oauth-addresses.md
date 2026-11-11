## MCP OAuth requests no longer reach internal addresses (#1947)

- **A remote MCP server can no longer make Corbanu send OAuth requests to
  your local machine, private network or cloud metadata service.** OAuth
  discovery, client registration, token and refresh requests (and each
  redirect) now go only to public addresses, unless the MCP server's own URL
  is `localhost`, a `*.localhost` name, or a loopback or private IP; then
  those addresses are allowed too. Link-local and cloud metadata addresses (`169.254.169.254`,
  `fd00:ec2::254`) and metadata host names are refused unless the server URL
  is that very IP address. Host names are checked after DNS resolution, and
  the request connects only to the address that was checked. Before, an MCP
  server could point these requests at a host name that resolves to an
  internal address (blind SSRF). rmcp's own fix for this
  (RUSTSEC-2026-0457, rmcp 3.7+) does not cover the OAuth HTTP client
  Corbanu uses, whatever the rmcp version.
- Not changed: public MCP servers, and local ones named by `localhost` or an
  IP address, sign in as before.
- **Changed:** an MCP server named by a host name that resolves to a loopback
  or private address (an intranet name, `host.docker.internal`, a `.local`
  name, an `/etc/hosts` alias) can no longer use OAuth; sign-in fails with
  `MCP OAuth request blocked: ...`, and a server there that doesn't use OAuth
  shows auth status unknown with a startup warning. Using the server's IP
  address in `url` works where its certificate allows that. With an HTTP
  proxy, only IP literals and host names themselves are checked. Corbanu does
  not send OAuth requests through a remote executor from before this change
  (sign-in fails until it is updated). Details: `docs/config.md#mcp-oauth-addresses`.
