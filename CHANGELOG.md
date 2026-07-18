# PFTerminal 0.1.15

## Added

- Added a first-party Kimi Code K3 provider for Kimi Code plans, including credential setup,
  onboarding, model selection, vault status, and doctor diagnostics.
- Added Kimi-specific action-turn completion handling so the model can continue after tool use
  without silently ending an unfinished turn.

## Fixed

- Centralized hierarchy role-graph validation across native tool, app-server v1/v2, and internal
  spawn paths, closing routes that could create invalid supervisors or bypass worker limits.
- Stale Nazgul root bindings now fail closed, surface a visible recovery message, and persist the
  corrected binding instead of silently rerouting work.
- Dispatch correction prompts are now confined to orchestration threads and bounded by consecutive
  failures, preventing unrelated panes from receiving hierarchy protocol instructions.
- Restored saved native crews without corrupting root-role metadata, and stopped unpinned workers
  from silently inheriting a supervisor's reasoning effort.
- Improved chat-stream finish-reason handling and removed hard-coded workflow/persona material from
  the default role prompts.

## Qualification status

- Provider/model, Kimi completion, hierarchy role, v1/v2 spawn validation, stale-binding,
  correction-bound, and native-crew restoration tests passed locally.
- Pull-request packaging, dependency, spelling, and blob-policy checks passed for both merged PRs.
- The real-TUI matrix did not complete: its runner lacked `rg` after the product binary built
  successfully. This release does not claim a completed real-TUI matrix or a fully green workspace
  suite.

Previous release: 0.1.14.

The changelog can be found on the [releases page](https://github.com/agtico/PfTerminal/releases).
