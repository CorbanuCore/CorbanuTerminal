# Closing the last three provider classes — 21 September

The received change collected on every provider whose *shape* the gates allowed,
but three classes were still refused outright, and "all models and providers" is
the requirement. This closes them.

## What was refused, and why each refusal was wrong

| shape | old reason | what is actually true |
| --- | --- | --- |
| `chat_completions_provider` | "overrides the serving provider" | It is an OpenRouter-compatible **configured** routing preference that the client puts in the body as `provider`. Configuration names who serves and bills the turn, so the turn is attributable to the selected provider. |
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

**Query parameters.** The route is pinned as the client builds it.
`canonical_query` sorts the provider's parameters, because they live in a
`HashMap` and `url_for_path` emits them in iteration order, so the string cannot
be reconstructed and compared byte for byte. `canonical_route` compares a path
exactly and the parameters as a sorted set, which is order-independent and still
rejects any parameter the configuration did not ask for. `AccountingMode::Provider`
carries the canonical query so `Sampling`'s stored endpoint is the route that was
actually requested.

## What is still not collected, and it is one thing

A **request-level** routing key the configuration did not ask for - `provider`
with a foreign value, `providerOptions`, or `plugins`. That body can send the work
to a different upstream vendor than the one selected, so attributing it to the
selected provider would be a lie. Such a request is served unrecorded, and the
transport logs the key that caused it. This is a per-request condition, not a
provider class: the same provider collects normally on every other turn.

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
