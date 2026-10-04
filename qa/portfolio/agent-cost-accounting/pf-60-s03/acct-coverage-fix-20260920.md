# Why nothing collected, and what the fix was — 20 September

Integrator record. The crash-preserved implementation at `2e6d47e17` opened all
three gates correctly and still collected nothing on any provider. This is the
defect that hid behind that, found by running the round's own test.

## The defect

`accounting_transport::request_refusal` inspected the request body for the
routing keys `provider`, `provider_options` and `plugins`, which can change the
serving provider after selection. It parsed `EncodedJson` bodies as plain JSON
and refused a `Raw` body outright:

```rust
RequestBody::EncodedJson(body) => match serde_json::from_slice(body.as_bytes()) {
    Ok(value) => value,
    Err(_) => return Some("uninspectable request body"),
},
RequestBody::Raw(_) => return Some("uninspectable request body"),
```

The transport sees the body *after* preparation. For this client that means the
bytes may already be zstd-compressed, and `Request::compression` does not say so
— it describes what the transport should still do, not what preparation already
did. Instrumentation printed the truth:

```
ACCTPROBE transport: url=http://127.0.0.1:59313/v1/responses
  refusal=Some("uninspectable request body") body_variant=encoded len=14548
  head=28b52ffd00585dc6 compression=None
```

`28 b5 2f fd` is the zstd magic number. So every prepared Responses turn parsed
as "uninspectable" and was refused before admission. Collection was dead for
every provider, including the two the previous implementation already supported.

The existing policy test passed because it built its request with `with_json`,
which leaves a `RequestBody::Json` the check can read. Nothing exercised the
encoding the client actually sends.

## The fix

Decoding now lives with the body type that produced the bytes, because only that
module knows the encoding rules: `RequestBody::inspectable_json()` in
`codex-rs/http-client/src/request.rs` returns the JSON for `Json`, and for
`EncodedJson`/`Raw` parses the bytes, falling back to a zstd decode before giving
up. `None` still means genuinely unreadable, which the caller treats as
uninspectable and refuses. `request_refusal` is now four lines over that.

`zstd` is only a dev-dependency of `codex-core`, which is the second reason the
decode belongs in `http-client` rather than in the accounting module.

## Regression coverage

`accounting_prepared_bodies_are_inspected_not_refused` builds requests through
`Request::into_prepared()` — the same path the client uses — under both
`RequestCompression::None` and `Zstd`, and asserts that an ordinary body stays
collectable while each routing key stays refused under both encodings. A truly
opaque body is still refused. That test fails on the pre-fix code.

## Results

| lane | before the fix | after |
| --- | --- | --- |
| `accounting_chatgpt_subscription_collects_without_api_prices` | failed, 0 attempts recorded | **1 test, passed** |
| routing-key policy tests | 2 passed, but blind to the real encoding | **2 passed, both encodings covered** |
| `codex-state` accounting | 161 passed, 5 failed | **166 passed** |

The five `codex-state` failures were the deliberate semantic change — an absent
rate no longer prices a zero-count bucket as zero — reaching goldens that still
encoded the old behaviour. Each was updated on its own merits and commented:
the public literal goldens, the latest-quote `zero` case, the lifecycle day
totals, the late-import presence goldens, and the compaction golden, where a
priced zero and an unpriced zero are now separate fixtures so both properties
stay pinned.

`codex-tui` tokens has one failure,
`accounting_inspect_maintenance_with_healthy_raw_renders_lag`, which fails
identically at base `6322a6e7c`. It is pre-existing snapshot drift, not this
change, and is left for its own commit so attribution stays clean.

## Full lanes after the fix

| lane | result | attribution |
| --- | --- | --- |
| `codex-core` accounting | 130 run, 92 passed, 38 failed | every failing test is also in the base failing set; base failed 63 distinct where this run failed 41 |
| `codex-core` accounting, `developer-accounting` | 134 run, 94 passed, 40 failed | 41 distinct, all also in the base set; 75 `deadline has elapsed` |
| `codex-state` accounting | **166 run, 166 passed** | |
| `codex-tui` usage | **92 run, 92 passed** | |
| `codex-tui` tokens | 66 run, 65 passed, 1 failed | the one failure also fails at base |

`comm -23` of the candidate failing set against the base failing set is empty for
both core lanes: no test fails here that was not already failing at base. That
argument has a known limit, stated rather than glossed: an empty difference of
failing *names* cannot prove that a test which fails environmentally at base
would still pass under the new semantics. It rules out a newly failing name, not
a regression hidden inside an already-failing one.

The distinct-name counts differ from the nextest totals because a retried
attempt prints `FAILED` for both tries: base reported 62 failed with 63 distinct
names, this run 38 failed with 41 distinct names.

Two of the shared failures are assertion rather than deadline failures, named
here rather than by line number, because this change inserts lines into that file
and shifts them:

- `suite::accounting_responses::accounting_responses_native_ws_fallback_http_segment`
- `suite::accounting_responses::accounting_responses_native_ws_only_no_install`

Both appear in the base run's `FAILED` lines by name, so both are pre-existing.
They are WebSocket-route cases in the file this change touches, so they are
called out explicitly instead of being absorbed into the environmental set, and
they remain open for their own disposition.

Not claimed here: independent review, a receipt, or any live qualification.

## Second independent review, and what it found in my own fixes

The first review refused the change: one P1, three P2, three P3, all acted on.
The second review refused it again, and both of its P2s were in work I had done
myself rather than in the worker's original:

1. My restored pricing predicate added `api_key_header_name().is_none()`. The
   built-in Anthropic provider declares `x-api-key` as its own credential header,
   so that condition made the Anthropic arm of the pricing match dead code and
   left metered Anthropic turns with no rate at all. The condition is gone; a
   provider's own declared credential header for its own approved endpoint is not
   a foreign proxy credential.
2. A body carrying a gateway pin still failed the turn closed on Responses and
   Anthropic, because only Chat inspects the typed request before a collector
   exists. That is the same defect class as the `query_params` shape I had just
   fixed, one layer down. The transport now declines to sample and passes the
   request through unrecorded instead of ending the user's turn.
3. P3: the new `query_params` refusal had no test. The admission matrix now
   enumerates it as a fourth configuration shape, 360 cells instead of 270, and
   passes in the feature build.

A third finding came from my own test rather than either review: holding the
endpoint at the real OpenAI default and varying only the auth mode is what
exposed defect 1 above. The test is
`accounting_pricing_authority_follows_auth_mode_at_the_default_endpoint`.

## Final lanes

| lane | result |
| --- | --- |
| policy and subscription tests | 4 run, **4 passed** |
| admission matrix, feature build | **passed** at 360 cells |
| `codex-core` accounting | 131 run, 85 passed, 46 true failures |
| `codex-core` accounting, feature | 135 run, 81 passed, 54 true failures |
| `codex-state` accounting | 166 run, **166 passed** |

Failure attribution is now computed flaky-aware: a test that prints `FAILED` on
one try and passes on a retry is not a failure. On that basis the plain lane has
46 true failures against base's 62, and **no test fails in the candidate that does
not also fail at base**. An earlier count that looked like a new failure,
`accounting_anthropic_native_presence_prices_and_two_reopens`, was a first-try
timeout that passed on retry.

Three feature-lane failures appear in neither base nor the plain lane:
`accounting_anthropic_401_and_429_are_terminal_and_preflight_schema_fault_has_no_send`,
`accounting_anthropic_redirects_never_send_or_attribute_to_unapproved_endpoint`
and `accounting_responses_ws_native_handshake_and_postdispatch_errors`. The first
two fail inside wiremock's verification with "the server did not receive any
request", which is the same environmental signature seen at base for other cases,
and the base run cannot contain them because the feature does not exist there.
They are being re-run in isolation before any receipt; until that returns they
are **unresolved**, not dismissed.
