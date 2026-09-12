# PF-60-S02 next increment — exact native quotation

Manager allocation, September 11 local / September 12 UTC. Same active PF-60
feature and PF-60-S02 sprint; **Product measurement** — “No commercial performance
numbers have been supplied.” Approved defaults remain accepted; no repeat
decision or live collection/billing permission is needed for this increment.
Journal `3f39d7a65` passed independent review and is integrated at
`c33d47f6ccd00a64fcb05a057472d8ca9f0139d4`, with 202 state tests passing there.

## Why this increment comes next

Implement exact USD quotation against supplied synthetic approved snapshots
inside the existing test-only native journal module. This gives later durable
estimate storage a reviewed exact amount/snapshot representation. It avoids
premature integer-micro-USD storage that would lose sub-micro estimates.
Dispatch callbacks, parser-presence preservation and transactional deletion span
several native boundaries; they cannot honestly be included in one small patch.
Pricing is the next bounded dependency, not an alternative accounting system.
Full S02 production wiring, retention/deletion, crash recovery and UI remain open.

## Exact ownership and OFF boundary

One Astra High worker in the existing accounting checkout/branch, with receiving
base above recorded in plan/sprint. Parent clean-fast-forwards it to the reviewed
allocation commit and records actual launch HEAD. Literal writes only:

- `codex-rs/state/src/runtime/accounting.rs`: only the two-line child registration
  `#[path = "accounting_pricing.rs"] mod pricing;`, delegated serially by manager.
- `codex-rs/state/src/runtime/accounting_pricing.rs`
- `codex-rs/state/src/runtime/accounting_pricing_tests.rs`
- `qa/portfolio/agent-cost-accounting/pf-60-s02/exact-pricing-increment.md`

Do not change journal SQL/types/tests, runtime.rs, public visibility, migrations,
dependencies, BUILD, lock/config, wallet, API/Core parsers, UI or shared plans.
The existing ancestor `#[cfg(test)]` excludes the entire module from normal
builds. No production constructor/caller, collector, price fetch, persistence
writer, clock/environment lookup or filesystem input. Tests may read a synthetic
on-disk journal. Target <=450 non-test lines, hard limit 500; <=800 total changed
lines including tests/evidence. Stop/re-slice rather than remove checks to fit.

## Contract

Use a private pure `quote_observations`-style entry with Attempt, ordered
observations and supplied approved snapshots. Validate Attempt and invoke the
existing replay reducer; do not accept unchecked Usage or duplicate normalization.

1. A snapshot has a bounded opaque UUID, exact provider/model/scope, USD and
   per-million-token unit, optional decimal rates for four disjoint buckets,
   opaque source-reference UUID, provider-published/native-catalog source kind,
   observation/approval/effective-from timestamps and optional exclusive end.
   It is a supplied synthetic approval record, not proof of production authority.
   Require effective-from >= observation and approval; reject backdating,
   inverted intervals, malformed rates, unsupported unit/currency, unknown
   fields and duplicate IDs. At most 64 candidates, bounded metadata as in Attempt.
2. Select exact provider/model/scope at dispatch in [effective-from, end).
   No match yields unavailable pricing; overlapping matches are explicit error;
   ordering must not affect selection. Never substitute today's or replay time.
   Legacy records lacking dispatch evidence stay outside this adapter, unknown.
3. Parse nonnegative decimal strings into checked u128 coefficient/scale;
   proposed support <=38 coefficient digits and <=18 fractional places.
   Reject signs, exponents, malformed syntax, numeric/bool JSON and overflow.
   Use checked alignment/products/sums with per-million divisor. These are
   explicit representation bounds, not rounding: no truncation, saturation,
   floats, micro-USD accumulator or treating out-of-range values as zero.
4. Price disjoint noncached input, cache read, cache write and output. Sum exact
   components before display rounding. Preserve known partial subtotal and
   per-bucket missing-usage/rate reasons plus optional all-buckets-priced quote.
   Unknown usage remains unknown even with zero rate; measured zero needs no
   rate. Native Anthropic noncached input remains priceable with unknown inclusive
   input. Unknown-compatible input stays unknown; reasoning is already an output
   subset, not another charge. Revisions replace cumulative values, never sum.
5. Bind result to exact Attempt, source revisions/positions and a copied selected
   snapshot identity/content. Pure returned values do not enforce durable
   snapshot immutability; storage is later work. No DTO name may imply terminal
   completion or final billed spend: the journal has no terminal-coverage fact.
6. Produce six-decimal half-even display text plus explicit rounded and
   nonzero-sub-micro flags, retaining exact underlying value. Compare remainders
   without overflow. No UI consumer or run aggregation is added.

## Source seams and compatibility

Reuse accepted `accounting_types::{Attempt, Observation, Usage, replay}`.
Wallet `CorbanuApiPricing` has rates/version but lacks approval/effective metadata;
do not import wallet/network into state. Its payment `parse_usd_micros` requires
positive amounts and six decimals, so it is not this exact token estimator.
Existing GPU hourly-rate storage is unrelated; leave both unchanged.

Later dispatch seams are EndpointSession's actual transport invocations after
auth, inside nested retries, with awaited durable intent before dispatch.
Presence must be captured before Responses/Chat/Anthropic default conversion.
Native `StateRuntime::init` runs migrations at startup; promoting cfg(test) or
adding a numbered migration alone would change today's OFF behavior. Later
accounting deletion belongs in native `delete_threads_strict`'s transaction.
These are future exact allocations, not writes permitted by this increment.

Canonical upstream `https://github.com/openai/codex.git`; verified common ancestor
`413492cd6c3a4d4f8dff6f406247ccda5a9d88aa`, cached upstream
`1d74c3ba1ee98be2025ab066dcc3fd654fe8a3b6`. No upstream upgrade qualified.
Immediate footprint is product-owned test-gated state code plus one child
registration. Retain provider wire, auth, history, deletion and transport APIs.
Existing Bazel source glob includes sibling tests; use literal fixtures, no
external resource registration/dependencies.

## Tests and independent handoff

Use literal synthetic snapshots whose observation/approval precede dispatch:

- Anthropic input50/read10 with write/output absent at 3/0.3 USD per million
  gives exact known 0.000153 USD; inclusive/full quote stays unknown.
- Inclusive disjoint arithmetic; absent/null/zero, missing price and unknown
  compatible semantics; measured zero without price versus unknown at zero rate.
- Exact snapshot scope/model, duplicate identity, gap/overlap, half-open boundary,
  prospective rules, order independence, currency/unit/unknown-field rejection.
- Decimal syntax/bounds and checked product/alignment/sum overflow. Half-even
  ties 0.0000005 -> 0.000000 and 0.0000015 -> 0.000002 retain exact values;
  two components of 0.0000004 sum before rounding to 0.000001.
- Close/reopen a real disposable accepted journal, read observations, quote twice
  and compare complete literal results plus unchanged rows/positions. A later
  prospective catalog entry cannot change old-dispatch selection or a returned
  quote. This is not durable estimate storage or process-crash evidence.

Run `just fmt` with writes guarded to allocated Rust paths; disclose known
out-of-scope Python/environment failures and use scoped rustfmt if needed.
No toolchain/dependency repairs or unrelated rewrites. Pinned cached 1.95.0,
offline, after formatting:

```text
just test -p codex-state runtime::accounting::pricing::tests
just test -p codex-state runtime::accounting::tests
just test -p codex-state
cargo check --offline --locked -p codex-state --lib
```

Also both governance and whitespace checks; no direct cargo test or process/
lock cleanup. Return uncommitted with hashes, exact nonzero counts, command
results, supported decimal limits and OFF proof. Parent owns independent Astra
High review, bounded corrections and combined-tree tests; no exhausted reset.

## Following work and human prerequisites

After quotation acceptance, manager allocates persistence of immutable snapshot/
result with per-thread/day aggregate contributions and atomic deletion/tombstone
proof. Storage lifecycle precedes separate OFF-preserving native production
promotion; then presence and durable synthetic dispatch bridge precede live
callers. Keep unsupported transport coverage OFF. Historical import, provider
identity ambiguity, lineage, process interruption and full S02 golden totals
remain required; S03 cannot start from this helper alone.

Technical human review needs exact diff/hashes, literals, decimal bounds, cached
tools and isolated-home setup, no account credentials. Product testing still
requires named tester/setup operator, exact candidate/binary, public inspection/
restart/export/deletion controls, code-blind intent cases, root plus two children,
retry/missing data/replay, 90/365 cutoffs and tombstone rejection. TUI/live-repo,
PF13/PF26, named runtime acceptance, benchmarks and release gates remain open.
