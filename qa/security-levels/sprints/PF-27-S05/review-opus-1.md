# PF-27-S05 review 1 (Opus 5.5 High, independent, `corbanu exec -m claude-opus-5-5-plan`, commit e57e55ef80)

VERDICT: CHANGES REQUESTED

I read the code only; I did not build or run tests. The broker side looks sound: model and agent credentials are kept apart, frames are single-use and bound to one exact request, and a dead broker fails closed. The problem is on the Core side. With the flag on, several paths still send the raw provider key straight from Core. That breaks the sprint mandate that raw credentials exist only in the trusted broker, with no raw-key fallback.

## High

**H1. Web search and image generation still send the key directly**
- **Where:** `ext/web-search/src/tool.rs:105-110` and `ext/image-generation/src/backend.rs:54`.
- **What happens:** both call `provider.api_auth()` and use a normal `create_client()`. That returns the real Bearer/API-key auth provider, not the brokered one. So under `broker_model_auth`, every web-search or image call sends the raw key from Core.
- **Fix:** add a provider-level hook that returns `BrokeredModelAuthProvider` plus the broker's Unix-socket `HttpClient`. Simplest option: wrap the provider in core so these extensions get the brokered pair. Failing that, make these extensions refuse to run under the flag.

**H2. The model catalog fetch sends the key directly**
- **Where:** `model-provider/src/models_endpoint.rs:87` uses `resolve_provider_auth` with a normal client.
- **What happens:** `/models` refreshes at startup and periodically, carrying the raw key.
- **Fix:** route it through the same broker credential (same base URL, so it falls under the prefix). If that isn't possible, skip the remote catalog under the flag.

**H3. Silent raw-key fallback for URLs that can't be brokered and on non-Unix**
- **Where:** `core/src/client.rs:2742-2749` and `core/src/client.rs:2766-2770`. The cause is `binding_for_base_url` in `model_broker_auth.rs`.
- **What happens:** for `http://` providers, HTTPS IPv6 hosts, base URLs with a query, and all of Windows, Core logs a warning and sends the key directly. That is a raw-key fallback. A user on Windows or with an IPv6 endpoint believes they are protected when they are not.
- **Fix:** when the flag is on and `provider_api_key` returns `Some`, fail the request with a clear error instead of returning `Ok(None)`. Add IPv6 support to the binding, or reject it explicitly. If plain-HTTP or loopback providers are meant to be exempt, document the exemption and get it accepted by product authority.

## Medium

**M1. Realtime conversation websocket is unverified**
- **Where:** `core/src/realtime_conversation.rs:1145-1150` and `:2495`.
- **What happens:** these read `auth_manager.auth()` directly and appear to build realtime websocket auth outside `current_client_setup`. Only Responses websockets are disabled under the flag, so the realtime conversation websocket probably still sends the key. The WebRTC sideband (`client.rs:817`) gets empty headers under the broker, so it fails closed, which is acceptable.
- **Fix:** disable realtime websockets under the flag, or make them fail when the provider is brokered. Add a test.

**M2. Other `auth_provider_from_auth` callers are unaudited**
- **Where:** `core/src/mcp_openai_file.rs:179`, `codex-mcp/src/connection_manager.rs:270`, `core-plugins`, `core-skills`, `analytics`.
- **What happens:** with an OpenAI API-key login, these send the same key to OpenAI backends or uploads. Each one is outside the broker.
- **Fix:** either broker them, or list each as an explicit, accepted exclusion in the sprint record. Today the mandate ("including the ones Core uses itself") is not met.

**M3. A failed broker start is permanent**
- **Where:** `model_broker_auth.rs`, the `get_or_insert_with` start path.
- **What happens:** one failed start (or a broker that later dies) makes every model request fail until Corbanu restarts. That fails closed, which is correct. But the global mutex is also held across the blocking spawn/register calls, so all sessions queue behind it.
- **Fix:** acceptable as is. Document it and add a test that the error message reaches the TUI.

## Low

**L1. Model credentials skip connection pinning**
- **Where:** `server.rs`, the `CredentialKind::Model` branch.
- **What happens:** model credentials skip the `pin_connections` / `pinned_addrs` check that agent (Provider) credentials get, so the broker does its own DNS lookup. That is fine if the `UpstreamClient` refuses redirects and private/loopback addresses.
- **Fix:** confirm that `UpstreamClient` doesn't follow redirects for model requests, and add a test for a 30x response.

**L2. Path matching is case-sensitive**
- **Where:** `ModelBindingWire::allows`.
- **What happens:** the query is stripped before the prefix and plain-path checks, and the full path plus query goes upstream. Encoded `%2e`, `%2f` and `%25` are rejected, and the prefix only matches on a segment boundary. That is all correct. The only gap: a server that treats paths case-insensitively could match a different-case route outside the prefix.
- **Fix:** none needed. Note it as accepted.

**L3. Missing tests**
- No test that, with the flag on, no outbound request from web-search, image generation, models or realtime carries `Authorization` or `x-api-key`.
- No test for the Windows/IPv6 refusal.
- No test for a frame being reused across two requests at the Core layer.

## Verified OK

- **Main model paths are switched.** Responses, chat, Anthropic, compact, memories, the realtime-calls route and all the no-redirect clients now go through the broker transport.
- **Headers are stripped both ways.** `apply_auth` removes `Authorization` and `x-api-key` before sending, and the broker removes them again.
- **Key doesn't leak.** The `RegisterModel` Debug output is redacted, the key is zeroized after registration, the header value is marked sensitive, and the cache stores only a salted hash of the key.
- **Host and port are normalized.** Hosts are lowercased on both sides and ports use the scheme default, so an explicit `:443` matches.
- **Flag off is unchanged.** `broker_model_auth` defaults to `None`, so every branch falls back to the original code.
- **Windows should compile.** On non-Unix, `BrokeredCredential` is an empty enum, so the broker branches can never run.
