---
sprint_id: "PF-27-S04"
title: "Isolated credential broker process"
status: in_progress
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-27"
execution_order: 28
owner: "/root"
parallel_lane: "isolated-broker"
write_scope: "codex-rs/linux-pidfd-spawn/, codex-rs/protected-state/src/lib.rs, codex-rs/protected-state/src/synthetic_fixture.rs, codex-rs/protected-state/src/native.rs, codex-rs/protected-state/src/native_tests.rs, codex-rs/protected-state/Cargo.toml, codex-rs/secret-broker-service/, codex-rs/secret-broker/, codex-rs/network-proxy/src/credential_broker.rs, codex-rs/network-proxy/src/credential_broker/, codex-rs/network-proxy/src/credential_broker_tests.rs, codex-rs/core/src/security/broker_client.rs, codex-rs/core/src/security/broker_client_tests.rs, codex-rs/core/src/config/network_proxy_credential.rs, codex-rs/core/src/config/network_proxy_credential_tests.rs, codex-rs/vault/src/capability.rs, codex-rs/vault/src/capability_tests.rs, qa/security-levels/sprints/PF-27-S04/, docs/sprints/current/p0-security-levels/pf-27-s04-isolated-credential-broker.md, codex-rs/Cargo.toml, codex-rs/Cargo.lock, MODULE.bazel.lock, docs/plans/active/p0-security-levels.md, docs/sprints/current/p0-security-levels/index.md, qa/security-levels/planning/parallel-handoffs-2026-09-04-round-5/, securityProgress.html"
integration_gate: "Codex /root serializes shared registration, audits scope and runs affected final-tree qualification on RTX plus synthetic lifecycle through TMUX. Six historical reviews spent; Travis granted five additional review slots replenishing every six hours, tracked in PF27 review-budget.md. Use Astra High and Fable 5.1 High Corbanu/TMUX without duplicate reviews. Production/native/all-OS gates remain open; no privileged installation or protected activation. Coordinate main writes with other integration owners."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/security-broker-resume-20260911"
branch: "feat/security-broker-resume-20260911"
base_commit: "d870c92dab2bf3fbb602dc3b8447fe9f3534aecb"
depends_on: "PF-27-S01, PF-13-S04, PF-27-S03, PF-41-S03"
created: 2026-08-28
updated: 2026-09-12
---
# PF-27-S04 — Isolated credential broker process

September 11 resumption: fresh main-based allocation after human acceptance. The earlier missing-checkout report was stale: `security-round5-broker` is clean
at pushed `cd7457da7`. Reconcile its existing service stage; do not redo it.
The [resume handoff](../../../../qa/security-levels/planning/parallel-handoffs-2026-09-04-round-5/RESUME-20260911.md) records exact coordinates, review history and unchanged native setup limits.

## Execution mandate

- Deliver: Raw credentials exist only in the trusted broker; a compromised agent process cannot call an unrestricted resolver.
- Excludes: adjacent feature implementation, Permissive policy changes, and unlisted integrations.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md#pf-27).
- Feature: `PF-27`.
- Product citation: **Non-negotiable controls** — “Permit agents to reference credentials only by label; resolve them solely inside the trusted execution boundary.”
- Acceptance advanced: Raw credentials exist only in the trusted broker; a compromised agent process cannot call an unrestricted resolver.
- Sources and archive disposition: [PF-27 reconciliation](../../../plans/security-source-reconciliation.md#pf-27).

## Code boundaries
- Current narrower mandate: [private root-session pump](../../../../qa/security-levels/sprints/PF-27-S04/descriptor-root-session-allocation-20260912.md), seven exact paths/700target800hard from2c63e4cd5 after accepted combinedeb5855959 proof; fixed bounded acceptance/admission/dispatch only. Reviews37/38 allocated, prior1–36 retained; no native/public bootstrap or broader edits.
- OpenClaw adoption reference: [OC-1](../../../plans/openclaw-source-review-2026-08-28.md#oc-1), [OC-2](../../../plans/openclaw-source-review-2026-08-28.md#oc-2) at `13adff02ca3897768d80d2bca18f5acf08c55d91`; see the review for named functions, callers, tests and limits. Reference tests are not candidate evidence.

- Existing/foundation: codex-rs/vault/src/lib.rs; codex-rs/network-proxy/src/credential_broker.rs; PF-13 Core capability store.
- Completed bounded source: service `src/launch/`, `src/probe/`, narrow registration and separate tests; [launcher recipe/identity preparation proof](../../../../qa/security-levels/sprints/PF-27-S04/launcher-20260912/README.md). No fixed listener, native client, PF20/Core/Vault edit or privileged execution.
- Tests: planned colocated Rust test modules prefixed `pf_27_s01`; fixtures use synthetic secrets and fake services only.

## Preconditions
- [x] All dependencies in front matter are completed and archived; plan remains active.
- [x] Read root and nearest implementation-path AGENTS.md; verified exact plan/worktree coordinates.
- [x] Accepted the PF-27-S03 Linux service, macOS launchd/XPC and Windows service/AppContainer candidates for construction; protected eligibility and sprint completion remain blocked until measured all-OS qualification passes.

## Done
- [x] Private session source74263bbbc/Rustb7275b0c:29 commands+suite0, seven real session cases, retained suites/strict lint/parity/RTX TMUX; Astra37/Fable38 exit0/findings[]. [Proof](../../../../qa/security-levels/sprints/PF-27-S04/descriptor-root-session-20260912/README.md). Received locally at51d1a64f5; combined proof pending, internal-only N/A; native/user gates remain.
- [x] Private dispatch sourcec7d48e482/Rustbd56b597:28 commands+suite0, six real pair/PF20 cases, retained suites/strict lint/parity/actual RTX TMUX; Astra35/Fable36 exit0/findings[]. [Proof](../../../../qa/security-levels/sprints/PF-27-S04/descriptor-root-dispatch-20260912/README.md). Internal-only N/A; no native/product readiness.
- [x] Descriptor compatibility d776e938d/Rustb48e9c4f: all25 commands+suite0, retained identity2 and both transport entries/five rejection scenarios, strict lint/parity; Astra33/Fable34 exit0/findings[]. [Proof](../../../../qa/security-levels/sprints/PF-27-S04/descriptor-root-compat-20260912/README.md). Internal/default-OFF only; original failures retained.
- [x] Manager accepted receiving Stage B e1cc38a70/Rust2588dfe0 after reading all17 command exits+suite0, actual RTX/TMUX/provenance/test summaries and unchanged module/lock pairs. Internal increment only; original failures/reviews retained.
- [x] StageB source7839f9f65/Rust5a4ee88b:892lines/same7paths within amended900; all18 repairedexits0, admission8/tenactualpeer scenarios, service59+11separatelycovered exclusions, preservedA/adapter suites, strictlint/parity/actual-keyRTXTMUX pass. Astra30 bufferedEOF reproduced/fixed, Fable31 receipt fixed, Fable32 patchcorrect/exit1 soleP3 count fixed without runtime change or extra review. [Proof](../../../../qa/security-levels/sprints/PF-27-S04/descriptor-admission-20260912/README.md). Privateincrement-only N/A; no product/native acceptance.
- [x] StageA identity/private pair lifecycle sourcee0eb9eac4/Rustf64820d0:614 changed lines, ten allocated paths; adapter4+8/default3/pair5+real1/owner8+real3/profiles2/service57 pass, ignored cases separately exercised; strict scoped Clippy/parity/actual-key RTX TMUX pass; Astra28/Fable29 clean. [Proof](../../../../qa/security-levels/sprints/PF-27-S04/descriptor-pair-20260912/README.md). WIP758-line archive retained; policy1.7 N/A only for this private increment, no product readiness. Manager receiving63cbce998 proof accepted.
- [x] Private single-child owner source4d830cbb3/Rust7e488200: default3/focused8/real3/profile2/service52 pass, four service exclusions exercised separately; actual-key RTX TMUX, strict Clippy and Cargo/Bazel parity pass. Astra25 clean; Fable26 lint finding repaired; Fable27 clean. [Proof](../../../../qa/security-levels/sprints/PF-27-S04/descriptor-owner-20260912/README.md). No two-child/native/product activation claim.
- [x] New single-feature record reconciled with current ownership and archived design input; no implementation claimed.
- [x] Manager accepted/integrated sealed-byte profile proof at main406aa3c5f; source635655c34 and original Astra21/Fable22 outcomes preserved. Travis approved isolated-adapter implementation/non-root proof September12; exact allocation recorded before code, existing branch/base retained.
- [x] Isolated adapter source30b471a47/Rust2d270c5c: focused3/OS7/service44/static-profile2/TMUX7 pass; second GNU2.39 host explicitly rejects unsupported libc (1 pass). Cargo/Bazel parity passes; Astra23/Fable24 clean. [Exact proof](../../../../qa/security-levels/sprints/PF-27-S04/descriptor-launch-20260912/README.md). No service wiring/privileged setup/completion claim.
- [x] Recovered reviewed broker leaves from `cdb821289` in provenance commit `90ae3a0cf`, without overwriting the current allocation or shared registrations.
- [x] Implemented digest-bound PF-41 journal integration and bounded native Linux
  peer/framing/channel teardown primitives, including concurrent disconnect
  cancellation and bounded partial-frame deadlines without idle timeouts.
  Final post-format remote suites pass 338/338 broker/Vault/proxy and 6/6
  focused Core tests; Cargo/Bazel parity passes. Astra and Fable repairs are
  verified; final Fable review has no blocking findings and one deferred P3
  signal-interruption follow-up (helper exit 1). Four of five reviews used.
  Production service, data-plane and all-OS qualification remain open; see `qa/security-levels/sprints/PF-27-S04/round5-evidence.md`.
- [x] Reconciled and qualified the service construction stage on main/0.1.42: default 1, synthetic 6, full affected 338, Core 6 and supporting TMUX pass. Fable review 6 found no runtime/security issue; one test-only lint finding was reproduced and fixed, strict Clippy and affected proof rerun successfully. Six reviews spent, no seventh; see `qa/security-levels/sprints/PF-27-S04/resume-20260911/README.md`. No native deployment or activation claimed.
- [x] Qualified admission/lifetime `bd70f0e6b` ([proof](../../../../qa/security-levels/sprints/PF-27-S04/child-admission-20260912/README.md)) and existing-root adapter `794080a4f` ([proof](../../../../qa/security-levels/sprints/PF-27-S04/root-composition-20260912/README.md)): default3/fixture17/affected356 (2 helper skips), scoped Clippy/parity/TMUX17+exit78 pass. Astra10 clean, Fable11 runtime correct with publication finding resolved; inherited transitive telemetry lint debt retained. Five new slots used. No native installation, executable launcher or protected activation.
- [x] Launch recipe/identity preparation `f0b1209e0`: default3/synthetic29/affected356 (2 existing skips), scoped lint/parity/build/TMUX pass; Astra14 and Fable15 exit0/no findings. P3 Python-path fixture follow-up fixed. No actual root-positive or native-eligibility claim; [evidence](../../../../qa/security-levels/sprints/PF-27-S04/launcher-20260912/README.md).
- [x] Fixed manifest/image inspection `099fa6f6d`: default3/synthetic35/affected356 (2 existing skips), scoped lint/parity/build/TMUX pass; Astra16/Fable17 clean. Initial fixture failures retained. No spawn authority or native qualification; [proof](../../../../qa/security-levels/sprints/PF-27-S04/manifest-20260912/README.md).
- [x] Sealed image `ee1ac023c`: default3/synthetic39/affected356 (2 existing skips), scoped lint/parity/build/TMUX pass; Astra18/Fable19 clean. Kernel write/truncate/seal denials and failure cleanup proven; no invocation or loader trust. [Proof](../../../../qa/security-levels/sprints/PF-27-S04/sealed-20260912/README.md).
- [x] Static probe build/linkage feasibility passes on unchanged Rust0aa65bd: staticPIE f6ca8e3368dc/no interpreter or external libs and GNU control, actual-key TMUX. Fable20 patch correct/P3 receipt gap repaired; original exit1 and failed attempts retained. No artifact invocation/native qualification. [Proof](../../../../qa/security-levels/sprints/PF-27-S04/static-probe-20260912/CURRENT.md).
## Remaining
- [ ] Obtain manager combined receiving proof for qualified private root session and the next literal source allocation. Dispatch eb5855959 already accepted; protected-user/PF26 policy1.7 gates remain.
- [ ] Include fresh connections after same-run re-registration with cached TLS handlers and admitted hosts, not only reuse of an old channel. Revocation fences queued dispatch, streams and uploads; new generations cannot inherit old credentials.
- [ ] Keep sentinel keys/raw registries outside agent-accessible processes; test open-channel revocation, upload cancellation, same-run-ID replacement and broker restart with old handles. The proxy's retained RegisteredRun concern requires a native regression, not just a copied new-connection test.
- [ ] Move raw credential resolution/substitution and its key material into a separately constrained trusted process; Core, model clients, and agent-accessible workers receive only opaque references.
- [ ] Reuse PF-16–19 decision, actor, mandate, expiry, and revocation types over versioned, bounded IPC; authenticate OS peer plus session/task/run, reject replay and malformed frames.
- [ ] Permit typed credential operations only; disallow generic resolve-to-string, arbitrary shell/URL/header/body injection, debug dumps, and unbounded credential enumeration.
- [ ] Make cancellation, run replacement, revocation, broker death, and restart close outstanding channels and invalidate capabilities; never restore a stale capability or fall back to raw auth.
- [ ] Add broker-crash, cross-run theft, wrong-peer, forged-reference, bounded-resource, and concurrent-revoke integration tests; preserve PF-13's exact OpenAI adapter.
- [ ] Add named `pf_27_s01` regression tests; update affected Cargo/Bazel/lock/schema edges together without broadening this feature.

## Verification
- [ ] Apply [independent isolated execution](../../../../qa/code-blind-functional/isolated-execution.md) to affected functional handoff; record schema-2 proof or integrator-accepted internal-only N/A and later gate. Historical tests are not upgraded.
- [ ] Run `cd codex-rs && just fix -p <affected-crate>` for each listed crate, then `just fmt`; inspect the final diff.
- [ ] Focused: `cd codex-rs && just test -p codex-secret-broker pf_27_s01 && just test -p codex-core pf_27_s01`; confirm tests actually ran.
- [ ] Integration: full affected crate suites via `just test -p <affected-crate>`; update Bazel locks when manifests change.
- [ ] TUI applicability: none; integration flows are re-run by PF-26-S02
- [ ] Record candidate/commit, commands, expected/actual outcomes and safe artifact digests; no production credentials or funds.

## Exit evidence
- [ ] Implementation commit and final-tree outputs under `qa/security-levels/sprints/PF-27-S04/`.
- [ ] Acceptance and source-mapping assertions proven; applicable true-TUI keys/checkpoints captured after formatting.
- [ ] PF-26 final-candidate and both-live-repository requalification remains mandatory; no release-complete claim here.
- [ ] Done/Remaining reflect reality; completed record moved to the archive and plan/navigation updated.
