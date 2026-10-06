# Response headers in the logs (#183/#201 follow-up)

`codex-http-client` logged every response's headers on its DEBUG "Request
completed" line, and the Responses websocket logged its handshake response
headers at INFO. The logs DB and the `/feedback` buffer record these at any
`RUST_LOG`, so `set-cookie` and any other credential-bearing header reached
them.

## Fix

- `codex_http_client::redact_headers` keeps every header name but shows values
  only for an allowlist of diagnostic headers:
  - exact names: `content-type`, `content-length`, `date`, `server`, `vary`,
    `etag`, `retry-after`, `x-request-id`, `cf-ray` and others;
  - prefixes: `openai-*`, `x-codex-*`, `x-oai-*`, `x-ratelimit-*`.

  Every other value, and any value marked sensitive, is `REDACTED`. It is used
  at the three log sites.
- `TransportError`'s `Debug` redacts the URL and the header values (from the
  review).
- The `/feedback` buffer drops events bridged from the `log` crate, as the logs
  DB already does. tungstenite logs its handshake request, including
  `Authorization`, through that bridge at trace level (from the review).

## Gate evidence

- **Tests:** after `just fmt` and `just fix`, all passed:
  - `just test -p codex-http-client` (80)
  - `just test -p codex-feedback` (10)
  - `just test -p codex-core --test all log_redaction` (3)
  - `just test -p codex-api` (241)

  New tests:
  - the allowlist, and the redacting `TransportError` Debug;
  - feedback drops the `log` target;
  - the core log-sink regression serves `set-cookie` on an HTTP response and on
    a websocket handshake. It checks that the value is absent from the trace
    file, the feedback buffer and the DB bytes, and that
    `"set-cookie": "REDACTED"` is present.
- **tmux run on GLM 5.2:** one Z.AI turn with `RUST_LOG=trace` (`tmux-run/`).

  | Build | `set-cookie` with a value in `codex-tui.log` | in the logs DB / WAL | `"set-cookie": "REDACTED"` |
  | --- | --- | --- | --- |
  | Before (main) | 1 | 1 (WAL) | 0 |
  | After | 0 | 0 | 1 |

  After the fix, Z.AI's headers log as follows:
  - shown: `date`, `content-type`, `transfer-encoding`, `connection`, `vary`
    and `x-request-id`;
  - `REDACTED`: `set-cookie`, `alt-svc`, `ga-traceid` and `x-log-id`.

  Only counts were printed; the cookie value was never displayed.
- **Review:** Opus 5.5 High, approve with fixes; see `review/`.
- **Video:** none. Nothing on screen changes; the evidence is the logs.
