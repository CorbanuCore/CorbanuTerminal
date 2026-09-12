# PF-60-S02 — Opt-in normal-library accounting store allocation

## Manager correction allowance — September 12, 15:19 UTC

Source dispatch occurred at ef13b347c; the original documents-only wording below
is history. First code review81230 found two P2s; corrected candidate1879total/
778non-test passed286 state tests with one disclosed LEAK. Corrective review13225
found one remaining production deletion-time race: time sampled before separate
cleanup and writer-lock acquisition can lag a concurrent accounting checkpoint.
Parent inspected and classified it in-scope at the same native deletion boundary.

The sole worker may correct production time sampling after the writer lock,
preserving explicit test timestamps and adding real reordered-deletion proof.
Same17 paths; original1900/950 target and1703/1879 candidates remain history.
Manager grants **2150 total/1000 non-test** solely for this second correction,
estimated30 runtime+120 regression+20 receipt lines. Report before exceeding;
never compress or remove proof. One additional corrective code pass is authorized;
if that fails, reclassify remaining findings under the two-cycle convergence rule.
No change to product, retention, collector, live authority or owner allocation.

September 12, 2026. Manager-approved single S02 execution unit, not another
prerequisite chain. This turn authorizes allocation documents only. Parent
inspects/reviews/integrates them before dispatching source implementation.

## Coordinates and accepted input

- Base: `8725e1ff755a5fa974058f465f3553b3f9b884eb`.
- Worker: `/Volumes/CorbanuDrive/Corbanu/worktrees/accounting-pf60-s01-20260911`;
  branch: `workstream/accounting-pf60-s01-20260911`; owner: Codex accounting
  production-store lane, existing PF-60-S02 reservation, no additional worker.
- Native six-file candidate `0eb98e8d3` integrated at that base; exact hashes
  parent-verified. First independent Astra High code review, helper70866,
  `native-ownership-code-review.json/txt`, exited0 clean. The [native receipt](../../../qa/portfolio/agent-cost-accounting/pf-60-s02/native-ownership-increment.md)
  records worker274/274 and950total/227non-test, original timestamp-fixture failure,
  fix restriction, warning/LEAK disclosures and prior review history.
- Native bridge accepted. Parent receiving at8725 reports exec46485 exit0:
  `just test -p codex-state -p codex-tasknode-session` passed354/354,0skipped14.997s,
  nextest `ef047f72-80c9-4700-863d-1f1e253b5d75`. No final-summary LEAK;
  21 fixture dead-code warnings,9duplicates. Parent governance also exited0,
  active3/current115/archived126. These are parent-attributed results, not reruns
  by this documents-only worker turn.
- [Active plan](../../plans/active/portfolio-agent-cost-accounting.md) and
  [current S02](../../sprints/current/portfolio-agent-cost-accounting/pf-60-s02-idempotent-usage-persistence-and-replay.md)
  own scope/dispatch. A/B/C1/C2/native assertions, receipts, original failures and
  all prior size/review dispositions remain frozen history, not work to repeat.

## Concrete output and approved allowance

Deliver a small typed codex-state facade usable from another Rust crate without
cfg(test): explicitly open a versioned store, admit an existing native owner's
request with its immutable original-price binding, atomically append observations
and exact estimates/contributions, read retained totals, maintain retention and
use actual native deletion. Reuse accepted journal/pricing/native/C2 operations.
Default StateRuntime initialization still installs and collects nothing. No real
Core/API caller, collector, CLI/config/environment switch or live activation.

Manager approves **1900 total / 950 conservative non-test**, including receipt,
test-gated implementation and both sides of moved bodies; estimate1515/795.
Above800 is justified by one coherent versioned-installation, normal-library,
native-lifecycle and atomic price-binding unit with failure/concurrency/reopen
proof. A migration alone would install unused state; visibility alone leaves
fixture DDL and separately committed journal/price/contribution writes. This is
not inherited C1/C2/native size authority. Report actual overage before expansion;
do not compress proof, silently add paths or introduce another allocation chain.
One NEW code review plus necessary correction, and one allocation review if
needed, are authorized to the parent; prior usage is unchanged. No reviews here.

## Exact 17 future implementation paths

These are source-dispatch boundaries, not permission to edit source during this
documents-only turn. Parent checks disjoint ownership of all shared state seams.

| Repository-relative path | Bounded responsibility | Estimated added+deleted |
| --- | --- | ---: |
| `codex-rs/state/src/runtime.rs` | Compile accounting normally; default startup remains OFF | 20 |
| `codex-rs/state/src/lib.rs` | Export only facade and bounded data types | 12 |
| `codex-rs/state/src/runtime/accounting.rs` | Register facade; gate fixture constructor; narrow journal access | 20 |
| `codex-rs/state/src/runtime/accounting_types.rs` | Minimal validated type visibility, no parallel identity model | 30 |
| `codex-rs/state/src/runtime/accounting_pricing.rs` | Narrow descriptor/result access; preserve arithmetic | 20 |
| `codex-rs/state/src/runtime/accounting_estimates.rs` | Factor persist_current connection body; preserve wrapper/tests | 100 |
| `codex-rs/state/src/runtime/accounting_lifecycle.rs` | Factor contribution connection body/read conversion; gate fixture DDL | 85 |
| `codex-rs/state/src/runtime/accounting_retention_atomic.rs` | Narrow facade forwarding/read conversion, no algorithm rewrite | 25 |
| `codex-rs/state/src/runtime/accounting_native.rs` | Same-connection admission, binding, estimate and contribution | 65 |
| `codex-rs/state/src/runtime/threads.rs` | Normal installed-store deletion hook, absent-store behavior unchanged | 20 |
| `codex-rs/state/src/runtime/accounting_store.rs` | New typed opt-in/open/maintenance/read/error facade | 210 |
| `codex-rs/state/src/migrations.rs` | Dedicated optional accounting migrator and ledger | 25 |
| `codex-rs/state/accounting_migrations/0001_usage.sql` | New ten-table DDL/constraints and inactive checkpoint | 95 |
| `codex-rs/state/BUILD.bazel` | Include accounting_migrations/** in compile data | 3 |
| `codex-rs/state/tests/accounting_store.rs` | New external-crate normal-library/OFF/upgrade/reopen proof | 420 |
| `codex-rs/state/src/runtime/accounting_store_tests.rs` | New internal migration/transaction/crash/contention proof | 300 |
| `qa/portfolio/agent-cost-accounting/pf-60-s02/production-store-increment.md` | Exact candidate evidence, failures and limitations | 65 |
| **Total / conservative non-test** | **Estimate, not actual results** | **1515 / 795** |

No Cargo/lock/dependency, Core/API/protocol/TUI, accepted test/receipt or shared
ledger edits belong to the future implementation worker. New tests use allocated
files only; any necessary additional boundary returns to the integrator first.

## Installation, OFF and native lifecycle contract

1. Ordinary STATE_MIGRATOR and StateRuntime::init must not create, adopt, validate
   as their own, or write the new accounting ledger. Fresh/default disabled
   profiles contain neither accounting tables nor ledger. Named explicit library
   opt-in is an engineering mode, not a new user-facing feature flag; there is no
   existing accounting feature flag to imply is already implemented.
2. Use separate `_accounting_migrations` in the same state DB and independent
   sequence0001 at the exact new SQL path above. Parent verified Migrator supports
   table_name; do not mix this history into `_sqlx_migrations`. Read-only search at
   base8725 found no accounting_migrations path/ledger reference; recheck receiving
   collisions before dispatch. No ordinary0053 is allocated. Do not inherit the
   ordinary runtime migrator's ignore_missing=true for unsupported newer accounting
   schemas: failed/partial/checksum-mismatched/newer schemas must fail closed.
3. Retain existing internal draft_accounting_* physical names to preserve proven
   SQL; naming does not imply fixture data is production-authorized. Do not silently
   adopt unversioned synthetic databases or repair unknown schemas/checksums.
   Existing native data and ordinary migration history must remain intact.
4. Gate create_for_tests constructors with cfg(test), preserving old assertions.
   Production open never invokes fixture setup or fabricates an Active checkpoint.
   DDL starts inactive; complete existing maintenance at one captured as-of performs
   activation. Crash during installation/activation must be recoverable: atomic
   rollback or a valid versioned inactive installation that retries safely. Never
   mistake interrupted/partial installation for completed activation.
5. Preserve public native delete signatures, counts, empty/missing-row semantics,
   last-delete ordering and no-schema behavior. Installed versioned accounting must
   still be deleted atomically with native rows when collection is disabled; OFF
   cannot strand old raw/compact ownership. Logs/memory/goals remain separate
   commits, not part of a global rollback/physical-erasure promise.
6. Require existing native owner equality/existence without creating a thread or
   auth principal/registry. Native upsert can recreate an ID; no permanent deleted
   identity fence or authorized historical resurrection is established here.

## Atomic original-price admission and observations

Current native append admits journal data but not price. EstimateStore::persist_current
and Lifecycle::refresh_current own separate write transactions. Factor their actual
connection bodies, keeping existing wrappers and tests, and call them through the
new facade's one BEGIN IMMEDIATE together with native admission and full maintenance.

Inputs use existing ThreadId/logical request/attempt UUIDs, provider/model/scope/
dialect, dispatch time and immutable original-price selection. Validate Active
store and selected native owner; commit intent, matching approved/effective-time
snapshot or explicit NULL binding, initial unknown estimate and contribution
together. Return only after commit. Unknown usage is not zero; an intent does not
prove an actual send, provider consumption or billing.

Later revisions atomically update journal/estimate/contribution using the original
binding. Later catalogs must not replace NULL or reprice existing rates. Reuse
snapshot UUID/content/provider/model/scope/time checks and exact arithmetic. A typed
descriptor validates content, not its claimed approval authority: real trusted
price provenance still belongs to the actual caller. No live rates, arbitrary raw
provider JSON, balance-as-invoice or post-hoc original-price invention.

## Required proof and future commands

- External state/tests/accounting_store.rs imports the public normal codex-state
  library, never private source or fixture constructors. Default fresh and populated
  pre-accounting profiles retain native data/ordinary checksums and zero accounting
  objects. Explicit opt-in validates all tables, indexes, foreign keys and ledger.
- Installation/activation failures and real process interruption/reopen at both
  boundaries prove recoverable state and retry without fake Active. Check partial,
  unversioned, failed, checksum-conflicting and unsupported-newer schemas without
  silent repair. Ordinary migrations must not consume accounting ledger entries.
- Owner mismatch/missing owner, conflicting snapshot, unsupported late import and
  journal/binding/estimate/contribution SQL faults leave all ten accounting plus
  three native tables unchanged. Exact markers after earlier writes, commit-time
  failure, two actual closes/reopens and successful retry are mandatory.
- Priced/NULL-bound intents, known zero/unknown, duplicate/reordered revisions and
  changed catalogs after reopen have one exact atomic visible result. Preserve
  original-price attribution. Reuse accepted C2 day-boundary tests; do not repeat
  its14-site matrix or remove prior proof.
- Real price-bound admission versus native deletion in both orders, actual lock
  contention, held old read/fresh snapshot and deleted-owner fresh UUID retries.
  Exercise both public deletion methods and collection-disabled deletion of
  previously installed raw/compact state, with separate-store limitations explicit.
- Preserve all A/B/C1/C2/native tests, original failures and review usage. Future
  scoped fix/literal formatting precede nonzero `just test -p codex-state --test accounting_store`,
  the registered accounting_store_tests selector, existing accounting/native
  selectors, full `just test -p codex-state`, then normal-library
  `cargo check --offline --locked -p codex-state --lib`. Record actual module
  registration/selector; no empty filter counts as evidence.
- Cached Rust1.95.0, offline/autoinstall OFF, assigned target; no direct cargo test
  or environment repair. Receipt includes literal diff/counts/digests, commands,
  exits, original failures, warnings/LEAKs and limitations. Parent owns independent
  review and combined state/TaskNode receiving verification. This allocation turn
  runs only governance/diff checks, not source formatting, builds/tests or reviews.

## Remaining caller/import gaps and authority

Real collection stays OFF and outside this store unit. Core run_sampling_request
must own logical identity across retries; core/client.rs adds auth/history/413
retry loops, and codex-api/endpoint/session.rs sends after auth inside transport
retries. Future collection must await durable admission immediately before each
actual send, without API-to-SQLite coupling or changing existing gateway identity.
Store commit alone is not that wiring or a qualified pre-/post-send failure policy.

Raw presence must precede AnthropicUsage::merge_into/ResponseCompletedUsage defaults;
cumulative TokenUsage cannot recover omitted/null/zero or provider dialect authority.
Actual original-price approval/source selection, optional StateDbHandle failure,
WebSocket/chat-completions coverage and complete root/two-child/consumed-retry/
ambiguity/native restart goldens remain S02 obligations, not new store prerequisites.

The90..365-day compact-only late-import guard stays unchanged and explicitly
unsupported, not waived. Future import needs authoritative dispatch identity,
presence and original-price provenance, atomic deduped compact-only writes without
aged raw detail, and a real finality/correction authority. Tombstones cannot subtract
revisions after per-attempt amounts expire. No invented sealed-final rule, longer
detail retention, shortened replay or claim of complete historical import.

Manager has accepted separate ledger/sequence, typed facade and default-OFF
engineering choices plus this size/review allowance. No new product decision is
needed for approved defaults. Changing horizons/daily expiry, retaining identifying
or numeric detail longer, permanently dropping imports, making ambiguous history
known, promising global erasure or enabling new user-visible/live behavior would
require new authority. S03 stays draft; full S02/S03, human/live/TUI/benchmark and
release readiness are not claimed. Native combined evidence is accepted above;
parent must accept/integrate these documents before source dispatch.
