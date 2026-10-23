# Provider URL credentials in user-visible errors (#201 follow-up)

#201 redacted URLs in log lines, but not the errors shown in the TUI. When a
provider's `base_url` held `user:pass@` or it had `query_params` such as
`key = "…"`, an HTTP error printed the full URL in the error cell, and app-server
clients received it too.

## Fix

The URL is redacted where errors are built and again where they are shown, so
`codex_http_client::redact_url` / `redact_reqwest_error` now cover these paths:
- `TransportError::Http.url` for HTTP, streaming and websocket errors;
- the `Display` text of `UnexpectedResponseError`, `ConnectionFailedError` and
  `ResponseStreamFailed`;
- from the review:
  - MCP streamable-HTTP request errors (shown as "MCP client for … failed to
    start"), through the new `RouteAwareRequestError::redacted_message`;
  - the `/model` base URL warning;
  - LM Studio request errors.

## Gate evidence

- **Tests:** after `just fmt` and `just fix`, all passed:
  - `just test -p codex-http-client` (80)
  - `just test -p codex-protocol error` (41)
  - `just test -p codex-api` (241)
  - `just test -p codex-exec-server route_aware` (the LM Studio crate has no
    tests; it was compiled)

  New tests cover:
  - the `execute` and `stream` error URLs against a local 401 server;
  - the `Display` impls;
  - `redacted_message`. reqwest already moves userinfo into a header, so only
    the query is left to redact there.
- **tmux run on GLM 5.2:** the `glm-5.2` model with a `demo401` provider that
  points at `tmux-run/fake401.py` and holds fake userinfo and a fake `key`.
  - **Before (main):** the error cell showed
    `http://demo-user:fake-demo-pass-0002@…/v1/responses?key=fake-demo-query-0001`.
  - **After:** it shows `http://REDACTED:REDACTED@…?key=REDACTED`.
- **Review:** Opus 5.5 High, approve with fixes; see `review/`.
- **Video:** `qa/demos/index/url-error-redaction.md`.
