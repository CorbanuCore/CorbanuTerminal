You are an independent, read-only security reviewer. Do not modify any file. Review commit 1a84d9fb79 on branch
fix/380-trace-header-leak (diff: `git show 1a84d9fb79` or `git diff origin/main...HEAD`) in this checkout.

Issue #380 (`gh issue view 380` if available): with RUST_LOG=trace, tungstenite's client handshake TRACE event
(`trace!("Request: {:?}", ...)`, a `log` crate record bridged into tracing) wrote the websocket upgrade request,
including `authorization: Bearer <API key>`, to codex-tui.log.

Required end state: no credential header (Authorization, api-key, x-api-key, cookies, ChatGPT tokens) from any HTTP or
websocket client library reaches ANY log sink (codex-tui.log, the logs sqlite DB, the /feedback buffer, exec stderr,
OTel) under any RUST_LOG, including user filters like `trace` or `tungstenite=trace`, in every subscriber (TUI, exec,
app-server, mcp-server, ...).

The fix: new crate codex-rs/log-guard. `guard(subscriber)` wraps the whole subscriber and returns false from
`enabled`/`Interest::never` from `register_callsite` for capped library targets, before any layer or per-layer filter
runs; `redact_credentials*` + `RedactingMakeWriter` add pattern-based redaction in the sink writers. Wired into every
`try_init` site. Tests: log-guard unit tests, core/tests/suite/log_redaction.rs (websocket + HTTP sentinel), and
exec/tests/suite/trace_log_credentials.rs (real codex-exec binary).

Please check, with evidence (file:line), and say which are blocking:
1. Correctness of the `Guarded<S>` Subscriber wrapper: does it delegate every method correctly (current_span,
   downcast_raw, try_close, on_register_dispatch, max_level_hint, event_enabled)? Any tracing-subscriber per-layer
   filter (FilterState) hazard? Does it really cover `log`-crate records (tracing-log's dispatch_record checks
   `dispatch.enabled(record.as_trace())` with the original target)? Could Interest::never caching break anything?
2. Coverage: is there any subscriber install site (registry().try_init(), fmt().init(), set_global_default) in the
   workspace that is NOT guarded and could see network library events? Search the repo (rg) yourself. Is the Windows
   sandbox/broker a sink?
3. Cap list completeness: audit the dependency tree (Cargo.lock; sources are under the cargo registry/git checkout
   dirs referenced by `cargo metadata` / $CARGO_HOME) for other crates that log request/response headers or raw bytes
   at TRACE/DEBUG (e.g. hyper, h2, reqwest, ureq/ureq-proto, eventsource-stream, rmcp, oauth2, tonic,
   opentelemetry-otlp, rama, tokio-tungstenite, tungstenite, sqlx). Are any capped crates' DEBUG logs still dangerous?
4. The redaction regexes in log-guard/src/redact.rs: false negatives for header shapes the libraries actually emit,
   and harmful false positives (e.g. hiding diagnostics in normal Codex logs); UTF-8 safety; performance on every event.
5. Test quality: would the tests have failed before the fix (the author reports the core websocket test and the exec
   websocket test fail with the guard removed)? Are they deterministic?
6. Anything else that would let the key reach a sink (e.g. errors carrying headers, Debug of request builders, panics).

Output: a verdict (approve / approve with fixes / request changes), then numbered findings with severity
(blocking / should-fix / nit), file:line, and a concrete suggested fix. Be concise.
