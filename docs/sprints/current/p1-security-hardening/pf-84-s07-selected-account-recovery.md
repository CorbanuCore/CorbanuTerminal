---
sprint_id: "PF-84-S07"
title: "Selected-account inference and credential recovery"
status: draft
plan_file: "docs/plans/active/p1-security-hardening.md"
plan_feature: "PF-84"
execution_order: 77
owner: "Contributor integration coordinator"
parallel_lane: "windows-host"
write_scope: "codex-rs/core/src/client.rs, codex-rs/core/src/client_anthropic_recovery.rs, codex-rs/core/src/client_anthropic_recovery_tests.rs, codex-rs/core/tests/suite/anthropic_payload_limit.rs, codex-rs/core/tests/suite/client.rs, codex-rs/cli/src/claude_oauth.rs, codex-rs/cli/src/claude_refresh_restore.rs, codex-rs/cli/src/claude_refresh_restore_tests.rs, codex-rs/codex-api/src/api_bridge.rs, codex-rs/codex-api/src/api_bridge_tests.rs, codex-rs/codex-api/src/lib.rs, docs/plans/active/p1-security-hardening.md, docs/sprints/current/p1-security-hardening/pf-84-s07-selected-account-recovery.md, docs/sprints/current/p1-security-hardening/index.md, qa/provider-auth/pf-84/s07/, qa/release/unreleased/notes/88-anthropic-recovery.md"
integration_gate: "Coordinator serializes main PR stages, requires exact independent review and green checks, reruns combined Anthropic and selected-source tests; upstream integrator reconciles adapters on sync only after main acceptance. Functional qualification is separate and not waived."
worktree: "/Volumes/CorbanuDrive/Corbanu/.codex-work/contrib-1111-88"
branch: "maint/contrib-88-1111"
base_commit: "5c7d8ed06095fabf6aa41d496996082c3dfa4384"
depends_on: "PF-84-S06"
created: 2026-11-11
updated: 2026-11-14
---

# PF-84-S07 — Selected-account recovery

## Owner decision update — recorded 2026-11-12

Travis accepted [88-B](https://github.com/CorbanuCore/CorbanuTerminal/pull/88#issuecomment-6810673461):
fail closed and require explicit sign-in for ambiguous damaged/deleted external Claude credentials.
Defer guaranteed restoration, including timeout, until all-writer coordination or identity-safe isolation exists.
Never guess identity, undo logout, overwrite concurrent credentials or switch accounts silently.
Remain `draft` until allocation and both checkers pass; no implementation/acceptance claim.
Opus recovered November 14; functional execution/demos remain outstanding. This draft is not reactivated or qualified.

The [November 11 handoff and Stage A evidence](https://github.com/CorbanuCore/CorbanuTerminal/blob/493f370ccf328840c4e052ceeffcda4e8dd132e9/docs/sprints/current/p1-security-hardening/pf-84-s07-selected-account-recovery.md#integration-handoff--2026-11-11)
remain preserved. PF-23-S04 and this sprint share the real owner and plan/index
write boundary; renaming an owner or omitting shared paths cannot resolve it.

## Execution mandate

Deliver the remaining approved recovery behavior from goodalexander's #88,
adapted to current selected-source and named-account boundaries. Exclude already
merged installers, new account types, flag removal and changes to Keychain ownership.

## Plan linkage

- Plan: [P1 security hardening](../../../plans/active/p1-security-hardening.md#approved-credential-recovery-follow-up-november-11).
- Feature: PF-84; predecessor PF-84-S06 is completed and archived.
- Product: **Shipping MVP — LIVE** — “Anthropic/Claude Plan” and “operational
  credential use without placing raw values in chat”.
- Acceptance: an existing selected account recovers safely without silent fallback.

## Code boundaries

Literal front-matter scope. Keep Core retry buffering in a private sibling module;
thin call-site adapter only. File restoration belongs beside CLI refresh handling;
native Keychain retains existing ownership. Tests use synthetic records only.

## Preconditions

- [x] Active P1 plan carries this owner-approved security follow-up, not a fourth plan.
- [x] PF-84-S06 is completed and archived.
- [x] Exact worktree, branch and base recorded in the plan.
- [x] No implementation scope overlaps other current executable sprint scopes.
- [x] Plan and sprint checkers pass before runtime edits (3 active plans; 102 current/166 archived sprints).

## Done

- [x] Original contribution reviewed; duplicate installer work excluded.
- [x] Existing selected-source, account-binding and zeroizing storage inspected.

## Remaining

- [ ] Stage A: bounded pre-output stream repair including metadata, one retry maximum,
  no retry after visible content/tool output; genuine entitlement versus expired auth.
- [ ] Stage B: fail closed with explicit sign-in for ambiguous corrupt/missing
  external Claude state, including timeout; qualify selected-source recovery
  without guessing identity or restoring ambiguous credentials.
- [ ] Preserve unknown fields, concurrent valid replacements and the approved
  boundary deferring guaranteed automatic restoration.
- [ ] Preserve explicit revocation/source drift and native Keychain isolation.
- [ ] Freeze independent code-blind design; map every case to evidence or accepted disposition.
- [ ] Final formatting, changed-crate and integration tests, exact-head security review.
- [ ] Enforced-isolation independent functional run, true-TUI/demo and live-repository evidence.
- [ ] Upstream sync receiving-boundary reconciliation.

## Verification

- [ ] Python plan and sprint checkers.
- [ ] Isolated codex-api library tests.
- [ ] Focused Core Anthropic and auth-recovery integration tests via just test.
- [ ] Focused CLI selected-store/refresh tests via just test.
- [ ] macOS/Linux/Windows affected CI and synthetic credential fixtures.
- [ ] Independent code-blind schema-2 isolation receipts and evidence review.
- [ ] TUI success/failure/cancel/recovery/resume in applicable live repositories.
- [ ] Demo reel with exact candidate, no credential values.

## Exit evidence

- [ ] Final implementation and combined integration commits.
- [ ] Raw test attempts and fresh replays linked under qa/provider-auth/pf-84/s07/.
- [ ] Case dispositions, remaining prerequisites and named-human acceptance disclosed.
- [ ] Contributor named and actual incorporated changes credited.
- [ ] Sprint archived only after all required evidence exists; source merge is not completion.
