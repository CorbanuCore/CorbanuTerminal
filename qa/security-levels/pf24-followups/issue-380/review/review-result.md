**Verdict: request changes.** The `Guarded` wrapper itself is sound, but one leak still breaks the required end state, and it happens with the default configuration. AWS SDK targets are not capped, so AWS credentials can reach the `/feedback` buffer and the logs database even when `RUST_LOG` is unset.

I did not run any tests; the checkout is read-only. Everything below comes from reading the source and the dependency checkouts.

## Findings

**1. Blocking: AWS SDK targets are not capped, so AWS credentials reach default sinks.**
- **Where:** `log-guard/src/guard.rs:14-40`. The leak comes from Codex's own `aws-auth/src/signing.rs:51`.
- **Session token on every signed Bedrock request:** that line calls `aws_sigv4::http_request::sign` with `SigningSettings::default()`, which includes the session token. aws-sigv4 1.3.7 then logs `trace!(canonical_request = %creq, …)` at `http_request/sign.rs:308`. The canonical request is printed with `Display` (`canonical_request.rs:464-473`), so it contains `x-amz-security-token:<session token>`. The same file logs `request = ?request` at line 252, which also prints the request body.
- **SSO and OIDC credentials:** aws-smithy-runtime 1.9.5 logs `trace!(request = ?request, "transmitting request")` unconditionally (`orchestrator.rs:401`). For SSO this includes `x-amz-sso_bearer_token`, which the generated code does not mark sensitive (`aws-sdk-sso …/shape_get_role_credentials.rs:119`). It also prints in-memory request bodies (`body.rs:98-101`), e.g. SSO-OIDC `CreateToken` with its refresh token and client secret.
- **Why it hits default sinks:** the feedback layer (`feedback/src/lib.rs:215`) and `log_db::default_filter()` (`state/src/log_db.rs:54-62`) capture TRACE for every non-`log` target. These are native `tracing` events, so no `RUST_LOG` is needed. The TUI log also gets them under `RUST_LOG=trace`.
- **Redaction misses it:** none of these names or shapes match `redact.rs`.
- **Fix:** add caps for `aws_sigv4`, `aws_smithy_*`, `aws_sdk_*` and `aws_config` at DEBUG. Also add `x-amz-security-token` and `x-amz-sso_bearer_token` to the redaction pattern. Add a regression test that signs with session credentials and asserts the token is absent from the feedback buffer and logs DB.

**2. Should-fix: the header pattern misses real `Debug` shapes** (`redact.rs:20`). The value capture stops at the first quote, so a wrapper type gets "redacted" instead of the secret:
- aws-smithy prints headers as `"authorization": HeaderValue { _private: H0("AWS4-…") }`, so only `HeaderValue { _private: H0(` is replaced.
- `x-api-key: Some("…")` is missed the same way.
- Tuple form `("api-key", "…")` has no `:` or `=` separator, so it is missed entirely.
- Bearer tokens survive these shapes only because of the separate bearer pattern. Azure `api-key` and other raw keys do not.
- **Fix:** allow a `,` separator for tuples. When the value does not start with a quote, also redact the first quoted string within a short window. Add tests for these three shapes.

**3. Should-fix: the header-name list is incomplete** (`redact.rs:20`). Add:
- `x-amz-security-token`, `x-amz-sso_bearer_token`
- `ocp-apim-subscription-key`, `x-auth-token`, `private-token`, `x-access-token`

Users can also put credentials under any name through provider `http_headers` / `env_http_headers`. Consider registering those configured names with the redactor at runtime.

**4. Should-fix: a marker prefix can hide a later credential** (`redact.rs:46-54`). `is_redacted` checks only the prefix of a capture that can run to end of line. So an unquoted line like `authorization: <redacted> x-api-key: real` is skipped as a whole. Only treat a capture as already redacted if it is exactly the marker, or re-scan the text after the marker.

**5. Nit: diagnostic false positives.** For unquoted output, the value runs to end of line. Lines like `authorization=required model=…` or `Authorization: failed for server X` lose the rest of the line. A grep found no current collisions in Codex's own log calls, so this is low risk. If it matters later, limit non-cookie values to an optional scheme plus one token.

**6. Nit: URL query credentials are not redacted.** tungstenite's DEBUG `Trying to contact {uri}` / `Redirecting to {uri:?}` and reqwest error messages include the query string. User `query_params` such as `key=`, `api_key=` or `access_token=` are not caught; only `api-key=` is. Suggest a pattern for `[?&](api[-_]?key|key|access_token|token|sig)=…`.

**7. Nit: a few install sites are unguarded, and nothing prevents new ones.** I searched with rg; the commit wires all the important sites (TUI, exec, app-server, mcp-server, exec-server telemetry, login, cloud-tasks, execve wrapper). The rest are low risk:
- `code-mode-host/src/main.rs:16-20`: fixed INFO level and ignores `RUST_LOG`, so it is safe today.
- `app-server-test-client/src/lib.rs:2378`: a dev tool that exports OTel spans only.
- `execpolicy-legacy` uses `env_logger` but does no networking.
- Every other `set_default` / `with_default` is inside `#[cfg(test)]`.
- **Windows sandbox:** it is not a tracing sink, so library events never reach it. It does write command previews to its own log (`windows-sandbox-rs/src/logging.rs:72-99`), which could carry user-typed `curl -H` credentials. That is outside #380; track it separately.
- **Fix:** guard `code-mode-host` for consistency, and add a CI check that any non-test `try_init`, `.init()` or `set_global_default` goes through `codex_log_guard::guard`.

**8. Nit: tests.**
- Would they fail without the guard? Yes, where it matters:
  - The log-guard unit test uses unredacted sinks plus a control test.
  - The core and exec websocket tests fail through the "no `tungstenite::handshake::client` Request line" assertion. The key check alone would not fail, because redaction masks the key.
- The HTTP tests have no positive control: reqwest, hyper and h2 never log that header today. They are canaries, not regression tests.
- Determinism looks fine. The positive checks ("Client handshake done.", "reqwest::") prevent a silent pass if events land on another thread.
- **Fix:** add a test that `tracing_opentelemetry` context and parent propagation still works through `Guarded`. That depends on the `downcast_raw` forwarding (`guard.rs:136-142`), which is correct, but nothing tests it.

## Your six questions

1. **Wrapper correctness: no defects found.**
   - **Delegation:** every method is delegated, including `current_span`, `try_close`, `clone_span`, `on_register_dispatch`, `max_level_hint` and `event_enabled`. `downcast_raw` checks for its own type and then forwards, so `WithContext` and registry downcasts still work.
   - **Per-layer filter state:** no hazard. The cap check runs before `inner.enabled` and `inner.register_callsite`, so the per-layer filter state and the stashed interest are never touched for capped callsites.
   - **`log` crate records:** covered. tracing-log 0.2.0 checks `dispatch.enabled(&record.as_trace())` with the original target (`lib.rs:166-171`). `LogTracer::enabled` checks again (`log_tracer.rs:191-194`), and the `interest-cache` feature is off.
   - **`Interest::never` caching:** safe. The caps are static, and the interest is recomputed whenever a dispatcher is added or `rebuild_interest_cache` runs.
2. **Coverage:** see #7.
3. **Cap list:**
   - **AWS: blocking**, see #1.
   - **DEBUG output of the crates already capped is safe:**
     - h2 0.4.16 logs frames at DEBUG, but its `Headers` `Debug` leaves out the header fields.
     - hyper, hyper-util and rama log no header values.
     - ureq 3.1.4 redacts sensitive headers itself (`util.rs:185-229`).
     - tungstenite's DEBUG output is URIs and close frames only; see #6.
     - The otlp and `rmcp::transport::auth` INFO caps cover their DEBUG logging of headers and tokens.
   - **Not capped, but no credential logging found:** oauth2 has no logging. tonic and sqlx do not log headers or bound values.
4. **Regexes:** see #2–#6. Both UTF-8 safety and performance are fine:
   - **UTF-8:** spans begin and end at ASCII delimiters or at the end of the buffer, and the code converts with lossy UTF-8 anyway.
   - **Performance:** the patterns compile with Unicode off (`(?-u)`), so the regex set takes the fast DFA path. Captures run only on a hit.
5. **Tests:** see #8.
6. **Other leak paths:**
   - The main one is AWS signing (#1).
   - Codex's own `http-client` `Request` derives `Debug`, but no production code logs it. Auth headers built in `model-provider/src/auth.rs` already call `set_sensitive`.
   - Websocket and reqwest errors carry response headers or URLs only; the URL case is #6.
   - No panic paths found.