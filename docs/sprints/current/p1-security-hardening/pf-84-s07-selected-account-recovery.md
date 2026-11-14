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
updated: 2026-11-11
---

# PF-84-S07 — Selected-account recovery

## Integration handoff — 2026-11-11

The coordinator returns this reservation to `draft` while serially integrating
PF-23-S04. Both records share one real owner and the plan/index write boundary;
renaming an owner or omitting shared paths would not make them independent.
Stage A is preserved at `0fa06783eda5157ddbbc129ebc82abc9425636f2` in #2115,
with independent review, isolated API242/Core10/Core-integration8 passes, a later
five-case command-fixture replay, and synthetic native-Windows fixture evidence.
Final CI is still pending; no functional acceptance or release is claimed.
Stage B remains blocked on owner decision 88-B, and independent functional
execution needs the requested Opus runtime restored. The original preservation
contract is not waived. Coordinates remain recorded for reallocation; before
further implementation, the integration owner must reactivate this reservation
with disjoint ownership or after the chroot handoff, then run both checkers.
No provider executor is running during this serialized integration.

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
- [ ] Stage B: selected-file refresh recovery for corrupt/missing state, timeout and
  normal failure, preserving unknown fields and concurrent valid replacement.
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
