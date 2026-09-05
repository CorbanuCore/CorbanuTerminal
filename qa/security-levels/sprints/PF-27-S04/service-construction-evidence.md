# Broker service construction checkpoint

Status: authored, not yet compiled or qualified. PF-27-S04 stays `in_progress`.
Allocation `db141e9cb`; source checkpoint `cde769497` is pushed on
`feat/security-round5-broker` and mirrored on the authorized RTX host.

The new crate composes the actual typed Vault backend and PF-41 journal adapter
with broker runtime state. It consumes trusted, process-owned session objects
and independently expected native peer identity. Default execution remains
unavailable (exit 78), while a separate optional synthetic fixture exercises
subprocess dispatch, journal settlement, death/restart, old-key/replay denial,
wrong-peer rejection and native signal interruption. These tests are authored,
not claimed as passing. EINTR retries preserve the existing absolute deadline.

## Pending build registration

The coordinator owns shared Cargo/workspace/Bazel/lock registration. The exact
requested workspace delta is adding member `secret-broker-service` and workspace
dependency `codex-secret-broker-service = { path = "secret-broker-service" }`.
The crate's own Cargo.toml and BUILD.bazel are already committed. Without member
registration, inherited workspace dependencies cannot be loaded by Cargo.

Next remote checks, serialized under the round's build lock with fresh TMPDIR:

1. Affected `just fix`, then `just fmt`, importing only scoped formatter changes.
2. `just test -p codex-secret-broker-service` (default unavailable binary).
3. `just test -p codex-secret-broker-service --features synthetic-fixture`
   (actual subprocess construction fixtures).
4. Full affected broker/Vault/proxy suites and focused Core regressions.
5. Coordinator-owned Cargo/Bazel lock parity, then governance.

The review ledger remains four of five consumed. No additional reviewer has
been invoked. The new substage needs independent review; the coordinator must
allocate the final invocation and explicitly resolve any conflict between one
remaining invocation and demanding both fresh Astra and Fable reviews.

See `service-qualification-boundary.md` for the absent production integrity-root
adapter, native bootstrap/privileged setup boundary and unimplemented streaming
and all-OS gates. No production credentials, principal creation, installation,
ownership changes or protected activation were used or claimed.
