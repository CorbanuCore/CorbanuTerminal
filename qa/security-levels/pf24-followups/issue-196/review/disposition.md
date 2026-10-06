# Review disposition (Opus 5.5 High, approve with fixes)

| # | Severity | Disposition |
| - | -------- | ----------- |
| 1 | High | Fixed. `codex_http_client::redact_url` and `redact_reqwest_error` hide query values and userinfo in every request log: transport trace, `Request completed/failed`, reqwest error text in `TransportError::Network`, and the Responses and realtime websocket connect logs. User-visible error messages that carry a URL (`TransportError::Http.url`) are unchanged; they show the provider URL the user configured. |
| 2 | Medium | Fixed. The core test also sends a header and a query sentinel, checks that wiremock received all three credentials, and checks that the log records the redacted bearer field, `"X-Sentinel": "<redacted>"` and `key=REDACTED`. |
| 3 | Low | The `TraceSinks` doc comment now states it captures this thread's events only. A global-subscriber test binary is not added. |
| 4 | Low | Fixed. Provider header values, the API-key header and bearer/agent `Authorization` values are marked sensitive, so `HeaderMap` Debug prints `Sensitive`. `ApiKeyHeaderAuthProvider` Debug redacts the key. The prost `ModelProvider` has no log site and is unchanged. |
| 5 | Low | Fixed. `base_url` userinfo and query are redacted in `Debug`. The doc comment says why `auth.args` are kept. |
| 6 | — | Agreed; nothing to change. |
| 7 | Nit | Fixed. A single `assert_eq!` on the full Debug string. The `Default` check was dropped. |
| 8 | Nit | Fixed. The tests moved to `suite/log_redaction.rs`, with an empty-needle guard and plain imports. |
| 9 | Nit | Fixed. The impl moved to `provider_debug.rs`. |
