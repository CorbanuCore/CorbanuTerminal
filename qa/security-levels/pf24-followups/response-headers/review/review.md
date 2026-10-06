**Verdict: approve with fixes.** The commit does what it says, and I didn't find a leak at any of its three call sites. But I found a pre-existing, higher-severity path where the bearer token in the websocket *request* headers can reach `/feedback`, and the denylist can be bypassed. I only read code; I ran no tests (the sandbox is read-only).

### Findings

1. **High (pre-existing, not from this commit): the websocket handshake request, including `Authorization`, can reach the `/feedback` buffer.** `feedback/src/lib.rs:214-217`, `tui/src/lib.rs:1394-1401`
   - **How it happens:** `tracing-subscriber` uses its default features, so `.try_init()` installs `tracing_log::LogTracer`. The pinned tungstenite fork logs the full handshake request at trace level (`src/handshake/client.rs:221`, `trace!("Request: {:?}", …)`), and that text includes `Authorization: Bearer …` and the account headers. Events bridged from the `log` crate arrive with target `"log"`.
   - `state/src/log_db.rs:57` turns off target `"log"`, so the logs DB is safe. The feedback filter defaults to TRACE and does **not** turn `"log"` off, so these lines reach the buffer users upload.
   - The Responses websocket, the realtime websocket and remote control all use this handshake.
   - The core regression can't catch this: it uses `set_default`, which never installs `LogTracer`.
   - **Fix:**
     - Add `.with_target("log", LevelFilter::OFF)` to `logger_layer`, ideally by sharing one sink filter with `log_db::default_filter`. Filtering on `"tungstenite"` won't work because the per-layer filter only sees the target `"log"`.
     - Add a regression test that installs `LogTracer`, runs a websocket turn with an auth header, and asserts that header's value never reaches feedback.

2. **Medium: the name denylist fails open for headers from third-party providers or MCP servers.** `http-client/src/log_redaction.rs:35-46`
   - Credential-bearing names it misses:
     - `location`, `content-location`, `link` and `refresh` can carry URLs with `code=`, `access_token=`, `X-Amz-Signature=` or `sig=` in the query.
     - Names like `cf-access-jwt-assertion`, `x-amzn-oidc-data`, `x-csrf`/`xsrf` variants and `*-jwt*`.
   - It also over-matches: `x-openai-authorization-error` (`auth`) is useful diagnostics and gets redacted.
   - Since these lines reach the DB and feedback at any `RUST_LOG`, failing closed is better.
   - **Fix:** use an allowlist of value-safe names: `content-type`, `content-length`, `content-encoding`, `date`, `server`, `vary`, `via`, `etag`, `cache-control`, `retry-after`, `x-request-id`, `x-oai-request-id`, `cf-ray`, `openai-*`, `x-ratelimit-*`, `x-codex-*` and `x-models-etag`. Every other name is kept with `REDACTED` as its value. `remote_control/enroll.rs:415` (`format_headers`) already follows this allowlist pattern. If you keep the denylist, at least send URL-valued headers through `redact_url` and add `jwt`, `assertion`, `csrf` and `xsrf`.

3. **Medium: the websocket log site isn't tested.** `codex-api/src/endpoint/responses_websocket.rs:601-605`
   - Only the HTTP "Request completed" path is exercised. If `redact_headers` were reverted at the INFO site, no test would fail.
   - **Fix:** add a case to `core/tests/suite/log_redaction.rs` that uses `start_websocket_server_with_headers` with `response_headers: vec![("set-cookie", …)]`. Assert both the negative check (value absent from all three sinks) and the positive check (`"set-cookie": "REDACTED"`).

4. **Low (latent): `TransportError` derives `Debug`, which prints the full `HeaderMap` and the unredacted `url`.** `http-client/src/error.rs:7-15`
   - I found no `?err` or `{err:?}` of `TransportError` or `ApiError` reaching tracing today. But one future debug log would leak both the headers and the URL.
   - **Fix:** write a manual `Debug` for `Http` that uses `redact_headers` and `redact_url`.

5. **Low (out of scope, same sink class): two more values reach the logs.**
   - `rmcp` (`transport/auth.rs:2009`) logs the MCP OAuth authorization code at DEBUG under target `rmcp::transport::auth`, which is captured at TRACE (only `rmcp::service` is capped). PKCE limits the impact.
   - `rmcp-client/src/auth_status.rs:183` logs the raw MCP `{url}`, plus an `anyhow` debug chain that may include a reqwest URL.
   - **Fix:** cap `rmcp::transport::auth` at INFO in both sink filters, and use `redact_url(url)` at that call site.

6. **Nit: the redaction marker differs between the two redactors.** Provider headers log `"<redacted>"` while URLs and response headers log `"REDACTED"` (`core/tests/suite/log_redaction.rs:321-323`). Pick one so it's easy to grep for.

7. **Nit: `pub struct RedactedHeaders` has no doc comment.** `http-client/src/log_redaction.rs:54` — add a one-line doc.

### Coverage checked, no issues
- **Header logging in our crates:** only the 3 sites changed in this commit print a full header map. `format_headers` in remote control is a safe allowlist (request id and cf-ray only). `response-debug-context` pulls out named fields only. `UnexpectedResponseError` stores selected fields, not a `HeaderMap`. The network-proxy logs targets, not header values.
- **`Request`/`Provider` Debug:** both derive `Debug` with headers (`http-client/src/request.rs:90`, `codex-api/src/provider.rs:43`), but I found no `?request`, `?provider` or `{response:?}` reaching tracing. The request body trace logs only byte counts.
- **Dependencies:** h2 deliberately leaves header fields out of its frame `Debug`. hyper 1.8.1 logs header counts and names only.
- **Tests:** the core test is sound. Its positive control (`"set-cookie": "REDACTED"`) proves the redacted line was emitted, the negative check covers all three sinks, and the current-thread `#[tokio::test]` makes `set_default` capture reliably. The unit test is a valid whole-string `assert_eq` with `pretty_assertions`.