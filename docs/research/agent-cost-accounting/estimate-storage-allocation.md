# PF-60-S02 — immutable synthetic estimate storage

Manager allocation, September 11 local / September 12 UTC. Same active PF-60,
PF-60-S02; **Product measurement**, “No commercial performance numbers have been
supplied.” Approved defaults unchanged. Quote candidate e8ffdad4e and journal
are accepted; receiving baseline `486d2fb9481c01f7d73a6f8d9992c6e1b96359dd`
passed 269 native tests (two process-leak warnings), 129 Python/governance tests
and normal state/Task Node library checks. No production or human pass implied.

## Exact worker scope

Reuse accounting checkout/branch in active plan, clean-fast-forward to reviewed
allocation commit before dispatch; parent records actual launch HEAD. Five files:

- `codex-rs/state/src/runtime/accounting_pricing.rs`: only Serialize import/
  derives below and `#[path = "accounting_estimates.rs"] mod storage;`.
- `codex-rs/state/src/runtime/accounting_types.rs`: only add Serialize to Usage.
- `codex-rs/state/src/runtime/accounting_estimates.rs`
- `codex-rs/state/src/runtime/accounting_estimates_tests.rs`
- `qa/portfolio/agent-cost-accounting/pf-60-s02/estimate-storage-increment.md`

Parent serially delegates these exact previously closed seams. All code remains
inside existing cfg(test) accounting. No accounting.rs/runtime.rs, old tests or
receipts, dependencies, BUILD, public visibility, production migration, UI,
provider, wallet, dispatch, auth or shared-plan edits. Target <=350 implementation/
seam lines + <=380 tests + <=50 receipt; hard <=500 non-test / <=800 total changed
lines (additions plus deletions). Stop/re-slice rather than omit checks.

## Native boundary and test schema

Make storage a private child of pricing, with sibling tests. It can access private
pricing fields and ancestor Journal/read_attempt/read_patches/native pool without
visibility changes. Reuse connection-taking readers inside transactions, not
Journal::read_observations which opens a separate transaction.
Add Serialize only to Snapshot, Rates, Currency, Unit, SourceKind,
ObservationQuote, BucketQuote and DisplayAmount; Usage's derive is above.
Implement Decimal serialization in the child as canonical exact decimal string.
Preserve the existing stricter rate Deserialize (38 digits/18 fractional places).
Quoted amounts can need 24 places: never deserialize them through the rate parser.

Private EstimateStore wraps Journal. Explicit fixture setup creates only:
`draft_accounting_price_snapshots(snapshot_id PRIMARY KEY, payload TEXT)`,
`draft_accounting_price_bindings(attempt_id PRIMARY KEY, snapshot_id nullable)`,
`draft_accounting_estimates(attempt_id, evidence TEXT, payload TEXT,
PRIMARY KEY(attempt_id,evidence))`. Add native foreign keys and NOT NULL where
appropriate. This is manager-allocated disposable test DDL, not a production
schema/version or migration. Normal StateRuntime initialization remains unchanged.
Use typed canonical JSON text, never floating point or SQL amount arithmetic.

Conceptual private entrypoints: create_for_tests; persist_current(attempt_id,
candidates) -> ObservationQuote; read_estimate(attempt_id, exact evidence) ->
Option<ObservationQuote>. Reopen wraps existing tables without repeating DDL.
No caller-supplied unchecked quote/Usage, external clock, catalog fetch or
production authority. Synthetic snapshots are inputs, not verified operator grants.

## Transaction contract

1. BEGIN IMMEDIATE. Load authoritative Attempt and ordered observations on that
   connection; missing Attempt errors. Validate and use accepted quote helper.
2. First persistence binds the attempt to selected immutable snapshot, including
   explicit unavailable NULL. Validate all candidates; any candidate whose ID
   conflicts with stored typed snapshot content errors even when not selected.
   Persist selected content only; identical duplicate is idempotent, no UPDATE.
3. On later observations, validate candidates but quote only from stored binding
   (or [] for unavailable). No historical automatic price fill/replacement or
   retrospective re-estimation. This is the conservative internal test behavior,
   not a new UI policy or a live catalog store.
4. Key each immutable estimate by exact ordered serialized observations including
   revision/source/sequence/presence. Equal evidence/payload is no-op; conflicting
   stored payload errors. Late lower revisions produce a distinct version even
   when highest revision is unchanged. Never add estimate versions as spend.
5. Commit snapshot, binding and estimate together. Any decode/validation/conflict/
   arithmetic/SQL failure rolls back all writes; source journal stays untouched.
   Use native writer serialization, no second database or cache authority.
6. Read in one transaction. Decode typed Attempt/observations/snapshot; validate
   binding and snapshot row, full Attempt equality, and every recorded source
   revision/position/patch against retained journal. Later extra revisions allowed;
   missing/changed recorded detail errors. Recompute using saved snapshot only,
   compare the entire canonical serialized result to stored payload, and return
   typed result only on equality. Reject added/missing fields, changed amounts/
   flags or unsupported values; no repair on read. Unknown key returns None.

The returned quote's exact strings/flags are checked by recomputation, not a
second permissive result parser. This proves retained-detail storage only;
retention-aware reads after pruning belong to the next lifecycle allocation.

## Proof and closeout

Test full-object roundtrips for Anthropic partial 0.000153, unknown/no snapshot,
measured zero/no rate, half-even ties and summed-before-rounding 0.0000008.
Amounts beyond 18 fractional places must roundtrip; rate bounds stay unchanged.
Real on-disk close/reopen twice must preserve snapshots/quotes/source rows.
Duplicate/concurrent same-key writes create one binding/version; conflicting
snapshot IDs, invalid inputs, missing attempts and SQL failures roll back.
Forged source position/amount/flag/snapshot fail read. New/late lower revisions
create separate immutable versions with old versions still readable. Prospective
catalog additions cannot reprice history; unavailable remains unknown.

Guard just fmt to allocated Rust paths; preserve/disclose unrelated formatter
failures. Use cached pinned 1.95.0 offline; no direct cargo test, toolchain/dependency
repair or process/lock cleanup. From codex-rs:

```text
just test -p codex-state runtime::accounting::pricing::storage::tests
just test -p codex-state runtime::accounting::pricing::tests
just test -p codex-state runtime::accounting::tests
just test -p codex-state
cargo check --offline --locked -p codex-state --lib
```

Both governance checkers and whitespace checks also required. Return uncommitted
with exact hashes, nonzero counts, failures, line budget, OFF proof and limitations.
Parent owns independent Astra High review plus scoped corrections and combined
tests; no exhausted allowance reset. Technical human review needs diff/hashes,
literal cases and cached tools/disposable homes, no credentials or product UI.

Upstream `https://github.com/openai/codex.git`, verified common ancestor
413492cd6c3a4d4f8dff6f406247ccda5a9d88aa, cached upstream
1d74c3ba1ee98be2025ab066dcc3fd654fe8a3b6. Product-owned test modules with tiny
serialized derives/registration only; retain native API/history/auth/deletion.
Existing source glob registers sibling tests; literal SQL/fixtures, no BUILD data.
No upstream upgrade, TUI/live-repository, benchmark or named runtime pass claimed.

## Subsequent required work

Next same-S02 allocation must join per-thread/day contributions with atomic
deletion/tombstones: one authoritative evidence version per attempt, replacement
on revision, exact sums/unknowns, 90-day detail and 365-day aggregates/replay.
Explicit deletion removes detail, copied quote provenance, attributable numerics
and associated identities; only minimal opaque replay tombstones survive within
the approved horizon. Resolve cutoff boundaries, shared-snapshot cleanup, replay
versus deletion concurrency, pruning and retained-evidence reads together.
Current FKs do not prove user deletion. Later connect native delete_threads_strict
transaction before ownership removal, then separately qualify OFF-preserving
production promotion, provider presence and durable pre-dispatch intent.
No production migration number is reserved. Process-crash, lineage, full S02
goldens and user inspection/export/deletion remain; S03 stays dependency-gated.
