# September 10 sprint identity reconciliation

The global sprint checker now passes: seven inherited errors at main
`0d1c3240d752335cc89faeb2149c5030063e5d69` were three duplicate sprint IDs and
four missing P0 feature/backlink entries. This routine ledger repair changes no
product behavior, authorization, sprint status, dependency or worker allocation.

## Canonical identities

All rows below belong to [P0 security](../plans/active/p0-security-levels.md).
Interpret a legacy ID only together with that owning plan and its old record
path. Bare PF-42–45 identifiers continue to belong to the existing Claude-auth
feature contracts; there is no global alias from those IDs to P0.

| Legacy P0 ID / filename | Canonical sprint | State retained |
| --- | --- | --- |
| PF-43-S01 / `pf-43-s01-provider-profile-persistence.md` | [PF-76-S01](current/p0-security-levels/pf-76-s01-provider-profile-persistence.md) | Draft, unallocated; reconcile shipped unified-provider evidence before implementing |
| PF-44-S01 / `pf-44-s01-tasknode-reliability.md` | [PF-77-S01](current/p0-security-levels/pf-77-s01-tasknode-reliability.md) | Draft with implementation/PTY evidence recorded; natural nonempty production observation outstanding |
| PF-42-S02 / `pf-42-s02-relink-recovery.md` | [PF-77-S02](archive/p0-security-levels/pf-77-s02-relink-recovery.md) | Completed serial relink repair; not completion of S01's observation |
| PF-45-S01 / `pf-45-s01-campaign-tracker.md` | [PF-78-S01](archive/p0-security-levels/pf-78-s01-campaign-tracker.md) | Completed for its recorded local integration/qualification scope |
| PF-45-S02 / `pf-45-s02-agent-profile-scope.md` | [PF-78-S02](archive/p0-security-levels/pf-78-s02-agent-profile-scope.md) | Completed profile-propagation repair |

The relink and propagation siblings are included to remove the same feature-ID
ambiguity across the whole touched P0 boundary, not just duplicate S01 records.
PF-76–78 were unused on the repair baseline. Filenames move with the canonical
IDs as required by the unchanged checker. Git retains the original paths at the
baseline commit; links in current docs and two release records now target the
renamed records. Those release records retain their original labels and claims.

The completed Claude-auth records and their dependency chain are byte-for-byte
unchanged. All five P0 records retain their status, owner, worktree, branch, base,
execution order, dependencies and checkbox ledger (apart from canonical ID text).
The Task Node S01 paragraph now agrees with its existing `draft` frontmatter;
the pending production observation is still unchecked. Historical QA evidence
is not re-executed or upgraded by this repair.

## Validation and handoff

The inventory remains 112 current / 121 archived sprints. P0's index is corrected
to 53 current / 28 completed archives and displays PF-27-S04's existing
`in_progress` state. PF-27-S04 and PF-35-S01 remain the only reserved sprints.
No proposal is activated, machine assigned, runtime deployed or Task Node record
written. The existing two-active-plan policy is unchanged.

Run the actual current validators before any implementation handoff:

```bash
python3 docs/plans/check.py
python3 docs/sprints/check.py
python3 qa/planning-ledger/2026-09-10/check.py
```

The last command is a fixed-baseline integrity audit for this repair, not a new
ongoing policy or allocation service. Evidence is under
`qa/planning-ledger/2026-09-10/`. Earlier portfolio receipt scripts intentionally
asserted seven inherited errors on their frozen publication candidates; keep
those receipts historical rather than treating their snapshot assertions as
current-tree gates. A passing planning check is not runtime, TUI, benchmark,
production-observation or human-acceptance evidence.
