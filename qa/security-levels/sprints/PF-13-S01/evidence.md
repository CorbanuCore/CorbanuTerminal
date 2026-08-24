# PF-13-S01 final-tree evidence

- Date: 2026-08-24 UTC
- Baseline commit: `3c1b2f6cbe11657ff4e3b72b11db029c9e7a92eb`
- Policy-contract commit: `0f62930018`
- Core-store commit: `61afdd805c`
- Tested code tree: `61afdd805c`
- TUI applicability: none; PF-26-S02 owns interactive qualification.

## Result

The security-policy crate now exports one versioned, secret-free
`CredentialCapabilityRequest` that binds the existing actor chain,
authorization request, bounded grant, optional existing action receipt, vault
label and scope identifiers, HTTP method, canonical HTTPS host/port, canonical
origin path, issue/expiry window, and revocation generation.

Core owns a hard-bounded concurrent lifecycle store. Issuance uses 256 bits of
operating-system entropy. The bearer `CapabilityToken` is private,
non-serializable, non-cloneable, redacted in `Debug`, and zeroized on drop.
Only a separate SHA-256 `CapabilityId` is safe to expose. The store contains
only the secret-free authority request, revalidates time and revocation on every
use, removes stale entries, and fails closed on malformed state, capacity,
clock, entropy, collision, forgery, authority mismatch, and poisoned locks.

## Public policy API

- `CredentialCapabilityRequest`
- `CredentialCapabilityError`
- `CredentialDestination`
- `CredentialHttpMethod`
- `CredentialReference`
- `CredentialTransport`
- `CapabilityId`
- `CREDENTIAL_CAPABILITY_SCHEMA_VERSION`
- `CAPABILITY_ID_HEX_LENGTH`

The Core store and opaque handle remain crate-private until PF-13-S02 connects
the trusted vault resolver.

## Secret-surface review

- There is no credential-value, bearer-header, private-key, seed, or arbitrary
  payload field in the policy contract.
- Vault references accept only bounded label/scope identifiers.
- Destination authority is typed HTTPS plus canonical lowercase DNS host and
  nonzero port; IP literals, userinfo, trailing dots, path smuggling, query,
  fragment, and dot-segment paths fail.
- Store entries never contain bearer token bytes or a credential value.
- The only token-bearing object stays inside Core and exposes no byte/string
  accessor.
- Errors and debug output do not echo token bytes or credential material.

## Final-tree commands

```text
cd codex-rs && just fix -p codex-security-policy
PASS — Clippy completed without new warnings.

cd codex-rs && just fix -p codex-core
PASS — Clippy completed; only seven pre-existing Core test dead-code warnings were emitted.

cd codex-rs && just fmt
PASS — formatting completed and the final diff was inspected.

cd codex-rs && just test -p codex-security-policy credential
PASS — 6 credential policy tests passed.

cd codex-rs && just test -p codex-core credential_capability
PASS — 7 Core store tests passed, including full-object, concurrency, forgery,
expiry, revocation, wrong actor/purpose/operation/method/host/path/scope,
capacity, clock, entropy, collision, and poison cases.

cd codex-rs && just test -p codex-security-policy
PASS — all 20 security-policy tests passed.

just bazel-lock-update
PASS — dependency graph resolved; MODULE.bazel.lock required no content change.

just bazel-lock-check
PASS — Bazel module lock parity check passed.
```

## Changed paths

- `codex-rs/Cargo.lock`
- `codex-rs/core/Cargo.toml`
- `codex-rs/core/src/security/mod.rs`
- `codex-rs/core/src/security/credential_capability.rs`
- `codex-rs/core/src/security/credential_capability_tests.rs`
- `codex-rs/security-policy/src/lib.rs`
- `codex-rs/security-policy/src/credential.rs`
- `codex-rs/security-policy/src/credential_tests.rs`
