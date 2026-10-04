# PF-60-S02 compact-only original-evidence import — worker receipt

Correction01 supersedes the original race-test coverage below. Parent material
review01 exited1 with one accepted P2; the correction section records the changed
test and new evidence. All original1970/463 results remain historical, not erased.

Product initiative; **Product measurement**, “No commercial performance numbers
have been supplied.” Current eight-path compact-late-import allocation applies.
This is an internal opt-in normal-library increment, not full S02, S03, billing,
live collection, a historical-source scanner, or human/release readiness.

## Coordinates and scope

- Worker: `/Volumes/CorbanuDrive/Corbanu/worktrees/accounting-late-import-20260913`.
- Branch: `workstream/accounting-late-import-20260913`.
- Clean launch: `37d6ffcc97bd3d3e1149ced136ccdae690c75658`; unchanged HEAD.
- Source base: `eb58559590dd2b9332f2445971ee086988c67080`; launch Rust tree
  `4222fd172edfdb04d0e69e019454894f783f379c`. No inherited code delta.
- Only eight allocated paths changed. No Core, Cargo, lock, BUILD, schema,
  shared plan, auth, collector, or old fixture/receipt edits; no commits/reviews.
- Manager target2000total/650non-test, STOP2200/750. Final counts below include
  additions plus deletions, new files and this entire receipt as non-test.
- Exclusive canonical Mac target was checked idle before builds. No RTX, old
  accounting target, cache invalidation, install, credentials or provider calls.

Evidence directory (E):
`/Volumes/CorbanuDrive/Corbanu/.codex-work/accounting-late-import.HLIhmu`.
`candidate.diff` is the literal complete patch, including untracked new files;
`frozen-manifest.json` records eight file hashes/counts and all log/JUnit digests.
Neither is a source commit or an independent review result.

## Implemented operation

`AccountingStore::import_retained(owner, bundle, as_of_ms)` owns BEGIN IMMEDIATE,
validation, maintenance, compact additions and one commit/rollback. It requires
an installed Active store and a present native owner under that same writer lock.
It never installs implicitly, changes default OFF or alters admit/observe guards.
Input caps are checked before cloning/sorting:1..64 attempts,256 observations per
entry,4096 total. Complete immutable snapshots are a caller contract, not a
fabricated provider-final boolean. Interrupted attempts may retain unknowns.

Dispatch+90days is inclusive; dispatch+365days is exclusive. Conservative UTC
day-start+365days can discard an aggregate before the attempt replay fence ends.
Checked explicit times reject negative/backward/future/overflow input without
clamping the durable clock. Expired-day imports retain only UUID/expiry fences.

Raw collisions reject before maintenance erases authority. Unseen identities,
request ownership, unique revision/source positions and retry ancestry are
validated against the bundle and retained raw authority. Identical revisions and
identical bundle entries contribute once. Erased external parents cannot prove
ancestry. All-seen bundles suppress without payload certification or advancing
maintenance/coverage; tombstones carry no owner, amount, price or payload hash.

Bound original prices must match dispatch eligibility and exact immutable stored
bytes. Unpriced stays unpriced; no fresh catalog fills an old unknown. Existing
quotation, prefix replay, DayTotals and CompactValues implement arithmetic.
Result validation checks the whole store and raw+compact sums, including stale
raw contributions. There is no aged raw insertion, even temporary staging.

## Original proof and its limits (1970/463 candidate)

- Normal-library root/two native children/unrelated owner, actual spawn edges,
  mixed young raw and aged compact, literal full DayTotals/coverage and exact
  snapshot/reference/tombstone sets. A second normal database admits original
  young evidence then maintains it; all ten accounting tables match after two
  reopens. Shared reducer equality is supplemental to the independent goldens.
- Omitted/null/zero, null-after-known, reordered cumulative revisions, split
  caches, all three dialects, partial/unpriced unknowns, exact1e-24 USD and
  half-even display; metric, unknown population, estimate and USD overflow.
- Exact90/365/+1ms boundaries, invalid clocks, raw guard/collisions, bounded
  inputs, wrong/missing owner, retry chains/cycles/erased parents, invalid prefix,
  and original snapshot corruption/ineligibility. Rejections compare the full
  main database: ten accounting tables, both ledgers, native owners/spawn edges.
- SQL fault markers: LATE_SNAPSHOT_INSERT, LATE_DAY_INSERT, LATE_DAY_UPDATE,
  LATE_REFERENCE_INSERT, LATE_TOMBSTONE_INSERT, LATE_CHECKPOINT_UPDATE. The UPDATE
  trigger requires attempts>1, reaching the new merge after maintenance. Each
  checks exact rollback, two disk reopens, removal of the test fault and retry.
  Deferred LATE_COMMIT_FK inserts complete inside the transaction but fail commit.
- Separate connections contend for same-ID and distinct-ID same-day imports,
  both import/maintenance orders, held old read snapshots and fresh complete
  reads. Public native deletion contends in both observed lock orders; a bounded
  race requires both actual outcomes, not an assumed SQLite FIFO ordering.
- Five raw INSERT-abort guards reject any aged staging. Collection-OFF public
  deletion preserves counts/missing-row behavior and another owner's snapshot;
  last owner deletion GCs it. Recreated native owners cannot replay fenced IDs.
- Actual test subprocesses cover public close/reopen, successful commit/lost
  acknowledgement, interruption while a polled import is writer-blocked, and
  killing the private real connection body after uncommitted writes. Two worker
  entry tests return without work when run alone; their substantive branches are
  executed by parent tests. No production hooks or agent children were added.
- Cancellation after commit may lose the acknowledgement: retry original IDs.
  Tests do not prove physical WAL erasure, power-loss durability, authentication
  of supplied history, permanent deleted-owner fencing or erased payload equality.
  Logs/memory/goals cleanup remains in separate databases; no cross-database
  atomic erasure is claimed.

## Original commands and actual outcomes

All Rust commands run in the worker's codex-rs with prefix P:
```sh
env RUSTUP_TOOLCHAIN=1.95.0 RUSTUP_AUTO_INSTALL=0 CARGO_NET_OFFLINE=true UV_OFFLINE=true CARGO_TARGET_DIR=/Volumes/CorbanuDrive/Corbanu/worktrees/management-workstreams-20260911/codex-rs/target
```
The manifest preserves literal commands, working directory, UTC timing and exits.
Final .time files record wall boundaries; earlier logs use filesystem creation/
last-write timestamps explicitly interpreted in UTC. JUnit was copied before
each following selector could overwrite the worker-local nextest output.

| Gate | Result |
| --- | --- |
| Scoped sandbox P just fix -p codex-state --locked | 101: TCP lock-listener bind denied, OS error1; no successful fix claim or repair. |
| Literal seven-file rustfmt --edition2024 --config skip_children=true | 0; final format-04 precedes all final tests; seven stable imports_granularity warnings. |
| First focused compile, focused-01 | 101 before execution: ambiguous assert_eq macro via support glob import, emitted in three test compilations; corrected explicit import. |
| Preliminary focused-02 | 19/19 pass,6.544s;287 filtered, no execution skips; run a76a9a8d-934e-472b-b822-dca0602b55c3. |
| Initial six-crate policy lint, clippy-01 | 101: new needless_collect; corrected by iterating original snapshot map and selecting referenced IDs. |
| Six-crate policy lint, clippy-02 | 0,1m20s; existing workspace levels with --no-deps, no global -D warnings. |
| Final focused new selector | 19/19 pass,5.552s;287 filtered, no execution skips;a7de88fa-e6e3-4adc-ba41-979bbdb5f18d. |
| Exact external targets | 5/5 pass,1.109s,0 skipped;073fc684-6b65-4fa6-be44-5f7b2bf1aa02. |
| Full state/API/TaskNode serial | 594/594 pass,35.357s,0 skipped;559a3d10-f539-4a32-b979-085d17245db7. |
| Existing 100 Core selector | 100/100 pass,20.620s;3466 filtered, no execution skips;c5ce52e8-ac37-47ad-b4de-e1b1953ed4fa. |
| Six normal libraries check | 0,49.64s;normal libraries only, offline/locked. |
| Governance and diff | Both checkers0:3active/115current/126archived; diff check0. |

Warnings are preserved verbatim, not a warning-free or workspace-strict-clean
claim. In particular the new original-price enum has a default-level large
variant warning; shared test helpers and existing fixture code also emit warnings.
No lint allowance, broad formatter, direct cargo test, workspace/full-Core run,
environment repair or lock mutation was used. LEAK/skip summaries are recorded
separately in the final evidence manifest, never inferred from a successful exit.
No final test log reports LEAK, FLAKY or a failed/retried test; execution skips0.

## Original frozen size

| Allocated file (under codex-rs unless qa/) | Added / removed |
| --- | --- |
| state/src/runtime/accounting_store.rs | 49 / 0 |
| state/src/runtime/accounting_retention_atomic.rs | 3 / 0 |
| state/src/runtime/accounting_late_import.rs | 247 / 0 |
| state/src/runtime/accounting_late_import_tests.rs | 837 / 0 |
| state/src/runtime/accounting_late_import_test_support.rs | 223 / 0 |
| state/tests/accounting_late_import.rs | 334 / 0 |
| state/tests/accounting_late_import_process.rs | 113 / 0 |
| qa/portfolio/agent-cost-accounting/pf-60-s02/compact-late-import-increment.md | 164 / 0 |

Runtime/registration299 + tests1507 + receipt164 = **1970total/463non-test**.
The manifest keeps the receipt hash external to the hashed receipt itself.

## Prior evidence and handoff

Manager-attributed receiving eb5855959 passed100/575, Clippy and normal check
before launch. Reviewed Core933/279 history (b320ef722 ->2cb69e429), earlier
835/249 failures, policy7/8-path returns, all A/B/C1/C2/native/store receipts,
and full-Core132/133 baseline-common plus14 serial-passing additional failures
remain unchanged. Those failed full gates and historical LEAKs are not waived or
relabelled by these bounded final results. Parent owns the one new material
review plus necessary correction and exact receiving combined gates.

The Corbanu skill/root1.7 isolated-execution policy governs applicability:
increment-specific internal/default-OFF N/A is manager-accepted here. No affected
user-facing import control or historical acquisition source is provided. The
integrator still provisions independent binary-only execution, actual negative
access probes, schema-2 receipts and separate evidence review before an applicable
collection/replay user handoff. S03 stays draft until all S02 gates; later native
user goldens, TUI/human/live-repository qualification are not claimed complete.

Next receiving action: inspect the frozen patch/proof, run the allocated material
review and combined checks. Subsequent acquisition/UI work must reuse this typed
original-evidence boundary; do not reconstruct raw presence/price authority from
lossy rollout TokenCount data or invent a new registry/approval prerequisite.

## Correction01 — deterministic orders, independent real contention

Parent `accounting-late-import-review-01.json` exited1, one P2 at external test
line82, confidence0.98 (overall0.93). Requiring both SQLite winners within16
races was not guaranteed by the writer barrier or alternating poll order. The
original tests happened to pass, but that was not a deterministic diversity proof.
The manager accepted the finding and authorized this same-eight-path correction;
one necessary parent correction rereview remains, not a new review budget.

Only `codex-rs/state/tests/accounting_late_import.rs` and this receipt changed
since1970. All six other Rust files (including runtime/registration) and the
private next-assignment note stay frozen. No production clock, lock, schema,
arithmetic, deletion, original-price, identity or API behavior changed.

The new serialized-outcomes test awaits public import then public delete, and
separately public delete then public import. It proves Imported/cleanup/fence in
the first order; missing-native-owner rejection with a complete unchanged main
DB dump in the second. Reading the actual completed checkpoint supplies an
explicit test time, isolating owner rejection from wall-clock ticks; no production
clock clamping is added. Both verify missing-row deletion and two native reopens.
These are explicitly sequential outcome checks, not called contention evidence.

The separate contention test polls both real public futures while a separate
connection holds BEGIN IMMEDIATE, proves both blocked, releases it, and accepts
either correct winner. It checks owner absence, complete compact/reference/price
cleanup and the exact outcome-dependent UUID/expiry fence set. No scheduler
diversity assertion, retry-until-both loop, test skip or weakened cleanup remains.

Correction evidence C:
`/Volumes/CorbanuDrive/Corbanu/.codex-work/accounting-late-import-correction.mBWkto`.
Original E artifacts and hashes remain immutable; C contains the original receipt,
accepted review JSON, final whole-candidate diff and corrected frozen manifest.
Exact commands, UTC timing, exits, file hashes and copied JUnit are in C's manifest.
Scoped fix again exited101 on the unchanged TCP lock-listener policy denial;
literal external-file rustfmt exited0 with one imports_granularity warning before
all corrected tests. No environment/cache/lock repair or broad formatter was used.

| Corrected final gate | Actual result |
| --- | --- |
| New focused / exact external | 20/20,6.148s,287 filtered /6/6,1.303s,0 skipped; run IDs in C manifest. |
| Full shared /100 Core | 595/595,41.529s,0 skipped /100/100,21.933s,3466 filtered; run IDs in C manifest. |
| Policy Clippy / normal check | Clippy0,3.87s; normal check0,0.86s. |
| Governance / diff | Both checkers0,3active/115current/126archived; diff check0. |

Corrected size:2099total/515non-test =299runtime +1584tests +216receipt. This is
99 above the original2000 target, within hard2200/750; original1970/463 preserved.
No corrected execution skips/LEAKs. Parent rereview/receiving and S02 gates remain.
