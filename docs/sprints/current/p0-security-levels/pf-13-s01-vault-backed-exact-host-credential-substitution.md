---
sprint_id: "PF-13-S01"
title: "Vault-backed exact-host credential substitution"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-13"
execution_order: 1
owner: "Jim Ricketts"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "none"
created: 2026-08-24
updated: 2026-08-24
---

# PF-13-S01 — Vault-backed exact-host credential substitution

## Execution mandate

- Deliver: one broker-supported HTTP credential can be used through an opaque,
  scoped vault capability and substituted only at its authorized transport.
- Excludes: `/security` TUI, generalized protected-data taxonomy, financial
  signing, browser isolation, content sanitization, classifiers, and new providers.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md)
- Feature: `PF-13` — Vault-backed egress capability boundary.
- Acceptance advanced: secrets stay out of model-visible and audit paths even
  when an agent performs an authorized provider request.

## Code boundaries

- Existing: `codex-rs/vault/src/lib.rs::Vault::reveal_for_programmatic_use`;
  `codex-rs/network-proxy/src/credential_broker.rs::CredentialBroker`.
- Planned: `codex-rs/vault/src/capability.rs`; minimum network-proxy adapter
  needed to resolve a capability at request-header injection.
- Tests: `codex-rs/vault/src/capability_tests.rs` and
  `codex-rs/network-proxy/src/credential_broker_tests.rs`.

## Functional requirements

- FR1: issue an unguessable opaque reference whose safe metadata binds the human
  principal, agent/session, task, purpose, operation, HTTP method, normalized
  destination, credential scope, issue/expiry time, and revocation generation.
- FR2: expose metadata and typed decisions only; no agent-facing API may return,
  serialize, format, log, or persist the raw credential.
- FR3: after deterministic validation, resolve and substitute the credential at
  the existing network-proxy header-injection boundary for one provider fixture.
- FR4: reject malformed, forged, expired, revoked, replayed, wrong-actor,
  wrong-purpose, wrong-operation, wrong-method, wrong-host, and broader-scope use.
- FR5: emit a secret-free decision and receipt identifying capability id,
  policy reason, destination, operation, and outcome.
- FR6: preserve existing Permissive behavior; the new path remains internal and
  opt-in until a later sprint composes it into Moderate or Aggressive.

## Non-functional requirements

- NFR1: use constant-time secret comparison where values are compared; redact
  `Debug`/error output and zeroize temporary raw material after substitution.
- NFR2: fail closed on unknown fields, clock failure, poisoned state, ambiguous
  host matching, or unavailable vault/broker state.
- NFR3: capability storage has a hard bound, removes expired/revoked entries,
  performs no additional network round trip, and is safe under concurrent use.
- NFR4: support Linux, macOS, and Windows without weakening exact-host checks;
  add no provider SDK or model dependency.

## Preconditions

- [ ] Plan is active, `PF-13` remains in scope, and dependencies remain absent.
- [ ] Worktree, branch, and base commit are exact and match the plan.

## Done

- [x] Sprint record defines one vertical slice, its boundaries, and measurable evidence.

## Remaining

- [ ] Implement FR1-FR6 in the declared boundaries without exposing a general raw-secret API.
- [ ] Implement NFR1-NFR4 and typed denial/receipt behavior.
- [ ] Add unit cases for every bound field, expiry/revocation, capacity, redaction, and cleanup.
- [ ] Add an integration canary across vault, child dummy value, proxy, and the
  captured exact-host request; scan model/tool/log/audit/artifact surfaces.
- [ ] Add adversarial host, redirect, concurrent-use, revocation-race, and replay cases.

## Verification

- [ ] Focused test: `cd codex-rs && just test -p codex-vault`.
- [ ] Integration test: `cd codex-rs && just test -p codex-network-proxy credential_broker`.
- [ ] Canary absent from model/tool payloads, logs, audit, artifacts, errors,
  receipts, and unauthorized child environments.
- [ ] Regression: existing broker provider and Permissive behavior tests pass unchanged.
- [ ] TUI applicability: none; no user-facing surface changes in this sprint.

## Exit evidence

- [ ] Implementation commit, changed paths, final tests, and canary scan recorded.
- [ ] Security review confirms raw-secret reachability did not expand; ledgers reflect reality.
- [ ] Completed record moved to `docs/sprints/archive/p0-security-levels/`.
