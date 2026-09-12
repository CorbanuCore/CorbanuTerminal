---
sprint_id: "PF-82-S01"
title: "Expose existing Team Context in Corbanu"
status: completed
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-82"
execution_order: 100
owner: "Codex Team Context integration"
parallel_lane: "tasknode-team-context"
write_scope: "codex-rs/skills/src/assets/samples/tasknode-usage/, codex-rs/cli/src/tasknode_cmd.rs, codex-rs/tui/src/chatwidget/tasknode_menu.rs, codex-rs/tui/src/chatwidget/tasknode_menu/, codex-rs/tui/src/chatwidget/slash_dispatch.rs, codex-rs/tui/src/app/event_dispatch.rs, codex-rs/tui/src/app_event.rs, docs/features/tasknode.md, qa/reliability/team-context-2026-09-11/, docs/sprints/current/p0-security-levels/pf-82-s01-team-context.md"
integration_gate: "Codex audits the read-only bridge and existing grant enforcement, runs combined CLI/TUI and backend checks, then rebuilds and verifies the installed debug launcher."
worktree: "/mnt/HC_Volume_101713660/pfrpc/scratch/corbanu-wallet-launch-hotfix"
branch: "fix/tasknode-team-context"
base_commit: "7ae35557657b2f2d1c51a8f4af4e4bb1180fe5eb"
depends_on: "none"
created: 2026-09-11
updated: 2026-09-11
---

# PF-82-S01 — Team Context terminal parity

Identity reconciliation, September 11: released as PF-76-S01 at `5f3a0ad7d`.
Renamed mechanically to PF-82-S01 to avoid main's pre-existing PF-76 profile
persistence record. Original implementation and qualification are unchanged.

## Execution mandate

Expose the existing authorized Team Context report through a read-only terminal bridge, `tasknode team context --json`, and `/tasknode team` with a menu entry. Preserve profile scope and grant checks. No private-doc sharing, permission changes, inference changes or release replacement.

## Plan linkage

[Active plan PF-82](../../../plans/active/p0-security-levels.md#pf-82--team-context-terminal-parity).

## Code boundaries

CLI tasknode_cmd.rs, TUI tasknode_menu child module and event/slash adapters, server tasknode-terminal-routes.js and route-policies.js. New route smoke and typed report snapshot tests.

## Preconditions

- [x] Active plan, no unfinished dependencies, exact worktree and disjoint allocation checked.

## Done

- [x] User authority, active plan, independent scope and exact worktree recorded.


- [x] Add the bridge using the existing collaboration route and authenticated terminal account.
- [x] Add CLI and TUI report access, explicit states and refresh.
- [x] Verify auth, grant revocation, CLI JSON, TUI snapshots and scope rejection.
- [x] Run final formatting, tests, build and keyboard workflows in both default disposable repositories.
- [x] Deploy the server route and verify `corbanu-debug --yolo`; update docs and evidence.


## Remaining

None.

## Verification

- [x] Backend route and Team Context smoke; `just test -p codex-cli` focused helper tests; `just test -p codex-tui` focused tasknode tests. Final true PTY on the candidate, including failure/recovery, cancellation and restart.

## Exit evidence

- [x] Record source commits, exact commands, live backend evidence, installed binary digest, PTY keys/screens and any remaining limitations in `qa/reliability/team-context-2026-09-11/`. Archive after all Remaining items are verified.

Implementation: `721d6f974d`; server `39e051c`, live release 737. [Final evidence](../../../../qa/reliability/team-context-2026-09-11/README.md) includes 24 Rust tests, backend permission checks and true PTY workflows in both default repositories.
