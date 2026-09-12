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

Inventory each historical report and queued event against its original source
and task mapping, record an explicit disposition, and review any proposed
migration before acting. Do not silently alias IDs, delete history or replay an
old event as new work. Live posting remains OFF pending its separate gates.

## Is a decision needed from Travis?

No. This is manager bookkeeping, not an unanswered product question. If a
specific migration needs new authority, the manager must present its exact scope,
alternatives, recommendation and consequence of waiting before asking.
