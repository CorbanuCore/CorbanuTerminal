# PF-27-S05 review 2 (Opus 5.5 High, independent re-check, `corbanu exec -m claude-opus-5-5-plan`, commit 3106a131bc)

**VERDICT: APPROVE**

This slice is safe to merge behind the flag. With the flag off, nothing changes. With the flag on, it does what the sprint record now claims for the main model request paths. I only read the code; I did not build it or run the tests.

**Flag off: nothing gets worse**
- `brokered_model_credential` (core/src/client.rs:2741) returns `Ok(None)` right away when `broker_model_auth` is `None`. So `api_auth` stays `resolved_auth.auth`, and `broker` stays `None`.
- With `broker` set to `None`, both `api_client_without_redirects` (:2792) and `build_api_transport` (:2810) take the original direct path, the same as before.
- The websocket check (:2690) only adds `|| self.broker_model_auth.is_some()`, so websocket behavior is unchanged with the flag off.
- The new Fatal error mapping in `broker_http_client` (:315) can only be reached when a credential is brokered.

**Flag on: H3 is fixed**
- On Unix, if the broker can't bind the base URL, the request now returns `CodexErr::Fatal` (:2750-2760). That covers plain HTTP, IPv6 literals and URLs with a query string.
- Broker start or registration failure is also Fatal (:2762-2765).
- On non-Unix, any provider API key returns Fatal (:2776-2785).
- `CodexErr::Fatal` is in the not-retryable list in `is_retryable` (protocol/src/error.rs:370). So these failures stop the request; they don't fall back to sending the key directly or loop on retries.
- Every HTTP caller of `current_client_setup` that I checked (:1482, 1630, 1726, 3182, 3739, 3995) passes `client_setup.broker` into the transport builder. I found no other direct client construction in client.rs.
- The two websocket callers (:3585 prewarm and :4304 stream) are both gated by `responses_websocket_enabled()`, which is always false with the flag on.
- L1 (no redirects followed) and M3 (accepted and documented) are as described.

**Findings (none blocking)**
1. **Low: the websocket fallback guard is indirect.** The prewarm path (client.rs:3585) wraps a setup error, including the new Fatal broker errors, into `ApiError::Stream`, which is retryable. Today that path can't run with the flag on, because websockets are turned off. It is still the one place where a broker failure would become retryable. **Fix:** add `debug_assert!(client_setup.broker.is_none())` in the websocket stream and prewarm paths, or return early with an error there when `broker_model_auth.is_some()`. That way a later change to websocket enablement can't quietly send a brokered `api_auth` over a socket.
2. **Low: no direct non-Unix check.** The non-Unix fail-closed branch has no test that runs it. **Fix:** add a `#[cfg(not(unix))]` unit test, or note in the sprint record that PF-27-S06 will cover it.
3. **Info: H1/H2/M1/M2 remain.** These are web search, image generation, model catalog, realtime websocket and the other `auth_provider_from_auth` users. They still send provider keys directly when the flag is on. The record lists them as Remaining, which is accurate, but the feature can't be called fail-closed until they're done. **Fix:** keep the flag `UnderDevelopment` and default-off, and make sure the feature docs and config description don't claim the broker covers all model traffic until those paths route through the broker or fail with Fatal.

## Disposition

- 1: `connect_websocket` now refuses outright under `broker_model_auth`.
- 2: the non-Unix branch has no direct test here; Windows is PF-27-S06 (P1). Recorded.
- 3: flag stays default-off and UnderDevelopment; the record lists the paths still outside the broker.
