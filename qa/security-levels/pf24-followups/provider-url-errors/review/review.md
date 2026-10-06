**Verdict: approve with fixes.**

The redaction itself is correct. The provider HTTP error URL is redacted where it is created (`transport.rs:125-127, 151-153`) and in the websocket handshake (`responses_websocket.rs:662`). It is redacted again in `UnexpectedResponseError::Display`, and running `redact_url` twice is harmless. Nothing depended on the raw URL: retries, auth recovery, telemetry, `response-debug-context` and the accounting route check read only `status`, `body` and `headers`, or the request's own URL. One existing expected string changes (`api_bridge_tests.rs:108`), but its URL has no query or userinfo, so it stays the same after redaction.

The findings are other places where a configured URL can still reach the user, plus some test gaps. I did not build or run anything; the sandbox is read-only.

1. **Medium: `exec-server/src/client/route_aware_http_client.rs:176-180`, `:201-204`, `:256`.** MCP streamable-HTTP errors still show the full server URL. A failed send returns `http/request failed: {error}`, where `error` is the raw reqwest error, and that text includes the URL. It then appears in the TUI as "MCP client for `x` failed to start: …" (`codex-mcp/src/connection_manager/startup.rs:114`). MCP servers set up with `url = "https://…?api_key=…"` will show the key. Oddly, the log line in the same file (`:312`) already strips the URL with `without_url()`. The body-read error and the streamed `error: Some(error.to_string())` leak the same way.
   **Fix:** use `codex_http_client::redact_reqwest_error(&e)` for all three. `RouteAwareRequestError::Request` needs this too.

2. **Low: `tui/src/chatwidget/model_popups.rs:354-358`, raw value at `:377`.** The `/model` menu warning shows the configured OpenAI `base_url` exactly as written, so `user:pass@` or query values appear on screen. The `/status` card already cleans this with `sanitize_base_url` (`status/card.rs:979`).
   **Fix:** pass the value through `codex_http_client::redact_url` or the same sanitizer.

3. **Low: `lmstudio/src/client.rs:86`, `:107`.** `Request failed: {e}` prints the reqwest error, which includes the provider `base_url`. That URL can be configured, and this text shows up in the `--oss` flow.
   **Fix:** use `redact_reqwest_error(&e)`.

4. **Low: `protocol/src/error.rs:491-494`, `:506-510`.** `ConnectionFailedError` and `ResponseStreamFailed` are never built in production; only tests create them. Their `Display` changes are defensive only. Their derived `Debug` still prints the reqwest URL with the query intact, so any future `{err:?}` would leak it.
   **Fix:** either remove the dead variants or write a manual `Debug` that uses `redact_reqwest_error`.

5. **Low: test gaps.**
   - `transport_tests.rs:57-91` covers only `execute()`. Add a case for `stream()` (`transport.rs:146-155`), which is the path the main streaming turn uses.
   - `error_tests.rs:776-813` doesn't test `ConnectionFailedError`.
   - The `ResponseStreamFailed` case uses only a query value, not userinfo.
   - Nothing tests the websocket `map_ws_error` HTTP branch. A unit test that calls `map_ws_error(WsError::Http(resp), &url_with_key)` and checks the `url` field would cover it.
   - Small flake risk in the transport test: the server reads once and then drops the socket. If any request bytes are left unread, the close can send a reset before the client reads the 401. Reading until `\r\n\r\n` removes that risk.

6. **Low (operational): leaks already on disk.** `ErrorEvent` and `StreamError.additional_details` are saved to rollout files. Sessions recorded before this fix may already contain the key on disk. Note this in the PR or release record and advise rotating any provider key that was set through `query_params` or userinfo.

7. **Nit: `http-client/src/log_redaction.rs:37-45`.** Every query value is now redacted, including harmless ones like Azure's `api-version=2025-04-01-preview`, which used to help debugging. This is acceptable as a safe default. If that version string matters, consider an allow-list such as `api-version`.

8. **Nit (process): change record and TUI proof.** This changes an error string shown in the TUI. Under AGENTS.md, a bounded fix needs a product-spec heading and requirement excerpt in the PR, plus true-TUI proof for the changed interactive path. For example: set a provider with `query_params = { key = "fake" }` pointing at a server that returns 401, and confirm the error cell shows `key=REDACTED`. Neither is in the commit.