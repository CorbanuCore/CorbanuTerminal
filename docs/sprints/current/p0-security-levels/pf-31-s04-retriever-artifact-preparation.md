---
sprint_id: "PF-31-S04"
title: "Retriever artifact and engine preparation"
status: blocked
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
updated: 2026-08-29
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
- [x] Pinned Scrapling 0.4.15 source, PyPI artifacts, OCI index/platform/config/provenance digests, exact observed runtime/browser versions, independent per-architecture SBOMs, license findings, signature gap and locked-rebuild policy.
- [x] Frozen side-effect-free existing-engine selection: preserve explicit choice; otherwise prefer eligible Podman over eligible Docker; never install, elevate, change defaults or touch unrelated resources.
- [x] Defined Corbanu ownership labels, one-worker/profile locking, reinspection and human-only pull/start/restart/reconciliation actions with visible no-fallback failures.
- [x] Added 27 fake-engine fixtures covering availability, per-platform identity, integrity, offline, architecture, lock/reinspection, same/cross-engine duplicates, explicit and automatic duplicate prevention, auto precedence, concurrency, explicit choice and unrelated resources; every fixture passes deterministic replay evaluation on macOS, Linux and Windows.
- [x] Measured the exact no-network image on macOS arm64, Linux amd64 and Windows 11 Pro amd64/Linux-container mode with Docker and Podman, generated 5,332/5,323-component SBOMs and recorded conservative preparation resource limits.
- [x] Selected and measured official Podman 5.8.3 client / 5.8.6 server on Windows in an isolated Corbanu-owned WSL machine; no product path installs, elevates, starts or selects an engine.
- [x] Codex GPT-5.5 autoreview finding fixed and regression-tested; clean rerun reports no accepted/actionable findings.
- [x] Completed the corrected-candidate disposition in the mandated Claude UI with Opus 5 and Max visibly selected; initial and follow-up findings, screenshots and fixes are preserved.
- [x] Restored the pinned Windows Tailscale peer, verified its SSH fingerprint and remote bundle SHA-256, then passed the final 27 fixtures, 27 deterministic replays, five manifest mutations and ten evidence-path checks on Windows 11 Pro with Python 3.13.15.

## Remaining

- [ ] Obtain integration-owner decisions for the Corbanu signing identity/rebuild and for the unresolved SBOM license records; do not activate the unsigned floating-input upstream image.
- [ ] Hand the final candidate to Jim Ricketts for scope audit, combined-tree reruns, archive transition and explicit G2 slot rotation before PF-33-S03 starts.

## Verification

- [x] Affected format/fix tools run before tests; exact commands and 27 fixture/27 deterministic-replay counts recorded.
- [x] Corrected artifact-manifest and engine-fixture tests pass on macOS, Linux and Windows; Windows exact-image qualification is retained separately from the corrected pure contract-suite rerun.
- [x] TUI applicability: none for this pure preparation/foundation boundary; user-facing consumer sprints retain true-TUI proof.
- [x] Scope audit proves no runtime route or profile becomes available from fixture-only preparation.
- [ ] Receiving integration owner reruns the final candidate on the combined tree after Claude finding disposition and before archive transition.

## Exit evidence

- [x] Candidate commits, contract/fixture versions, local and Claude review dispositions, cross-host bundle identity and final-tree outputs are recorded under `qa/security-levels/sprints/PF-31-S04/`.
- [ ] PF-31-S01 consumes these frozen pins and reruns actual engine/launch/egress/isolation tests; preparation does not qualify protected retrieval.
- [ ] Scope audit and provisional handoff are recorded; integration owner must resolve blockers, rerun the combined tree and complete/archive the ledger.
