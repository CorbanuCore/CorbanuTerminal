# Unified provider-auth execution sprints

The active [unified provider onboarding and management plan](../../../plans/active/unified-provider-auth.md)
allows one serial sprint. PF-42 through PF-47 are the merged Claude-auth
foundation. PF-48 through PF-53 are completed and archived; PF-54 is the next
unallocated serial sprint.

| Order | Sprint | Outcome | Status | Depends on |
| ---: | --- | --- | --- | --- |
| 7 | [PF-48-S01](../../archive/unified-provider-auth/pf-48-s01-provider-catalog-contract.md) | Typed provider catalog and capability contract | completed | PF-47-S01 |
| 8 | [PF-49-S01](../../archive/unified-provider-auth/pf-49-s01-status-and-eligibility.md) | Shared metadata status and eligibility persistence | completed | PF-48-S01 |
| 9 | [PF-50-S01](../../archive/unified-provider-auth/pf-50-s01-api-key-flow-controller.md) | Shared typed controller and API-key adapter | completed | PF-49-S01 |
| 10 | [PF-51-S01](../../archive/unified-provider-auth/pf-51-s01-openai-account-adapter.md) | OpenAI account adapter | completed | PF-50-S01 |
| 11 | [PF-52-S01](../../archive/unified-provider-auth/pf-52-s01-claude-auth-adapter.md) | Merged Claude backend adapter | completed | PF-51-S01 |
| 12 | [PF-53-S01](../../archive/unified-provider-auth/pf-53-s01-multi-provider-onboarding.md) | Configure-many onboarding and deferred Corbanu | completed | PF-52-S01 |
| 13 | [PF-54-S01](pf-54-s01-provider-management.md) | `/providers` eligibility management | draft | PF-53-S01 |
| 14 | [PF-55-S01](pf-55-s01-startup-provider-convergence.md) | Startup, current-model, and custom-provider convergence | draft | PF-54-S01 |
| 15 | [PF-56-S01](pf-56-s01-final-qualification.md) | Final automated, TMUX, review, documentation, and release evidence | draft | PF-55-S01 |

## Delivery contract

- Implementation owner: GPT-5.6 Sol high implementation agent.
- Accountable integration and feature-completeness owner: Codex primary agent.
- Formal review maximum: four unless a review uncovers and records a major issue.
- External review: one Claude Fable 5 high reviewer spawned and controlled
  through the TMUX harness.
- True-TMUX qualification begins in PF-53 and expands through PF-56.

## Machine checks

```bash
python3 docs/plans/check.py
python3 docs/sprints/check.py
```
