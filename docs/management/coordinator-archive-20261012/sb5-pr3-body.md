PF-27-S05 slice 1. Behind the new default-off `broker_model_auth` flag (UnderDevelopment); nothing changes with the flag off.

**What it does**

When a provider's auth is one plain API key, Core hands the key to the contained isolated broker (Seatbelt on macOS, seccomp on Linux). The key can come from:
- the environment, or a key stored in the vault or legacy store;
- an OpenAI API-key login;
- `experimental_bearer_token`.

It is sent as either `Authorization: Bearer` or `x-api-key`. Core does this once per process. After that, Core keeps only an opaque `ModelCredential`. That credential is bound to the base URL's HTTPS origin and path prefix.

Every `ModelClient` request then goes as plain HTTP over the broker's private Unix socket, with a single-use HMAC frame for its exact origin, method and path. This covers Responses, Chat, Anthropic Messages, compact, memories and the realtime call. The broker attaches the key and makes the HTTPS request. Redirects come back to Core without being followed.

**Fail-closed**

Core never falls back to sending the key itself. Each of these cases returns a `Fatal` error, which is not retried:
- the broker cannot start, or has died (it is never restarted inside the same process, the same rule as PF-27-S04);
- the provider URL cannot be brokered (plain HTTP, an IPv6 literal, or a query);
- the platform is not Unix.

Responses websockets are off under the flag.

**Not covered yet** (listed as Remaining in the sprint record; the flag stays under development)
- Vault labels are still decrypted by Core, not inside the broker.
- Env keys are not yet removed from Core's environment.
- These paths still attach the key themselves: web search, image generation, the model catalog refresh, the realtime conversation websocket, and other `auth_provider_from_auth` users.
- ChatGPT sign-in is not brokered.

**Changes**
- network-proxy:
  - `RegisterModel` and a `ModelBindingWire` that the broker enforces (exact host and port, a plain path under the prefix);
  - a public `model_auth` facade with no call that returns the key;
  - protocol v3.
- model-provider: `provider_api_key()`, which returns the key only when direct auth would send exactly one header.
- http-client: `HttpClient::unix_socket`.
- core:
  - `model_broker_auth.rs`: a process-wide broker, a credential cache keyed by a salted hash, `BrokeredModelAuthProvider`, and the URL rewrite;
  - `client.rs`: every model transport goes to the broker socket when the key is brokered;
  - `with_broker_model_auth` at the three `ModelClient` construction sites.
- features: `broker_model_auth`. The config schema is regenerated.

**Gate**
- Tests:
  - `pf_27_s05` in network-proxy (3), model-provider (2) and core (3);
  - `just test -p codex-network-proxy -p codex-secret-broker -p codex-model-provider -p codex-http-client -p codex-features`: 577 passed;
  - core lib subsets (client, config, session, memory_stage_one, model_broker_auth): 910 passed.
- Linux (RTX box): clippy `-D warnings` on the six crates, plus the `pf_27_s05` tests.
- Opus 5.5 High:
  - [review 1](qa/security-levels/sprints/PF-27-S05/review-opus-1.md): CHANGES REQUESTED. H3 is fixed: an unbrokerable key now fails instead of being sent. H1, H2, M1 and M2 are recorded as Remaining.
  - [review 2](qa/security-levels/sprints/PF-27-S05/review-opus-2.md): APPROVE. Nits applied: `connect_websocket` refuses under the flag.
- GLM 5.2 tmux videos, using the real Z.AI key from the vault:
  - [model key brokered](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-27-s05-pf27s05-model-key-brokered-ead7691092c1-2026-10-06.mp4): the answer streams back, and the log shows `containment=seatbelt` and `model provider key held by the credential broker (host=api.z.ai, port=443, path=/api/paas/v4, header=Bearer)`.
  - [broker death fails closed](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-27-s05-pf27s05-broker-death-fails-closed-ead7691092c1-2026-10-06.mp4): after the broker is killed, the next prompt fails at once with "Fatal error: the isolated credential broker is unavailable…".
- Evidence: `qa/security-levels/sprints/PF-27-S05/README.md`.
