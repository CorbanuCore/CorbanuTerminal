# PF-13-S02 final-tree evidence

- Date: 2026-08-25 UTC
- Baseline commit: `1bdc515bff48a4d9048dae7d06c6214e884265bc`
- Implementation commit: `9e5789c1ae59f75b9e10771ce16a98ed21d48bbe`
- Tested code tree: `9e5789c1ae59f75b9e10771ce16a98ed21d48bbe`
- TUI applicability: none; this sprint changes no interactive surface.

## Result

Vault now accepts a `VaultCredentialRef` created at the trusted Core adapter
only after opaque capability authorization. The reference carries the
secret-free capability id and fully bound request, is neither cloneable nor
serializable, has no `Display` implementation, and emits only
`VaultCredentialRef(<redacted>)` from `Debug`.

Every use revalidates the complete request binding, current time, and current
revocation state before vault access. Missing, deleted, ineligible credential
types, wrong label or scope, expired authority, revoked authority, storage
failure, callback failure, cancellation, and panic all reduce to stable
secret-free errors.

## Temporary-secret lifetime and redaction review

1. Core consumes its crate-private `AuthorizedCredentialCapability` into a
   `VaultCredentialRef`; the opaque bearer and raw credential never cross this
   adapter.
2. Vault revalidates label, scope, expiry, and revocation before decrypting.
3. While holding the vault storage lock, Vault validates metadata and credential
   type, reads the secret, and immediately moves its allocation into
   `Zeroizing<String>`.
4. The storage lock is released before a trusted synchronous callback receives
   a borrowed `&str`. The callback cannot return data from the resolver API.
5. On success, stable error, cancellation, or caught panic, Vault explicitly
   drops the zeroizing allocation before mapping or returning the outcome.
   Panic payloads are discarded without formatting.
6. The implementation performs no comparisons on credential material, so no
   secret comparison requires a constant-time primitive. Label and scope checks
   compare bounded public metadata only.

Tests exercise callback success, error, cancellation, and panic; prove the vault
lock is reusable afterward; compile-check absence of `Serialize` and
`Display`; and verify `Debug`, tracing, and returned errors do not contain
the credential canary.

## Final-tree commands

```text
cd codex-rs && just fix -p codex-vault && just fix -p codex-core
PASS — Clippy completed; only seven pre-existing Core test dead-code warnings were emitted.

cd codex-rs && just fmt
PASS — formatting completed and the final diff was inspected.

just bazel-lock-update && just bazel-lock-check
PASS — Cargo/Bazel dependency parity passed; MODULE.bazel.lock required no content change.

cd codex-rs && just test -p codex-vault capability
PASS — 5 resolver tests passed.

cd codex-rs && just test -p codex-vault
PASS — all 28 vault tests passed.

cd codex-rs && just test -p codex-core credential_capability
PASS — 7 targeted Core capability tests passed; 3381 unrelated tests were skipped.
```

## Changed paths

- `codex-rs/Cargo.lock`
- `codex-rs/core/Cargo.toml`
- `codex-rs/core/src/security/credential_capability.rs`
- `codex-rs/core/src/security/credential_capability_tests.rs`
- `codex-rs/vault/Cargo.toml`
- `codex-rs/vault/src/lib.rs`
- `codex-rs/vault/src/capability.rs`
- `codex-rs/vault/src/capability_tests.rs`
