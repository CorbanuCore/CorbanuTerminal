# PF-13-S03 final-tree evidence

- Date: 2026-08-25 UTC
- Baseline commit: `8185a6f3f88ad078c6b56b9a4a70a041ab8cd509`
- Implementation commit: `2ae49b3bf0daa3606a71ca8a0f6683b9633a5e34`
- Tested code tree: `2ae49b3bf0daa3606a71ca8a0f6683b9633a5e34`
- Product requirement: `docs/corbanu-product-spec.md`, **Required trust boundaries** — “Permit agents to reference credentials only by label; resolve them solely inside the trusted execution boundary.”
- TUI applicability: none; this sprint changes no interactive surface.

## Result

Core can install one vault-backed OpenAI credential route in the managed network
proxy. The child receives only an opaque OpenAI-shaped reference. Immediately
before outbound header injection, the proxy validates the complete
transport-derived request and passes its scheme, normalized host, port, method,
path, capability id, and full actor/session/task-bound authority to Core. Core
then rechecks current time and revocation state and asks Vault to resolve the
credential through its zeroizing callback boundary.

The opt-in route permits only the authorized `POST` request to HTTPS
`api.openai.com:443` and an exact authorized path under `/v1/`. It refuses
plaintext HTTP, lookalike and subdomain hosts, alternate ports, other methods,
adjacent or query-mutated paths, missing opaque references, explicit
authorization collisions, MITM hook collisions, stale authority, and a second
use caused by retry or redirect. Existing Permissive broker behavior remains
unchanged when the scoped route is not installed.

## Exact request and denial matrix

| Case | Expected result | Evidence |
| --- | --- | --- |
| HTTPS, exact host and port, POST, exact authorized `/v1/responses` path, matching opaque bearer reference | Resolve once and inject the vault bearer value | `scoped_openai_route_injects_once_and_passes_complete_context`; Core integration test |
| Plaintext HTTP | Deny before resolution | `scoped_openai_denial_matrix_fails_before_resolution`; plaintext proxy guard |
| `api.openai.com.evil.example` or `sub.api.openai.com` | Deny before resolution | Denial matrix |
| Port `8443` | Deny before resolution | Denial matrix |
| Method `GET` | Deny before resolution | Denial matrix |
| Adjacent `/v1/chat/completions` or query-mutated authorized path | Deny before resolution | Denial matrix |
| Missing opaque bearer reference | Deny before resolution | Denial matrix |
| Caller-supplied or MITM-hook-supplied Authorization | Deny before resolution | Denial matrix and MITM collision guard |
| Expired or revoked authority | Stable secret-free resolution failure | `scoped_openai_stale_authority_and_unsupported_route_fail_closed` |
| Redirect/retry attempts to reuse the route | Deny as already used | Exact-route success test |

Each denial-matrix case asserts that the trusted resolver was not called and
that the returned error does not contain the credential canary.

## Secret lifetime and storage review

1. Core owns the opaque capability and `VaultCredentialRef`; the proxy route
   stores only secret-free authority, a capability id, a resolver object, an
   opaque child reference, and one-shot state.
2. Installing the scoped route removes any legacy OpenAI raw-value record.
   Child virtualization ignores an untrusted raw `OPENAI_API_KEY` value and
   overwrites it with the opaque reference.
3. The proxy derives and validates the final HTTPS request context before
   invoking the resolver. Failed routing checks do not touch Vault.
4. Vault decrypts into `Zeroizing<String>` and exposes only a borrowed value
   to the synchronous trusted callback. The source allocation is cleared as
   soon as the callback returns.
5. The callback constructs the outbound Authorization header at the transport
   boundary. Those request-scoped bytes live only with the outbound request;
   the broker and resolver do not retain them across requests.
6. Resolver and transport errors are fixed enums/messages. Credential values
   are not formatted into logs, errors, audit metadata, or debug output.
7. Existing legacy credential records now use `Zeroizing<String>`, while the
   new scoped OpenAI path stores no raw credential at all.

## Final-tree commands

```text
cd codex-rs && just fix -p codex-network-proxy && just fix -p codex-core
PASS — Clippy completed; only seven pre-existing Core test dead-code warnings were emitted.

cd codex-rs && just fmt
PASS — formatting completed and the final diff was inspected.

just bazel-lock-update && just bazel-lock-check
PASS — Cargo/Bazel dependency parity passed; MODULE.bazel.lock required no content change.

cd codex-rs && just test -p codex-network-proxy credential_broker
PASS — 14 focused tests passed; 194 unrelated tests were skipped.

cd codex-rs && just test -p codex-network-proxy
PASS — all 208 network-proxy tests passed.

cd codex-rs && just test -p codex-core network_proxy_spec
PASS — 15 targeted tests passed, including the encrypted-vault-to-proxy integration; 3374 unrelated tests were skipped.
```

## Changed paths

- `codex-rs/Cargo.lock`
- `codex-rs/core/Cargo.toml`
- `codex-rs/core/src/config/mod.rs`
- `codex-rs/core/src/config/network_proxy_credential.rs`
- `codex-rs/core/src/config/network_proxy_spec.rs`
- `codex-rs/core/src/security/credential_capability_tests.rs`
- `codex-rs/network-proxy/Cargo.toml`
- `codex-rs/network-proxy/src/credential_broker.rs`
- `codex-rs/network-proxy/src/credential_broker/providers.rs`
- `codex-rs/network-proxy/src/credential_broker/providers/openai.rs`
- `codex-rs/network-proxy/src/credential_broker/resolver.rs`
- `codex-rs/network-proxy/src/credential_broker_tests.rs`
- `codex-rs/network-proxy/src/http_proxy.rs`
- `codex-rs/network-proxy/src/lib.rs`
- `codex-rs/network-proxy/src/mitm.rs`
- `codex-rs/network-proxy/src/runtime.rs`
