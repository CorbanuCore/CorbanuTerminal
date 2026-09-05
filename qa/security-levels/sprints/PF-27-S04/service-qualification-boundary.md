# Linux service construction and qualification boundary

This record separates executable construction from protected deployment. The
new composition crate links the broker, typed Vault backend and PF-41 journal
without a dependency cycle. `TrustedSession` consumes an owned socket, expected
native peer, binding, MAC and expiring grants; the runtime also requires the
existing `ProtectedModeAuthorization`. None is a worker-deserialized setup API.

The normal `codex-secret-broker-service` binary still exits 78. The separately
named `codex-secret-broker-service-fixture` requires the `synthetic-fixture`
build feature and an explicit fixture argument. It uses only a newly created
synthetic Vault, mock keyring, volatile test integrity root and fixed receipt
transport. Its inherited socket-pair peer is the parent that created the pair,
not the child's post-exec PID. Native peer checks do not prove process isolation.

## Exact missing code boundary

Repository inspection finds `IntegrityRootStore` implementations only in test
fixtures. PF-20's production protected-root owner must supply a durable native
adapter and its authenticated bootstrap handle. A same-user JSON/file root is
not an acceptable replacement. The new service accepts the preconstructed
PF-41 journal adapter; constructing it from untrusted claims is not implemented.
An authenticated native launcher must also supply the validated platform report,
per-boot key, live grants and connected socket. Installing the default exit-78
binary as a system service does not solve either missing adapter.

## Minimal proposed privileged setup, requiring separate approval

After those code adapters exist, first qualify only synthetic data on the RTX
host. Proposed reversible targets are:

- Create a dedicated non-login `corbanu-broker-test` system principal and a
  separate non-login `corbanu-worker-test` principal, after verifying neither
  name exists. No existing account or group membership is modified.
- Create only `/var/lib/corbanu-broker-test` for broker-owned synthetic Vault,
  journal and integrity state, and `/run/corbanu-broker-test` for native sockets;
  set narrowly scoped ownership and permissions on these new directories only.
- Install one clearly test-named systemd service and socket unit with explicit
  UID, executable, inherited-handle contract, restricted filesystem/network
  access and process-debug restrictions; keep original production services
  untouched. Record exact unit contents and native capability results before
  claiming any protection.
- Launch the real untrusted test worker separately and probe denial of Vault,
  root, config, memory/debug, inherited handles and unauthorized socket access.
  Verify service restart, generation replacement and live revocation using
  synthetic credentials. Stop/disable test units and preserve evidence on exit.

No principal creation, service installation, ownership/ACL changes or real Vault
transfer has been performed. Per-request streaming, cached TLS handlers, upload
cancellation, response filtering, macOS and Windows isolation are still separate
mandatory gates. This construction stage cannot enable protected activation.
