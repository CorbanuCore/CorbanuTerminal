---
sprint_id: "PF-13-S05"
title: "Credential boundary adversarial qualification"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-13"
execution_order: 13
owner: "Jim Ricketts"
worktree: "/home/pfrpc/repos/CorbanuTerminal-security-levels"
branch: "feat/p0-security-levels"
base_commit: "3c1b2f6cbe11657ff4e3b72b11db029c9e7a92eb"
depends_on: "PF-13-S04"
created: 2026-08-24
updated: 2026-08-24
---

# PF-13-S05 — Credential boundary adversarial qualification

## Execution mandate

- Deliver: final-tree canary and adversarial evidence for the complete PF-13 boundary.
- Excludes: new credential behavior, other providers, `/security` TUI, live-repository release QA, and finished docs.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md)
- Feature: `PF-13`
- Acceptance advanced: authorized use succeeds while every unauthorized or observable surface remains secret-free.

## Code boundaries

- Existing: PF-13 S01-S04 implementation and tests
- Planned: `scripts/security-credential-canary`; `qa/security-levels/sprints/PF-13-S05/`
- Test surfaces: Core context/tool events, child env, proxy capture, tracing, audit, errors, receipts, and artifacts

## Preconditions

- [ ] PF-13-S04 is completed and archived.
- [ ] A named independent security reviewer is recorded before acceptance.
- [ ] Exact worktree coordinates match the active plan.

## Done

- [x] Sprint record defines one bounded qualification outcome.

## Remaining

- [ ] Generate a unique canary and exercise one authorized OpenAI request without printing or persisting the value.
- [ ] Scan exact outgoing request capture, model context, tool payloads, child environment, logs, audit, errors, receipts, crash output, and artifacts.
- [ ] Exercise malformed, forged, expired, revoked, replayed, wrong-actor/purpose/operation/method/host/scope, redirect, concurrent-use, and revocation-race cases.
- [ ] Prove bounded-store cleanup and absence of additional provider network round trips.
- [ ] Run Linux locally and attach CI evidence for macOS and Windows without weakening host checks.
- [ ] Obtain independent security review of raw-secret reachability and record findings/corrections.

## Verification

- [ ] Fix and format all affected crates before the final run; inspect the final diff.
- [ ] Final affected tests: `cd codex-rs && just test -p codex-security-policy && just test -p codex-vault && just test -p codex-network-proxy && just test -p codex-core`.
- [ ] Canary: `python3 scripts/security-credential-canary --candidate <binary> --output qa/security-levels/sprints/PF-13-S05/`.
- [ ] Reviewer, commit, commands, platform results, and canary digest are recorded.
- [ ] TUI applicability: none; PF-26-S02 consumes this boundary in true-TUI workflows.

## Exit evidence

- [ ] Final candidate commit and artifact manifest recorded.
- [ ] Canary absent from every unauthorized/model-visible surface.
- [ ] Independent security review passes with no open P0 finding.
- [ ] Ledgers reflect reality and the completed record is archived.
