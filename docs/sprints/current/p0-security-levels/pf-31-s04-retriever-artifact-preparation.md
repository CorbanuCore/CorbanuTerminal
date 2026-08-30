---
sprint_id: "PF-31-S04"
title: "Retriever artifact and engine preparation"
status: ready
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-31"
execution_order: 17
owner: "Codex browser/retrieval lane"
parallel_lane: "browser-retrieval"
write_scope: "scripts/security-retriever-artifact-check, qa/security-levels/retriever/, qa/security-levels/sprints/PF-31-S04/, docs/sprints/current/p0-security-levels/pf-31-s04-retriever-artifact-preparation.md"
integration_gate: "Jim Ricketts receives the PF-31-S04 candidate at G2, audits the literal scope, reruns manifest/fake-engine/governance checks on the combined tree, then archives the sprint before PF-33-S03 allocation."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/p0-security-browser-retrieval"
branch: "feat/p0-security-browser-retrieval"
base_commit: "ea23dfa38bc4f2cbfe0aceadd6777c3e436a53d4"
depends_on: "none"
created: 2026-08-28
updated: 2026-08-28
---

# PF-31-S04 — Retriever artifact and engine preparation

## Execution mandate

- Deliver: Prepare a reproducible pinned retrieval artifact and engine-selection contract without waiting for live broker integration.
- Excludes: protected-mode activation, adjacent feature implementation and Permissive behavior changes.

## Plan linkage

- Plan: [P0 security levels](../../../plans/active/p0-security-levels.md#pf-31).
- Feature: `PF-31`.
- Product citation: **Reconciled security scope — TO BUILD** — “Unknown or unsupported protected paths fail visibly rather than falling back to raw secrets or unscreened execution.”
- Acceptance advanced: [accepted architecture refinements](../../../plans/security-architecture-refinements-2026-08-28.md).
- Source input: [OpenClaw source review](../../../plans/openclaw-source-review-2026-08-28.md) at `13adff02ca3897768d80d2bca18f5acf08c55d91`; reference behavior is not candidate acceptance.

## Code boundaries

- Planned: scripts/security-retriever-artifact-check; qa/security-levels/retriever/{artifact-manifest.json,engine-fixtures}/
- Existing integration paths are read-only until the named consumer sprint; shared manifests/lockfiles require serialized ownership.

## Preconditions

- [x] Plan active; dependencies in front matter are `none`.
- [x] Named execution owner and exact plan-matching worktree/branch/base assigned; governance checkers pass before readiness.
- [x] Root and nearest implementation AGENTS.md read; disjoint write scope and receiving integration gate reserved.

## Done

- [x] Bounded preparation/foundation mandate created from the accepted review; no implementation or platform acceptance claimed.
- [x] Browser/retrieval lane allocated from dispatch base `ea23dfa38bc4f2cbfe0aceadd6777c3e436a53d4` with Jim Ricketts as receiving integration owner.

## Remaining

- [ ] Evaluate current supported Scrapling/runtime/browser dependencies; record exact image/platform digests, source/build lock, signature policy, licenses, SBOM, security review date and rebuild/update policy. No floating latest tag as the installed identity.
- [ ] Specify existing Podman/Docker detection: reuse a supported existing engine without replacing it, prefer Podman when both are equally eligible and no user choice exists; explain unsupported configurations and preserve explicit selection.
- [ ] Define idempotent check/start/restart/pull/test flow with ownership labels and concurrency locking; distinguish Corbanu-owned workers from shared/unrelated engines and containers. Never stop or remove unrelated resources or create duplicate workers on retries.
- [ ] Build manifest validators and fake-engine tests for absent/stopped/stalled/mismatched workers, multiple clients, wrong architecture, tampered images and offline failure; measure image/disk/CPU/RAM needs on all three OSes.
- [ ] Record the human installation/elevation contract and cancellation path without storing credentials; no automatic install/start on a user's machine in this preparation sprint. Final UI/install work and actual containment remain PF-31-S01.

## Verification

- [ ] Run affected format/fix tools before final tests; record exact commands and actual test counts.
- [ ] Run artifact-manifest and engine-fixture tests using the planned validator; record all three platform artifact identities and measured resource requirements.
- [ ] TUI applicability: none for this pure preparation/foundation boundary; user-facing consumer sprints retain true-TUI proof.
- [ ] Verify no runtime route or profile becomes available from fixture-only preparation.

## Exit evidence

- [ ] Commit, contract/fixture versions, owner review and final-tree outputs under `qa/security-levels/sprints/PF-31-S04/`.
- [ ] PF-31-S01 consumes these frozen pins and reruns actual engine/launch/egress/isolation tests; preparation does not qualify protected retrieval.
- [ ] Record integration handoff and scope audit; complete all ledgers before archive and update plan/navigation.
