---
sprint_id: "PF-13-S05"
title: "Credential boundary adversarial qualification"
status: completed
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-13"
execution_order: 13
owner: "Jim Ricketts"
parallel_lane: "foundation-platform"
integration_gate: "PF-13-S07 consumes the frozen component evidence after remaining repairs; Jim Ricketts owns integration."
write_scope: "scripts/security-credential-canary, scripts/security_credential_canary.py, scripts/test_security_credential_canary.py, codex-rs/vault/src, codex-rs/network-proxy/src/credential_broker.rs, codex-rs/network-proxy/src/credential_broker/providers.rs, codex-rs/network-proxy/src/credential_broker_tests.rs, codex-rs/cli/tests/vault.rs, codex-rs/tui/src/lib.rs, codex-rs/tui/src/tui.rs, codex-rs/tui/src/credential_panic_tests.rs, codex-rs/core/src/security/credential_capability_tests.rs, codex-rs/core/src/agent/control.rs, codex-rs/core/src/agent/control/spawn.rs, codex-rs/core/src/agent/control_tests.rs, codex-rs/core/src/security/effective_policy.rs, codex-rs/core/src/security/effective_policy_tests.rs, codex-rs/core/src/session/handlers.rs, codex-rs/core/src/session/session.rs, codex-rs/core/src/session/tests.rs, codex-rs/core/src/session/turn.rs, codex-rs/core/src/shell_snapshot.rs, codex-rs/core/src/shell_snapshot_tests.rs, codex-rs/core/src/tools/handlers/multi_agents.rs, codex-rs/core/src/tools/handlers/multi_agents_tests.rs, codex-rs/core/src/tools/handlers/multi_agents_v2.rs, codex-rs/core/src/tools/handlers/multi_agents_v2/interrupt_agent.rs, codex-rs/core/src/tools/spec_plan.rs, codex-rs/core/src/tools/spec_plan_tests.rs, codex-rs/core/src/client.rs, codex-rs/core/tests/suite/auto_review.rs, codex-rs/core/tests/suite/client.rs, codex-rs/core/tests/suite/code_mode_elicitation.rs, codex-rs/core/tests/suite/compact.rs, codex-rs/core/tests/suite/multi_agent_resume.rs, codex-rs/core/tests/suite/otel.rs, codex-rs/core/tests/suite/prompt_caching.rs, codex-rs/core/tests/suite/tool_parallelism.rs, codex-rs/protocol/src/models.rs, qa/security-levels/sprints/PF-13-S05"
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/p0-security-foundation-platform"
branch: "feat/p0-security-foundation-platform"
base_commit: "6a35712cd5731b191d875e8c6468f1abe23eb66e"
depends_on: "PF-13-S04"
created: 2026-08-24
updated: 2026-08-30
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
- Integrated-tree repair classification: the transferred lane owns only the 18 named Core failures recorded in `repair-core-triage.md`. The amended `write_scope` enumerates each affected runtime/test path; no Vault, broker, provider, profile, or Permissive contract is widened.
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
- [x] Transferred the sprint explicitly to the CorbanuDrive foundation worktree at `6a35712cd5731b191d875e8c6468f1abe23eb66e`; prior evidence remains historical and is not relabeled.
- [x] Re-ran complete Core on the transferred integrated tree with Rust 1.95 and all four companion executables: 3,393/3,411 passed, 18 failed and 19 skipped. All credential tests passed and the earlier prompt-cache failure passed; the new JUnit is preserved as repair input, not a clean qualification result.
- [x] Scope-reviewed and repaired all 18 transferred-tree failures without weakening the credential boundary or test assertions; a final complete Core rerun passed 3,411/3,411 with 19 platform-filtered skips and no retry/flaky classification. JUnit run `fd5920a2-8b87-4e14-a2b8-a7201aed6304` is preserved as `repair-core-final-macos-junit.xml.gz` (SHA-256 `9eb1c35509c4cd4480f8491ed218b2b59a8e765d39c8fd71fdb8f7381f1f1a7e`).
- [x] Re-ran the affected security crates after the Core repairs: 295/295 passed, zero skipped, run `c7938288-cff5-496f-b802-03d95adf7f19`; `repair-focused-final-macos-junit.xml.gz` SHA-256 `1c2d35ab9d5fc6a82884cac060446050a319f68e1f388c9e71a89b1d1c9c296a`. The canary harness unit suite also passed 11/11.
- [x] Published integrated repair commit `be8153f2e29c360d83776441aed50deb204eafa7`; a clean macOS canary passed 47/47 tests and recorded exact executable/report hashes.
- [x] On Windows 11 AMD64, fetched the same published commit into a clean `D:` checkout, pinned Rust/Cargo 1.95.0 and MSVC, and passed all nine canary groups / 47 tests. The four-test raw-export group executed the unprivileged `mklink /J` directory-junction posture case; independent `Get-FileHash` matched the report's executable SHA-256.
- [x] Compared every path/source hash embedded by the retained Linux report with the integrated macOS report; all probe sources are identical. The Linux result remains honestly bound to `f6ec1c75f` rather than being relabeled after the non-probe Core repairs.
- [x] Completed the immutable repair-chain review through TMUX + Corbanu Terminal + Claude Opus 5.0 Max; the final pass returned no findings after two accepted-finding repair rounds.
- [x] Stabilized two qualification-only fixture-lifetime races at `d021d017b`; telemetry passed 50/50 retry-free stress iterations and isolated Core passed 3,414/3,414 with 19 platform skips and no retries.
- [x] Completed a separate Claude Opus 5.0 Max review of the final stabilization diff with no findings; the raw TMUX captures and hashes are recorded in `qa/security-levels/sprints/PF-13-S05/claude-integrated-repair-review.md`.

## Remaining

None.

The amended mandate authorizes only the repairs above; additional runtime fixes return to scope review and require affected reruns.

## Verification

- [x] Record applicable upstream adapter evidence: permanent Vault guard, thin native TUI hook checks, unchanged legacy provider path, focused tests and final PTY; verified upstream ancestor is recorded in repair evidence.
- [x] Fix and format all affected crates before the final run; inspect the final diff.
- [x] Final affected tests: `cd codex-rs && just test -p codex-security-policy -p codex-vault -p codex-network-proxy --test-threads 4` passed 295/295; `just test -p codex-core --test-threads 4` passed 3,411/3,411 with 19 platform-filtered skips.
- [x] Canary: `python3 scripts/security-credential-canary --candidate <binary> --output qa/security-levels/sprints/PF-13-S05/`.
- [x] Final Windows canary on published commit `be8153f2e29c360d83776441aed50deb204eafa7`: 47/47 passed, including the directory-junction posture case; clean source and independent executable identity matched.
- [x] Claude Opus 5.0 Max review of the integrated Core repair through the mandated TMUX + Corbanu Terminal harness, with immutable results and dispositions.
- [x] Reviewer, reviewed commit, commands and historical platform/canary identities are recorded without claiming a final integrated pass.
- [x] TUI: production panic-hook subprocess proof plus final candidate startup/cancel/resume in a PTY; no new UI. PF-26-S02 retains integrated feature/live-repository qualification.

## Exit evidence

- [x] Published integrated repair candidate `be8153f2e` and artifact/source manifests are recorded; historical `f6ec1c75f` evidence retains its original identity.
- [x] Final all-platform component canary proof: macOS and Windows passed the published integrated candidate; Linux's clean report remains bound to its recorded candidate and has an identical complete probe-source hash set. Native PF-23 wiring is not claimed.
- [x] Independent accepted-repair security review passes with no findings; native integration review remains downstream in PF-23/PF-26.
- [x] Ledgers reflect the clean integrated repair and stabilization reviews; the completed record is archived.
