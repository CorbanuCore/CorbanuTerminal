---
sprint_id: "PF-23-S02"
title: "Aggressive deny and grant enforcement"
status: in_progress
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-23"
execution_order: 41
owner: "untrusted-content lane"
parallel_lane: "untrusted-content"
write_scope: "codex-rs/core/src/security/aggressive.rs, codex-rs/core/src/security/aggressive_tests.rs, codex-rs/core/src/security/protected_surface.rs, codex-rs/core/src/security/protected_surface/, codex-rs/core/src/security/tainted_action.rs, codex-rs/core/src/security/tainted_action/, codex-rs/core/src/security/tainted_action_tests.rs, codex-rs/core/src/client.rs, codex-rs/core/src/mcp_openai_file.rs, codex-rs/core/src/tools/orchestrator.rs, codex-rs/core/src/tools/runtimes/unified_exec.rs, codex-rs/core/src/tools/handlers/unified_exec/write_stdin.rs, codex-rs/core/tests/suite/pf_23_s01.rs, codex-rs/sandboxing/src/seatbelt.rs, codex-rs/sandboxing/src/seatbelt_tests.rs, qa/security-levels/sprints/PF-23-S02/, qa/demos/specs/, qa/demos/index/PF-23-S02.md, docs/sprints/current/p0-security-levels/pf-23-s02-aggressive-deny-and-grant-enforcement.md"
integration_gate: "Per-sprint gate of 2026-10-06: focused tests, GLM 5.2 tmux demos, one independent Opus 5.5 High review, Linux clippy on the RTX box; merge behind source_envelopes. Not reserved: codex-rs/core/src/security/mod.rs, codex-rs/core/src/tools/sandboxing.rs, codex-rs/core/src/unified_exec/process_manager.rs, codex-rs/Cargo.lock."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/pf23-s02-20261007"
branch: "feat/pf-23-s02"
base_commit: "64137b7189"
depends_on: "PF-17-S01, PF-23-S01"
created: 2026-08-24
updated: 2026-10-07
---

# PF-23-S02 — Aggressive deny and grant enforcement

## Execution mandate

- Deliver: Aggressive denies every named sensitive surface unless one matching human grant is active.
- Excludes: grant TUI, signing adapters, new tools/providers, downgrade flow, and qualification.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md)
- Feature: `PF-23`
- Reconciliation: [source decisions and archive mapping](../../../plans/security-source-reconciliation.md).
- Product citation: **P0 `/security` levels** — “Existing approval, sandbox, vault, wallet, tool, network, and agent policies are unchanged.”
- Acceptance advanced: one narrow grant cannot authorize an adjacent actor, resource, destination, operation, child, or post-expiry use.

## Code boundaries

- `core/src/security/aggressive.rs`: the one grant enforcement point (`issue` host-only, `admit`, `revoke_all`).
- `core/src/security/protected_surface/read_denials.rs`: protected-path rules (reads denied, persistence read-only).
- `core/src/security/protected_surface/confined.rs`: which processes started inside the rules (`write_stdin`).
- `core/src/tools/orchestrator.rs` (lift by Moderate approval or Aggressive grant), `core/src/mcp_openai_file.rs`
  (sandboxed upload reads), `sandboxing/src/seatbelt.rs` (rename guard), `tainted_action/{paths,shell}.rs` (Persistence).

## Preconditions

- [x] PF-17-S01 and PF-23-S01 are completed and archived.
- [x] Read root, `codex-rs/AGENTS.md`, and `codex-rs/core/AGENTS.md`; worktree recorded above.

## Done

All behind `source_envelopes` with Moderate/Aggressive; flag off and Permissive unchanged.
Gate: [qa/security-levels/sprints/PF-23-S02/gate.md](../../../../qa/security-levels/sprints/PF-23-S02/gate.md).

- [x] Aggressive applies the protected-path rules from the session start; an approval never lifts them, only a
  `BoundedGrant` matching lineage, surface, exact operation, session, expiry, uses and policy epoch. Another level,
  the kill switch or no live policy drops every grant; grants are never inherited (a child has its own lineage and
  session; derived grants keep the parent's context and are refused).
- [x] The rules also make persistence files read-only (shell start-up, login items, `~/.local/bin`, `~/.claude`,
  git hooks/config incl. worktree git folders, project `.codex`/`.agents`, Corbanu home entries that stay readable),
  only where the profile allowed writes; profile denials stay. macOS denies renaming any folder above a protected
  path and removing a protected link.
- [x] Command-text net: writes, moves and removals of those paths (and `git config` run keys, cron, launchd,
  `systemctl --user`) are a new Persistence kind.
- [x] Codex Apps uploads read through the protected sandbox (moved from PF-23-S01); no sandbox, no upload.
- [x] A process started without the rules is recorded at spawn; typing into it after untrusted content asks once
  per start under Moderate and needs a grant under Aggressive (moved from PF-23-S01).

## Remaining

- [ ] Gate round 2 (review, Linux clippy, re-recorded videos), merge, archive.
- Grant issuance UI is PF-25-S01 (`aggressive::issue`, `confined::grant_operation`); revocation UI PF-25-S02.
- Not covered (gate record): broker, retrieval, browser-login and derived-data adapters do not exist yet and must
  call `aggressive::admit` when built; Linux symlinked dotfiles (only the target is protected).

## Verification

- [ ] `just fix -p codex-core`, `just fmt`, focused `just test -p codex-core pf_23_s02 pf_23_s01 pf_30_s03
  mcp_openai_file aggressive confined`, `just test -p codex-sandboxing`, full `just test -p codex-core`.
- [ ] Linux clippy (`-D warnings`) on the RTX box; GLM 5.2 videos; Opus 5.5 High review.

## Exit evidence

- [ ] Commit, denied-surface matrix and changed paths in the gate record; record archived.
