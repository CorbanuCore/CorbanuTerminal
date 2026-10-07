---
sprint_id: "PF-25-S01"
title: "Temporary grant TUI"
status: completed
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-25"
execution_order: 44
owner: "tui lane worker (round 8, 2026-10-07)"
parallel_lane: "tui"
write_scope: "codex-rs/core/src/security/grant_offer.rs, codex-rs/core/src/security/grant_offer_tests.rs, codex-rs/core/src/security/aggressive.rs, codex-rs/core/src/security/mod.rs, codex-rs/core/src/lib.rs, codex-rs/core/src/tools/orchestrator.rs, codex-rs/app-server-client/src/lib.rs, codex-rs/tui/src/security/grant_view.rs, codex-rs/tui/src/security/grant_view_tests.rs, codex-rs/tui/src/security/mod.rs, codex-rs/tui/src/bottom_pane/approval_overlay.rs, codex-rs/tui/src/bottom_pane/approval_overlay_grant_tests.rs, codex-rs/tui/src/bottom_pane/security_level_picker.rs, codex-rs/tui/src/bottom_pane/snapshots/, qa/security-levels/sprints/PF-25-S01/, qa/demos/specs/, qa/demos/index/PF-25-S01.md"
integration_gate: "Per-sprint gate of 2026-10-06 behind security_levels; grants are offered only under a live Aggressive policy."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/pf-25-s01-20261007"
branch: "feat/pf-25-s01-grant-tui"
base_commit: "e4d17dbdc6f0d6a4c460d9005fd7cc73aebc2566"
depends_on: "PF-17-S01, PF-23-S02, PF-24-S02"
merged_behind_flag: "security_levels"
gate_evidence: "qa/security-levels/sprints/PF-25-S01/README.md"
created: 2026-08-24
updated: 2026-10-07
---

# PF-25-S01 — Temporary grant TUI

## Execution mandate

- Deliver: Aggressive users can inspect and confirm one narrow, expiring grant on a trusted surface.
- Excludes: kill switch, revocation management, arbitrary policy editing, financial signing adapters, and release qualification.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md)
- Feature: `PF-25`
- Reconciliation: [source decisions and archive mapping](../../../plans/security-source-reconciliation.md).
- Product citation: **P0 `/security` levels** — “Existing approval, sandbox, vault, wallet, tool, network, and agent policies are unchanged.”
- Acceptance advanced: the exact actor, action, resource, destination, limits, and expiry are visible before human confirmation.

## Code boundaries

- Existing: `tui/src/bottom_pane/approval_overlay.rs`; `security-policy/src/grant.rs`
- Planned: `tui/src/security/grant_view.rs`; typed Core grant event
- Tests: sibling behavior tests, Core event tests, and reviewed snapshots

## Preconditions

- [x] PF-17-S01, PF-23-S02 archived; PF-24-S02 merged (#253) and archived with its GLM pass.
- [x] Read root, Rust, Core, TUI, and TUI style instructions.
- [x] Worktree coordinates above.

## Done

Gate: [qa/security-levels/sprints/PF-25-S01/README.md](../../../../qa/security-levels/sprints/PF-25-S01/README.md).

- [x] Sprint record is linked only to PF-25.
- [x] Typed, secret-free request: while Core waits for the approval of one command under a live Aggressive policy it
  records an offer (`core/src/security/grant_offer.rs`): actor chain, session, action, resource, command, folder,
  digest, expiry. It is data, never authority, ends when the approval is answered, and at most 8 are open per
  session (rate limit). A confirmed grant for an operation already held is refused (deduplication).
- [x] Grant review (`tui/src/security/grant_view.rs`), opened with `g` from the command approval: every field above,
  the limit (1 run, or `u` for any run until it expires; 10 minutes), and what stays denied. Enter grants and
  approves the command; Esc goes back to the approval with nothing granted; a refusal is shown and grants nothing.
- [x] Only the review's Enter calls `security_grant::confirm`; no `Op`, app-server method, tool or model output
  reaches it. Core issues only an open offer equal to the one shown, under the same policy epoch, with the
  session's human principal as issuer and the asking session's actor chain (a descendant's grant reaches only it).
- [x] `/security` lists the grants held now with their command, runs left and expiry.
- [x] Applied only when the approval is approved: `confirm` records the choice on the offer; the orchestrator applies
  it ("1 run" for that run, never held; "until it expires" held) under the epoch it was confirmed in. The review opens
  on "Back" (no double-Enter grant), cannot grant when cut off, and shows every field escaped.
- [x] Tests: Core unit (one run, until expiry, no offer off Aggressive, unapproved approval, forged offer, state
  changes, depth-4 descendant, rate limit, shared ids, nonce-scoped guards, escaping) and end to end
  (`core/tests/suite/pf_25_s01.rs`); TUI (option rules, review snapshots, Esc through the pane, double Enter, clipped,
  dismissed, Ctrl+C, success with Core's offer, escaped folder, only the review calls `confirm`).

## Remaining

Nothing in this sprint (PR #260). Handed to PF-25-S02: revoking held grants and the kill switch.
- Decision: grants stay in memory only (PF-23-S02's design: a restart ends them), so nothing is persisted; the sprint
  text's "persist the signed grant record" is not done, on purpose.
- Not offered: patches (their approval has no grant option yet) and typing into an unconfined process (still
  refused under Aggressive; start a new process). Commands an automatic reviewer answers never show the option.

## Verification

- [x] `just fix -p codex-core -p codex-tui -p codex-app-server-client`; `just fmt`.
- [x] `just test -p codex-core` (`grant_offer`, `aggressive`, `pf_23_s02`, `transition`); `just test -p codex-tui`
  (`pf_25_s01`, `approval_overlay`, `security`).
- [x] Snapshots reviewed and accepted (PF-25 grant output only).
- [x] GLM 5.2 tmux run and three videos; Linux clippy on the RTX box; Opus 5.5 High review (four rounds, APPROVE).

## Exit evidence

- [x] Commit, snapshots, changed paths, and key script recorded (gate record).
- [x] Test output under `qa/security-levels/sprints/PF-25-S01/`.
- [x] Ledgers reflect reality and the completed record is archived.
