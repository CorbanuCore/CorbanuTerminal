# Claude subscription-auth execution sprint

The active [Claude subscription-auth plan](../../../plans/active/claude-subscription-auth.md)
allows one serial sprint. CSA-01 is completed and archived; CSA-02 is the sole
in-progress unit.

| Order | Sprint | Outcome | Status | Depends on |
| ---: | --- | --- | --- | --- |
| 1 | [CSA-01 / PF-42-S01](../../../sprints/archive/claude-subscription-auth/pf-42-s01-auth-source-contract.md) | Typed source, selection, health, and persistence | completed | none |
| 2 | [CSA-02 / PF-43-S01](pf-43-s01-managed-token-lifecycle.md) | Encrypted managed-token lifecycle | in progress | PF-42-S01 |

## Machine checks

```bash
python3 docs/plans/check.py
python3 docs/sprints/check.py
```
