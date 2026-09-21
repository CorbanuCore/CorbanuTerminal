# Closing the last three provider classes — 21 September

The received change collected on every provider whose *shape* the gates allowed,
but three classes were still refused outright, and "all models and providers" is
the requirement. This closes them.

## What was refused, and why each refusal was wrong

| shape | old reason | what is actually true |
| --- | --- | --- |
| `chat_completions_provider` | "overrides the serving provider" | It is an OpenRouter-compatible **configured** routing preference that the client puts in the body as `provider`. It does not determine which upstream vendor serves the turn - with fallbacks permitted, that is OpenRouter's choice - but the selected gateway is the account that is billed, and that is what the provider id records. |
| AWS signing | "does not establish the supported endpoint/dialect" | Signing changes how a request is authenticated, not where it goes or which dialect it speaks. The endpoint pin still proves the destination. It cannot supply monetary rates, which is now enforced in the pricing predicate rather than by refusing collection. |
| `query_params` | "not part of the pinned endpoint" | They are part of the route. The client appends them to every URL, so the pin has to carry them. Refusing was a workaround for a pin that was built as `{base}/{path}` while the client requested `{base}/{path}?{query}`. |

`route_refusal` now refuses no provider shape at all.

## How each is attributed honestly

**Configured routing.** `chat::eligible` requires `request.provider ==
provider.chat_completions_provider`: the body must carry exactly what
configuration says this provider sends. A request-level preference that
configuration did not ask for is still refused, and the transport check applies
the same rule by comparing against the configured value threaded in from the
client. The matrix proves both directions - the configured value collects, a
foreign `{"order": ["someone-else"]}` never does.

**AWS.** Collection is admitted; `turn_mode` adds `provider.aws.is_none()` to the
pricing predicate, so an AWS route records tokens with money unavailable rather
than borrowing API-key rates.

**WebSocket routes.** Admitting query parameters also required the Responses
WebSocket lane to carry them: `Provenance::capture` pins
`websocket_url_for_path("responses")`, which includes the query, while the
accounting pin built `{base}/responses` and hard-failed on any query. Left
unfixed, every such turn would have been rejected outright rather than merely
uncollected. `websocket::endpoint` now takes the canonical query, and a query the
configuration did not declare is still refused.

**Query parameters.** The route is pinned as the client builds it.
`canonical_query` sorts the provider's parameters, because they live in a
`HashMap` and `url_for_path` emits them in iteration order, so the string cannot
be reconstructed and compared byte for byte. `canonical_route` compares a path
exactly and the parameters as a sorted set, which is order-independent and still
rejects any parameter the configuration did not ask for. `AccountingMode::Provider`
carries the canonical query so `Sampling`'s stored endpoint is the route that was
actually requested.

## Configuration-emitted gateway pins collect as well

An earlier version of this record claimed the only remaining refusal was a
request-level key, and that was false: the client itself emits `providerOptions`
from configuration whenever the selected provider is the Vercel gateway and the
model carries a vendor pin. Under the old rule that silently excluded whole
provider and model classes on every turn, not occasionally.

Those now collect on the same principle as the OpenRouter preference: the
transport is given the configured value for both keys and admits a body that
carries exactly it. What it records is the gateway that bills the account. The
upstream vendor inside that gateway's pool is not pinned by configuration and is
not claimed to be.

## All three routing keys this client emits are admitted

The record has now been wrong twice about what remains refused, in the same way
each time: a key I assumed came from the turn is in fact emitted by the client
itself. `provider` comes from `chat_completions_provider`, `providerOptions` from
the Vercel gateway vendor pin, and `plugins` from OpenRouter web search, which the
client derives from the provider and the session's tool set. Under the old rule
each of those silently excluded whole provider, model or session classes on every
turn.

All three are now threaded to the transport as the expected value, and a body
carrying exactly what the client constructed is admitted. What remains refused is
a routing key whose value differs from that - which, since the client builds the
body, is a belt-and-braces invariant rather than a load-bearing gate. It is kept
because it is cheap and because it fails safe: such a request is served unrecorded
and the transport logs the key.

The honest caveat for all three: the recorded provider id is the account that is
billed. Which upstream vendor serves the turn inside a gateway's pool is not
pinned by configuration and is not claimed to be.

## Verification

Clean-host lanes at this tree, RTX workstation:

| lane | result |
| --- | --- |
| `codex-core` accounting | **132 run, 132 passed** |
| `codex-core` accounting, `developer-accounting` | **136 run, 136 passed** |
| `codex-state` accounting | **166 run, 166 passed** |
| `codex-tui` usage | **92 run, 92 passed** |
| `codex-tui` tokens | 66 run, 65 passed, 1 pre-existing failure |

The admission matrix still enumerates 360 cells over six provider ids, three
dialects, five authentication setups and four configuration shapes, and now
asserts every shape collects, with the stored route equal to the one the client
requests for that shape.

Not claimed: independent review of this increment, or any live run.
