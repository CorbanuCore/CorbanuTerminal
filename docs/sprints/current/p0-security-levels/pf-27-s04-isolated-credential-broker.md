---
sprint_id: "PF-27-S04"
title: "Isolated credential broker process"
status: in_progress
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-27"
execution_order: 28
owner: "/root/broker"
parallel_lane: "isolated-broker"
write_scope: "codex-rs/secret-broker/, codex-rs/network-proxy/src/credential_broker.rs, codex-rs/network-proxy/src/credential_broker/, codex-rs/network-proxy/src/credential_broker_tests.rs, codex-rs/core/src/security/broker_client.rs, codex-rs/core/src/security/broker_client_tests.rs, codex-rs/core/src/config/network_proxy_credential.rs, codex-rs/core/src/config/network_proxy_credential_tests.rs, codex-rs/vault/src/capability.rs, codex-rs/vault/src/capability_tests.rs, qa/security-levels/sprints/PF-27-S04/, docs/sprints/current/p0-security-levels/pf-27-s04-isolated-credential-broker.md"
integration_gate: "Codex /root serializes shared Core/Vault/network-proxy and Cargo/Bazel/lock registration, audits scope, reruns broker/network-proxy/Vault/Core and governance suites on RTX plus TMUX, Astra High and Fable 5.1 High reviews (maximum five per lane). Native service, provider data-plane and all-OS qualification remain mandatory for completion; intermediate leaves cannot enable protected activation."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/security-round5-broker"
branch: "feat/security-round5-broker"
base_commit: "07791288b6feeccfaee5a57c12452359cc666957"
depends_on: "PF-27-S01, PF-13-S04, PF-27-S03, PF-41-S03"
created: 2026-08-28
updated: 2026-09-04
---

# PF-27-S04 — Isolated credential broker process

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

- OpenClaw adoption reference: [OC-1](../../../plans/openclaw-source-review-2026-08-28.md#oc-1), [OC-2](../../../plans/openclaw-source-review-2026-08-28.md#oc-2) at `13adff02ca3897768d80d2bca18f5acf08c55d91`; see the review for named functions, callers, tests and limits. Reference tests are not candidate evidence.

- Existing/foundation: codex-rs/vault/src/lib.rs; codex-rs/network-proxy/src/credential_broker.rs; PF-13 Core capability store.
- Planned: codex-rs/secret-broker/src/{main,ipc,resolver}.rs; codex-rs/core/src/security/broker_client.rs.
- Tests: planned colocated Rust test modules prefixed `pf_27_s01`; fixtures use synthetic secrets and fake services only.

## Preconditions

- [x] All dependencies in front matter are completed and archived; plan remains active.
- [x] Read root and nearest implementation-path AGENTS.md; verified exact plan/worktree coordinates.
- [x] Accepted the PF-27-S03 Linux service, macOS launchd/XPC and Windows service/AppContainer candidates for construction; protected eligibility and sprint completion remain blocked until measured all-OS qualification passes.

## Done

- [x] New single-feature record reconciled with current ownership and archived design input; no implementation claimed.
- [x] Recovered reviewed broker leaves from `cdb821289` in provenance commit
  `90ae3a0cf`, without overwriting the current allocation or shared registrations.
- [x] Implemented digest-bound PF-41 journal integration and bounded native Linux
  peer/framing/channel teardown primitives; final post-format tests pass 42/42
  on RTX. Reviews and production service qualification remain open;
  see `qa/security-levels/sprints/PF-27-S04/round5-evidence.md`.

## Remaining

- [ ] Implement the completed PF-27-S03 OS identity/IPC/handle design and PF-41-S03 durable-event contract; verify controller/broker state cannot be read or rewritten by the real agent process.
- [ ] Include fresh connections after same-run re-registration with cached TLS handlers and admitted hosts, not only reuse of an old channel. Revocation fences queued dispatch, streams and uploads; new generations cannot inherit old credentials.

- [ ] Keep sentinel keys/raw registries outside agent-accessible processes; test open-channel revocation, upload cancellation, same-run-ID replacement and broker restart with old handles. The proxy's retained RegisteredRun concern requires a native regression, not just a copied new-connection test.

- [ ] Move raw credential resolution/substitution and its key material into a separately constrained trusted process; Core, model clients, and agent-accessible workers receive only opaque references.
- [ ] Reuse PF-16–19 decision, actor, mandate, expiry, and revocation types over versioned, bounded IPC; authenticate OS peer plus session/task/run, reject replay and malformed frames.
- [ ] Permit typed credential operations only; disallow generic resolve-to-string, arbitrary shell/URL/header/body injection, debug dumps, and unbounded credential enumeration.
- [ ] Make cancellation, run replacement, revocation, broker death, and restart close outstanding channels and invalidate capabilities; never restore a stale capability or fall back to raw auth.
- [ ] Add broker-crash, cross-run theft, wrong-peer, forged-reference, bounded-resource, and concurrent-revoke integration tests; preserve PF-13's exact OpenAI adapter.
- [ ] Add named `pf_27_s01` regression tests; update affected Cargo/Bazel/lock/schema edges together without broadening this feature.

## Verification

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
