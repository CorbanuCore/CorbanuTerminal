# PF-27-S05 evidence: model-client auth in the broker (slice 1)

Feature flag: `broker_model_auth` (UnderDevelopment, default off). With the flag off nothing changes.

## What slice 1 covers

When a provider's auth is one plain API key, Core registers the key once per process with a contained broker. The
key can come from the environment or a vault/stored provider key, an OpenAI API-key login, or
`experimental_bearer_token`. The registration binds the key to the base URL's HTTPS origin, its path prefix, and
whether it is sent as `Authorization: Bearer` or `x-api-key`. After that, Core keeps only an opaque reference.

Every `ModelClient` request goes as plain HTTP over the broker's private Unix socket, carrying a single-use signed
frame for that request's exact origin, method and path. The broker then attaches the key and makes the HTTPS
request. This covers Responses, Chat, Anthropic Messages, compact, memories and the realtime call.

There is no fallback to sending the key directly. The request fails, and is not retried, when:
- the broker cannot start or has died;
- the provider URL cannot be brokered (plain HTTP, an IPv6 literal, or a query string);
- Core is running on a platform other than Unix.

Responses websockets are turned off. The paths that still send keys directly are listed under Remaining in the sprint
record.

## Tests

| Crate | Tests | Result |
| --- | --- | --- |
| network-proxy | `pf_27_s05_model_key_is_attached_only_inside_the_broker` (both header styles, single-use frames, redirect returned unfollowed), `pf_27_s05_model_key_is_bound_to_its_origin_and_path_prefix` (checked by Core and by the broker; bad bindings refused), `pf_27_s05_model_broker_death_fails_closed` | pass (macOS; Linux on the RTX box) |
| model-provider | `pf_27_s05_plain_api_keys_are_extracted_with_their_header` (matches direct auth), `pf_27_s05_sign_in_and_other_auth_are_not_brokered` | pass |
| core | `pf_27_s05_base_url_binds_origin_and_path_prefix`, `pf_27_s05_requests_are_rewritten_to_plain_http_for_the_broker`, `pf_27_s05_unbrokerable_provider_key_fails_closed_under_the_flag` (Fatal, not retried, websockets off; flag-off control) | pass |

Wider runs:
- `just test -p codex-network-proxy -p codex-secret-broker -p codex-model-provider -p codex-http-client -p codex-features`: 577 passed.
- `codex-core` lib subsets (client, config, session, memory_stage_one, model_broker_auth): 910 passed.
- Linux clippy (`-D warnings`) on the RTX box for the six changed crates: clean.

## Live checks (GLM 5.2 via Z.AI, real key from the vault, disposable profile)

| Demo | Observed |
| --- | --- |
| `pf27s05-model-key-brokered` | The answer streams back normally. The log shows `isolated credential broker started containment=seatbelt` and `model provider key held by the credential broker (host=api.z.ai, port=443, path=/api/paas/v4, header=Bearer)`. |
| `pf27s05-broker-death-fails-closed` | The first prompt is answered. After the broker process is killed, the next prompt fails with "The isolated credential broker is unavailable; restart Corbanu to start a new one" and nothing is sent with the key. |

Videos are listed in [`qa/demos/index/PF-27-S05.md`](../../../demos/index/PF-27-S05.md).

## Independent review

Opus 5.5 High via `corbanu exec -m claude-opus-5-5-plan`, run read-only.

[Review 1](review-opus-1.md): **CHANGES REQUESTED**.
- H3 is fixed. A key the broker cannot hold now fails instead of being sent.
- L1: a test now shows that redirects come back unfollowed.
- M3 is documented.
- H1, H2, M1 and M2 are recorded as Remaining. These are web search, image generation, the model catalog refresh,
  the realtime conversation websocket and other users of `auth_provider_from_auth`.

[Review 2](review-opus-2.md): **APPROVE**. Its nits have been applied or recorded.
