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
carrying exactly what the client constructed is admitted.

Threading alone was not enough, and review caught that too: `chat::eligible` runs
BEFORE the transport exists and still demanded that `provider_options` and
`plugins` be absent, so the threading was dead code and OpenRouter web-search
sessions and Vercel-gateway Chat sessions still never collected. The predicate now
accepts each field when the provider configuration explains it - the gateway for
`providerOptions`, OpenRouter for `plugins` - while the exact value is still
checked at the transport. `accounting_chat_collects_the_fields_the_client_itself_emits`
pins both, and pins that an ordinary provider still refuses both. What remains refused is
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
| `codex-core` accounting | **135 run, 135 passed** |
| `codex-core` accounting, `developer-accounting` | **140 run, 140 passed** |
| `codex-state` accounting | **166 run, 166 passed** |
| `codex-tui` usage | **92 run, 92 passed** |
| `codex-tui` tokens | 66 run, 65 passed, 1 pre-existing failure |

The counts include the two tests this increment added:
`accounting_chat_collects_the_fields_the_client_itself_emits` and
`accounting_websocket_pin_matches_the_client_route`.

The admission matrix still enumerates 360 cells over six provider ids, three
dialects, five authentication setups and four configuration shapes, and now
asserts every shape collects, with the stored route equal to the one the client
requests for that shape.

Not claimed: independent review of this increment, or any live run.

## Checked against the real catalogue, and what is still outside collection

`accounting_every_built_in_provider_collects` iterates
`built_in_model_providers(None)` - the product's actual provider list, twenty-one
entries, asserted exactly so a shrinking catalogue is visible - and asserts each one selects a collecting mode at its own wire dialect, binds a mode
that collects, is admitted by the per-turn dialect gate as well as the selector,
and carries pricing authority under API-key authentication only for the two
metered entries at their own default endpoints. An earlier version of this test
bound with no auth mode at all, which made its pricing assertion unfalsifiable. Six synthetic identities in the matrix proved the rule; this
proves the catalogue obeys it. A future provider with a shape this code cannot
attribute fails here rather than silently going uncollected.

Collection keys on provider identity and wire dialect only. There is no
model-level gate, so "all models" follows from "all providers" for any model a
provider serves. Pricing is a separate question and is per model, by catalogue
billing.

Review found a third class I had missed, and it was the one that mattered:
**compaction**. A compaction builds its own model client session and streamed on
it with no collector attached, so those turns never recorded - including
`/compact`, which the operator asks for directly. My stated rationale for the
other exclusions ("not turns the operator asked for") was simply false for it.

Compaction now collects, on **both** compaction paths. The first attempt wired
only the local one, and review pointed out that providers supporting remote
compaction - OpenAI and Azure Responses, which is where operator `/compact`
usually lands - route to `compact_remote_v2` and still recorded nothing. Both now
call the same helper: the attachment logic `session/turn.rs` performed inline is
extracted into `accounting::attach_turn`, so the paths cannot drift again.

The remote path attaches only when the compaction owns its client session. When
the session is borrowed from a live turn, that turn's collectors are already
attached and the compaction request belongs to it, so nothing is replaced
mid-flight.

The turn label is `accounting::compaction_turn_label`, which keeps the identity
inside the store's 128-byte bound: a long submission id would otherwise make its
compaction unrecordable while the ordinary turn recorded fine.
`accounting_compaction_label_stays_recordable` pins that against
`Attempt::validate`, including a multi-byte boundary. In compaction the attachment is deliberately best effort: if collection cannot be
attached the compaction still runs and a warning says it proceeded unrecorded.

Honest limit, because review pushed on it and the first answer was worse than the
problem: best effort covers ATTACH time only. Once collectors are attached, an
accounting fault mid-stream still fails the compaction, exactly as it fails an
ordinary turn. I tried making that path retry the attempt unrecorded and backed it
out - it re-sends the request, so a bookkeeping fault would have cost a second
compaction call. Failing closed and consistently is the better of the two, and the
inconsistency with the sentence above is stated rather than hidden.

Two exclusions remain, both session classes rather than provider or model
classes, and neither introduced by this work:

- **Agent-identity telemetry sessions.** When the client resolves agent-identity
  telemetry, the Responses WebSocket route is excluded before a collector exists.
- **Startup prewarm and auxiliary inference.** Outside sampling collection by
  design; they are not turns the operator asked for. That rationale is true for
  these two.
