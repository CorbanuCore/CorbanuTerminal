# PF-60-S02 — retention decision and mutation handoff

Planning only, not an executable allocation. Product: **Product measurement**,
“No commercial performance numbers have been supplied.” The approved accounting
defaults stand; this resolves their implementation without enabling collection.

## One newly delivered product decision

On September 12 the manager asked Travis whether each daily aggregate may expire
when its UTC day start reaches 365 days old. Recommendation: approve conservative
daily expiry, with the oldest retained day explicitly visible. This never retains
a contribution past 365 days, but can discard a newer member up to 24 hours early.
Alternative: require exact rolling availability and revisit aggregate representation.
Status: unanswered. Do not infer approval, repeatedly ask, or implement either
behavior. This is not another request to approve the existing 90/365-day defaults.

Why it requires a decision: after raw detail is removed, A+B for two requests on
the same day cannot yield B alone when A expires earlier. UUID+expiry tombstones
carry no subtraction amounts. Keeping per-request amounts for 365 days defeats
the accepted detail-retention boundary. Whole-day early loss or late retention
changes availability or privacy, so neither is a silent engineering choice.
The exact rolling 365-day replay horizon and 90-day detail boundary do not change.

## Manager-owned design after the answer

Use independently deletable thread/day aggregate values, not global-only sums.
Reuse B1's exact CompactValues codec; a separate thread/day/snapshot reference set
keeps immutable price versions while any surviving raw or compact owner needs them.
No copied observation, per-attempt amount, source-position or price-binding ledger
may remain as disguised detail after raw cleanup.

The mutation allocation must own one atomic transfer boundary: validate source and
destination, combine exact amounts/unknowns, retain needed snapshot references,
insert opaque attempt tombstones, then remove all source/estimate/contribution/
binding copies in FK-safe order. Repeating a committed batch cannot add it again.
Rollback must restore all representations. Do not nest transaction-owning methods.
Existing EstimateStore::read_on_connection validates old versions only while their
source remains; compact values cannot later reconstruct those erased observations.

Source disposition: derive latest measured usage from validated retained authority.
An existing price binding is immutable and must be reused, including a null binding.
Do not freeze a stale quote, use a fresh catalog, or treat unquoted intent as zero.
Before dispatch, allocate a precise connection-level latest-quote path for stale
estimates and never-bound intent using no new price candidates. Corrupt source,
missing bound snapshot or invalid arithmetic must be distinguishable from honest
unknown usage; transaction rollback is not evidence of successful retention cleanup.

Admission/maintenance: use checked explicit as-of times and a monotonic durable
checkpoint. Raw expiry is dispatch+90 days; replay expiry is dispatch+365 days.
Advance the replay admission floor atomically with eligible tombstone removal,
rejecting expired imports even after their UUID record disappears. The current
journal writer must not admit aged detail merely because it uses a new UUID.
Manager must size a compact-only 90..365-day late-import path or record that import
coverage as unsupported and unqualified; do not silently shorten replay to 90 days.

Reads and deletion must ship with transfer: one read transaction combines compact
values and only untransferred current raw contributions. Expose coverage and
drill-down cutoffs separately from unknown quantities. Partial sweeps or elapsed
unapplied cleanup cannot claim a complete current total. Delete compact-only
threads, remove attributable aggregates and GC snapshots only after both raw and
compact owners disappear. Retain anonymous replay records without thread metadata.
Native deleted-thread admission and physical database/WAL erasure remain later
production integration; this fixture cannot promise either.

## Scope and human-style proof before dispatch

Parent next records exact paths/interfaces, baseline and nonoverlapping ownership
in the active plan/sprint. Likely seams are lifecycle read/delete, journal admission
and a private retention child; no worker owns those writes yet. Budget the code and
proof together under the existing 500 non-test/800 total limits. If oversized,
split a coherent non-mutating prerequisite; never expose a compactor while deferring
its deletion, admission or read obligations. No automatic fourth lane or S03 start.

Technical test plan: two owners sharing snapshots, mixed raw/compact partial days,
seven unknown metrics, 24-place USD, repeated batches, exact boundary times,
dispatch zero, backward/overflow clocks, compact-only deletion after two reopens,
SQL abort at each transfer/checkpoint stage, and actual writer contention plus
both serialization orders. Inspect complete before/after tables and literal totals;
no sleeps as race proof. Old detail must disappear without losing another owner's
snapshot or admitting replay. Boundary-day availability follows only the approved
answer. Runtime human/TUI/live-repository, crash, benchmark and release proof remain
unqualified. No collector, scheduler, billing, deletion job or live price fetch.
