---
sprint_id: "PF-42-S01"
title: "Task Node profile isolation"
status: completed
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-42"
execution_order: 77
owner: "Codex Task Node identity lane"
parallel_lane: "tasknode-identity"
write_scope: "codex-rs/Cargo.lock, codex-rs/cli/src/main.rs, codex-rs/cli/src/tasknode_cmd.rs, codex-rs/tasknode-session/, codex-rs/tui/src/chatwidget/tasknode_menu.rs, docs/features/tasknode.md, qa/reliability/2026-08-31-tasknode-profile-isolation.md, docs/plans/active/p0-security-levels.md, docs/sprints/current/p0-security-levels/index.md, docs/sprints/archive/p0-security-levels/pf-42-s01-tasknode-profile-isolation.md"
integration_gate: "Product authority receives commit 4cb0a21352331999f6292db89dcfc5d702fc2759 on fix/tasknode-profile-isolation; the integration owner audits scope, reruns the session plus focused CLI/TUI tests and true-PTY profile flows, then includes it only through a normal qualified release."
worktree: "/home/pfrpc/repos/CorbanuTerminal"
branch: "fix/tasknode-profile-isolation"
base_commit: "5e681b388e4d19d0bdd49d07c08803591bad31e5"
depends_on: "none"
created: 2026-08-31
updated: 2026-08-31
---

# PF-42-S01 — Task Node profile isolation

## Execution mandate

- Deliver: bind all Task Node active, pending, request, chat, status, and logout
  authority to the selected Corbanu profile in both TUI and CLI.
- Excludes: Task Node server changes, new identity providers, wallet/provider
  credentials, release publication, and recovery of a token already overwritten
  before this fix.

## Plan linkage

- Plan: [P0 security levels](../../../plans/active/p0-security-levels.md#pf-42).
- Feature: `PF-42`.
- Product citation: **Shipping MVP — LIVE** — “Task Node and identity: Tasks,
  evidence, verification, rewards, balances, chat, context, linked identity,
  and live Task Node-linked Nostr identity.”
- Decision: the CEO explicitly classified the reported cross-profile authority
  bleed as P0 and directed immediate repair on 2026-08-31.
- Acceptance advanced: multiple named users can safely link Task Node on one
  machine without sharing a bearer token or pending/logout state.

## Code boundaries

- Session store: `codex-rs/tasknode-session/` and its Cargo dependency edge.
- Callers: `codex-rs/tui/src/chatwidget/tasknode_menu.rs`,
  `codex-rs/cli/src/tasknode_cmd.rs`, and root CLI profile propagation in
  `codex-rs/cli/src/main.rs`.
- Evidence/docs: `docs/features/tasknode.md` and
  `qa/reliability/2026-08-31-tasknode-profile-isolation.md`.

## Preconditions

- [x] Existing active P0 security plan authorized the repair as PF-42 under
  explicit product authority without consuming a third plan slot.
- [x] Worktree, branch, and base commit are exact and recorded in the plan.
- [x] Runtime write paths are disjoint from all concurrently in-progress sprint
  scopes; the Corbanu API wallet snapshot directory was not changed.
- [x] Product contract, root policy, Rust/TUI instructions, Corbanu development
  skill, and true-TUI skill were applied.

## Done

- [x] Added deterministic per-profile encrypted-vault namespaces while
  preserving legacy unprofiled wrappers.
- [x] Threaded the selected validated profile through every TUI and CLI Task
  Node auth operation, including link promotion, requests, chat, and logout.
- [x] Made the Task Node menu show the active Corbanu profile.
- [x] Added conservative legacy migration: exact case-insensitive identity
  match imports once; mismatch never returns authority.
- [x] Added generalized two-profile, pending-state, logout, mismatch, migration,
  CLI parsing, root-profile gate, and rendered-menu regressions.
- [x] Updated finished user guidance and recorded redacted PTY evidence.

## Remaining

- None.

## Verification

- [x] `cargo clippy -p codex-tasknode-session --tests -- -D warnings` passed.
- [x] `cargo test -p codex-tasknode-session` passed 13/13.
- [x] Focused root and Task Node CLI profile tests passed 1/1 each.
- [x] Focused TUI profile render test passed 1/1; existing unrelated dead-code
  warnings remain disclosed.
- [x] `cargo check -p codex-cli -p codex-tui` passed before final review.
- [x] Root-level `-p goodalexander tasknode link status --json` returned the
  selected profile with no credential value.
- [x] True PTY launched both reported profile names, sent `/tasknode` text and
  Enter separately, displayed each profile scope, and never displayed or
  authorized the other identity.
- [x] `git diff --check` passed; no `.snap.new` artifact remains.
- [x] `python3 docs/plans/check.py` passed at two active plans. The sprint
  checker parsed PF-42-S01 without a PF-42 finding but remains globally red on
  pre-existing duplicate IDs, dependency resolution, and Corbanu API allocation
  errors outside this sprint's scope; the exact output is disclosed in QA.

## Exit evidence

- [x] Implementation commit:
  `4cb0a21352331999f6292db89dcfc5d702fc2759`.
- [x] Final-tree commands, PTY inputs, visible checkpoints, expected migration,
  and limitations are recorded in the linked QA artifact.
- [x] Upstream touch is limited to Corbanu-owned Task Node adapters; unprofiled
  compatibility wrappers and baseline are explicitly tested.
- [x] No network action, credential disclosure, push, install, or release was
  performed as part of qualification.
- [x] `Done` and `Remaining` ledgers reflect reality.
- [x] Completed record is under `docs/sprints/archive/p0-security-levels/`.
