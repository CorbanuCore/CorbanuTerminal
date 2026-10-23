You are an independent senior security and Rust reviewer (Opus 5.5, high effort). Review the last commit on this branch (`git show HEAD`, base origin/main). Do not edit files; read the code and report.

Context: follow-up to #201. Log lines already redact provider URL query values and userinfo, but user-visible errors did not: `TransportError::Http.url` became `UnexpectedResponseError.url` and is printed as `, url: …` in the error shown in the TUI (and sent to app-server clients); `ConnectionFailedError` / `ResponseStreamFailed` print a `reqwest::Error` whose text includes the URL. A provider configured with `query_params = { key = "…" }` or a base_url with `user:pass@` showed the key on screen. Change: redact at the source (`http-client/src/transport.rs`, `codex-api` websocket `map_ws_error`) and again in the three `Display` impls in `protocol/src/error.rs`, using `codex_http_client::redact_url` / `redact_reqwest_error`.

Look for:
- Other user-visible error strings (TUI history cells, app-server error notifications, `corbanu exec` stderr, login / model-list / MCP / realtime errors) that can include a configured provider URL with credentials, or the base_url itself.
- Anything that relied on the unredacted `url` (retries, auth, telemetry) and would now break.
- Test validity.
