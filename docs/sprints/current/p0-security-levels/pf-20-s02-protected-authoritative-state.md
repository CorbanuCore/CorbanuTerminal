---
sprint_id: "PF-20-S02"
title: "Protected authoritative-state persistence"
status: in_progress
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
- [x] Read root and nearest implementation AGENTS.md; run the sprint checker before readiness.

## Done

- [x] Follow-up separated from the accepted upstream foundation; no new implementation or qualification claimed.
- [x] Implement controller-owned level, authority generations, kill state, recovery state, and owner epochs outside agent-editable preferences.
- [x] Bind append-only state, intent, and commit records to a protected external anchor with monotonic compare-and-store semantics.
- [x] Reject rollback, deletion, truncation, symlink replacement, metadata weakening, stale-owner recovery, and forged pending records without weakening active restrictions.
- [x] Keep non-Unix persistence and every PF-27-ineligible host visibly unavailable.

## Remaining

- [ ] Integration owner merges after PF-19, reruns combined-tree gates, archives the sprint, and preserves the explicit activation blocker.

## Verification

- [x] `cd codex-rs && just fix -p codex-config && just fix -p codex-core && just fmt` before final affected tests.
- [x] `cd codex-rs && just test -p codex-config && just test -p codex-core config:: && just test -p codex-core security::`; `just write-config-schema` produced no diff.
- [x] Reconcile against current PF-27-S03 macOS, Linux, and Windows probes: all report the protected store unsupported and protected mode ineligible, so activation remains blocked; no unsupported persistence pass is claimed.
- [x] TUI applicability: storage has no direct UI; an authenticated Corbanu/TMUX smoke passed, while PF-24 and PF-26 retain human transition/recovery proof.
- [ ] Integration-owner combined-tree config/security and governance rerun.

## Exit evidence

- [x] Record implementation commit, changed paths, contract version and exact final-tree commands/results under `qa/security-levels/sprints/PF-20-S02/`.
- [x] Preserve S01 archive/evidence unchanged; do not relabel historical passes as proof of these new cases.
- [x] Record consumer integration handoff; lane-owned ledgers are complete and plan/navigation/archive remain integration-owner-only.
- [ ] Integration owner records the merged candidate and moves this record to the archive with plan/navigation updates.
