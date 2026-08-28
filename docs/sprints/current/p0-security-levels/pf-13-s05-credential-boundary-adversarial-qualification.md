---
sprint_id: "PF-13-S05"
title: "Credential boundary adversarial qualification"
status: in_progress
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-13"
execution_order: 13
owner: "Jim Ricketts"
lane: "qualification"
write_scope: "scripts/security-credential-canary, scripts/security_credential_canary.py, scripts/test_security_credential_canary.py, codex-rs/vault/src, codex-rs/network-proxy/src/credential_broker.rs, codex-rs/network-proxy/src/credential_broker/providers.rs, codex-rs/network-proxy/src/credential_broker_tests.rs, codex-rs/cli/tests/vault.rs, codex-rs/tui/src/lib.rs, codex-rs/tui/src/tui.rs, codex-rs/tui/src/credential_panic_tests.rs, codex-rs/core/src/security/credential_capability_tests.rs, qa/security-levels/sprints/PF-13-S05"
worktree: "/Users/travisgood/Documents/ChatGPT/corbanu-pf13-s02"
branch: "feat/pf-13-s02-scoped-vault-resolver"
base_commit: "1bdc515bff48a4d9048dae7d06c6214e884265bc"
depends_on: "PF-13-S04"
created: 2026-08-24
updated: 2026-08-28
---

# PF-13-S05 — Credential boundary adversarial qualification

## Execution mandate

- Deliver: accepted Kimi/controller repairs and final-tree adversarial qualification of the PF-13 component boundary; Travis authorized all accepted fixes after the review checkpoint.
- Excludes: PF-23 native profile wiring, other providers, Permissive policy changes, new `/security` UI, live-repository release QA, and finished docs.

## Plan linkage

- Upstream: [plan touch record](../../../plans/active/p0-security-levels.md#upstream-touch-record); resolve this sprint's adapter rows.
- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md)
- Feature: `PF-13`
- Acceptance advanced: authorized use succeeds while every unauthorized or observable surface remains secret-free.

## Code boundaries

- Existing: PF-13 S01-S04 implementation and tests
- Planned: full-output scanner, Vault-owned scoped panic guard, thin TUI panic-hook adapter, scoped header hardening, alternate-home tests; exact paths are in `write_scope`.
- Kimi repair-review hardening: apply the same scoped guard to the terminal-restoration hook and report the non-secret byte count on capture overflow; neither changes the authorized feature contract.
- Test surfaces: Core context/tool events, child env, proxy capture, tracing, audit, errors, receipts, and artifacts

## Preconditions

- [x] PF-13-S04 is completed and archived.
- [x] Travis replaced the interrupted Fable High review with Kimi 3.0 through Corbanu Terminal (`moonshotai/kimi-k3`, High); no fallback is authorized.
- [x] Exact worktree coordinates match the active plan.
- [x] Review checkpoint committed at `f0a160eee`; plan records the repair authority, verified upstream baseline and literal adapter/test boundaries.

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
- [x] C1: scan complete stdout/stderr before capture limits, deny overflow/incomplete capture, and pass all ten Python regressions.
- [x] C2: contain scoped callback panic output with Vault's permanent guarded hook and thread-local scope; production logging/terminal hooks, nested/concurrent/unscoped and recovery proof passed on macOS.
- [x] F4/F6: zeroizing scoped header temporary, sensitive final header and corrected label comment; 295 Vault/proxy/policy tests passed with legacy behavior unchanged.
- [x] Final Kimi K3 High repair review returned no findings; both initial P3 suggestions resolved. Artifacts and final source hashes are in `qa/security-levels/sprints/PF-13-S05/repair-evidence.md`.

## Remaining

- [ ] F3: copied encrypted vault/canonical home keyring tests pass; run the new native symlink-home CLI tests. Preserve custom-home support and defer parent-policy composition to PF-23.
- [ ] Triage the 135 complete-Core failures and record a clean full rerun; all 13 credential-named tests already pass.
- [ ] Repeat affected tests and platform canaries against the final integrated candidate; historical results are not relabeled as merge qualification.

Mac failure triage and qualification repairs may run concurrently within this
sprint. This amended implementation mandate authorizes only the repairs above;
additional runtime fixes return to scope review and require affected reruns.

## Verification

- [ ] Record applicable upstream adapter evidence or justified non-applicability; structural checks alone are not qualification.
- [x] Fix and format all affected crates before the final run; inspect the final diff.
- [ ] Final affected tests: `cd codex-rs && just test -p codex-security-policy && just test -p codex-vault && just test -p codex-network-proxy && just test -p codex-core`.
- [x] Canary: `python3 scripts/security-credential-canary --candidate <binary> --output qa/security-levels/sprints/PF-13-S05/`.
- [x] Reviewer, reviewed commit, commands and historical platform/canary identities are recorded without claiming a final integrated pass.
- [ ] TUI: production panic-hook subprocess proof plus final candidate startup/cancel/resume in a PTY; no new UI. PF-26-S02 retains integrated feature/live-repository qualification.

## Exit evidence

- [ ] Final integrated candidate commit and artifact manifest recorded after review repairs.
- [ ] Canary absent from every required unauthorized/model-visible surface, including complete output and production panic-hook coverage.
- [ ] Independent security review passes with no open P0 finding.
- [ ] Ledgers reflect reality and the completed record is archived.
