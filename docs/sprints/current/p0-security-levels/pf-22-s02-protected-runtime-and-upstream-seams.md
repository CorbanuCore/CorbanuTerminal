---
sprint_id: "PF-22-S02"
title: "Protected runtime integration and upstream seams"
status: in_progress
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-22"
execution_order: 27
owner: "/root/pf22_protected_runtime"
parallel_lane: "protected-runtime"
write_scope: "codex-rs/core/Cargo.toml, codex-rs/Cargo.lock, codex-rs/core/src/security/effective_policy.rs, codex-rs/core/src/security/effective_policy_tests.rs, codex-rs/core/src/security/protected_runtime.rs, codex-rs/core/src/security/protected_runtime_tests.rs, codex-rs/core/src/security/mod.rs, codex-rs/core/src/agent/control.rs, codex-rs/core/src/agent/control/spawn.rs, codex-rs/core/src/agent/control_tests.rs, qa/security-levels/upstream-seams.json, scripts/security-upstream-seams-check, scripts/security_upstream_seams_check.py, scripts/tests/test_security_upstream_seams.py, qa/security-levels/sprints/PF-22-S02/, docs/sprints/current/p0-security-levels/pf-22-s02-protected-runtime-and-upstream-seams.md"
integration_gate: "The Codex ingress/classifier integration owner audits PF-22-S02 scope and consumed contract versions, runs Core/security-policy/security-audit/seam/compatibility/governance checks on the combined tree, performs TMUX and Opus 5 Max closure, integrates and archives PF-22 before rebasing or registering PF-27's Core broker seam, then releases the sprint slot for PF-30-S01."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/p0-security-protected-runtime"
branch: "feat/p0-security-protected-runtime"
base_commit: "43d2d86488d5c1b2eb5cbc401ee8371dbdb76bf4"
depends_on: "PF-22-S01, PF-19-S02, PF-20-S02, PF-21-S02, PF-41-S03"
created: 2026-08-28
updated: 2026-08-31
---

# PF-22-S02 — Protected runtime integration and upstream seams

## Execution mandate

- Deliver: Integrate new protected state, dispatch fences and durable events through small audited Core hooks.
- Excludes: individual ingress/egress adapters, broker implementation, TUI and release qualification.

## Plan linkage

- Plan: [P0 security levels](../../../plans/active/p0-security-levels.md).
- Feature: `PF-22`.
- Product citation: **Reconciled security scope — TO BUILD** — “Unknown or unsupported protected paths fail visibly rather than falling back to raw secrets or unscreened execution.”
- Acceptance advanced: [architecture refinements](../../../plans/security-architecture-refinements-2026-08-28.md); preserve the completed S01 and its historical evidence, with only added guarantees in this follow-up.

## Code boundaries

- OpenClaw adoption: OC-8 in the [pinned source review](../../../plans/openclaw-source-review-2026-08-28.md), commit `13adff02ca3897768d80d2bca18f5acf08c55d91`; named source tests are references, not candidate passes.

- Existing: `codex-rs/core/src/security/{mod,effective_policy}.rs`, sibling tests, config/session and `agent/{control,registry}.rs` integration hooks.
- Planned: `qa/security-levels/upstream-seams.json`, `scripts/security-upstream-seams-check`, `scripts/tests/test_security_upstream_seams.py`.
- Use existing policy/inheritance types; adapter-owning sprints extend the seam register and their own tests.

## Preconditions

- [x] Plan active; all dependencies completed and archived.
- [x] Assigned a named execution owner and exact plan-matching worktree/branch/base with a disjoint scope and integration gate.
- [x] Read root and nearest implementation AGENTS.md; run the sprint checker before readiness.

## Done

- [x] Follow-up separated from the accepted upstream foundation; no new implementation or qualification claimed.
- [x] Composed PF-19 dispatch/revocation fences, PF-20 authoritative state and PF-41 durable intent/terminal recovery into one fail-closed protected-runtime contract.
- [x] Bound configured, creator-required and effective containment plus owner/policy/run/revocation generations, measured readiness, expiry and recovery status without adding an adapter activation path.
- [x] Added an exact-symbol upstream seam register and mechanical checker; unverified adapter-owned seams remain explicitly pending.
- [x] Bound grant and mandate authority to the exact durable request/approved preview, with negative cross-request substitution coverage.
- [x] Recorded focused, affected, compatibility and real-TMUX evidence under `qa/security-levels/sprints/PF-22-S02/`.

## Remaining

- [ ] Complete the required TMUX/Corbanu/Claude Opus 5 Max read-only review and remediate/rereview any actionable P0/P1/P2 findings.
- [ ] Have the integration owner rerun combined-tree gates, archive the sprint and update shared plan/navigation ledgers.

## Verification

- [x] `cd codex-rs && just fix -p codex-core && just fmt` before final affected tests.
- [x] `cd codex-rs && just test -p codex-core effective_policy && just test -p codex-core security_inheritance` plus focused new integration cases.
- [x] `python3 scripts/security-upstream-seams-check --manifest qa/security-levels/upstream-seams.json` and `python3 -m unittest discover -s scripts/tests -p 'test_security_upstream_seams.py'`; missing hook/owner/commit/command/evidence must fail.
- [x] TUI applicability: no new UI here; PF-24/PF-25/PF-26 retain transition, stop and resume proof. A supporting post-remediation TMUX smoke passed.
- [ ] Run the required read-only Claude Opus 5 Max review through Corbanu Terminal in TMUX after provider authentication is restored.

## Exit evidence

- [x] Record implementation/remediation commits, changed paths, contract versions and exact final-tree commands/results under `qa/security-levels/sprints/PF-22-S02/`.
- [x] Preserve S01 archive/evidence unchanged; do not relabel historical passes as proof of these new cases.
- [x] Record the consumer integration handoff and pending adapter owners in evidence.
- [ ] Complete the Opus review, integration-owner combined-tree matrix and all shared ledgers before archive and plan/navigation updates.
