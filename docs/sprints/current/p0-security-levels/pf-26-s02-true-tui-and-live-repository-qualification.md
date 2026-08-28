---
sprint_id: "PF-26-S02"
title: "True-TUI and live-repository qualification"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-26"
execution_order: 72
owner: "Jim Ricketts"
worktree: "/Users/travisgood/Documents/ChatGPT/corbanu-security-levels"
branch: "feat/p0-security-levels"
base_commit: "7cc15ae0762664d6d01765de407329887da9f876"
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
- Tooling: existing Rust `TmuxServer` support and `test-tui` skill, PTY, actual key sends, isolated `RUST_LOG=trace`; do not reimplement the driver in shell.

## Preconditions

- [ ] PF-26-S01 is completed and archived.
- [ ] Final formatted candidate binary/version/commit is recorded.
- [ ] Both disposable live-repository worktrees are resolved and safe for chaotic edits.
- [ ] The typed tmux harness is merged and its required Ubuntu lane is green.
- [ ] Product authority has approved numeric usability targets for the fixed research workflows.

## Done

- [x] Sprint record is linked only to PF-26.

## Remaining

- [ ] Measure both live repositories' Moderate research tasks: completion, prompt/approval counts, blocked false positives, time-to-first-safe-output and end-to-end latency against product-approved targets; fail safely without taint laundering.
- [ ] Repeat typed descendant request/cancel/grant, policy-tamper recovery, engine reuse/stall recovery and submitted-unknown financial kill in the final candidate TUI, then obtain named human acceptance in PF-26-S03.

- [ ] Exercise full-plan true-TUI flows: migration cancel/failure/recovery, broker failure, screened web outage, quarantine, download promotion, origin-bound login/human challenge, derived-data export, exact financial sign/broadcast, Sweep and inspector/audit.
- [ ] Use fake credentials/venues only; forced detector misses must still deny extraction and unauthorized actions. Include multi-turn poisoned summary/memory, child handoff and restart with revoked authority.
- [ ] Run applicable flows in both disposable live repositories with actual keys; text and Enter are separate inputs, and artifact capture must not contain secrets or sensitive human-auth keystrokes.

- [ ] In TensorCash, prove Permissive compatibility and Moderate hostile-content/protected-action success, denial, cancel, and recovery.
- [ ] In Isometric Game, prove Aggressive default denial, one narrow grant, child inheritance, expiry, revocation, kill switch, downgrade, restart, and resume.
- [ ] Send command text and Enter separately; capture actual keys, visible checkpoints, terminal dimensions, logs, and outcome.
- [ ] Exercise the PF-13 credential boundary through a user-visible Moderate workflow without exposing the canary.
- [ ] If a failure requires code changes, return to the owning feature sprint, reformat/retest, rebuild, and rerun all affected final-candidate flows.

## PF-13 tmux qualification contract

- PF-26-S01 supplies the local capture proxy and scanner; drive them with the Rust test `tmux_pf13_moderate_credential_is_brokered_without_disclosure`.
- Start the candidate in private tmux with isolated `CODEX_HOME`, temporary vault, unique canary, fixed dimensions and no live provider.
- Send text and Enter separately. Exactly one approved `POST https://api.openai.com/v1/*` has `Authorization: Bearer <canary>` in transport capture alone.
- Show success and a secret-free receipt; a raw `vault auth-helper` attempt is visibly denied under Moderate.
- Scan model context, tool payloads, child environment, viewport, scrollback, logs, audit, errors, receipts, crash output and artifacts.
- Exit and prove cleanup of the control reader, tmux server, proxy, children, private socket and temporary secret material.

## Verification

- [ ] TensorCash success, failure/cancel, and recovery artifacts pass.
- [ ] Isometric Game success, failure/cancel, and restart/resume artifacts pass.
- [ ] Candidate binary hash matches the automated-evidence candidate.
- [ ] Exact-key scripts and screenshots/logs contain no secret or protected financial value.
- [ ] PF-13 passes with `CORBANU_TMUX_REQUIRED=1`, zero retries, exactly one transport-only canary occurrence and zero forbidden-surface occurrences.
- [ ] TUI applicability: required and satisfied only by these true PTY runs.

## Exit evidence

- [ ] Candidate, repositories, base commits, keys, checkpoints, and outcomes recorded.
- [ ] Artifacts linked under `qa/release/<version>/security/tui/`; PF-13 uses `moderate/credential-boundary/` beneath it.
- [ ] No flow relies on `corbanu exec`, mocks, snapshots, or smoke tests as its proof.
- [ ] Ledgers reflect reality and the completed record is archived.
