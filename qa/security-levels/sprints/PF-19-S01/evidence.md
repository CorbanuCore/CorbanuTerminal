# PF-19-S01 final-tree evidence

- Date: 2026-08-24 UTC
- Original implementation: `8a3b416c26`
- Corrective implementation: `46e9121ebe`
- Tested code tree: `46e9121ebe`
- Public API reviewed: `RevocationEvent`, `RevocationTarget`, `RevocationState`, grant/mandate invalidation, generation, and kill-switch ordering.
- TUI applicability: none.

## Review result

The original state made specific revocations and all-authority timestamps monotonic, but it had no revocation generation for cache invalidation, an older kill-switch event could overwrite a newer state, and actor revocation could not invalidate a pending mandate by its acting-agent chain.

Corrective commit `46e9121ebe` adds a validated generation tied to the applied-event ledger, deterministic `(timestamp, event id)` kill-switch ordering, convergence under reversed delivery, monotonic all-authority invalidation, corrupt-state rejection, and the complete actor chain on mandates. A revoked acting agent now invalidates both grants and pending mandates. Duplicate events remain idempotent and do not advance generation; distinct stale switch events are recorded without rolling policy back.

Regression coverage includes specific grant revocation, acting-agent mandate revocation, duplicate delivery, restart/serialization, same-time reversed delivery, stale rollback, unknown targets, generation corruption, and kill-switch dominance.

## Final-tree commands

```text
cd codex-rs && just fix -p codex-security-policy
PASS — clippy completed.

cd codex-rs && just fmt
PASS — formatting completed; final diff inspected.

cd codex-rs && just test -p codex-security-policy revocation
PASS — 2 revocation tests passed, 12 filtered out.

cd codex-rs && just test -p codex-security-policy
PASS — 14 tests passed.
```

## Changed paths

- `codex-rs/security-policy/src/mandate.rs`
- `codex-rs/security-policy/src/revocation.rs`
- `codex-rs/security-policy/src/security_policy_tests.rs`
