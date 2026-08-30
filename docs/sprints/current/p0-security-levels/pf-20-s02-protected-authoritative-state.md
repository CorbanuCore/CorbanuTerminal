---
sprint_id: "PF-20-S02"
title: "Protected authoritative-state persistence"
status: ready
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-20"
execution_order: 24
owner: "Codex authoritative-state lane"
parallel_lane: "authoritative-state"
write_scope: "codex-rs/config/src/security_state.rs, codex-rs/config/src/lib.rs, codex-rs/core/src/security/authoritative_state.rs, codex-rs/core/src/security/authoritative_state_tests.rs, codex-rs/core/src/security/mod.rs, qa/security-levels/sprints/PF-20-S02/, docs/sprints/current/p0-security-levels/pf-20-s02-protected-authoritative-state.md"
integration_gate: "The Codex ingress/classifier integration lane receives the PF-20-S02 candidate after PF-19-S02, audits the literal scope, serializes any required generated config schema or shared manifest edits, reruns codex-config plus focused codex-core security/config tests and PF-27-S03 platform probes on the combined tree, archives PF-20-S02, and keeps protected activation unavailable."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/p0-security-authoritative-state"
branch: "feat/p0-security-authoritative-state"
base_commit: "5521b681fff0ecb50b17c10bc1dd1356cbecc1b6"
depends_on: "PF-20-S01, PF-27-S03"
created: 2026-08-28
updated: 2026-08-30
---

# PF-20-S02 — Protected authoritative-state persistence

## Execution mandate

- Deliver: Store authoritative protected policy and recovery state under the reviewed controller identity.
- Excludes: OS mechanism selection, effective runtime integration, event-store implementation and TUI.

## Plan linkage

- Plan: [P0 security levels](../../../plans/active/p0-security-levels.md).
- Feature: `PF-20`.
- Product citation: **Reconciled security scope — TO BUILD** — “Unknown or unsupported protected paths fail visibly rather than falling back to raw secrets or unscreened execution.”
- Acceptance advanced: [architecture refinements](../../../plans/security-architecture-refinements-2026-08-28.md); preserve the completed S01 and its historical evidence, with only added guarantees in this follow-up.

## Code boundaries

- OpenClaw adoption: OC-6 and OC-11 in the [pinned source review](../../../plans/openclaw-source-review-2026-08-28.md), commit `13adff02ca3897768d80d2bca18f5acf08c55d91`; named source tests are references, not candidate passes.

- Planned: `codex-rs/config/src/security_state.rs`; authenticated update adapter and tamper/restart tests in `codex-rs/core/src/security/`.
- Existing config editing/schema paths only as adapters; PF-27-S03 owns OS identities and PF-41-S03 owns durable event records.

## Preconditions

- [x] Plan active; all dependencies completed and archived.
- [x] Assign a named execution owner and exact plan-matching worktree/branch/base; reserve disjoint scopes and integration gate if parallel.
- [ ] Read root and nearest implementation AGENTS.md; run the sprint checker before readiness.

## Done

- [x] Follow-up separated from the accepted upstream foundation; no new implementation or qualification claimed.

## Remaining

- [ ] Implement controller-owned level, grant, revocation/kill generation and recovery state separate from agent-editable preferences. Authenticate mutation callers using the PF-27-S03 contract; do not trust a model-supplied role or ordinary config file as authority.
- [ ] Reject overwrite, deletion, rename, symlink/replacement, permission changes, old snapshots and rollback by the untrusted context, including restart/resume. Distinguish a genuinely new legacy installation from missing state after protected activation; the latter cannot become Permissive.
- [ ] Implement compare-and-activate revision checks and ownership-scoped rollback so stale recovery cannot overwrite a later credential/provenance owner. Test crash boundaries and durable recovery, not just in-memory snapshots.
- [ ] Preserve PF-20-S01 legacy/config/schema behavior and the frozen Permissive oracle; document platform-specific unsupported cases as activation blockers. Hand any required Cargo/Bazel/schema synchronization to the integration owner; those shared surfaces are outside this lane.

## Verification

- [ ] `cd codex-rs && just fix -p codex-config && just fix -p codex-core && just fmt` before final affected tests.
- [ ] `cd codex-rs && just test -p codex-config && just test -p codex-core config:: && just test -p codex-core security::`; generate schema with `just write-config-schema` in `codex-rs` if affected.
- [ ] Run PF-27-S03 tamper probes against this store on every supported OS; record actual identity, crash/restart and expected/actual denial.
- [ ] TUI applicability: none for storage API; PF-24 and PF-26 own human transition/recovery flows.

## Exit evidence

- [ ] Record implementation commit, changed paths, contract version and exact final-tree commands/results under `qa/security-levels/sprints/PF-20-S02/`.
- [ ] Preserve S01 archive/evidence unchanged; do not relabel historical passes as proof of these new cases.
- [ ] Record consumer integration handoff; complete all ledgers before archive and update plan/navigation.
