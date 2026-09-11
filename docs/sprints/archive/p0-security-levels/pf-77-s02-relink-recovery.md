---
sprint_id: PF-77-S02
title: Multiple-account relink recovery
status: completed
plan_file: docs/plans/active/p0-security-levels.md
plan_feature: PF-77
execution_order: 82
owner: Codex account recovery owner
parallel_lane: account-recovery
write_scope: codex-rs/tasknode-session/, codex-rs/tui/src/chatwidget/tasknode_menu.rs, codex-rs/tui/src/chatwidget/snapshots/, codex-rs/cli/src/tasknode_cmd.rs, docs/features/tasknode.md
integration_gate: Focused session and CLI tests plus actual TUI relink and restart
worktree: /home/pfrpc/repos/worktrees/corbanu-release-0.1.39
branch: fix/tasknode-agent-profile-scope
base_commit: 7e99c5a06bdb7b0f01f655cb4d84739b4f70bb86
depends_on: none
created: 2026-09-08
updated: 2026-09-10
---

# PF-77-S02 — Multiple-account relink recovery

Renumbered from P0 PF-42-S02; see the [identity reconciliation](../../identity-reconciliation-2026-09-10.md). Historical evidence and completion state are retained.

## Execution mandate

Restore named-profile relinking and current Corbanu authentication guidance.

Product initiative repairing the existing authorization boundary. Product:
**Shipping MVP — LIVE**, **Task Node and identity**, "linked identity".
User authority: September 8 report requesting multiple accounts on one machine.

## Plan linkage

Active [P0 plan](../../../plans/active/p0-security-levels.md), feature PF-77.

## Code boundaries

Shared Task Node session resolver, TUI/CLI adapters and focused regression tests.

## Preconditions

- [x] Active plan, concrete user defect and exact worktree recorded.
- [x] Prior PF-77 implementation is preserved; remaining observation returned
  to draft to serialize this repair. Existing unrelated checker failures persist.

## Done

- [x] Confirmed installed 0.1.40 honors a named profile without a config file.
- [x] Reproduced server rejection of its saved token and the pending-link
  starvation in the actual TUI. Other profile status succeeds.
- [x] Located obsolete PFTerminal guidance and absent GitHub account selection.

- [x] Repair completed relink resolution while preserving profile isolation.
- [x] Cover stale active tokens, pending/cancel/failure, concurrent checks,
  distinct accounts, default scope and restart.
- [x] Replace obsolete active authentication guidance.
- [x] Run formatting, affected tests and actual TUI keys on the final candidate.

- [x] Delivery and remaining owner authentication recorded in the QA report.

## Remaining

None for implementation. The owner must complete GitHub sign-in for the revoked account.


## Verification

- [x] Final affected tests and actual TUI recovery/restart checks pass.

## Exit evidence

- [x] Source `f20a2a7389`, local installation and live server delivery recorded.
- [x] Installed PTY matrix passed; owner GitHub sign-in remains a live dependency.
- [x] Sprint archived with accurate implementation and qualification evidence.

Private evidence: `/mnt/HC_Volume_101713660/pfrpc/scratch/corbanu-multi-account-20260908`.
Application repository coding benchmarks do not exercise this authentication
boundary. Real TUI account checks and local HTTP authentication fixtures do.
No full public release or new model benchmark is included in this repair.

Evidence: [account recovery](../../../../qa/reliability/2026-09-08-corbanu-account-recovery.md).
