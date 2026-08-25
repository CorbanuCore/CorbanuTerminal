---
sprint_id: "PF-13-S03"
title: "OpenAI exact-host proxy substitution"
status: in_progress
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-13"
execution_order: 11
owner: "Jim Ricketts"
worktree: "/home/pfrpc/repos/CorbanuTerminal-pf13-s02"
branch: "feat/pf-13-s02-scoped-vault-resolver"
base_commit: "1bdc515bff48a4d9048dae7d06c6214e884265bc"
depends_on: "PF-13-S02"
created: 2026-08-24
updated: 2026-08-25
---

# PF-13-S03 — OpenAI exact-host proxy substitution

## Execution mandate

- Deliver: substitute one vault-backed bearer credential only for approved OpenAI HTTPS requests.
- Excludes: other providers, redirects, arbitrary headers, legacy CLI gating, receipts, and TUI.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md)
- Feature: `PF-13`
- Acceptance advanced: approved `POST https://api.openai.com/v1/*` receives `Authorization: Bearer <secret>` only at transport.

## Code boundaries

- Existing: `network-proxy/src/credential_broker.rs::CredentialBroker`; `credential_broker/providers/openai.rs`
- Planned: `network-proxy/src/credential_broker/resolver.rs`; Core adapter in `core/src/config/network_proxy_spec.rs`
- Tests: `network-proxy/src/credential_broker_tests.rs`; affected Cargo/Bazel files

## Preconditions

- [x] PF-13-S02 is completed and archived.
- [x] Read root, `codex-rs/AGENTS.md`, and `codex-rs/core/AGENTS.md`.
- [x] Exact worktree coordinates match the active plan.

## Done

- [x] Sprint record names the provider, scheme, host, method, path, and header.

## Remaining

- [ ] Replace broker-owned `real_value: String` state with a capability reference plus a non-secret resolver interface.
- [ ] Pass scheme, normalized host, port, method, path, actor/session/task, and capability id into validation before resolution.
- [ ] Permit only HTTPS, default port 443, exact host `api.openai.com`, method `POST`, path prefix `/v1/`, and the OpenAI bearer header.
- [ ] Resolve at header injection, zeroize immediately afterward, and refuse redirect reuse or caller-supplied conflicting authorization.
- [ ] Deny lookalike/subdomain hosts, alternate ports/schemes/methods/paths, redirects, stale references, and explicit-header collisions.
- [ ] Preserve existing Permissive broker behavior outside the new opt-in route.

## Verification

- [ ] Fix: `cd codex-rs && just fix -p codex-network-proxy && just fix -p codex-core`.
- [ ] Format: `cd codex-rs && just fmt`; then inspect the final diff.
- [ ] Focused test: `cd codex-rs && just test -p codex-network-proxy credential_broker`.
- [ ] Core integration: `cd codex-rs && just test -p codex-core network_proxy_spec`.
- [ ] TUI applicability: none; no user-facing surface changes.

## Exit evidence

- [ ] Implementation commit and changed paths recorded.
- [ ] Captured exact-host request and denial matrix linked under `qa/security-levels/sprints/PF-13-S03/`.
- [ ] Review confirms the proxy stores no raw credential across requests.
- [ ] Ledgers reflect reality and the completed record is archived.
