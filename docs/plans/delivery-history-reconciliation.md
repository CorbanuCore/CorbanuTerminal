# Historical PF-76-S01 — delivery-control identity reconciliation

This is the explanatory record for **historical delivery-control reports**, not
the modern provider-persistence sprint with the same ID.

## What happened

The recovery source assigned PF-76-S01 to internal delivery control. Main already
used [PF-76-S01 for provider-profile persistence](../sprints/current/p0-security-levels/pf-76-s01-provider-profile-persistence.md).
The receiving delivery-control work therefore uses
[PF-80-S01](../sprints/current/initiative-delivery-control/pf-80-s01-delivery-control.md).
See the [original source-transition record](main-workstreams-2026-09-11.md).

## Impact and owner

The Corbanu manager owns reconciliation. Ambiguous historical reports are held
out of current initiative joins; they cannot establish progress for either
modern sprint. Historical queued events remain held, with original IDs, payloads
and receipts preserved. New explicitly mapped delivery-control observations are
separate. This notice does not itself block current delivery-control development.

## Next action

The inventory is done, 2026-09-16: **nothing is held.** No queued Task Node event
exists in live state and no outbox has ever been written there; the only
PF-76-S01 delivery-control report in the repository is a synthetic test fixture,
which stays where it is. There is nothing to migrate, replay or delete.

The bounded fix now distinguishes report provenance. A PF-76-S01 report with
explicit `source_namespace: main-provider-profile-persistence` reaches its normal
task mapping. The synthetic historical namespace
`synthetic-recovery-delivery-control` remains refused with a historical-source
reason. Missing or unrecognized provenance remains held; a mapping is not proof
of provenance. Modern reports join the provider sprint normally; historical
notices continue to link here, and unknown provenance is labelled unresolved.

The inventory's namespace was on the fixture wrapper, not its nested report.
Reports now accept an optional namespace, which new queue records retain locally
outside the unchanged immutable event payload. Producers must supply it explicitly;
old reports/events without it are not reclassified, rewritten or replayed.
`prepare`, `flush` and `retry` read this local metadata and retain the hold when
it is absent. The single-event sender remains restricted to PF-80-S01.

Product authority: **Internal delivery control — TO BUILD**, “Task Node receives
only explicitly mapped, supported progress”. This restores an existing route;
posting, enrollment and authorization gates do not change.
[Inventory](../../qa/initiative-control/pf-80-s01/pf76-inventory-20260916.md) and
[revision evidence](../../qa/initiative-control/pf-80-s01/pf76-discrimination-receipt.md).
Do not silently alias IDs, delete history or replay an old event as new work.
Live posting remains OFF pending its separate gates.

## Is a decision needed from Travis?

No. This is manager bookkeeping, not an unanswered product question. If a
specific migration needs new authority, the manager must present its exact scope,
alternatives, recommendation and consequence of waiting before asking.
