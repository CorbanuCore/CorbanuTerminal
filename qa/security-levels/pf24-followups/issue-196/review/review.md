**Verdict: approve with fixes.** The `ModelProviderInfo` `Debug` fix is correct and complete for that struct. Because every field is destructured, adding a field without a redaction decision won't compile. It also fixes every derived `Debug` that contains the struct, such as `Config` and `model_providers`. But `query_params` values still reach the same log sinks through request URLs, and the regression test can pass without ever seeing the line it guards. I only read the code; I didn't build or run anything because the sandbox is read-only.

### Findings

1. **High: `query_params` values still reach the trace log, logs DB and `/feedback` through request URLs.**
   - **Where:**
     - `http-client/src/transport.rs:88-96`: `trace_request` logs `"{method} to {url}"` on every model request. The model client uses `create_client()`, which has request logging on, and the log DB layer makes `enabled!(TRACE)` true.
     - `http-client/src/client.rs:118-140` and the matching code in `send()`: they log `url = %url` at DEBUG.
     - `http-client/src/transport.rs` `map_error`: `reqwest::Error::to_string()` includes the URL.
     - `codex-api/src/endpoint/responses_websocket.rs:587,608`: they log the websocket URL at INFO.
   - **Problem:** `Provider::url_for_path` adds `k=v` for every query param. So a provider using `?key=…` (Gemini-style) leaks the key on every request, even though `Debug` now hides it. This is the same class of bug as #196.
   - **Fix:**
     - Add one `url_for_log(&str) -> String` helper in `codex-http-client`. It should keep query names, replace values with `<redacted>` and remove userinfo. Use it at all of these log sites.
     - Use `err.without_url()` in `map_error`.
     - Add a `query_params` sentinel to the integration test (finding 2). It should fail until this is fixed.

2. **Medium: the integration test has no positive check for the line it guards** (`core/tests/suite/user_shell_cmd.rs:744-806`).
   - **Problem:** The only positive check is `"spawn_child_async"`. If `Configuring session … provider=` moves to another thread, changes level or is removed, the `PROVIDER_KEY` check passes without testing anything.
   - **Missing coverage:** `http_headers` and `query_params` are only tested by the unit test, not end to end.
   - **Fix:**
     - Also check that the trace file and the DB rows contain `experimental_bearer_token: Some("<redacted>")`.
     - Set `http_headers = {"X-Sentinel": HEADER_SECRET}` and `query_params = {"key": QUERY_SECRET}` on the provider, and add both to `secrets`.
     - Check that wiremock received both, the same way the test already checks the bearer header.

3. **Low: the test can't see events from other threads** (`user_shell_cmd.rs:605`, `set_default`).
   - **What it catches:** `tokio::spawn` tasks on the current-thread runtime are captured, so the session, turn and tool tasks are covered.
   - **What it misses:** Events from `spawn_blocking` threads and from `std::thread` runtimes go to the global dispatcher, which is unset, and are silently dropped. For example, guardian review (`core/src/guardian/review.rs:686`) builds its own runtime and session and logs its own `Configuring session` line.
   - **Fix:** Document this limit on `TraceSinks`. For full coverage, add a separate test binary that calls `set_global_default`. This doesn't block the PR.

4. **Low: other types still carry secrets in their `Debug` output.** I found no current log site for any of these, but each is one `?x` away from a leak.
   - `model-provider/src/auth.rs:88-92`: `ApiKeyHeaderAuthProvider` derives `Debug` with `api_key: String`. This is the resolved `env_key`/vault key.
   - `codex-api/src/provider.rs:42`: `Provider` derives `Debug` over a `HeaderMap` holding `http_headers` values and resolved `env_http_headers` values.
   - `config/src/thread_config/proto/…`: the prost `ModelProvider` derives `Debug` with `experimental_bearer_token` and `http_headers`.
   - **Fix:** Call `HeaderValue::set_sensitive(true)` in `build_header_map` (`model-provider-info/src/lib.rs:1136,1149`) and in every auth provider's `add_auth_headers`, so `HeaderMap` `Debug` prints `Sensitive`. Hand-write `Debug` for `ApiKeyHeaderAuthProvider`.

5. **Low: `base_url` userinfo is printed in clear** (`model-provider-info/src/lib.rs:985`).
   - **Problem:** A `base_url` like `https://user:token@host/v1` puts the token in the `Debug` output and in every logged URL (finding 1).
   - **Fix:** In `Debug`, parse the URL and replace the username and password with `<redacted>`. Use the same logic in the helper from finding 1.
   - The `auth` command args (line 992) are usually subcommands, not secrets, so keeping them is acceptable. Mention it in the doc comment.

6. **Redaction policy: correct as written.**
   - Redacting all `http_headers` and `query_params` values is right. There's no reliable way to tell which names hold credentials (`api-key`, `x-goog-api-key`, `key`, `sig`, `code`), and keeping the names keeps the output useful for diagnosis.
   - `env_http_headers` values are environment variable names, so keeping them is right.
   - `chat_completions_provider` holds routing preferences only, so it's fine to keep.

7. **Nit: unit test style** (`model-provider-info/src/model_provider_info_tests.rs:1813-1816`).
   - `assert_eq!(….contains("<redacted>"), false)` should be `assert!(!…)`.
   - The output is deterministic because keys are sorted, so per `codex-rs/AGENTS.md` ("prefer deep equals") one `assert_eq!` on the full `format!("{provider:?}")` string would replace the five substring checks.
   - The `Default` assertion is close to "testing statically defined values"; consider dropping it.

8. **Nit: test layout** (`core/tests/suite/user_shell_cmd.rs`).
   - `TraceSinks` and the provider-key test are about log redaction, not `!` commands. Move both into a `log_redaction.rs` suite module; this file is now 807 lines.
   - The `contains` closure panics if a needle is empty (`windows(0)`). Assert that the needles are non-empty, or use `memchr::memmem`.
   - Import `std::sync::Arc` instead of writing the inline paths.

9. **Nit: module size** (`model-provider-info/src/lib.rs`, now 2149 lines). Consider moving the `Debug` impl and `RedactedValues` into a small `debug.rs` submodule, per the 500-line module target.