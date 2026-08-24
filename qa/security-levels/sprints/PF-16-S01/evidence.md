# PF-16-S01 final-tree evidence

- Date: 2026-08-24 UTC
- Product citation: **P0 `/security` levels** — “Moderate and Aggressive are deterministic. Model judgment can warn but cannot grant authority.”
- Original implementation: `d183036cb0`
- Corrective implementation: `6af50d0a5f`
- Tested code tree: `6af50d0a5f`
- Public API reviewed: `ActorChain`, `AuthorizationContext`, `AuthorizationRequest`, `AuthorizationDecision`, `ProtectedResource`, `PolicyAction`, and deterministic request digests.
- TUI applicability: none.

## Review result

The original request already bound the human/agent actor chain, resource, typed action, destination, quantity, time, and grant identifier. Review found that session, task, purpose, and concrete operation were missing from the canonical request and therefore from its digest. Corrective commit `6af50d0a5f` makes all four required bounded fields. It also adds mutation tests for every new binding and malformed/incomplete-input tests proving failure messages do not echo a protected canary value. Dependent Permissive fixtures in Core and Vault were updated mechanically to construct the strengthened request.

No parallel policy concepts or unauthorized allow behavior remain in the reviewed PF-16 surface.

## Final-tree commands

```text
cd codex-rs && just fix -p codex-security-policy
PASS — clippy completed.

cd codex-rs && just fmt
PASS — formatting completed; diff inspected.

cd codex-rs && just test -p codex-security-policy authorization
PASS — 3 authorization tests passed, 9 filtered out.

cd codex-rs && just test -p codex-security-policy
PASS — 12 tests passed.
```

## Changed paths

- `codex-rs/security-policy/src/authorization.rs`
- `codex-rs/security-policy/src/security_policy_tests.rs`
- `codex-rs/core/src/agent/registry_tests.rs` (constructor compatibility only)
- `codex-rs/vault/src/tests.rs` (constructor compatibility only)
