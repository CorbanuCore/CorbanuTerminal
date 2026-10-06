You are an independent senior security and Rust reviewer (Opus 5.5, high effort). Review the last commit on this branch (`git show HEAD`, base origin/main). Do not edit files; read the code and report.

Context: follow-up to #183/#201 (credentials in logs). The logs DB and `/feedback` buffer record DEBUG/TRACE at any RUST_LOG. `codex-http-client` logged `headers = ?response.headers()` on every "Request completed" debug line, and the Responses websocket logged the handshake response headers at INFO, so `set-cookie` values (and any credential-like response header) reached those sinks. Change: `codex_http_client::redact_headers(&HeaderMap)` returns a Debug view that keeps every header name and replaces the value of sensitive (`is_sensitive`) headers and of any header whose name contains cookie/auth/token/key/secret/session/signature/credential/password. Used at the three log sites. Tests: a unit test and the existing core log-sink regression now also serves a `set-cookie` response header and checks the log file, feedback buffer and DB bytes.

Look for:
- Other places where HTTP response (or request) headers, or `reqwest::Response`/`http::Response` Debug, still reach tracing (any crate: codex-api, core, login, model-provider, mcp clients, rmcp-client, backend-client, network-proxy, app-server-transport).
- Whether the name-fragment denylist is right (too narrow / too broad), versus an allowlist.
- Test validity.
