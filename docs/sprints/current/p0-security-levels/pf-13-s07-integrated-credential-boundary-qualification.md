---
sprint_id: "PF-13-S07"
title: "Integrated credential boundary qualification"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-13"
execution_order: 73
owner: "Jim Ricketts"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-13-S05, PF-13-S06, PF-27-S02, PF-28-S02, PF-29-S02, PF-33-S02"
created: 2026-08-28
updated: 2026-08-28
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

## Remaining

- [ ] Trace request/response capture through final redaction/persistence and scan headers, bodies, trailers, SSE, errors, debug output, receipts, crash output and artifacts without retaining raw canaries.
- [ ] Rerun process-memory/debug/handle/filesystem/IPC, policy-tamper/restart and actual-connection canaries from the real agent context on Linux, macOS and Windows.
- [ ] Exercise malformed, forged, exhausted, expired, revoked, replayed, wrong-identity/purpose/operation/model/resource/method/host/scope, redirect, concurrency and open-channel revocation cases.
- [ ] Prove the isolated broker/launch, reflected-output gate, migration preflight, destination enforcement, reservation settlement and cleanup compose without additional provider round trips or Permissive drift.
- [ ] Obtain independent security review of the exact candidate with no open critical finding; preserve limitations and return fixes to owners.

## Verification

- [ ] Fix/format owning crates before freezing the candidate; run final affected policy, Vault, proxy and Core suites without filtering failures.
- [ ] Run the canary harness on all promised platforms with candidate/source identity and complete-output scans.
- [ ] TUI applicability: component-only here; PF-26-S02 retains the integrated true-TUI and live-repository workflows.

## Exit evidence

- [ ] Record candidate, commands, platform results, reviewer and artifact/source hashes under `qa/security-levels/sprints/PF-13-S07/`.
- [ ] No S01–S05 evidence is relabeled as proof of S06 or composed downstream controls.
- [ ] Ledgers reflect reality; archive only after every required result passes.
