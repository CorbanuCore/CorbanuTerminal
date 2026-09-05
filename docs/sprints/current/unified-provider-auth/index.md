# Unified provider-auth execution sprints

The active [unified provider onboarding and management plan](../../../plans/active/unified-provider-auth.md)
allows one serial sprint. PF-42 through PF-47 are the merged Claude-auth
foundation. PF-48 through PF-57 are completed and archived; no implementation
sprint from that sequence remains open. PF-57-S02 also completed the user's
2026-09-04 Astra reconciliation repairs in its isolated serial worktree.
PF-57-S03 is the sole current sprint, reconciling those repairs with Travis's
two subsequent integration-branch commits under the user's 2026-09-05 mandate.

| Order | Sprint | Outcome | Status | Depends on |
| ---: | --- | --- | --- | --- |
| 7 | [PF-48-S01](../../archive/unified-provider-auth/pf-48-s01-provider-catalog-contract.md) | Typed provider catalog and capability contract | completed | PF-47-S01 |
| 8 | [PF-49-S01](../../archive/unified-provider-auth/pf-49-s01-status-and-eligibility.md) | Shared metadata status and eligibility persistence | completed | PF-48-S01 |
| 9 | [PF-50-S01](../../archive/unified-provider-auth/pf-50-s01-api-key-flow-controller.md) | Shared typed controller and API-key adapter | completed | PF-49-S01 |
| 10 | [PF-51-S01](../../archive/unified-provider-auth/pf-51-s01-openai-account-adapter.md) | OpenAI account adapter | completed | PF-50-S01 |
| 11 | [PF-52-S01](../../archive/unified-provider-auth/pf-52-s01-claude-auth-adapter.md) | Merged Claude backend adapter | completed | PF-51-S01 |
| 12 | [PF-53-S01](../../archive/unified-provider-auth/pf-53-s01-multi-provider-onboarding.md) | Configure-many onboarding and deferred Corbanu | completed | PF-52-S01 |
| 13 | [PF-54-S01](../../archive/unified-provider-auth/pf-54-s01-provider-management.md) | `/providers` eligibility management | completed | PF-53-S01 |
| 14 | [PF-55-S01](../../archive/unified-provider-auth/pf-55-s01-startup-provider-convergence.md) | Startup, current-model, and custom-provider convergence | completed | PF-54-S01 |
| 15 | [PF-56-S01](../../archive/unified-provider-auth/pf-56-s01-final-qualification.md) | Final automated, TMUX, review, documentation, and release evidence | completed | PF-55-S01 |
| 16 | [PF-57-S01](../../archive/unified-provider-auth/pf-57-s01-latest-main-integration.md) | Latest-main integration and credential-store liveness | completed | PF-56-S01 |
| 17 | [PF-57-S02](../../archive/unified-provider-auth/pf-57-s02-reconciliation-auth-repairs.md) | Reconciliation credential-lifecycle repairs | completed | PF-57-S01 |
| 18 | [PF-57-S03](pf-57-s03-travis-release-reconciliation.md) | Travis/provider UX and release repair reconciliation | in_progress | PF-57-S02 |

## Delivery contract

- PF-48 through PF-56 implementation owner: GPT-5.6 Sol high implementation agent.
- PF-57 integration and feature-completeness owner: Codex primary integration agent.
- PF-57 preserved archived commit identities by merging rather than rebasing.
- Its recorded review exception completed with Fable 5.1 through Corbanu and
  TMUX after all applicable integration findings were repaired.
- True-TMUX qualification from PF-56 passed on combined candidate `a935e507b`.

## Machine checks

```bash
python3 docs/plans/check.py
python3 docs/sprints/check.py
```
