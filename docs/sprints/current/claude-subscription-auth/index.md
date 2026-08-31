# Claude subscription-auth execution sprint

The active [Claude subscription-auth plan](../../../plans/active/claude-subscription-auth.md)
allows one serial sprint. CSA-01 is the current typed-contract unit; later units
remain plan-only until their dependencies are completed and archived.

| Order | Sprint | Outcome | Status | Depends on |
| ---: | --- | --- | --- | --- |
| 1 | [CSA-01 / PF-42-S01](pf-42-s01-auth-source-contract.md) | Typed source, selection, health, and persistence | in progress | none |

## Machine checks

```bash
python3 docs/plans/check.py
python3 docs/sprints/check.py
```
