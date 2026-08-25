---
sprint_id: "PF-15-S01"
title: "Security-level domain foundation reconciliation"
status: completed
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-15"
execution_order: 1
owner: "Jim Ricketts"
worktree: "/Users/travisgood/Documents/ChatGPT/corbanu-security-levels"
branch: "feat/p0-security-levels"
base_commit: "7cc15ae0762664d6d01765de407329887da9f876"
depends_on: "none"
created: 2026-08-24
updated: 2026-08-25
---

# PF-15-S01 — Security-level domain foundation reconciliation

## Execution mandate

- Deliver: verify and accept the existing typed security-level domain as the plan foundation.
- Excludes: authorization decisions, runtime enforcement, persistence, and TUI behavior.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md)
- Feature: `PF-15`
- Acceptance advanced: Permissive, Moderate, and Aggressive have one bounded typed representation.

## Code boundaries

- Existing: `codex-rs/security-policy/src/{level,bounded,lib}.rs`
- Build: `codex-rs/{Cargo.toml,Cargo.lock}`; `codex-rs/security-policy/{Cargo.toml,BUILD.bazel}`
- Tests: `codex-rs/security-policy/src/security_policy_tests.rs`

## Preconditions

- [x] Plan remains active and worktree coordinates match this record.
- [x] Inspect repository-root and `codex-rs/AGENTS.md` before altering Rust.
- [x] Worktree contains commit `a4f178fe15` without unreviewed overlap.

## Record creation

- [x] Sprint record is linked to one plan feature.
- [x] Commit `a4f178fe15` added the level domain, bounded values, crate manifest, Cargo lock entry, and Bazel target.

## Done

- [x] Review the existing diff against PF-15 and remove scope outside the domain foundation.
- [x] Confirm unknown serialized levels fail visibly and legacy absence resolves only through the documented compatibility path.
- [x] Run `just bazel-lock-update` if dependency metadata changes and include lock/build parity in the same commit.
- [x] Record any corrective commit and final changed paths.

## Remaining

- None.

## Verification

- [x] Fix: `cd codex-rs && just fix -p codex-security-policy`.
- [x] Format: `cd codex-rs && just fmt`; then inspect the final diff.
- [x] Focused final-tree test: `cd codex-rs && just test -p codex-security-policy level`.
- [x] Build parity: Cargo and Bazel compile the same public crate surface.
- [x] TUI applicability: none; this sprint changes no interactive surface.

## Exit evidence

- [x] Implementation and corrective commits recorded.
- [x] Final-tree command output linked under `qa/security-levels/sprints/PF-15-S01/`.
- [x] `Done` and `Remaining` reflect reality.
- [x] Completed record moved to `docs/sprints/archive/p0-security-levels/`.
