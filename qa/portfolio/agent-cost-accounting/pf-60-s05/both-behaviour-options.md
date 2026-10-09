# PF-60-S05 open decision: "both" behaviour

*2026-10-08. For Travis's decision. Nothing here is decided or implemented. PF-60-S05 stays blocked until an option is
chosen.*

## The problem

The sprint gives each provider a declared billing basis ([defaults](billing-basis-defaults.md)). Three real situations
don't fit a single label:

1. **Membership and API balance with the same vendor.** These are different products with different keys and usually
   different URLs. Examples: a Kimi Code membership and a Kimi Open Platform balance; a GLM Coding Plan and a Z.AI
   balance; a Claude subscription and an Anthropic API key; a ChatGPT login and an OpenAI API key.
2. **Quota, then paid overflow on the same login.** Claude usage credits are charged at API rates once plan limits are
   reached. Kimi Extra Usage draws on a separate balance once membership quota runs out. ChatGPT plans can buy Codex
   credits. Corbanu Terminal can't see, for any one request, whether the quota or the overflow paid for it.
3. **One route that can bill either way.** Z.AI's Anthropic URL is the Coding Plan route, but it bills a balance for
   accounts that never bought a plan and have been allowlisted. A user can also point a built-in provider at another URL.

## Worked example used for every option

Prices here are made up to keep the arithmetic easy: Claude API at $5 per million input tokens and $25 per million
output; GLM on Z.AI at $1.40 / $4.40. One day:

| Work | Requests | Tokens in / out | What really happened |
| --- | --- | --- | --- |
| `claude-plan` (Max subscription, usage credits on) | 40 | 2.0M / 0.2M | The last 10 (0.5M / 0.05M) ran after the weekly limit and were paid from usage credits: **$3.75** |
| `anthropic` (API key) | 8 | 0.4M / 0.04M | Billed **$3.00** |
| `zai` with its URL changed to the Coding Plan endpoint | 30 | 1.0M / 0.1M | Covered by the GLM Coding Plan: **$0** (at API price it would be $1.84) |

Real money that day: **$6.75**. API value of all the Claude subscription work: $15.00.

## Option A: one basis per provider entry

The declared basis (built-in default or the user's `billing` override) applies to everything sent through that provider
entry. Overflow is never counted.

- **`/cost` shows:** Spent $3.00 (Anthropic API). Subscription: Claude, 40 requests, API value $15.00, not spent. Z.AI:
  "Pay per use $1.84", because `zai` is declared pay per use. A note says paid overflow beyond a plan isn't visible here
  and to check the provider's usage page.
- **Counted as spent:** $4.84. The Z.AI figure is wrong unless the user adds the override. The $3.75 of overflow is
  missing.
- **User configures:** `model_providers.zai.billing = "subscription"` after changing the URL. Nothing for Claude.
- **Pros:** simplest, and this is what the sprint builds anyway. Subscription work is never shown as spending.
- **Cons:** overflow is invisible. If someone changes a URL and forgets the override, the result is silently wrong.

## Option B: the basis follows the key and route actually used (recommended)

The built-in table declares a basis per credential and per route, not one per provider. For example: the Z.AI coding URL
is subscription and the general URL is pay per use; the Kimi Code URL is subscription and the Kimi Open Platform URL is
pay per use; a ChatGPT login is subscription and an OpenAI API key is pay per use. A route the table doesn't know is
shown as "billing basis not declared", with a next step, instead of being guessed. Overflow is handled as in A.

- **`/cost` shows:** Spent $3.00. Subscription: Claude, 40 requests (API value $15.00, not spent); Z.AI Coding Plan, 30
  requests. The same overflow note as A.
- **Counted as spent:** $3.00. The only gap is the $3.75 of overflow, and the screen says it can't see it.
- **User configures:** nothing. The `billing` override stays available for unusual setups.
- **Pros:** right without any setup for "membership plus API balance", which is the common case. A mistake shows up as
  "not declared" instead of a silently wrong number. Still declared, not guessed from the auth type.
- **Cons:** overflow is still invisible. We have to maintain and verify a list of routes per provider, each with a
  real-provider check. That adds a few rows to the table and some test work to S05.

## Option C: option B plus a "possible extra charges" line

Same as B. In addition, the user marks a subscription that has paid overflow switched on (`overflow = true`, or once in
`/providers`). Where a provider reliably signals that a request was paid from overflow, that request is counted as
spent at its overflow price. Otherwise `/cost` shows an upper bound.

- **`/cost` shows:** Spent $3.00, plus a separate line: "Your Claude plan may have charged up to $15.00 in usage credits
  today; check claude.ai › Settings › Usage."
- **Counted as spent:** $3.00. The "up to $15.00" is shown but not added in.
- **User configures:** one overflow flag per subscription.
- **Pros:** users with overflow switched on are warned that money may have been spent.
- **Cons:** the bound is usually far above the truth ($15.00 against $3.75). No reliable per-request overflow signal has
  been verified for any provider, and Kimi bills overflow in RMB. Adds UI and per-provider research. Better as a later
  sprint if wanted.

## Option D: when unsure, count it as spent

Any provider that can bill either way is counted at API price as spent unless the user marks it subscription-only.

- **`/cost` shows:** Spent $19.84 ($3.00 Anthropic, $15.00 Claude, $1.84 Z.AI).
- **Counted as spent:** $19.84 against a true $6.75.
- **User configures:** marks `claude-plan` and `zai` as subscription-only to get back to $3.00.
- **Pros:** never understates money.
- **Cons:** it brings back the review's Blocker (subscription work shown as spending) for most users by default.

## Recommendation

**Option B.** It needs no setup for the common case of a membership plus an API balance with the same vendor. It is
still a declared rule rather than a guess from the type of login. Anything it can't place is shown as "not declared"
instead of being given a wrong number. Overflow stays out of "spent", and the screen says so in plain words. Revisit
option C only if a provider is found to signal reliably, per request, that overflow paid for it.

If B is chosen, S05's acceptance criterion 12 becomes: for each built-in provider with more than one route or credential
in the table, a real request on each route is recorded with the declared basis; an unknown route shows "not declared";
and the overflow note appears only on subscription work.
