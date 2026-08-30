---
sprint_id: "PF-21-S02"
title: "Expanded compatibility and upstream drift"
status: in_progress
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-21"
execution_order: 26
owner: "Codex compatibility/drift lane"
parallel_lane: "compatibility-drift"
write_scope: "scripts/security-level-compat, scripts/security_level_compat.py, scripts/test_security_level_compat.py, qa/security-levels/compatibility/, qa/security-levels/sprints/PF-21-S02/, docs/sprints/current/p0-security-levels/pf-21-s02-expanded-compatibility-and-upstream-drift.md"
integration_gate: "The Codex ingress/classifier integration lane receives the scripts-and-evidence-only PF-21-S02 candidate, verifies the immutable permissive-baseline-v1.json is byte-identical and no Rust/runtime path changed, merges after the PF-19/PF-20 candidates, reruns the compatibility harness plus governance checkers, and archives PF-21-S02 before PF-22-S02 allocation."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/p0-security-compatibility-drift"
branch: "feat/p0-security-compatibility-drift"
base_commit: "5521b681fff0ecb50b17c10bc1dd1356cbecc1b6"
depends_on: "PF-21-S01"
created: 2026-08-28
updated: 2026-08-30
---

# PF-21-S02 — Expanded compatibility and upstream drift

## Execution mandate

- Deliver: Extend independent compatibility coverage without rewriting the accepted pre-feature baseline.
- Excludes: protected-mode implementation, user-visible workflow changes and release-level TUI qualification.

## Plan linkage

- Plan: [P0 security levels](../../../plans/active/p0-security-levels.md).
- Feature: `PF-21`.
- Product citation: **Reconciled security scope — TO BUILD** — “Unknown or unsupported protected paths fail visibly rather than falling back to raw secrets or unscreened execution.”
- Acceptance advanced: [architecture refinements](../../../plans/security-architecture-refinements-2026-08-28.md); preserve the completed S01 and its historical evidence, with only added guarantees in this follow-up.

## Code boundaries

- Existing: `scripts/security_level_compat.py`, `scripts/security-level-compat`, `scripts/test_security_level_compat.py`.
- Immutable oracle: `qa/security-levels/permissive-baseline-v1.json`; new independent controls, inventory and drift ledger under `qa/security-levels/compatibility/`.
- Any discovered Core/Vault regression becomes evidence for a later separately
  scoped repair; this lane does not edit Rust runtime paths.

## Preconditions

- [x] Plan active; all dependencies completed and archived.
- [x] Assign a named execution owner and exact plan-matching worktree/branch/base; reserve disjoint scopes and integration gate if parallel.
- [x] Read root and nearest implementation AGENTS.md; run the sprint checker before readiness.

## Done

- [x] Follow-up separated from the accepted upstream foundation; no new implementation or qualification claimed.
- [x] Read the repository and Rust workspace policies, exact product requirement, active plan, sprint contract and compatibility-lane handoff; validated the allocated branch with both governance checkers before implementation.

## Remaining

- [ ] Preserve the accepted baseline manifest byte-for-byte and its pre-feature commit `3c1b2f6cbe11657ff4e3b72b11db029c9e7a92eb`. Add an independently built upstream-aligned control and reviewed drift ledger; same-candidate on/off comparisons are supplemental, never a source of golden expectations.
- [ ] Expand executable inventory to inherited environment/auth helpers, web.run history/native search, browser, MCP/plugins/children, wallet, clipboard/export and persisted sessions. Added broker, screening, secretless launch and migration must remain opt-in above Permissive.
- [ ] Record baseline/upstream/candidate commits, config and environment digests, exact probes and owner-reviewed intentional upstream differences. Unknown or unexplained drift fails acceptance rather than regenerating expected results.
- [ ] Add harness self-tests for missing surfaces, mismatched control identities, candidate-derived expectations, stale evidence and expanded case failures; retain the original executable probes and failure behavior.

## Verification

- [ ] Format affected Python/Rust before final tests; `python3 -m unittest scripts.test_security_level_compat -v`.
- [ ] Run the extended compatibility harness against independently built baseline/upstream controls and the final candidate; record exact supported CLI arguments and hashes.
- [ ] If Rust fixtures change: `cd codex-rs && just fix -p codex-core && just fix -p codex-vault && just fmt`, then affected `just test` suites.
- [ ] TUI applicability: automated oracle only here; PF-26-S02 still requires both live repositories with actual keys.

## Exit evidence

- [ ] Record implementation commit, changed paths, contract version and exact final-tree commands/results under `qa/security-levels/sprints/PF-21-S02/`.
- [ ] Preserve S01 archive/evidence unchanged; do not relabel historical passes as proof of these new cases.
- [ ] Record consumer integration handoff; complete all ledgers before archive and update plan/navigation.
