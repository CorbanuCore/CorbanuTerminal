# PF-60-S02 — exact compact values (B1)

Executable after this allocation's independent review and manager dispatch.
Product: **Product measurement**, “No commercial performance numbers have been
supplied.” The approved v1 defaults remain unchanged. This is a private foundation
for retention, not a retention service, production migration or billing feature.

## Baseline and ownership

Receiving `3898eaa658924361b94cdbfc6751f930550ddee4` includes reviewed contributions
and recorded-attempt deletion `b08fff66d`. All 299 native tests (226 state/73 Task
Node) pass on that combined tree. One fresh-context Astra High accounting worker
uses the existing checkout/branch in the active plan and PF-60-S02. Parent first
fast-forwards its clean idle checkout to the reviewed allocation commit and records
the actual launch HEAD/agent ID. No concurrent accounting writer.

Literal writes (four files only):

- `codex-rs/state/src/runtime/accounting_lifecycle.rs`: private child registration only.
- `codex-rs/state/src/runtime/accounting_compact_values.rs`: new pure value codec/composition.
- `codex-rs/state/src/runtime/accounting_compact_values_tests.rs`: new sibling tests.
- `qa/portfolio/agent-cost-accounting/pf-60-s02/compact-values-increment.md`: new receipt.

No journal, pricing, storage, old tests/receipts, manifests, dependencies, BUILD,
production schema, Core/TUI, plans, credentials, live prices or network writes.
Manager serializes the registration seam. Keep the entire existing cfg(test)
module chain and no production call sites. Return uncommitted, no merge/push.
Upstream identity and ownership remain in the active plan/S02 allocation; this
adds only a product-owned private descendant, not an upstream API change.

## Exact behavior

Reuse lifecycle's DayTotals and pricing's Decimal; do not relax the rate parser.
Its 18-place rate limit does not decode all 24-place computed amounts. Add a
strict stored-amount decoder accepting nonnegative canonical strings with u128
coefficient and scale 0..24, including u128::MAX (39 digits). Checked accumulation,
exact canonical reserialization equality; reject signs, whitespace, exponent,
empty components, JSON numbers/booleans, leading whole zeros, fractional trailing
zeros, excess scale and coefficient overflow. No f64 or display-rounding storage.

Define a versioned compact value containing fixed seven-element known/unknown
integer arrays in DayTotals order (input, noncached, read, write, output,
reasoning, total), exact known USD, unknown-estimate count and attempt count.
Reject unknown/missing fields, wrong arity, negative/noninteger/overflow counts,
unknown counts above attempts and inconsistent empty values. Do not invent
cross-metric equations across different known populations. No attempt IDs,
observations, provider/model, timestamps or snapshot payload in this value.

Expose only the minimum pub(super) type and decode/encode/from-DayTotals/
to-DayTotals/checked-add methods inside the already private test-only chain.
Fields/parsing remain private. Composition validates both inputs and checks
every addition; failures return no partially modified result. Preserve all seven
unknown counts and known subtotals. Full USD stays unavailable for unknown
estimates or zero constituents. Ownership/snapshot references belong to a later
storage envelope, not this codec. Preserve original quote/rate serialization.

## Proof and bounds

Targets: 220 implementation + 3 registration + 310 tests + 35 receipt lines.
Hard limits remain 500 non-test / 800 total changed lines, additions AND deletions
including evidence. Do not remove cases or compress code to fit; stop for manager
re-slicing if necessary. One independent Astra High review plus scoped corrections,
no nested reviewers, extra panels or exhausted-budget resets.

Test canonical round trips at 0/6/18/24 places, u128 max/overflow, malformed syntax,
types and scales, all seven metrics, unknown versus explicit zero, count overflow,
amount alignment/sum overflow, and conversion from actual existing quote reduction.
Two 0.0000004 amounts must compose to 0.0000008 before six-place half-even display;
known 0.000153 plus an unknown estimate must retain an unavailable full amount.
Assert complete literal objects and rejected inputs, not fake disk-lifecycle proof.

Use pinned cached Rust 1.95.0, offline, no auto-install, own target directory.
Run scoped fix/guarded just fmt before final just test selectors for compact_values,
lifecycle, storage, pricing and journal, plus the full state crate. Every selector
must run nonzero tests. Run normal offline locked state --lib check, both plan/
sprint checkers and whitespace check. No direct cargo test, lock/process kill or
environment repair. Record command outputs, hashes, failures and leak markers;
existing Python formatter/cargo-fix restrictions are not a successful full format.
Parent reruns affected tests after integration. Technical human inspection shows
literal decoded totals, exact sum, unknown result and malformed rejection; no
interactive path changes, TUI/live repositories/human acceptance are not qualified.

## Next boundary — manager work, not another defaults question

Do not introduce pruning, watermark advancement, tombstone removal or compact
storage here. Later atomic mutation must include retention-aware reads/deletion,
shared snapshot ownership, 90-day detail cleanup and 365-day replay admission.
Never retain per-attempt numeric detail for 365 days disguised as a compact ledger.
The manager must resolve partial-day aggregate coverage before mutation: a daily
sum alone cannot subtract individually expired members at a rolling 365-day edge.
Do not silently choose early loss, over-retention or midnight-rounded expiry.
Also resolve unquoted/stale source disposition without fresh repricing, 90..365-day
late imports, completed-sweep versus partial-batch reads and corruption handling.
B1 does not depend on these choices; production and complete S02 remain unfinished.
