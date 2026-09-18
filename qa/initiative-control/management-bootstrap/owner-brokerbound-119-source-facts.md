# Header contract — owner-brokerbound-119

Inspected repository source at assigned base `989f23eeb43bccfda5360398ad59062b78fa23aa`.
No server source, VM, real credential, or live backend request was inspected.
These findings distinguish client behavior from unverified backend enforcement.

## Window identifier: x-codex-window-id

The client places its existing window ID in both client metadata and compatibility
headers: [responses_metadata.rs:227](../../../codex-rs/core/src/responses_metadata.rs#L227),
[responses_metadata.rs:255](../../../codex-rs/core/src/responses_metadata.rs#L255).
Both Responses and compact extend these compatibility headers:
[client.rs:2396](../../../codex-rs/core/src/client.rs#L2396),
[client.rs:1170](../../../codex-rs/core/src/client.rs#L1170).
This is a client correlation identity, not a bearer/account credential.

Broker decision: forward the supplied header unchanged on both supported routes;
do not mint a replacement identity. It is no longer silently dropped. If absent,
the broker omits it. The known consequence is absence of that direct correlation
header (body metadata may still contain the window ID). This client source does
not establish server rejection, a mandatory-header rule, or the exact server
effect when absent. There is no source-demonstrated obstacle to fronting here.

## Attestation: x-oai-attestation

The provider enables attestation only for cached ChatGPT auth:
[provider.rs:412](../../../codex-rs/model-provider/src/provider.rs#L412).
Core returns no header when the provider does not support it or no attestation
provider exists: [client.rs:1341](../../../codex-rs/core/src/client.rs#L1341).
The header is inserted only for `Some`:
[client.rs:2401](../../../codex-rs/core/src/client.rs#L2401),
[client.rs:1175](../../../codex-rs/core/src/client.rs#L1175).
The host integration supplies the value; Core does not derive it from OAuth
tokens: [attestation.rs:13](../../../codex-rs/core/src/attestation.rs#L13).

The app-server seeks an attestation-capable connection, otherwise returning None;
it allows 100 ms for generation. When a capable connection fails, it may send
a status-only envelope, without a token:
[app-server attestation.rs:19](../../../codex-rs/app-server/src/attestation.rs#L19),
[attestation.rs:69](../../../codex-rs/app-server/src/attestation.rs#L69),
[attestation.rs:82](../../../codex-rs/app-server/src/attestation.rs#L82),
[attestation.rs:151](../../../codex-rs/app-server/src/attestation.rs#L151).
The app-server-client's in-process and remote initialization both explicitly set
`request_attestation: false`:
[lib.rs:525](../../../codex-rs/app-server-client/src/lib.rs#L525),
[remote.rs:113](../../../codex-rs/app-server-client/src/remote.rs#L113).
The synthetic API-key worker also does not qualify as ChatGPT auth at Core's gate.

Broker decision: forward an actual supplied opaque header unchanged and never
journal it; do not invent a valid token, success envelope, or failure envelope.
Absent stays absent, matching a supported client construction path. The source
proves that requests can be sent without it; it does NOT prove the real backend
will accept this worker/account/model without it. If the real backend requires
valid attestation for this route, the synthetic API-key fronting arrangement
cannot satisfy that requirement with this broker: it has no attestation issuer.
That would be a real-run blocker, not a reason to manufacture attestation or
refresh credentials repeatedly. No such backend requirement was demonstrated
by this offline assignment.

## Responses-lite: x-openai-internal-codex-responses-lite

The value is `true` only when model metadata selects `use_responses_lite`:
[client.rs:5227](../../../codex-rs/core/src/client.rs#L5227).
The same selection is used on Responses and compact:
[client.rs:2404](../../../codex-rs/core/src/client.rs#L2404),
[client.rs:1178](../../../codex-rs/core/src/client.rs#L1178).
It changes request semantics: tools and instructions move into input items,
ordinary instructions/tools fields differ, and parallel tool calls are disabled:
[client.rs:1675](../../../codex-rs/core/src/client.rs#L1675),
[client.rs:1750](../../../codex-rs/core/src/client.rs#L1750).

Broker decision: preserve the client-provided header and body together on both
routes; do not infer or mint it based on the model name. For a non-lite request
the native client intentionally omits it. Dropping it from a lite request removes
the explicit protocol selection while retaining the lite-form body; exact server
rejection or interpretation is not specified by this client source. The broker
therefore fixes the known lossy forwarding. If an actual lite client omits it,
qualification must record that mismatch; this broker cannot reconstruct the
model metadata decision from arbitrary JSON. No unconditional refusal on absent
header is justified, since normal non-lite requests omit it.

## Account identity and authority

[BearerAuthProvider:38](../../../codex-rs/model-provider/src/bearer_auth_provider.rs#L38)
adds ChatGPT-Account-ID only when present. The broker now likewise permits missing
or null account_id and omits the header, including after refresh, without falling
back to worker headers or decoded account claims. Malformed present values still
refuse. Supplied-account consistency validation remains when the account is known.

Production subscription authority is now exactly
`https://chatgpt.com/backend-api/codex`, matching the default base at
[model-provider-info lib.rs:1003](../../../codex-rs/model-provider-info/src/lib.rs#L1003).
Refresh remains exactly `https://auth.openai.com/oauth/token`. Literal-loopback
HTTP overrides exist only for offline fixtures. API-key upstream configuration
retains its existing contract.

## Evidence limit

The new loopback test covers exact forwarding, absence without minting, both
routes, and exclusion of opaque attestation from the journal. It proves broker
mechanics, not backend acceptance. A later manager-authorized real exact-package
turn must establish server acceptance, model entitlement and attestation policy;
that qualification is explicitly excluded from this worker allocation. The prior
round's unconditional “can be fronted entirely” conclusion is therefore narrowed:
source compatibility is supported, real acceptance remains unproved.
