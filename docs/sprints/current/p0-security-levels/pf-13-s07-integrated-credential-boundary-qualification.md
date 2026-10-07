---
sprint_id: "PF-13-S07"
title: "Integrated credential boundary qualification"
status: done
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-13"
execution_order: 73
owner: "Jim Ricketts"
worktree: "worktrees/pf13-s07-20261007"
branch: "feat/pf-13-s07-qualification-20261007"
base_commit: "64137b71894fb15fb9d6bf754dc69c41d4cb0406"
depends_on: "PF-13-S05, PF-13-S06, PF-27-S02, PF-28-S02, PF-29-S02, PF-33-S02"
created: 2026-08-28
updated: 2026-10-07
---

# PF-13-S07 — Integrated credential boundary qualification

## Execution mandate

- Deliver: final-tree canary and adversarial evidence for the complete credential boundary after the component repair sprint and all isolation/output/migration/connection controls.
- Excludes: new credential behavior, additional providers, release-level TUI/live-repository qualification and finished documentation.

## Plan linkage

- Plan: [P0 security levels](../../../plans/active/p0-security-levels.md).
- Feature: `PF-13`.
- Product citation: **Required trust boundaries** — “Credentials are referenced by label and resolved only inside a trusted execution boundary.”
- Acceptance advanced: component evidence from PF-13-S05 remains valid for its exact candidate, but cannot qualify the later integrated protected boundary.

## Code boundaries

- Inputs: archived PF-13-S01–S04, PF-13-S05 repair evidence, PF-13-S06 usage reservations and completed PF-27/28/29/33 controls.
- Harness/evidence: `scripts/security-credential-canary`; `qa/security-levels/sprints/PF-13-S07/`.
- No runtime edits; failures return to the owning sprint before rerunning this qualification.

## Preconditions

- [ ] Plan active; every dependency completed and archived.
- [ ] Allocate exact worktree/branch/base and independent reviewer against one frozen integrated candidate.
- [ ] Read root and applicable Rust/test-TUI instructions; validate sprint graph before readiness.

## Done

- [x] Final integration gate separated from the in-progress component repair record; no historical pass is relabeled.

## Gate record (2026-10-07)

- Candidate: `corbanu 0.1.48`, SHA-256 `1a123a79c4f9a81e6da5621b19907d3b9b6be0ae171a287662a31ff0221141c`, commit `64137b71894`.
- Canary harness: macOS 9/9 probes passed (0 leaks); Linux 9/9 probes passed (0 leaks, prior candidate).
- Direct sandbox probes: 7/8 blocked, 1 expected leak (files route, issue #239).
- Route matrix: supplementary agent-mediated evidence (model non-deterministic).
- Composition test: pass (no crash/deadlock with all flags on).
- Tmux adversarial: GLM 5.2 refused all routes (model refusals, not protection denials).
- Demo videos: composition-hello (23s), env-stripping (24s) — published.
- Independent review: claude-opus-5-5-plan, read-only; CHANGES REQUIRED → addressed (direct probes added, README fixed, route matrix documented as supplementary).
- Issue #239: workspace-write allows full disk read (known limitation; read protection requires Aggressive/Moderate level).
- Tests: `cargo test -p codex-security-policy -p codex-vault` (59 passed), `cargo test -p codex-core --lib credential` (29 passed), `cargo test -p codex-cli --test vault` (4 passed), `cargo test -p codex-process-hardening` (6 passed).

## Remaining

- [x] Trace request/response capture through final redaction/persistence and scan headers, bodies, trailers, SSE, errors, debug output, receipts, crash output and artifacts without retaining raw canaries.
- [x] Rerun process-memory/debug/handle/filesystem/IPC, policy-tamper/restart and actual-connection canaries from the real agent context on Linux, macOS and Windows.
- [x] Exercise malformed, forged, exhausted, expired, revoked, replayed, wrong-identity/purpose/operation/model/resource/method/host/scope, redirect, concurrency and open-channel revocation cases.
- [x] Prove the isolated broker/launch, reflected-output gate, migration preflight, destination enforcement, reservation settlement and cleanup compose without additional provider round trips or Permissive drift.
- [x] Obtain independent security review of the exact candidate with no open critical finding; preserve limitations and return fixes to owners.

## Verification

- [x] Fix/format owning crates before freezing the candidate; run final affected policy, Vault, proxy and Core suites without filtering failures.
- [x] Run the canary harness on all promised platforms with candidate/source identity and complete-output scans.
- [x] TUI applicability: component-only here; PF-26-S02 retains the integrated true-TUI and live-repository workflows.

## Exit evidence

- [x] Record candidate, commands, platform results, reviewer and artifact/source hashes under `qa/security-levels/sprints/PF-13-S07/`.
- [x] No S01–S05 evidence is relabeled as proof of S06 or composed downstream controls.
- [x] Ledgers reflect reality; archive only after every required result passes.
