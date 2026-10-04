# Residual dispositions

Both remain blocked for positive UI qualification on this frozen candidate.
These are explicit prerequisites, not passes or requests to waive a gate.
No production code, provider identity, observation, price or bill was fabricated.
Source excerpts and hashes are preserved in `residual-source-evidence.json`.

## Two collected providers — blocked in the authorized loopback rig

The collector is not inherently single-provider: `core/src/accounting.rs`,
`developer_accounting_mode`, accepts **anthropic + Anthropic**, **openai +
Responses**, and **openai + Chat**. Every other provider ID selects `Disabled`.
Consequently, two OpenAI transports or models do not constitute two providers,
and the real `reader_local` emission in round 61 does not supply a second
collected provider. Its stored provider count is still one.

To collect a genuinely second provider here requires the built-in `anthropic`
identity. In this CLI's top-level configuration, `model-provider-info/src/lib.rs`
constructs it with `https://api.anthropic.com/v1`. The built-in merge calls
`apply_transport_overrides`, which copies retry/timeouts, **not base_url**.
`core/src/config/mod.rs` supplies a dedicated `openai_base_url`; there is no
corresponding Anthropic root override in that load path. Setting a custom provider
with an Anthropic wire loses collector eligibility instead of solving this.

This is backed by the preserved real-key [Anthropic timeout](../acct-readers-61/reader-run-01/anthropic-sample-timeout.txt.gz),
[raw attempt](../acct-readers-61/reader-run-01/anthropic-sample.raw.gz),
[driver](../acct-readers-61/reader-run-01/driver.py.gz) and
[emissions](../acct-readers-61/reader-run-01/emissions.json): no Anthropic emission
reached the loopback fixture. The frozen source explains why that configuration
could not redirect it. This revision does not repeat a known denied external
transport or relax the loopback boundary.

The accounting role overlay is not a general escape hatch: in
`core/src/agent/role.rs` it preserves the current accounting mode and overlays
provider fields only when the next provider ID equals the current provider ID.
An OpenAI root switching to Anthropic does not satisfy that condition. Changing
a bound endpoint is still subject to pre-send eligibility, not permission to
route around denial. A private embedding that supplies its own admitted providers
could exercise both collectors, but would not be this exact CLI/PTY flow.

**What must change:** an authorized fixture-capable Anthropic endpoint selection
that retains the built-in ID and correct admission/auth binding, followed by a
new exact candidate and a real-key two-provider reconciliation; alternatively,
a separately authorized isolated live-provider lane with synthetic/dedicated
credentials. A broader custom-provider collector would also need explicit product
scope, stable pricing/usage semantics and tests. None is authorized or implemented
in this QA-only lane. No live credential access is needed to establish this limit.

## Positive measured-dollar comparison — blocked by absent settlement support

Four newly collected requests provide real **reported token counts** and native
**estimated dollars**. Neither is a provider-settled dollar measurement. The
stale/restored control in round 61 therefore cannot be relabelled as a positive
measured-cost comparison.

`state/src/runtime/accounting_pricing.rs::ObservationQuote` contains an attempt,
observations, usage, price snapshot, buckets and estimated subtotals; it has no
settled-dollar amount. `Inspection` in `accounting_store.rs` carries recorded
usage/estimate totals and requests, without a settlement comparison field.
`chatwidget/tokens.rs::inspection_pages` inserts **“Billed cost: unavailable — no
settlement evidence”** and unconditionally appends **“Estimate versus billed
difference: unknown — no settlement evidence”** to ready pages. These are
unconditional absence paths in this inspector, not a recoverable fixture-price
problem. The actual fresh-session, root and attempt captures show those strings.

**What must change:** a defined trusted settlement-evidence input, its persistent
binding to provider/account/request/attempt and currency, the timing and coverage
semantics for comparison with the retained estimate, and UI support for the
matched measured amount and difference. Then execute a real-key case where an
independently supplied nonzero settlement amount is compared against an
independently calculated estimate, including mismatch and missing-evidence cases.
That requires product-authorized financial/data scope and implementation outside
this lane. Editing the SQLite estimate into a supposed bill, using a tariff times
usage as measured dollars, or treating a plan purchase as request settlement
would not fulfill the residual. **Positive measured-dollar comparisons: 0.**
