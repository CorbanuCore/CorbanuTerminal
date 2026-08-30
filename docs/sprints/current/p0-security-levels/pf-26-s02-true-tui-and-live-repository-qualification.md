---
sprint_id: "PF-26-S02"
title: "True-TUI and live-repository qualification"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-26"
execution_order: 75
owner: "Jim Ricketts"
lane: "qualification"
write_scope: "qa/release"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-26-S04"
created: 2026-08-24
updated: 2026-08-27
---

# PF-26-S02 — True-TUI and live-repository qualification

## Execution mandate

- Deliver: final-candidate true-TUI evidence with actual keys in disposable TensorCash and Isometric Game worktrees.
- Excludes: runtime fixes, finished docs, human sign-off, and release decision; failures return to the owning implementation sprint.

## Plan linkage

- Upstream: [plan touch record](../../../plans/active/p0-security-levels.md#upstream-touch-record); resolve this sprint's adapter rows.
- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md)
- Feature: `PF-26`
- Acceptance advanced: success, cancel/failure, recovery, and resume work end to end; Corbanu exec is not accepted proof.

## Code boundaries

- Evidence: `qa/release/<version>/security/tui/`
- Inputs: exact disposable TensorCash and Isometric Game paths/base commits
- Tooling: repository `TmuxServer` test support, `test-tui` skill, actual key
  sends, isolated `RUST_LOG=trace`; do not reimplement the driver in shell

## Preconditions

- [ ] Plan upstream baseline, adapter ownership, and exact contract tests are resolved before readiness.
- [ ] Every listed dependency is completed and archived.
- [ ] Final formatted candidate binary/version/commit is recorded.
- [ ] Both disposable live-repository worktrees are resolved and safe for chaotic edits.
- [ ] The typed tmux harness is merged and its required Ubuntu lane is green.

- [ ] Allocate lane/worktree/base in the plan and validate disjoint write scopes before readiness.

## Done

- [x] Sprint record is linked only to PF-26.

## Remaining

- [ ] Run platform/live-repository lanes in isolated state against the same PF-26-S04 candidate; collate one evidence manifest.
- [ ] In both repositories, prove separate browser/content health, hostile acquisition, cancellation, backend-unavailable denial, and isolated recovery/resume.

- [ ] In TensorCash, prove Permissive compatibility and Moderate hostile-content/protected-action success, denial, cancel, and recovery.
- [ ] In Isometric Game, prove Aggressive default denial, one narrow grant, child inheritance, expiry, revocation, kill switch, downgrade, restart, and resume.
- [ ] Send command text and Enter separately; capture actual keys, visible checkpoints, terminal dimensions, logs, and outcome.
- [ ] Exercise the PF-13 credential boundary through a user-visible Moderate workflow without exposing the canary.
- [ ] If a failure requires code changes, return to the owning feature sprint, reformat/retest, rebuild, and rerun all affected final-candidate flows.

## PF-13 tmux qualification contract

- PF-26-S01 supplies a deterministic local capture proxy and canary scanner;
  PF-26-S02 drives them through a Rust TUI integration test named
  `tmux_pf13_moderate_credential_is_brokered_without_disclosure`.
- Start the candidate in a private tmux server with an isolated `CODEX_HOME`,
  temporary vault, unique canary, fixed terminal size, and no live provider.
- Send prompt text and Enter separately. One approved Moderate workflow must
  produce one `POST https://api.openai.com/v1/*` whose transport capture alone
  contains `Authorization: Bearer <canary>`.
- The viewport must show success and a secret-free receipt. A raw
  `vault auth-helper` attempt must be visibly denied under Moderate.
- Scan model context, tool payloads, child environment, viewport, scrollback,
  logs, audit, errors, receipts, crash output, and artifacts for the canary.
- Exit normally and prove the control reader, tmux server, proxy, child
  processes, private socket, and temporary secret material are gone.

## Verification

- [ ] Include remote Linux/tmux detach/reattach and provider-recovery proof with synthetic actions; preserve taint/epochs and verify no duplicate execution without assuming SSH caused provider failure.
- [ ] Record applicable upstream adapter evidence or justified non-applicability; structural checks alone are not qualification.
- [ ] TensorCash success, failure/cancel, and recovery artifacts pass.
- [ ] Isometric Game success, failure/cancel, and restart/resume artifacts pass.
- [ ] Candidate binary hash matches the automated-evidence candidate.
- [ ] Exact-key scripts and screenshots/logs contain no secret or protected financial value.
- [ ] The PF-13 test passes with `CORBANU_TMUX_REQUIRED=1`, zero retries, exactly
  one transport-only canary occurrence, and no forbidden-surface occurrence.
- [ ] TUI applicability: required and satisfied only by these true PTY runs.

## Exit evidence

- [ ] Candidate, repositories, base commits, keys, checkpoints, and outcomes recorded.
- [ ] PF-13 evidence is linked under
  `qa/release/<version>/security/tui/moderate/credential-boundary/`.
- [ ] No flow relies on `corbanu exec`, mocks, snapshots, or smoke tests as its proof.
- [ ] Ledgers reflect reality and the completed record is archived.
