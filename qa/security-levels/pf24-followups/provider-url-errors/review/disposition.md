# Review disposition (Opus 5.5 High, approve with fixes)

| # | Severity | Disposition |
| - | -------- | ----------- |
| 1 | Medium | Fixed. The MCP streamable-HTTP send, body-read and stream errors are redacted through the new `RouteAwareRequestError::redacted_message` and `redact_reqwest_error`. |
| 2 | Low | Fixed. The `/model` warning passes the base URL through `redact_url`. |
| 3 | Low | Fixed. LM Studio's "Request failed" messages use `redacted_message`. |
| 4 | Low | Kept. `ConnectionFailedError` and `ResponseStreamFailed` are not built in production; only their `Display` changed. A manual `Debug` was not added. |
| 5 | Low | Fixed. Tests now cover the `stream()` path and `redacted_message`. The test server reads up to the end of the headers before it answers. `map_ws_error` is not unit-tested. |
| 6 | Low | Recorded. Error events in rollout files written before this fix may hold the URL. Rotate any provider key that was set through `query_params` or userinfo. |
| 7 | Nit | Kept. Redacting every query value is the safe default. |
| 8 | Nit | Done. There is a tmux run with a local 401 server, and a video. |
