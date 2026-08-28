---
sprint_id: "PF-26-S02"
title: "True-TUI and live-repository qualification"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-26"
execution_order: 62
owner: "Jim Ricketts"
worktree: "/home/pfrpc/repos/CorbanuTerminal-security-levels"
branch: "feat/p0-security-levels"
base_commit: "3c1b2f6cbe11657ff4e3b72b11db029c9e7a92eb"
depends_on: "PF-26-S01"
created: 2026-08-24
updated: 2026-08-28
---

# PF-26-S02 — True-TUI and live-repository qualification

## Execution mandate

- Deliver: final-candidate true-TUI evidence with actual keys in disposable TensorCash and Isometric Game worktrees.
- Excludes: code changes beyond discovered fixes, finished docs, human sign-off, and release decision.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md)
- Feature: `PF-26`
- Reconciliation: [source decisions and archive mapping](../../../plans/security-source-reconciliation.md).
- Product citation: **P0 `/security` levels** — “Existing approval, sandbox, vault, wallet, tool, network, and agent policies are unchanged.”
- Acceptance advanced: success, cancel/failure, recovery, and resume work end to end; Corbanu exec is not accepted proof.

## Code boundaries

- Evidence: `qa/release/<version>/security/tui/`
- Inputs: exact disposable TensorCash and Isometric Game paths/base commits
- Tooling: repository `test-tui` skill, PTY, actual key sends, isolated `RUST_LOG=trace`

## Preconditions

- [ ] PF-26-S01 is completed and archived.
- [ ] Final formatted candidate binary/version/commit is recorded.
- [ ] Both disposable live-repository worktrees are resolved and safe for chaotic edits.

## Done

- [x] Sprint record is linked only to PF-26.

## Remaining

- [ ] Exercise full-plan true-TUI flows: migration cancel/failure/recovery, broker failure, screened web outage, quarantine, download promotion, origin-bound login/human challenge, derived-data export, exact financial sign/broadcast, Sweep and inspector/audit.
- [ ] Use fake credentials/venues only; forced detector misses must still deny extraction and unauthorized actions. Include multi-turn poisoned summary/memory, child handoff and restart with revoked authority.
- [ ] Run applicable flows in both disposable live repositories with actual keys; text and Enter are separate inputs, and artifact capture must not contain secrets or sensitive human-auth keystrokes.

- [ ] In TensorCash, prove Permissive compatibility and Moderate hostile-content/protected-action success, denial, cancel, and recovery.
- [ ] In Isometric Game, prove Aggressive default denial, one narrow grant, child inheritance, expiry, revocation, kill switch, downgrade, restart, and resume.
- [ ] Send command text and Enter separately; capture actual keys, visible checkpoints, terminal dimensions, logs, and outcome.
- [ ] Exercise the PF-13 credential boundary through a user-visible Moderate workflow without exposing the canary.
- [ ] If a failure requires code changes, return to the owning feature sprint, reformat/retest, rebuild, and rerun all affected final-candidate flows.

## Verification

- [ ] TensorCash success, failure/cancel, and recovery artifacts pass.
- [ ] Isometric Game success, failure/cancel, and restart/resume artifacts pass.
- [ ] Candidate binary hash matches the automated-evidence candidate.
- [ ] Exact-key scripts and screenshots/logs contain no secret or protected financial value.
- [ ] TUI applicability: required and satisfied only by these true PTY runs.

## Exit evidence

- [ ] Candidate, repositories, base commits, keys, checkpoints, and outcomes recorded.
- [ ] Artifacts linked under `qa/release/<version>/security/tui/`.
- [ ] No flow relies on `corbanu exec`, mocks, snapshots, or smoke tests as its proof.
- [ ] Ledgers reflect reality and the completed record is archived.
