---
sprint_id: "PF-13-S05"
title: "Credential boundary adversarial qualification"
status: in_progress
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-13"
execution_order: 13
owner: "Jim Ricketts"
lane: "qualification"
write_scope: "scripts/security-credential-canary, scripts/test_security_credential_canary.py, qa/security-levels/sprints/PF-13-S05"
worktree: "/Users/travisgood/Documents/ChatGPT/corbanu-pf13-s02"
branch: "feat/pf-13-s02-scoped-vault-resolver"
base_commit: "1bdc515bff48a4d9048dae7d06c6214e884265bc"
depends_on: "PF-13-S04"
created: 2026-08-24
updated: 2026-08-28
---

# PF-13-S05 — Credential boundary adversarial qualification

## Execution mandate

- Deliver: final-tree canary and adversarial evidence for the complete PF-13 boundary.
- Excludes: new credential behavior, other providers, `/security` TUI, live-repository release QA, and finished docs.

## Plan linkage

- Upstream: [plan touch record](../../../plans/active/p0-security-levels.md#upstream-touch-record); resolve this sprint's adapter rows.
- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md)
- Feature: `PF-13`
- Acceptance advanced: authorized use succeeds while every unauthorized or observable surface remains secret-free.

## Code boundaries

- Existing: PF-13 S01-S04 implementation and tests
- Planned: `scripts/security-credential-canary`; `qa/security-levels/sprints/PF-13-S05/`
- Test surfaces: Core context/tool events, child env, proxy capture, tracing, audit, errors, receipts, and artifacts

## Preconditions

- [x] PF-13-S04 is completed and archived.
- [x] Travis replaced the interrupted Fable High review with Kimi 3.0 through Corbanu Terminal (`moonshotai/kimi-k3`, High); no fallback is authorized.
- [x] Exact worktree coordinates match the active plan.

## Done

- [x] Sprint record defines one bounded qualification outcome.
- [x] Generated a unique canary and exercised one authorized in-process header injection without printing or persisting the value; no live provider request was measured.
- [x] Scanned test-constructed model/tool/environment, header, log/audit/receipt/error and artifact surfaces; production panic-hook output and truncated captures remain gaps.
- [x] Exercised malformed, forged, expired, revoked, replayed, wrong-actor/purpose/operation/method/host/scope, redirect, concurrent-use, and revocation-race cases.
- [x] Proved bounded-store cleanup and denial before repeat resolution in component tests; no provider network round trips were measured.
- [x] Ran Linux locally and attached passing commit-bound CI evidence for macOS and Windows without weakening host checks.
- [x] Ran the approved complete Core suite on macOS and preserved its failing JUnit report without claiming a pass.
- [x] Re-ran the complete PF-13 credential qualification on Windows 2022 at the clean current branch tip and committed its machine-readable report.
- [x] Merged all prior work at `044491b8b` before the Fable High outside review; recorded its provider-triggered interruption in `qa/security-levels/sprints/PF-13-S05/fable-outside-review.md` without accepting the automatic Opus substitution.
- [x] Completed Kimi 3.0 High review of `044491b8b`; preserved raw findings and controller dispositions in `qa/security-levels/sprints/PF-13-S05/kimi-outside-review.md`. Qualification remains not ready.

## Remaining

- [ ] Resolve accepted review findings, including output-truncation scanning C1 and production panic-hook proof C2; preserve PF-23 ownership of native profile integration.
- [ ] Triage the 135 complete-Core failures and record a clean full rerun; all 13 credential-named tests already pass.
- [ ] Repeat affected tests and platform canaries against the final integrated candidate; historical results are not relabeled as merge qualification.

Mac failure triage and qualification repairs may run concurrently within this
sprint. Pin each result to its candidate; runtime fixes return to an authorized
implementation sprint and require affected reruns.

## Verification

- [ ] Record applicable upstream adapter evidence or justified non-applicability; structural checks alone are not qualification.
- [x] Fix and format all affected crates before the final run; inspect the final diff.
- [ ] Final affected tests: `cd codex-rs && just test -p codex-security-policy && just test -p codex-vault && just test -p codex-network-proxy && just test -p codex-core`.
- [x] Canary: `python3 scripts/security-credential-canary --candidate <binary> --output qa/security-levels/sprints/PF-13-S05/`.
- [x] Reviewer, reviewed commit, commands and historical platform/canary identities are recorded without claiming a final integrated pass.
- [x] TUI applicability: none; PF-26-S02 consumes this boundary in true-TUI workflows.

## Exit evidence

- [ ] Final integrated candidate commit and artifact manifest recorded after review repairs.
- [ ] Canary absent from every required unauthorized/model-visible surface, including complete output and production panic-hook coverage.
- [ ] Independent security review passes with no open P0 finding.
- [ ] Ledgers reflect reality and the completed record is archived.
