# Claude subscription-auth execution sprint

The active [Claude subscription-auth plan](../../../plans/active/claude-subscription-auth.md)
allows one serial sprint. CSA-01 through CSA-06 are completed and archived.
Remaining live-account, human, live-repository, and release gates stay on the
active plan.

| Order | Sprint | Outcome | Status | Depends on |
| ---: | --- | --- | --- | --- |
| 1 | [CSA-01 / PF-42-S01](../../../sprints/archive/claude-subscription-auth/pf-42-s01-auth-source-contract.md) | Typed source, selection, health, and persistence | completed | none |
| 2 | [CSA-02 / PF-43-S01](../../../sprints/archive/claude-subscription-auth/pf-43-s01-managed-token-lifecycle.md) | Encrypted managed-token lifecycle | completed | PF-42-S01 |
| 3 | [CSA-03 / PF-44-S01](../../../sprints/archive/claude-subscription-auth/pf-44-s01-platform-auth-resolution.md) | Platform-authoritative exact-source resolution | completed | PF-42-S01 |
| 4 | [CSA-04 / PF-45-S01](../../../sprints/archive/claude-subscription-auth/pf-45-s01-auth-choice-and-recovery.md) | Explicit auth choice, migration, and recovery UX | completed | PF-43-S01, PF-44-S01 |
| 5 | [CSA-05 / PF-46-S01](../../../sprints/archive/claude-subscription-auth/pf-46-s01-final-qualification.md) | Automated final qualification, documentation, and review | completed | PF-45-S01 |
| 6 | [CSA-06 / PF-47-S01](../../../sprints/archive/claude-subscription-auth/pf-47-s01-first-run-anthropic-account.md) | First-run Anthropic-account onboarding | completed | PF-46-S01 |

## Machine checks

```bash
python3 docs/plans/check.py
python3 docs/sprints/check.py
```
