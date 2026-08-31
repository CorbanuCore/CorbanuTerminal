# PF-27-S04 serialized integration handoff

Apply this only after PF-22-S02 is integrated and archived. Keep activation
fail closed until the complete broker/backend/service seam and all-OS evidence
are present.

## Shared dependency lock

The workspace and Core dependency registrations already exist on the allocation
base:

- `codex-rs/Cargo.toml` contains workspace member `secret-broker` and alias
  `codex-secret-broker = { path = "secret-broker" }`.
- `codex-rs/core/Cargo.toml` contains `codex-secret-broker = { workspace = true }`.
- `codex-rs/secret-broker/BUILD.bazel` already owns the Bazel crate target.

The shared manifest parity delta adds `codex-secret-broker = { workspace = true }`
to `codex-rs/vault/Cargo.toml`, adds `codex-secret-broker` to the Vault lock
entry, and applies this `codex-secret-broker` lock-entry change:

```diff
 [[package]]
 name = "codex-secret-broker"
 version = "0.1.35"
 dependencies = [
  "chrono",
+ "hmac 0.12.1",
  "pretty_assertions",
+ "serde",
  "serde_json",
+ "sha2 0.10.9",
+ "thiserror 2.0.18",
+ "zeroize",
 ]
```

After applying that exact lock delta, run from the repository root with all
cache/output variables pointed at the integration owner's CorbanuDrive lane:

```text
just bazel-lock-update
just bazel-lock-check
```

Inspect and commit any resulting `MODULE.bazel.lock` change with the shared
lock registration. No such change was needed by the leaf Bazel test before
integration.

## Serialized code seam

The adapter implementations are in the scoped leaf files. Apply these minimal
registrations in one serialized integration commit:

1. add `pub(crate) mod broker_client;` to `core/src/security/mod.rs`;
2. re-export `IsolatedCredentialDispatchError`,
   `IsolatedCredentialDispatcher`, `IsolatedCredentialReceipt`,
   `IsolatedCredentialRoute`, and `IsolatedCredentialUse` from
   `network-proxy/src/lib.rs`;
3. in `network-proxy/src/runtime.rs`, import those isolated route/receipt/error
   types and expose `install_isolated_credential_route` plus
   `dispatch_isolated_credential`, each delegating to `CredentialBroker`;
4. re-export `SystemVaultBrokerClock`, `VaultBrokerBackend`,
   `VaultBrokerBackendError`, `VaultBrokerClock`, and `VaultBrokerTransport`
   from `vault/src/lib.rs`; and
5. apply the Vault Cargo/lock deltas above, then update/check Bazel locks.

These registrations were applied temporarily, formatted, and used for focused
Core plus full network-proxy/Vault tests, then restored before branch handback.

Do not activate by merely importing `BrokerRuntime`. The integration owner then
activates, in this order:

1. a qualified OS transport that creates `ObservedPeer` from `SO_PEERCRED`, an
   XPC audit token/code requirement, or a named-pipe client token;
2. the implemented in-broker Vault backend plus a PF-41 `DurableBrokerAudit`
   adapter;
3. the implemented Core broker client/config leaf;
4. the network-proxy exact-host activation call site, with cached-handler, fresh-connection,
   run-replacement, stream, upload, and revocation regressions; and
5. protected-runtime activation only after measured platform authorization.

Any missing adapter must return typed unavailable. The current `main.rs` exit-78
behavior is the required safe default until those seams exist.
