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
- [x] Scanned test-constructed model/tool/environment, header, log/audit/receipt/error and artifact surfaces; the original production-hook and truncated-capture gaps are closed by the repair evidence below.
- [x] Exercised malformed, forged, expired, revoked, replayed, wrong-actor/purpose/operation/method/host/scope, redirect, concurrent-use, and revocation-race cases.
- [x] Proved bounded-store cleanup and denial before repeat resolution in component tests; no provider network round trips were measured.
- [x] Ran Linux locally and attached passing commit-bound CI evidence for macOS and Windows without weakening host checks.
- [x] Ran the approved complete Core suite on macOS and preserved its failing JUnit report without claiming a pass.
- [x] Recorded historical pre-repair PF-13 credential qualification on Windows 2022 and committed its machine-readable report; it does not qualify repaired candidate `f6ec1c75f`.
- [x] Merged all prior work at `044491b8b` before the Fable High outside review; recorded its provider-triggered interruption in `qa/security-levels/sprints/PF-13-S05/fable-outside-review.md` without accepting the automatic Opus substitution.
- [x] Completed Kimi 3.0 High review of `044491b8b`; preserved raw findings and controller dispositions in `qa/security-levels/sprints/PF-13-S05/kimi-outside-review.md`. Qualification remains not ready.
- [x] C1: scan complete stdout/stderr before capture limits, deny overflow/incomplete capture, and pass all ten Python regressions.
- [x] C2: contain scoped callback panic output with Vault's permanent guarded hook and thread-local scope; production logging/terminal hooks, nested/concurrent/unscoped and recovery proof passed on macOS.
- [x] F4/F6: zeroizing scoped header temporary, sensitive final header and corrected label comment; 295 Vault/proxy/policy tests passed with legacy behavior unchanged.
- [x] Final Kimi K3 High repair review returned no findings; both initial P3 suggestions resolved. Artifacts and final source hashes are in `qa/security-levels/sprints/PF-13-S05/repair-evidence.md`.
- [x] F3: copied-vault/canonical-home and both-home-variable symlink CLI tests passed on Mac/Linux; custom homes remain supported and parent-policy composition stays in PF-23.
- [x] Restored and identified the production executable after all CLI probes; eleven Python tests and Kimi's artifact-identity review passed.
- [x] Final Mac/Linux canaries each passed all nine groups / 47 tests at clean candidate `f6ec1c75f`; report hashes match the on-disk executables. Final Mac PTY success/cancel/recovery/resume passed on a byte-identical candidate.
- [x] Re-ran complete Core with companion executables: 3,388/3,407 passed, 19 failed; all 13 credential tests passed. Preserved full JUnit and classified every remaining failure without weakening tests.
- [x] Rechecked corrected Windows endpoint `100.111.98.11`: reachable and supplied host fingerprint verified; login rejected. Recorded the authentication blocker in `qa/security-levels/sprints/PF-13-S05/windows-access-2026-08-28.md` without claiming test execution.
- [x] Recorded the later agent-reported SSH/fingerprint success at `100.111.98.12` and independently verified publication through `a9ebfcc2f`, including required candidate `f6ec1c75f`. No Windows test result is inferred.

## Remaining

- [ ] Scope the remaining Core lifecycle/tool/shell/serialization repairs, then obtain a clean complete rerun. Nineteen failures remain; the additional prompt-cache failure passes in isolation but is not removed from the full-run result.
- [ ] On the remote agent's working Windows route, fetch the published candidate, prepare Rust 1.95/Python and required test tools, then run the final canary including the directory-junction posture test. Final source/artifact identity must be recorded; historical results are not relabeled as qualification of the repaired candidate.

Mac failure triage and qualification repairs may run concurrently within this
sprint. This amended implementation mandate authorizes only the repairs above;
additional runtime fixes return to scope review and require affected reruns.

## Verification

- [x] Record applicable upstream adapter evidence: permanent Vault guard, thin native TUI hook checks, unchanged legacy provider path, focused tests and final PTY; verified upstream ancestor is recorded in repair evidence.
- [x] Fix and format all affected crates before the final run; inspect the final diff.
- [ ] Final affected tests: `cd codex-rs && just test -p codex-security-policy && just test -p codex-vault && just test -p codex-network-proxy && just test -p codex-core`.
- [x] Canary: `python3 scripts/security-credential-canary --candidate <binary> --output qa/security-levels/sprints/PF-13-S05/`.
- [x] Reviewer, reviewed commit, commands and historical platform/canary identities are recorded without claiming a final integrated pass.
- [x] TUI: production panic-hook subprocess proof plus final candidate startup/cancel/resume in a PTY; no new UI. PF-26-S02 retains integrated feature/live-repository qualification.

## Exit evidence

- [x] Final integrated candidate `f6ec1c75f` and artifact/source manifests recorded after review repairs; Core and Windows limitations remain explicit.
- [ ] Final all-platform canary proof: Mac/Linux required component surfaces, complete output and production panic hooks passed; Windows remains pending. Native PF-23 wiring is not claimed.
- [x] Independent accepted-repair security review passes with no findings; native integration review remains downstream in PF-23/PF-26.
- [ ] Ledgers reflect reality and the completed record is archived.
