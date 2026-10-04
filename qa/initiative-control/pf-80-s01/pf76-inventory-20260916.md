# Historical PF-76-S01 inventory, and the collision that is still live

**Fable, 2026-09-16.** The delivery sprint carries an obligation to "reconcile
historical PF-76 reports/queued events with PF-80 explicitly; no silent replay,
deletion or automatic flushing". This is the inventory half, done by looking
rather than by assuming there is a backlog.

## Inventory: there is nothing held

| Where I looked | What is there |
| --- | --- |
| `…/initiative-control.oGQGyA/state/` | `control.json`, `coordinator/`, `decisions.fixture.json`, `decision-slack-status.json`, `events/`, `slack-operator/`, `source.json`. No `outbox/`, no `outbox-index.json`, no report store. |
| `…/initiative-control.oGQGyA/` | `publisher-report.json`, `fable-report.json`, `manager-report.json` — current publication receipts, not queued Task Node events. |
| `scripts/initiative_control/fixtures/recovery-old-state.json` | One report, `run_id` `recovery-synthetic-1`, `source_namespace` `synthetic-recovery-delivery-control`, explicitly synthetic. |

**Disposition: nothing to migrate, replay or delete.** No historical queued
event exists in live state. The only PF-76-S01 delivery-control report in the
repository is a synthetic test fixture, and it stays exactly where it is. The
holding behaviour is real code, not a promise: `tasknode.enqueue` refuses the
legacy ID outright and three further call sites re-check it.

## The collision is not just historical

`PF-76-S01` is also a live sprint in the security plan —
`docs/sprints/current/p0-security-levels/pf-76-s01-provider-profile-persistence.md`,
status `draft`. The guard keys on the sprint **ID**, at
`scripts/initiative_control/tasknode.py` lines 60, 191, 450 and 495. So the
modern provider-profile sprint can never report progress through Task Node: its
reports would be refused as historical delivery-control, forever, with no path
other than renaming the sprint.

Live `control.json` already carries a `task_mappings` entry for `PF-76-S01`.
That mapping is unreachable today — anything that matched it would be refused
before the mapping was read. Configuration that cannot ever apply is worse than
absent, because it reads like a working route.

This is not urgent: PF-76-S01 is draft and posting is OFF. It is recorded now
because the reconciliation item is the right place to find it, and because the
next person to read that mapping would reasonably assume it works.

## Frozen next unit

Discriminate on provenance rather than on the identifier. The historical reports
are distinguishable by `source_namespace`; the identifier is not a
discriminator, because two unrelated sprints legitimately share it.

Located files, verified present:

- `scripts/initiative_control/tasknode.py` — four legacy-ID call sites.
- `scripts/initiative_control/test_tasknode.py` — must gain a case proving a
  modern PF-76-S01 report is accepted while a historical one is still refused.
- `scripts/initiative_control/control.py` — `LEGACY_DELIVERY_SPRINT` definition.
- `scripts/initiative_control/attention.py` and `test_attention.py` — the notice
  and its link resolution both special-case the same ID.
- `docs/plans/delivery-history-reconciliation.md` — the explanatory record.

No decision is needed from Travis. Posting stays OFF; this changes which reports
are refused, not whether anything is sent.
