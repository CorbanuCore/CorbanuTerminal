---
sprint_id: "PF-13-S05"
title: "Credential boundary adversarial qualification"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-13"
execution_order: 34
owner: "Jim Ricketts"
worktree: "/Users/travisgood/Documents/ChatGPT/corbanu-security-levels"
branch: "feat/p0-security-levels"
base_commit: "7cc15ae0762664d6d01765de407329887da9f876"
depends_on: "PF-13-S04, PF-27-S02, PF-28-S02, PF-29-S02, PF-33-S02"
created: 2026-08-24
updated: 2026-08-28
---

# PF-13-S05 — Credential boundary adversarial qualification

## Execution mandate

- Deliver: final-tree canary and adversarial evidence for the complete PF-13 boundary.
- Excludes: new credential behavior, other providers, `/security` TUI, live-repository release QA, and finished docs.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md)
- Feature: `PF-13`
- Reconciliation: [source decisions and archive mapping](../../../plans/security-source-reconciliation.md).
- Product citation: **P0 `/security` levels** — “Existing approval, sandbox, vault, wallet, tool, network, and agent policies are unchanged.”
- Acceptance advanced: authorized use succeeds while every unauthorized or observable surface remains secret-free.

## Code boundaries

- Existing: PF-13 S01-S04 implementation and tests
- Planned: `scripts/security-credential-canary`; `qa/security-levels/sprints/PF-13-S05/`
- Test surfaces: Core context/tool events, child env, proxy capture, tracing, audit, errors, receipts, and artifacts

## Preconditions

- [ ] PF-13-S04, PF-27-S02, PF-28-S02, PF-29-S02, PF-33-S02 are completed and archived.
- [ ] A named independent security reviewer is recorded before acceptance.
- [ ] Exact worktree coordinates match the active plan.

## Done

- [x] Sprint record defines one bounded qualification outcome.

## Remaining

- [ ] Trace the concrete request/response capture adapter through final redaction and persistence; scan headers, bodies, trailers, SSE, errors and debug artifacts. Final transport contains the authorized credential, but evidence stores only canary digests/results, never raw capture values.
- [ ] Rerun PF-27-S03 process-memory/debug/handle/filesystem/IPC and PF-20 policy tamper/restart canaries from the actual agent context on all three OSes; design probes or CI platform labels alone are not proof.

- [ ] Qualify the PF-27 isolated process/launch contract, PF-28 reflected-response/output gates, PF-29 migration preflight and PF-33 actual-connection enforcement; the earlier in-process thin slice is not the final secret boundary.

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
