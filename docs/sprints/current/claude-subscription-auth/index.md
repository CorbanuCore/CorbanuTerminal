# Claude subscription-auth execution sprint

The active [Claude subscription-auth plan](../../../plans/active/claude-subscription-auth.md)
allows one serial sprint. CSA-01 through CSA-03 are completed and archived; no
Claude subscription-auth sprint is currently allocated.

| Order | Sprint | Outcome | Status | Depends on |
| ---: | --- | --- | --- | --- |
| 1 | [CSA-01 / PF-42-S01](../../../sprints/archive/claude-subscription-auth/pf-42-s01-auth-source-contract.md) | Typed source, selection, health, and persistence | completed | none |
| 2 | [CSA-02 / PF-43-S01](../../../sprints/archive/claude-subscription-auth/pf-43-s01-managed-token-lifecycle.md) | Encrypted managed-token lifecycle | completed | PF-42-S01 |
| 3 | [CSA-03 / PF-44-S01](../../../sprints/archive/claude-subscription-auth/pf-44-s01-platform-auth-resolution.md) | Platform-authoritative exact-source resolution | completed | PF-42-S01 |

## Machine checks

```bash
python3 docs/plans/check.py
python3 docs/sprints/check.py
```
