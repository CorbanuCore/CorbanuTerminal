# PF-17-S01 final-tree evidence

- Date: 2026-08-24 UTC
- Original implementation: `d68c4dbc95`
- Corrective implementation: `5a03e1e0ec`
- Tested code tree: `5a03e1e0ec`
- Public API reviewed: `GrantScope`, `BoundedGrant`, delegation, expiry, and exact request matching.
- TUI applicability: none.

## Review result

The original grant contract preserved the human issuer and actor-chain prefix, bound grant integrity, required expiry, and checked exact resource/action/destination and quantitative limits. Review found two general widening defects:

1. a child scope could add a quantitative asset not authorized by its parent; and
2. a grant could match a request timestamped before the grant was issued.

Corrective commit `5a03e1e0ec` requires every bounded child asset to exist within the parent's limits, rejects child issuance before parent issuance, and makes grants inactive before issuance. Regression coverage now includes pre-issuance, expiry, wrong actor, wrong action, adjacent resource, wrong destination, extra asset, excessive quantity, missing quantity, child scope widening, child action widening, and integrity mutation.

## Final-tree commands

```text
cd codex-rs && just fix -p codex-security-policy
PASS — clippy completed.

cd codex-rs && just fmt
PASS — formatting completed; final diff inspected.

cd codex-rs && just test -p codex-security-policy grant
PASS — 2 grant tests passed, 10 filtered out.

cd codex-rs && just test -p codex-security-policy
PASS — 12 tests passed.
```

## Changed paths

- `codex-rs/security-policy/src/grant.rs`
- `codex-rs/security-policy/src/security_policy_tests.rs`
