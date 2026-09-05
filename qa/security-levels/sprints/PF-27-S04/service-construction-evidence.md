# Broker service construction checkpoint

Status: scoped construction and remote tests passed; independent service-stage
review remains pending. PF-27-S04 stays `in_progress`. Allocation `db141e9cb`;
formatted source checkpoint `6bdc84195` is pushed on `feat/security-round5-broker`
and mirrored cleanly on the authorized RTX host. No production service or
protected eligibility is claimed.

The new crate composes the actual typed Vault backend and PF-41 journal adapter
with broker runtime state. It consumes trusted, process-owned session objects
and independently expected native peer identity. Default execution remains
unavailable (exit 78), while a separate optional synthetic fixture exercises
subprocess dispatch, journal settlement, death/restart, old-key/replay denial,
wrong-peer rejection and native signal interruption. The actual six-test
subprocess suite passed. EINTR retries preserve the existing absolute deadline.

## Registration and exact final checks

The coordinator registered the workspace member/dependency in `2a6cfb3d0` and
delegated only service-caused lock deltas. Cargo added one 17-line service
package entry; Bazel lock update and check both passed with no MODULE delta.
The persistent Bazel server shut down normally before releasing the build lock.

Executed remotely after scoped fix and full formatting, serialized under the
round's build lock with fresh `service-tmp.Qj3cCA` beneath the broker evidence
root (not the previously contaminated shared TMPDIR):

1. Affected `just fix`, then `just fmt`: passed; exact scoped changes imported.
2. `just test -p codex-secret-broker-service`: **1/1** default-denial test passed.
3. `just test -p codex-secret-broker-service --features synthetic-fixture`
   **6/6** actual subprocess construction tests passed.
4. Full affected broker/Vault/proxy suites: **338/338**. Focused Core
   broker-client/config tests: **6/6**, with 2387 unrelated tests filtered out.
5. Cargo/Bazel parity passed; governance passed (2 active plans, 58 current and
   114 archived sprints).

Nextest IDs, in the same order: `4517d034-c8c7-4d2f-b3f2-3473d383c652`,
`da0c39af-1741-44ed-825d-1772599c2ff6`,
`b6e2d87e-71ca-44cf-85dd-2523588ef655`,
`a356981b-03a7-430f-923b-2dec3c4f8c0e`.
Log `/home/travis/security-round5/evidence/broker/service-check-3.log`, SHA-256
`d052a150103cde9e3eacd977b5da30c7885c08784d9faedd4ba5fc1f1e6bcfbc`.
Earlier development runs diagnosed an absent `Default` runtime configuration
and missing Nix `process` feature; both were fixed before final tests. Test
Clippy emitted unwrap-use warnings; the final fix command exited successfully.

Both binaries were rebuilt with `--locked` at the exact source and copied while
holding the build lock to
`/home/travis/security-round5/evidence/broker/service-6bdc84195/`:

- `codex-secret-broker-service`: SHA-256
  `83a47d31342df3b37c82bd409ed0db9f48af7cf80f5fdedabe364fad6db23abf`.
- `codex-secret-broker-service-fixture`: SHA-256
  `dc2cd318f005f92cb7cbd1678fbd14017d6e618bd986bbd01acbc9a61f9e4d32`.

Four prior reviews remain recorded without resetting the ledger. The
coordinator allocated review **5, Astra High**, after this immutable evidence
checkpoint, against the service diff from `db141e9cb`. It has not yet returned
a verdict. A sixth invocation is not authorized; fresh Fable service-stage
coverage awaits the user's budget decision.

See `service-qualification-boundary.md` for the absent production integrity-root
adapter, native bootstrap/privileged setup boundary and unimplemented streaming
and all-OS gates. No production credentials, principal creation, installation,
ownership changes or protected activation were used or claimed.
