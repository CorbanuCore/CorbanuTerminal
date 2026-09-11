---
sprint_id: "PF-77-S01"
title: "Task Node durable command and transport recovery"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-77"
execution_order: 79
owner: "Task Node reliability integration"
parallel_lane: "tasknode-reliability"
write_scope: "codex-rs/tui/src/bottom_pane/mod.rs, codex-rs/tui/src/bottom_pane/bottom_pane_view.rs, codex-rs/tui/src/bottom_pane/list_selection_view.rs, codex-rs/tui/src/chatwidget/constructor.rs, codex-rs/tui/src/app.rs, codex-rs/tui/src/chatwidget/agent_control.rs, codex-rs/secrets/src/local.rs, codex-rs/tasknode-session/, codex-rs/cli/src/tasknode_cmd.rs, codex-rs/tui/src/chatwidget/tasknode_menu.rs, codex-rs/tui/src/chatwidget/tasknode_recovery.rs, codex-rs/tui/src/chatwidget/tasknode_recovery_tests.rs, codex-rs/tui/src/chatwidget.rs, codex-rs/tui/src/app/event_dispatch.rs, codex-rs/tui/src/app_event.rs, codex-rs/tui/src/chatwidget/snapshots/, codex-rs/Cargo.lock, MODULE.bazel.lock"
integration_gate: "Task Node reliability integration owns sequential adapter edits, focused session/CLI/TUI tests and final PTY recovery proof."
worktree: "/home/pfrpc/repos/CorbanuTerminal"
branch: "fix/tasknode-profile-isolation"
base_commit: "ec549c0c687f50a682487e9d68289c05557ff579"
depends_on: "none"
created: 2026-09-05
updated: 2026-09-10
---

# PF-77-S01 — Task Node durable command and transport recovery

Renumbered from P0 PF-44-S01; see the [identity reconciliation](../../identity-reconciliation-2026-09-10.md). Historical evidence and completion state are retained.

September 8 allocation handoff: implementation and PTY checks are already
recorded below. Return the outstanding production observation to draft while
PF-77-S02 serially repairs the same authentication adapters. Preserve all
completed work and the unchecked observation; this is not completion.

## Execution mandate

- Deliver shared profile/server-bound Task Node transport and recoverable task
  requests across failure, closing a view and restarting Corbanu.
- Excludes inference selection, portfolio behavior and a public versioned release.
- Product authority and excerpt: PF-77 in the linked active plan.

## Plan linkage

- Plan: [P0 security](../../../plans/active/p0-security-levels.md), feature `PF-77`.
- Repeated submission recovers the original account-owned receipt; stale
  profile/view responses cannot replace the current display.

## Code boundaries

- Shared transport, encrypted command persistence and scoped credential cache.
- CLI Task Node adapter, TUI request menus, async fences and control inbox.
- Evidence: `qa/reliability/2026-09-05-tasknode-recovery/`.

## Preconditions

- [x] Active plan and explicit user authorization for reliability work.
- [x] No unfinished dependency; worktree, branch and base match the plan.
- [x] Existing unrelated global checker conflicts recorded separately.

## Done

- [x] Product citation and baseline dirty diff preserved.
- [x] Shared transport, task payload/error semantics and origin/profile binding.
- [x] Encrypted pending keys/text/receipts and reopen/restart recovery.
- [x] Stale async identity/view fencing and request pagination/retry.
- [x] Structured agent inbox with busy/draft protection and duplicate suppression.
- [x] Adjacent-case regression and rendered snapshot coverage.
- [x] `just fix`, `just fmt`, affected tests and final diff inspection.
- [x] Real PTY success/failure, reopen, process restart, retry and control resume.
- [x] Optimized operator binary and matching tool host built and PTY checked.
- [x] Task Node v706 deployed; Kimi resumed its original production thread.
- [x] Real tool calls, scoped API journal receipt and active supervisor verified.
- [x] Finished guidance and exact candidate hashes/evidence updated.

## Remaining

- [ ] Observe the first naturally occurring nonempty scoped production round.

## Verification

- [x] 18 session/cache, seven CLI, five Task Node TUI and two agent-control tests.
- [x] Required real PTY checks under the repository `test-tui` workflow.
- [x] Development and optimized candidate evidence recorded separately.
- [x] Production credential scope, committed journal and supervisor ticks verified.
- [ ] Inspect live per-duty completion/resume when a mandatory duty arrives.

## Exit evidence

- [x] Exact candidate base, working-tree manifest and executable hashes recorded.
- [x] Automated output and PTY keys/checkpoints linked in the QA directory.
- [ ] Archive after the remaining live observation; preserve an accurate ledger.

## Current qualification

Kimi's normal exit and candidate resume preserved thread
`01a07030-4786-72d2-a7e4-87b077c8788f`. The production API reported no mandatory
duties at cutover. The supervisor is active with successful quiet ticks and zero
restarts. Per-duty outcomes and partial resume passed PostgreSQL fixtures; this
is not represented as a live nonempty round. No artificial task or payment was
created for qualification. All implementation and deployment work is complete;
the pending observation keeps this sprint in draft pending reallocation. Public release is excluded.
