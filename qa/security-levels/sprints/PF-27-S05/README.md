# PF-27-S05 evidence: model-client auth in the broker

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

Responses websockets are turned off. (Slice 1 text, kept for history; round 6 below brokers or refuses the paths
that still sent keys directly.)

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
| `pf27s05-broker-death-fails-closed` | The first prompt is answered. After the broker process is killed, the next prompt fails at once (not retried) with "Fatal error: the isolated credential broker is unavailable; restart Corbanu to start a new one", and nothing is sent with the key. |

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

## Round 6: the rest of the sprint (PR from `feat/pf27-s05-broker-finish-20261006`)

What changed is listed under Done in the [sprint record](../../../../docs/sprints/archive/p0-security-levels/pf-27-s05-model-client-auth-broker.md),
together with the known limits.

### New tests

| Crate | Tests |
| --- | --- |
| network-proxy | `pf_27_s05_stored_key_is_read_inside_the_broker`, `pf_27_s05_env_keys_are_handed_over_removed_and_unregistered`, `pf_27_s05_raw_key_is_not_left_in_core_memory` (best-effort scan of writable memory with a positive control; Linux also checks `/proc/self/environ`), `pf_27_s05_vault_lock_is_created_and_never_a_symlink`, `pf_27_s05_take_env_var_*` |
| model-provider | `pf_27_s05_brokered_provider_keys_are_named_not_read`, `..._held_values_and_sign_in_tokens_go_to_the_broker`, `..._sign_in_for_an_env_key_provider_is_brokered`, `..._required_broker_not_running_fails_closed`, `..._command_auth_is_sent_as_before`, `..._flag_off_direct_auth_is_unchanged`, `..._header_only_first_party_auth_gets_no_credential`, `..._brokered_provider_hands_out_a_placeholder_not_the_key` |
| http-client | `pf_27_s05_broker_frame_requests_go_only_to_the_broker_socket` |
| core | `pf_27_s05_non_unix_start_is_the_refusing_broker`, `pf_27_s05_platform_without_broker_refuses_every_credential` |
| secrets | `already_private_permissions_are_not_rewritten` (a read needs no chmod, so the sandboxed broker can open the vault) |

### Results

- Affected crates: 1312 passed.
- Core subsets: 1099 passed. Two `suite::client::skills_*` tests fail because the real `~/.agents/skills` leaks into
  them; this is unrelated to the sprint.
- Linux, RTX box: clippy `-D warnings` is clean on the changed crates, and the `pf_27_s05` tests pass (10).

### Live runs

All runs used GLM 5.2 via Z.AI with keyring isolation (`CORBANU_TEST_NO_NATIVE_KEYRING=1`) in a disposable profile.
The vault key for each demo profile stayed in the profile's `keyring-fallback` file, never the login keychain.

| Demo | Observed |
| --- | --- |
| `pf27s05-env-key-handed-over` | The session answers through the broker. `ps -E` on Core shows `ZAI_API_KEY` overwritten, and the log reads "provider keys handed to the credential broker and removed from the environment: ZAI_API_KEY". |
| `pf27s05-vault-key-in-broker` | The key exists only in the disposable profile's vault, and Core's launch environment has no `ZAI_API_KEY`. The session answers, with `containment=seatbelt` and "credential held by the credential broker (host=api.z.ai, …)". |

During the first vault run the broker could not read the vault: the secrets layer chmods `local.age` on every read,
and the broker's sandbox denies that. Fixed in `secrets/src/local.rs`.

### Independent review

Opus 5.5 High via OpenRouter, run read-only in a keyring-isolated profile.

- [Review 3](review-opus-3.md): **CHANGES REQUESTED**. Fixed:
  - fail closed before the broker runs;
  - a sign-in used for an env-key provider is brokered;
  - the vault lock is created before containment, and never through a symlink;
  - a refused hand-over now fails closed;
  - registration happens outside the lock;
  - slots are stable;
  - the tests were fixed.

  Scrub thread-safety and the scope of the memory test are recorded as known limits.
- [Review 4](review-opus-4.md): **CHANGES REQUESTED**. Fixed:
  - command auth is sent as before;
  - the broker is installed whenever the process requires one;
  - keys are scrubbed when the broker fails to start;
  - stale snapshots no longer replace a newer registration;
  - the record and claims are corrected.
- [Review 5](review-opus-5.md): **APPROVE**. Its two documentation notes are applied. Its third note is recorded
  as a follow-up: a stale snapshot that starts after a refresh can still swap the copy back, costing one failed
  request; monotonic versions for sign-in slots would end that.
