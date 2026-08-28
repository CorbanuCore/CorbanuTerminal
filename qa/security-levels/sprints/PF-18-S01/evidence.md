# PF-18-S01 final-tree evidence

- Date: 2026-08-24 UTC
- Original implementation: `e22a35ccf2`
- Corrective implementation: `4b438c46bb`
- Tested code tree: `4b438c46bb`
- Public API reviewed: `ProtectedActionPreview`, `ProtectedActionMandate`, `ReplayLedger`, and `ActionReceipt`.
- TUI applicability: none; PF-25 owns the trusted human interaction.

## Review result

The original mandate digest bound the complete protected-action preview to the human principal and rejected preview mutation, duplicate consumption, replay, and expiry. Review found that `matches_preview` accepted timestamps earlier than `approved_at_unix_seconds`, allowing pre-approval use. Corrective commit `4b438c46bb` restricts use to the exact approval validity window.

Regression coverage now mutates every bound dimension: actor chain, resource, action, request time, session, task, purpose, operation, destination, quantity, grant identifier, preview expiry, and nonce. It also covers pre-approval use, stale use, duplicate consumption, replay, malformed fields, negative clock input, receipt integrity mutation, and the exact secret-free receipt serialization surface.

## Final-tree commands

```text
cd codex-rs && just fix -p codex-security-policy
PASS — clippy completed and applied only test-style fixes.

cd codex-rs && just fmt
PASS — formatting completed; final diff inspected.

cd codex-rs && just test -p codex-security-policy mandate
PASS — 2 mandate tests passed, 11 filtered out.

cd codex-rs && just test -p codex-security-policy
PASS — 13 tests passed.
```

## Changed paths

- `codex-rs/security-policy/src/mandate.rs`
- `codex-rs/security-policy/src/security_policy_tests.rs`
