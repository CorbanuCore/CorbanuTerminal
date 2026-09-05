---
sprint_id: "PF-20-S03"
title: "Local controller integrity root"
status: in_progress
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-20"
execution_order: 78
owner: "/root/provenance"
parallel_lane: "local-controller-anchor"
write_scope: "codex-rs/protected-state/, codex-rs/core/src/security/authoritative_state.rs, codex-rs/core/src/security/authoritative_state_anchor.rs, codex-rs/core/src/security/authoritative_state_anchor_tests.rs, qa/security-levels/sprints/PF-20-S03/, docs/sprints/current/p0-security-levels/pf-20-s03-local-controller-integrity-root.md"
integration_gate: "Codex /root owns shared Core exports and workspace Cargo/Bazel/locks, audits PF20 leaf dependency DAG and PF41 root-last versus PF20 anchor-first adapters, and reruns affected config/security-audit/Core/native synthetic tests plus actual-key TMUX on RTX. PF27 owns privileged launcher/containment composition; no protected activation from unprivileged fixtures. Astra High and Fable5.1 High via Corbanu TMUX, max five reviews on this new PF20 track."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/security-local-anchor"
branch: "feat/security-local-anchor"
base_commit: "601602fa7e53fcb5b41753a0b3607addd45d4415"
depends_on: "PF-20-S02, PF-41-S03, PF-27-S03"
created: 2026-09-04
updated: 2026-09-04
---

# PF-20-S03 — Local controller integrity root

## Execution mandate

- Deliver: a Linux local-controller durable checkpoint implementation and narrow adapters to existing policy/audit anchor contracts, protecting against restored agent-accessible data.
- Excludes: whole-machine snapshot resistance, TPM/off-host services, real Vault migration, protected activation, PF27 process-isolation qualification, policy/event protocol redesign and other platforms.

## Plan linkage

- Plan: [P0 security levels](../../../plans/active/p0-security-levels.md), feature PF-20.
- Product citation: **Non-negotiable controls** — “Record tamper-evident policy decisions, tool calls, approvals, signatures, and transaction or order IDs without secrets.”
- Product decision: **Data-rollback scope decision — 2026-09-04** permits local controller authority under trusted kernel/administrator/controller-store assumptions; whole-machine rollback is outside scope.
- [Design](../../../../qa/security-levels/planning/parallel-handoffs-2026-09-04-round-5/protected-audit-root-design.md) supplies the construction proposal, subject to that resolved threat model and this exact allocation.
- S02 and PF41S03 remain completed foundations, not native implementation proof. PF27S04 consumes this independent root; it is not a dependency of this sprint.

## Code boundaries

- New leaf crate protected-state may depend on existing audit/config/policy and crypto/OS crates, never Core or Vault.
- Existing Core private AuthoritativeStateAnchorStore receives a narrow wrapper; shared security/mod.rs registration remains root-owned.
- PF41 IntegrityRootStore is consumed unchanged. No new ambiguity variant or second policy journal without root allocation.
- Broker service/platform probes remain exclusively PF27-owned. No worker-visible arbitrary path/UID/file capability constructor.
- Read Rust/Core policies and path-types skill before implementing path-bearing types.

## Preconditions

- [x] Active plan; PF20S02, PF41S03 and PF27S03 archived.
- [x] User authorized rolling allocations and resolved the local-data rollback threat model.
- [x] Exact new worktree/base/branch reserved; PF30S01 returned to draft with frozen handoff, without claiming completion.
- [x] Inspect actual interfaces and required nested policies; verify leaf DAG before requesting shared registration (coordinator registration `695704d8e`).

## Done

- [x] Read-only architecture identifies enrollment, exact durable CAS, narrow capability bootstrap, ambiguity and missing-native-provider requirements.

## Remaining

- [x] Implement bounded, owner/key/namespace-bound authenticated checkpoints using existing config/journal types where appropriate; no self-selected authority.
- [x] Explicit one-time enrollment and authenticated Genesis; normal startup opens existing only. Lost/corrupt registry, head or key denies. Interrupted enrollment requires human reconciliation.
- [x] Exact full-value CAS, checked monotone successor, process-wide exclusive ownership, no-follow directory-relative operations, sync file then atomic publication then directory sync before success.
- [x] Fail closed on stale/foreign/overflow/torn state and unsupported Linux filesystem/kernel operations. Post-publication ambiguity latches unavailable; no blind retry or fallback to an older root.
- [x] Build narrow authenticated native client/service construction with actual post-exec peer and generation binding; root-ready witness is not full ProtectedModeAuthorization. Production entry stays gated; coordinator accepted root-anchor/distinct-child composition only as a staged design, not deployment approval.
- [x] Reuse PF41 journal root-last recovery and PF20 policy anchor-first recovery; implement thin adapters without changing their authority/persistence semantics.
- [x] Prove restored/deleted agent data is rejected against intact controller state; document whole-controller snapshot rollback as excluded rather than a passing protection test (unprivileged real PF41 consumer and synthetic Core adapter tests, not privileged deployment).
- [x] Test enrollment interruption, restart, competing processes, wrong identity, short writes/fsync/ENOSPC/ack loss, stale channels and inherited-socketpair negative control with synthetic data.
- [x] Produce exact PF27 consumer handoff and proposed privileged qualification requirements. No sudo, principal/ACL/service setup or real credentials under this allocation. Broker and coordinator accepted the seam; revised privileged installation manifest remains unapproved.

## Verification

- [x] RTX only under shared build flock: scoped just fix, full just fmt, focused protected-state/config/security-audit/Core adapter suites; root serializes Cargo/Bazel changes. Final leaf17/Core17/audit46/config229 passed; Cargo/Bazel parity passed. Any subsequent source change must rerun affected gates.
- [x] Use unprivileged real subprocess tests plus fault injection; never claim separate-principal isolation from same-user fixtures.
- [x] No new UI: supporting actual-key TMUX /security, /status, cancel/exit/restart retained visible unavailable protected state. Final source `a0825d720`; immutable CLI SHA256 `449488d50c3f240ff0bee857f865577a3269f488f2742a94f37513498e2fd1c1`; eleven captures and exact commands in PF20S03 QA.
- [ ] Astra High and Fable5.1 High via Corbanu/private TMUX; max five for this new track. No review-budget reset of frozen PF30S01 or PF27.
- [ ] Coordinator combined-tree tests and lock/governance checks; native privileged qualification remains PF27 integration evidence.

## Exit evidence

- [ ] Exact implementation commit, tests, fault outcomes, keys/captures and review dispositions under QA.
- [ ] Separate algorithm/native transport proof from missing principal-isolation/production activation proof. No platform, human or release acceptance implied.
- [ ] Record TensorCash/Isometric applicability: no repository-specific behavior in this leaf; final release qualification in both remains required.
- [ ] Done/Remaining reflect reality; archive only this bounded dependency after required evidence and root integration, not the composed protected workflow.
