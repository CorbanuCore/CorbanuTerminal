# PF27 consumer handoff — staged Linux root dependency

Coordinator accepted this exact staged composition after broker compatibility
confirmation. It supersedes the historical nonlogin-controller proposal for this
design only; actual privileged installation needs a revised manifest and explicit
approval. There is no established same-process dual-namespace requirement.

## What PF20 supplies

`codex-protected-state` is a leaf depending on existing config, audit and policy
contracts, not Core/Vault/broker. `ControllerRoot` implements unchanged PF41
`IntegrityRootStore` and the narrow `PolicyRootStore` contract. The Linux-only Core
`NativeAuthoritativeStateAnchor` copies the complete existing private anchor.
No policy-transition authorization is minted by either adapter.

`Enrollment::journal` and `Enrollment::policy` construct data only. Production
`ControllerRoot::enroll_system` and `open_{journal,policy}_system` require the actual
root principal and fixed, root-controlled directory ancestry. They never create
directories, change permissions, overwrite enrollment, or reset lost roots.

Each root owns one namespace. The registry resides in
`/etc/corbanu-protected-state/{journal,policy}`; key, lock and authenticated head
reside in `/var/lib/corbanu-protected-state/{journal,policy}`. Both are outside
agent-accessible Corbanu data. Owner/key rotation requires a separately authorized
migration; arbitrary rotation is rejected, not implemented as re-enrollment.

## Exact native seam

1. PF27's trusted launcher creates and retains an actual `std::process::Child`.
2. That post-exec child connects to the installed root-owned
   `/run/corbanu-protected-state.sock` using `NativeAnchorClient::connect_system()`.
3. PF27 accepts the stream and selects the appropriate enrolled namespace using
   trusted launch metadata, never a worker path/UID/namespace assertion.
4. It calls `ControllerRoot::serve_child(stream, &mut child)` on that root. PF20
   checks the live Child/pidfd and kernel peer PID; parent-created socketpairs
   fail this check. The child authenticates the kernel root peer.
5. A fresh channel-generation key, sequenced authenticated bounded frames, and
   absolute ten-second frame deadlines bind requests to this channel. Connection
   loss consumes the client capability; uncertain CAS never retries silently.

The API supplies no arbitrary-file/path/UID authority constructor. It also does
not prove that the selected Child is the correct broker executable, that worker
credentials are separated, or that ptrace/process/IPC restrictions exist. PF27
must establish and measure those independently. Journal and policy clients need
separately authorized channels; listener routing is intentionally PF27-owned.
An idle channel expires after ten seconds and requires explicit trusted
generation bootstrap; there is no transparent reconnect/replay.

## Required deployment approval and qualification

No privileged setup was performed in PF20. Before production use, explicitly
authorize installing the root-owned service, its fixed private registry/storage
directories, the socket ownership/mode and child-access rule, plus distinct
controller/broker/worker principal and containment policy. Authorize enrollment
as a one-time administrative action, independently of protected activation.

On the actual selected host, qualify positive authorized-child CAS/restart and
negative worker read/write/delete/rename/hardlink/symlink access to both stores;
worker socket/handshake attempts; wrong executable/child principal; ptrace and
process-memory reads; inherited descriptors/socketpairs; controller/child death;
concurrent controller starts; missing key/registry/lock/head; and restore of old
Corbanu data while retaining the newer controller root. Measure supported
filesystem/kernel behavior and crash recovery. Same-user tests are not this
two-principal evidence. Root's existence alone must not activate protection.

## Persistence semantics and limits

Exact full-value CAS publishes file-sync → atomic rename → directory-sync before
success. Invalid/torn state, permission drift and ambiguous publication latch
unavailable. Enrollment's independent durable marker distinguishes first install
from loss; incomplete enrollment requires human reconciliation, not reset.

PF41 remains record-first/root-last with its existing recovery behavior; PF20
policy remains anchor-first with exact pending-state recovery. Do not invert
either protocol. The PF41 first-record ambiguous-receipt limitation is unchanged.
The Linux prototype admits ext-family/XFS and required native APIs only; other
filesystems/platforms deny. Physical power-loss durability is not inferred from
injected sync faults. Whole-machine/controller-store rollback is explicitly
outside the user's resolved threat model, not claimed resistant.

Real Vault migration, installer/service management, full broker qualification,
protected activation and macOS/Windows adapters are not implemented here.
