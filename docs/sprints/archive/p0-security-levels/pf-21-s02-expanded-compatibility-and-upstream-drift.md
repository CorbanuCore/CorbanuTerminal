---
sprint_id: "PF-21-S02"
title: "Expanded compatibility and upstream drift"
status: completed
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
- [x] Preserved the accepted baseline manifest byte-for-byte and its pre-feature commit `3c1b2f6cbe11657ff4e3b72b11db029c9e7a92eb`; selected the buildable, genuinely Rust-distinct upstream-aligned 0.1.34 control `af5a4e39b590e7517120fd935ccfac8cbf7cf131` with pinned unequal `codex-rs` tree identities and added a reviewed, fail-closed drift ledger. Candidate output never sources golden expectations.
- [x] Expanded exact executable inventory to inherited environment/auth helpers, web.run history/native search, browser, MCP/plugins/children, wallet, clipboard/export and persisted sessions. Added dispatch-base-anchored exact protected-boundary cases that directly exercise `codex-secret-broker`, `codex-network-proxy`, `codex-browser-isolation`, and `codex-content-security`; no self-asserted behavior string is treated as evidence and no Rust/runtime path changed.
- [x] Recorded baseline/upstream/candidate identities, configuration/environment digests, exact probes and the zero-entry reviewed drift result. Unknown, duplicate, unaccepted, stale, future-dated, candidate-derived, mismatched or unobserved drift fails acceptance; protected-boundary source drift is never ledger-allowlisted.
- [x] Added self-tests for every mandated drift branch plus broad test filters, dirty recipe/runtime masking, fixed-time ledger checks, bounded external artifact roots, cleanup failures, lexical test extraction and ambient secret exclusion; retained the five immutable probes and fail-closed behavior.

## Verification

- [x] Formatted and linted affected Python; `python3 -m unittest scripts.test_security_level_compat -v` passed 37/37. No Rust file or fixture changed.
- [x] The remediated extended harness passed 36/36 against independently built baseline/upstream controls and the clean final implementation candidate; exact CLI arguments, identities and hashes are recorded in the candidate evidence.
- [x] Rust fix/format is not applicable because no Rust path changed.
- [x] TUI applicability is automated-oracle-only; a supporting real-candidate TMUX `/status`/clean-exit smoke passed, while PF-26-S02 still owns both live repositories with actual keys.
- [x] Completed the mandatory read-only Corbanu Terminal + Claude Opus 5 Plan/max independent review in TMUX session `pf27-opus5-g1-review`; the final transcript SHA-256 is `72dd6300b905da2ef3e703e28dc9d038fdb09f36b1678c6d88130e77b6feb86c` and the exact verdict was `NO FINDINGS`.
- [x] Integration owner reran the combined-tree qualification after merging this lane and the prerequisite PF-19/PF-20 lanes; the final report passed 36/36 with a clean source tree.

## Exit evidence

- [x] Recorded implementation commit, changed paths, contract version and exact clean-tree commands/results under `qa/security-levels/sprints/PF-21-S02/`.
- [x] Preserved the S01 archive/evidence unchanged and recorded the remediated three-source plus protected-boundary 36-test report separately from historical qualification.
- [x] Integration owner merged after PF-19/PF-20; updated all four out-of-scope callers with required `--upstream <commit>`; reran combined-tree checks; updated shared navigation; and archived PF-21-S02.

## Remaining

- [x] Closed the integration-owner Exit evidence item above at merge `c02568c71` and caller-update commit `c8ada313d`.
