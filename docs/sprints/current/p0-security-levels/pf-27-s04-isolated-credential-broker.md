---
sprint_id: "PF-27-S04"
title: "Isolated credential broker process"
status: in_progress
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-27"
execution_order: 28
owner: "/root/pf27_isolated_broker"
parallel_lane: "isolated-broker"
write_scope: "codex-rs/secret-broker/, codex-rs/network-proxy/src/credential_broker.rs, codex-rs/network-proxy/src/credential_broker/, codex-rs/network-proxy/src/credential_broker_tests.rs, codex-rs/core/src/security/broker_client.rs, codex-rs/core/src/security/broker_client_tests.rs, codex-rs/core/src/config/network_proxy_credential.rs, codex-rs/core/src/config/network_proxy_credential_tests.rs, codex-rs/vault/src/capability.rs, codex-rs/vault/src/capability_tests.rs, qa/security-levels/sprints/PF-27-S04/, docs/sprints/current/p0-security-levels/pf-27-s04-isolated-credential-broker.md"
integration_gate: "After PF-22-S02 is integrated and archived, the Codex ingress/classifier integration owner rebases PF-27-S04, audits the literal scope, serializes Core module/Cargo/Bazel/lock registration, reruns secret-broker/network-proxy/Vault/Core/platform/governance suites, completes TMUX and Opus 5 Max closure, and pauses before final Linux/Windows qualification until the user confirms the tailnet switch."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/p0-security-isolated-broker"
branch: "feat/p0-security-isolated-broker"
base_commit: "43d2d86488d5c1b2eb5cbc401ee8371dbdb76bf4"
depends_on: "PF-27-S01, PF-13-S04, PF-27-S03, PF-41-S03"
created: 2026-08-28
updated: 2026-08-31
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

- [x] Reconciled the record with current ownership and archived design input.
- [x] Added bounded authenticated IPC and a typed exact-host operation with canonical opaque references, replay/binding checks, and no generic resolver API.
- [x] Added a fail-closed PF-27-S03-authorized runtime with OS-observed peers, resource caps, lifecycle fences, and PF-41 intent-before-effect semantics.
- [x] Added named `pf_27_s04_pf_27_s01` regressions for malformed frames, wrong peers, forged references, replay, cross-run theft, expiry, replacement, restart, audit failure, bounds, and concurrent upload cancellation.
- [x] Verified leaf/Bazel/proxy/Vault/audit/platform/governance checks and recorded evidence plus the serialized shared-integration handoff.
- [x] Added the Core authenticated broker client/config adapter, the
  network-proxy isolated exact-host route with no header injection, and the
  in-broker Vault typed backend; compiled and tested them under temporary
  integration-owner registrations that were restored before handback.

## Remaining

- [ ] Integrate and qualify the completed PF-27-S03 OS service transports and
  PF-41-S03 durable journal adapter; verify controller/broker state cannot be
  read or rewritten by the real agent process.
- [ ] Add real broker-crash/service-death integration tests and measured
  Linux/macOS/Windows wrong-peer, state-isolation, resource, replacement, fresh
  connection, cached-handler, open-channel, upload, and concurrent-revoke proof.
- [ ] Let the integration owner apply the recorded Cargo/Bazel/shared-lock and
  Core module registrations without broadening this lane.

## Verification

- [x] Ran fix/format for the owned leaf, inspected the final diff, and kept all
  targets/caches/logs on CorbanuDrive.
- [x] Ran focused secret-broker/Core/network/Vault tests; the Core and adapter
  leaves were compiled under temporary serialized registrations that were
  restored before handback.
- [x] Ran full secret-broker, network-proxy, Vault and security-audit suites,
  the Bazel unit target, platform-probe contracts, and governance checks.
- [x] Captured a supporting real-TMUX Corbanu Terminal smoke in
  read-only/never mode with Claude Opus 5 Plan Max selected.
- [x] Recorded implementation candidate `75bce53ef`, commands, outcomes and
  safe artifact digests; no production credentials or funds were used.
- [ ] Complete the preserved TMUX/Corbanu/Claude Opus 5 Max review after the
  current Claude credential blocker is cleared; no clean verdict is claimed.

## Exit evidence

- [x] Implementation commit and final-tree outputs recorded under
  `qa/security-levels/sprints/PF-27-S04/`.
- [x] Acceptance and source-mapping assertions proven for the scoped leaf;
  supporting true-TUI status checkpoints captured after formatting.
- [ ] PF-26 final-candidate and both-live-repository requalification remains mandatory; no release-complete claim here.
- [ ] Done/Remaining reflect reality; completed record moved to the archive and plan/navigation updated.
