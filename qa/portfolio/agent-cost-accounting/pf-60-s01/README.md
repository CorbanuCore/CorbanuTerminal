# PF-60-S01 synthetic accounting evidence

Status: proposal v1; bounded local fixtures, not product or human acceptance.
Contract: [source mapping and decisions](../../../../docs/research/agent-cost-accounting/contract.md).
All prices, invoice/account figures, token counts and identifiers are synthetic.
Provider names identify source adapter families, not actual usage or quotes.
`pfterminal-plan` is the native compatibility routing ID for the Corbanu API
fixture; it does not restore legacy Plan entitlements.

## Exact local verification

Run from the worker repository root, using Python's standard library only:

```bash
python3 -m unittest discover -s qa/portfolio/agent-cost-accounting/pf-60-s01 -p 'test_*.py'
python3 docs/plans/check.py
python3 docs/sprints/check.py
git diff --check
```

`test_contract.py` registers nonempty unittest discovery. `fixtures.json` is the
raw input and literal golden expectation source. `reference.py` is a proposed
contract oracle, not an adapter to Rust, SQLite, rollouts, or provider services.
Tests use exact rational arithmetic, with decimal text for prices. No dependency
installation, private data, network or native product invocation is needed.

## Hand calculations (USD)

Rates are invented dollars per million tokens. Arithmetic below was written
separately from the reducer and expected values are literals in the fixture.
Terms before division can also be read as micro-USD for these synthetic rates.

| Attempt | Explicit calculation | Expected estimate / unknowns |
| --- | --- | --- |
| `root-1`, OpenAI current | `(100 - 40 - 0) * 2 + 40 * 0.5 + 0 * 2.5 + 20 * 4 = 120 + 20 + 0 + 80 = 220`; divide by 1,000,000 | `0.000220`; 8 reasoning tokens are within the 20 output. Separate invented billed amount `0.000300`, not added. |
| `child-a-1`, Anthropic interrupted | Native inclusive input `50 + 10 + 20 = 80`; known cost `50 * 3 + 10 * 0.3 + 20 * 3.75 = 150 + 3 + 75 = 228`; divide by 1,000,000 | Known subtotal `0.000228`; output and full estimate unknown. Failure does not erase the input. |
| `child-a-2`, Anthropic retry | Native input `100 + 20 + 40 = 160`; cumulative output `0 → 12 → 30`; `100 * 3 + 20 * 0.3 + 40 * 3.75 + 30 * 6 = 300 + 6 + 150 + 180 = 636` | `0.000636`; total tokens `160 + 30 = 190`; separately measured reasoning unknown. |
| `child-b-1`, Corbanu API | Native Chat input 80 includes read 20; write unreported; output 10; total `80 + 10 = 90`; no model price | Full estimate unknown; zero known-priced subtotal is labeled as subtotal, never displayed as free work. Native fixed cache-write zero is not measured zero. |
| `historical-priced`, OpenAI old | `(100 - 40) * 1 + 40 * 0.25 + 20 * 2 = 60 + 10 + 40 = 110` | `0.000110` using old price before September 1; current price would wrongly produce `0.000220`. |
| `historical-unknown` | Input 50, output 10, total 60; cache split absent; only known output cost `10 * 4 = 40` | Known subtotal `0.000040`, full estimate unknown; missing cache/reasoning details stay null. |

Root subtree is exactly `root`, `child-a`, `child-b`: three logical requests,
four dispatched attempts. Retry input is consumed independently. Known cost
`220 + 228 + 636 + 0 = 1084` micro-USD = `0.001084`; full total is unknown for two
attempts. Child-a alone has `228 + 636 = 864` micro-USD known and unknown full cost.
Known billed subtotal is `0.000300`; three attempts lack billed evidence.

Root known tokens: input `100 + 80 + 160 + 80 = 420`; cache read
`40 + 10 + 20 + 20 = 90`; cache write `0 + 20 + 40 + ? = 60 known, 1 unknown`; output
`20 + ? + 30 + 10 = 60 known, 1 unknown`; reasoning
`8 + ? + ? + 0 = 8 known, 2 unknown`; total
`120 + ? + 190 + 90 = 400 known, 1 unknown`. No extra cache/reasoning addition.
Historical attempts belong to a separate thread and do not enter these totals.

The synthetic API account snapshot has balance 10,000,000 micro-USD, reserved
2,000,000, available 8,000,000 (`10 - 2 = 8 USD`). Its availability is not a
request estimate. A provider window with 25% used has no inferred cash value.

## Negative and recovery cases

The unit suite checks these expected counterexamples, with successful assertions
meaning the bad interpretation was rejected, not that a real provider failed:

- Additive stream output would produce 42 instead of 30; adding reasoning cost
  would produce `0.000252` instead of `0.000220`. Both are explicitly disproved.
- Dropping the consumed retry would omit `0.000228`; missing price/usage cannot
  produce a full zero total. A genuinely explicit all-zero complete observation
  has zero token-only cost even without prices.
- Duplicate/reordered replay and all six permutations of the three child retry
  revisions produce identical results. Conflicting duplicate revisions fail
  without committing the batch. Missing/reused provider response IDs do not
  silently collapse local attempt identities.
- A partial JSON checkpoint passes through stdin to a fresh Python process;
  replaying the full stream twice produces the same aggregate. This is neither
  a native database restart test nor an actual kill-during-request test.
- Invalid counts (negative, noninteger, boolean), impossible cache/reasoning
  subsets, conflicting totals, wrong logical owner, unknown attempt/parent,
  cyclic retries/lineage, duplicate attempt IDs and invoice allocation are rejected.
- Price overlap, duplicate identity, unsupported unit/currency and invalid
  money are rejected. Effective bounds are half-open; missing dispatch time or
  a historical price gap returns no price. Adding a future price preserves old
  results. A missing output price exposes the known input subtotal only.
- Changing provider allowance or API balance cannot alter request totals. The
  oracle has no path from these snapshots into request-cost arithmetic.

## Boundaries deliberately left open

The raw synthetic wire observations preserve field presence; the suite does not
claim that a deserialized legacy native zero can recover lost presence. Native
snapshot import, compaction, inherited history exclusion, rollback, provider-ID
reconciliation, multiple-parent edge ingestion, transport-layer retry capture,
invoice ingestion, precision display rounding, durable price immutability and
retention/tombstone behavior need S02+ design and tests. The oracle receives a
single-parent mapping and a stable price input; it is not a production validator.
Unknown lineage is rejected here; a product quarantine view is not implemented.
Source-only inspection found no authoritative historical invoice join or price
effective-date history. No upstream SHA was independently verified.

Actual run outputs, timestamps, artifact digests and scope receipt are in
[results.md](results.md) and [handoff.md](handoff.md). Independent review and
Travis's acceptance remain pending with the manager. No UI, live-repository,
benchmark, release or runtime enablement claim follows from this evidence.
