# Money on every provider, and plan work stated as plan work

*PF-60 S03, 21 September 2026. Author: Fable. Branch
`bootstrap/acct-activation-20260916`.*

## What was wrong

Collection reached every provider in the previous increments. **Money did not.**

Two separate holes, both of which made accounting "unavailable" in the sense
that matters to an operator looking at a bill:

1. **Money existed on two providers.** `turn_mode` granted pricing authority
   only to `("anthropic", Anthropic)` and `("openai", Responses|Chat)` at their
   own default endpoints. Every other built-in provider - OpenRouter, Vercel,
   DeepSeek, Ambient, Baseten, Kimi, Meta and the rest - recorded tokens with no
   rate at all, even under plain API-key authentication, even though the
   catalogue states exact per-token prices for those rows.
2. **Subscription turns recorded no economics whatsoever.** Under plan
   authentication `api_key_pricing` is false, which mapped to
   `Pricing::Unavailable`: no snapshot, no rate, no plan figure. A ChatGPT-plan
   or Claude-plan session recorded token counts and nothing else, and the
   catalogue's `plan_relative_burn_millis` - which the spawn tooling has been
   printing to agents for months - never reached the ledger.

## What money means under a plan

A plan does not charge per token, so pricing a plan turn with API rates and
adding the result to spend would claim money that was never charged. Both
numbers are worth having, and they are different statements:

- **The plan rate that applied**, 1000 meaning 1.0x, and the consumption it
  implies: total tokens scaled by that rate.
- **The API equivalent**: what the same tokens would have cost at the
  catalogue's API rates, when the catalogue states any.

The ledger now carries a `Basis` on every price snapshot - `Billed` or
`PlanEquivalent` - and `ObservationQuote` splits the arithmetic accordingly:
`known_subtotal` is money charged and stays exactly zero for plan work, while
`known_equivalent` carries the counterfactual. `DayTotals` gains
`equivalent_usd`, `unknown_equivalents`, `plan_burn_milli_tokens` and
`plan_attempts`. `Snapshot::validate` refuses the two shapes being confused: a
plan rate without the plan basis, or the plan basis without a plan rate.

A plan attempt still increments `unknown_estimates`, because billed money for
that turn genuinely cannot be stated. The display no longer leaves it at that:
it says how many attempts ran on subscription capacity, what they consumed at
the plan rate, and what they would have cost on the API side.

## The rate that applied, not the rates the row could charge

`ModelBilling::plan_burn_millis_at` resolves a row against the dispatch instant:

- `Plan` and the plan side of `AuthDependent` are flat.
- `PlanSchedule` resolves the peak window as `[start, end)` in UTC hours,
  including windows that wrap past midnight, where the hours after midnight
  belong to the window that opened the previous day - so a weekday restriction
  is tested against the opening day, not the calendar day of the instant.
- A promotion applies only while it is valid. A promotion with no end, or an end
  this client cannot parse, is **not** applied: the discount has to stop without
  a catalogue release, which is exactly what the field was added for.
- A degenerate window (`start == end`) charges off-peak rather than charging
  peak forever.

`Metered` and `Local` rows have no plan side and state none. Inventing a burn
for a metered row would put a number in the ledger no catalogue ever stated.

## Money follows the route, and authentication chooses the basis

There are three situations, not two, and an earlier revision of this work
collapsed them into the old `api_key_pricing` bit. Independent review caught it:
"API key on a route this client will not price" was being treated as
subscription capacity, so an operator pointing a provider at a proxy would have
had their API-key spend booked as plan work.

`PriceAuthority` now states which economics a turn admits:

- **`ApiKeyRates`** - the provider's own default route, in its own dialect,
  resolved under the turn's own credential, with no credential-bearing custom
  headers and no provider-held credential shape (AWS signing, command auth, an
  experimental bearer token).
- **`PlanRate`** - that same route under subscription-style authentication. The
  route is resolved per credential, because a ChatGPT plan turn goes to the
  Codex route and an API key to the API one, and both are OpenAI's own.
- **`Unavailable`** - anything else. Tokens are still collected; no economics of
  either kind are claimed for a destination the catalogue quotes nothing for.

`accounting_every_built_in_provider_collects` asserts all three per provider:
rates on its own route under an API key, the plan side on that route under plan
authentication, and nothing at all through a relay.

`billed` also stopped refusing `AuthDependent` rows. Under API-key
authentication those rows' API rates are precisely what the provider charges;
refusing them left auth-dependent models priceless on the one auth mode that is
billed per token.

Provenance is unchanged in kind and unified in shape: every billed snapshot's
`source_reference` is a UUIDv5 over `(source, provider, model, "api_key",
"default", "USD/million", input, output, read)`, and every plan snapshot's is
over `("plan-equivalent-bundled-v1", provider, model, "plan", burn,
"USD/million", input, output, read)`. The Anthropic tuple gained the
authentication and tier terms the OpenAI one already carried, so its identities
differ from the previous release by construction.

## Verification

- `codex-protocol`: flat rows, scheduled rows, weekday peaks, wrapping
  overnight windows, promotion expiry, unparsable and absent promotion ends,
  degenerate windows.
- `codex-core`: `accounting_plan_projection_states_the_rate_and_only_stated_equivalents`
  pins the shipped catalogue rows - `claude-opus-5-plan` (burn only, no invented
  price), `gpt-5.6-luna` (burn 0.2x plus exact API rates), `glm-5.3` (3.0x
  inside the weekday peak, 1.0x outside) - and that nothing is stated for a
  metered row, another provider's row, an unknown slug, or an unquoted tier.
- `codex-core`: `accounting_responses_ws_subscription_uses_resolved_endpoint_without_api_prices`
  admits a real attempt on the **real** Codex subscription route and asserts the
  stored snapshot is `PlanEquivalent` at 1.0x carrying the catalogue's own API
  side - $5/M uncached, $30/M output, $0.50/M cached input - for `gpt-5.6-sol`.
- `codex-core`: `accounting_chatgpt_subscription_off_route_collects_without_economics`
  drives a subscription turn end to end against a wiremock endpoint, which is
  nobody's own route, and asserts tokens are collected while no snapshot, no
  spend and no plan figure are claimed.
- `codex-state`: `plan_basis_separates_plan_consumption_from_money_spent` proves
  spend stays zero while the equivalent accrues, that a burn-only row records
  the rate and no money of either kind, and that a snapshot mixing the two bases
  is rejected.
- `codex-state`: `version_one_days_decode_as_days_with_no_plan_work` proves a
  compact day written before plan accounting still reads, as no plan work.
- `codex-tui`: `accounting_inspect_plan_work_is_never_reported_as_money_spent`
  pins the copy, including the case where the catalogue prices none of the plan
  attempts and the display says so instead of reporting zero.

Clean-host lanes at this tree, RTX workstation: `codex-core` accounting
138/138, with `developer-accounting` 143/143, `codex-state` accounting 168/168,
`codex-tui` usage 92/92, `codex-tui` tokens 66 of 67 - the one failure,
`accounting_inspect_maintenance_with_healthy_raw_renders_lag`, is a stale
snapshot that fails identically at the integration tip without any of this work,
checked on the same host. Raw logs under `rtx-20260921/`.

## Known limits, stated

- The API equivalent exists only where the catalogue states API rates for the
  same row. `claude-plan` and `zai` rows are burn-only today, so those sessions
  record the plan rate and explicitly no money. That is the catalogue's
  statement, not an estimate this client can improve without guessing.
- Plan consumption is expressed in the catalogue's own relative units. It is
  what the plan charges relative to a 1.0x model, not a share of a quota: the
  size of the operator's pool is not something this client is told.
- A plan turn still counts toward `unknown_estimates`, because its billed money
  is unknowable per turn. The plan figures sit beside that count rather than
  replacing it.
- Provider-held credential shapes (AWS signing, command auth, bearer tokens)
  carry no per-token rates: what that account is charged is not something the
  catalogue states for this client. On the provider's own route they do take the
  plan side when the credential is not an API key, which is exactly how the
  built-in `claude-plan` provider - command auth, plan rows - gets its burn
  recorded.
- Off a provider's own route, nothing is claimed at all. That is deliberate: a
  proxy or relay may or may not charge what the catalogue quotes, and this client
  is not told which.
