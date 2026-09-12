# PF-20 native protected integrity root — allocation proposal

Date: 2026-09-04. Read-only design, not implementation or qualification.
Prepared by provenance lane for the integration owner. PF-30-S01 source remains
frozen at `2a4fb5857`, final evidence `e890ae4a9`, review ledger 5/5 unchanged.

## Recommendation and the decision that cannot be hidden

**Resolved by Travis Good, 2026-09-04:** protect against restored older Corbanu
data; whole-machine snapshot rollback is not required. The local controller
design below is selected under its explicit trusted-host assumptions.
PF-20-S03 allocates implementation separately. Missing/corrupt protected state
still denies; elevated deployment and destructive recovery are not authorized.
The options/questions below remain historical design rationale, not open
product blockers or a requirement to add TPM/off-host machinery.

Implement a Linux-only **controller-owned anchor service plus authenticated
client capability**, not a file-backed `IntegrityRootStore::new(path, uid)`.
The service is separate from the broker/worker and owns the durable checkpoint
namespace outside their writable state. Normal service startup is open-existing
only. A separate administrator-authorized enrollment operation creates a new
namespace exactly once. Reuse PF-41's journal and PF-20's policy-state algorithms;
do not build another event log or weaken either activation gate.

The first implementable guarantee is resistance to an untrusted worker restoring,
deleting or changing journal/policy files while the trusted controller root
survives. A root-owned file, HMAC, second partition or local enrollment marker
cannot detect an administrator/offline restore of **all** trusted anchor state
and its key together. The architecture says the agent must not roll authority
back; the platform contract also mentions old snapshots. Therefore root must
record this threat-model distinction before allocation/acceptance. Do not label
whole-host snapshot resistance as implemented. If that stronger requirement is
mandatory now, select and qualify a genuinely independent monotonic authority
(off-host controller service or hardware-backed monotonic state) first. TPM
presence, persistence semantics, write limits and availability have not been
inspected; adding a nominal TPM API is not sufficient.

Suggested user decisions, in order:

1. Approve a Linux-only staged adapter under trusted kernel/administrator/storage
   assumptions, with worker/journal rollback protection and **explicitly no
   whole-controller-disk snapshot protection**, or require an off-host/hardware
   monotonic anchor before this lane can claim production qualification.
2. Approve only synthetic, reversible privileged setup on the RTX host once the
   source and exact installation manifest are reviewed. No real Vault migration
   or protected-mode activation is part of that approval.
3. Agree that lost enrollment/key/root requires human recovery, never automatic
   reenrollment. Keep protected dispatch blocked and preserve evidence; destructive
   reset or backup restore requires a separate explicit decision.

This distinction is an explicit product decision, not permission to narrow the
existing platform guarantee silently. Until resolved, a local-only candidate may
be implemented/tested as a staged dependency but cannot claim the full old-snapshot
acceptance gate.

No additional question is needed to choose buffer sizes or implement CAS tests.
No privileged action has been taken in preparing this proposal.

## Source findings and reuse

Inspected provenance worktree `e890ae4a9` and broker worktree
`6bdc84195cbfc2dfcee477be73ee9c22cc313aa5` (the latter contains the new service).
Paths below are relative to the recorded CorbanuTerminal worktrees.

| Existing source | Actual contract and implication |
| --- | --- |
| `codex-rs/security-audit/src/journal_types.rs:112` | `IntegrityRootStore::load` permits `None` only for authorized first install; `compare_and_store` requires exact old checkpoint and durable success. There is no production implementation. |
| `codex-rs/security-audit/src/journal.rs:339` | PF-41 writes/syncs/publishes the event first, advances root last, then acknowledges. Only root `Timeout` maps to `CommitUnknown`; definite errors have distinct outcomes. |
| `codex-rs/security-audit/src/journal_recovery.rs:258` | Journal startup validates owner/key/checkpoint and history. Existing explicit ambiguous-event reconciliation grants no dispatch permit. Reuse this recovery, never replay effects from an anchor client retry. |
| `codex-rs/core/src/security/authoritative_state.rs:71` | PF-20 defines another external CAS trait, but no native anchor provider. Its types/trait are Core-private. Unix uid/mode validation is expressly defense in depth, not containment. |
| `codex-rs/core/src/security/authoritative_state.rs:183` | Policy activation advances the anchor **before** immutable state/intent/commit files. Its anchored-pending recovery handles that order; do not change it to PF-41's reverse order. |
| `codex-rs/config/src/security_state.rs` | Reuse `AuthoritativeStateOwner`, schema/revision validation, monotone owner and authority generations, forward-only `recovered_successor`. Do not introduce another policy-state schema. |
| `codex-rs/secret-broker/src/platform_contract.rs:166` | The public report validator checks shape, target/probe strings, freshness and ten observations. It is not an OS probe or authenticated transport; trusted construction must supply independent observations and expected identities. |
| `codex-rs/secret-broker/src/linux_transport.rs:29` | Existing kernel peer observation is useful. A socketpair made by the parent retains parent credentials; it cannot authenticate a post-exec child merely by supplying its PID string. |
| `codex-rs/secret-broker-service/src/linux.rs` | `TrustedSession` and `BrokerService` consume trusted composition inputs; they do not establish privileged bootstrap. Normal binary still exits 78. |
| `qa/security-levels/sprints/PF-27-S04/service-qualification-boundary.md` | Requires real durable root and launcher; synthetic fixture only uses volatile root. Existing account/process isolation remains unqualified. |

Read archived PF-20-S02 and PF-41-S03 and their evidence. Both completed
foundations explicitly defer the native protected adapter. PF-20 excluded OS
mechanism selection; PF-27 owns process/access controls. This follow-up must not
retroactively mark their fixtures as native proof. The authority reference is
**Reconciled security scope — TO BUILD**, “Unknown or unsupported protected paths
fail visibly rather than falling back to raw secrets or unscreened execution,”
and the platform/authoritative-state acceptance section of
`docs/plans/security-architecture-refinements-2026-08-28.md`.

## Smallest concrete construction contract

### Identities and namespaces

- Trusted controller/anchor process uses a dedicated non-login principal,
  distinct from broker and untrusted worker. A narrowly scoped root-owned
  launcher installs/starts it. It does not run model code or spawn tools.
- Root-owned, worker-immutable enrollment metadata binds a random installation
  ID, stable controller target identity, exact producer identity, owner generation,
  integrity-key ID and algorithm, namespace kind (`journal` versus `policy`),
  permitted executable identity and storage location. Key bytes live only in
  controller-readable protected storage. A key ID is correlation, not authority.
- Journal capability authorizes exactly one namespace and existing `JournalOwner`.
  The service checks every checkpoint's owner/key/schema, sequence and nondecreasing
  policy/run generations. Initial sequence is one; normal successor is exactly
  previous sequence plus one with checked overflow. Changing key/owner or rebinding
  a namespace is not an ordinary CAS; defer rotation until an explicit migration
  protocol is allocated. Existing owner mismatch continues to deny.
- Bind target identity separately from per-boot identity. Installation survives
  reboot; each controller/broker boot and channel gets a fresh random generation
  and fresh channel secret. Old capabilities cannot reconnect after replacement.
- Bounded records/frames before allocation; a proposed 16 KiB maximum checkpoint
  frame is ample for current bounded fields and must be checked against actual
  serialized maxima before freezing. Reject unknown schema/fields and unexpected
  descriptors. Do not let worker-provided paths or UID strings select a store.

### Enrollment is not `load(None)` on ENOENT

1. A separately invoked setup command requires the authenticated administrator
   operation, verifies the exact new namespace and enrollment identity, and
   refuses any existing/reused namespace. Ordinary broker/control RPC has no
   `create`, `reset`, `delete`, `set_owner` or `restore` operation.
2. Persist an enrollment-in-progress record under the independent setup registry,
   sync it and its directory; create key/store and a authenticated `Genesis`
   checkpoint; sync all new files/directories; finally mark enrollment complete
   durably. Do not issue a usable native capability before completion. Interrupted
   enrollment remains unavailable and needs exact human reconciliation, not
   blind rerun with new keys.
3. `load == None` maps only to the intact, authenticated `Genesis` state of this
   independently authorized enrollment. A completed root at sequence one or
   later never turns into `Genesis`. Missing namespace/registry/key, wrong key,
   malformed data or incomplete enrollment returns an error. Missing registry
   at normal startup is an error too, not evidence of a fresh host.
4. Once the controller checkpoint advances, startup opens that existing protected
   identity only. A separate persistent registry detects loss of the anchor
   directory, but is not represented as an off-host antirollback witness.

### Durable exact CAS

- One controller service owns the namespace and takes a lifetime exclusive lock
  on a fixed, protected lock inode; duplicate service startup refuses. An in-process
  mutex serializes each namespace's operations. Neither a process mutex alone
  nor locking the replaceable checkpoint inode is sufficient.
- Before CAS, open and validate the current persisted checkpoint under that lock;
  compare the **complete typed checkpoint**, not only sequence or hash. Do not
  promote a cached value after I/O failure. Validate the authenticated envelope,
  namespace/owner/key binding and successor constraints before writing anything.
- Write the complete next authenticated checkpoint into an exclusive, no-follow
  temporary file in the same protected directory; check every write; sync the
  file; atomically replace the head through directory-relative operations; sync
  the containing directory; only then return success. No background flush or
  queue acknowledgment. Enrollment and parent-directory creation need their own
  directory syncs. File sync alone does not persist a new directory entry.
  [Linux fsync contract](https://man7.org/linux/man-pages/man2/fsync.2.html).
- Resolve from a trusted directory descriptor with `openat2` containment/no-symlink
  constraints; validate the opened object's identity/type/owner/mode/link count
  and approved mount, not a prior pathname stat. Require the selected Linux API
  and filesystem semantics; unsupported cases deny rather than silently fall back.
  This is defense in depth inside the separately enforced principal boundary.
  [Linux openat2 contract](https://man7.org/linux/man-pages/man2/openat2.2.html).
- Definite stale CAS -> `Conflict`; malformed/binding mismatch -> `Invalid`;
  absent key -> `MissingKey`; proven no-publish I/O denial -> `Unavailable`.
  A failure after publication may have committed, including directory-sync failure
  or lost acknowledgment: latch the namespace/channel unavailable and surface
  an **ambiguous** error, not success or an automatic rollback. Existing PF-41
  represents ambiguity by `Timeout`; document that mapping explicitly or obtain
  a separately allocated additive ambiguity variant before changing its enum.
- Never retry a possibly committed write as a new operation. After reopening,
  service can report the exact durable root; PF-41 recovery decides next steps.
  A journal record beyond root is ambiguous, root beyond journal or wrong digest
  denies, unresolved dispatch remains unknown until existing reconciliation.
  The first-record ambiguous case is especially important: current reconciliation
  requires an existing root, so no new automatic first-record repair is promised.
- A torn temporary file never becomes authority. Reject a torn/corrupt published
  head; do not fall back to a valid older slot or choose a convenient highest file.
  Preserve failure artifacts. Hardware lying about completed writes and whole
  trusted-store snapshot restoration remain outside the initial qualified claim.

### Native bootstrap, not a deserializable claim

- Create a non-serializable `LinuxAnchorCapability` only inside the trusted
  launcher/client factory after observing actual peer credentials and matching
  the authorized launch registry, installation, executable and owner binding.
  Do not expose `from_file(File)`, `from_uid(u32)`, `from_report(json)` or an
  environment-variable bypass that grants protected ownership.
- Spawn/track the actual child with a stable process handle (pidfd on the selected
  Linux target), observe exec completion and credential/namespace identity, then
  have that child establish its authenticated connection **after exec**. Match
  kernel-observed peer PID/UID to that live launch entry and challenge bound to
  the channel. A pre-fork socketpair peer describes its creator, not the child.
  [Linux Unix socket credentials](https://man7.org/linux/man-pages/man7/unix.7.html).
- Keep bootstrap key off argv/environment, workspace, worker-readable config,
  logs and inherited worker handles. Only the trusted broker/control component
  receives its capability. Close all unintended fds and prevent ptrace/process
  memory/fd duplication from the actual worker principal; test these denials.
- Reuse existing broker binding/MAC/transcript and expiry/revocation semantics at
  their consumer boundary. A disconnected or replaced controller/broker cancels
  that generation; stale fd, PID reuse, replay, wrong executable and wrong
  namespace fail before CAS. Worker traffic has no enrollment or anchor endpoint.
- Avoid a bootstrap cycle: a private **root-ready** observation is narrower than
  `ProtectedModeAuthorization`. Establish/test storage using the administrator's
  setup authority first, then feed real independently observed protected-store
  evidence into the full PF-27 report. Never invent all-supported capabilities
  merely to construct an anchor. All ten real checks remain mandatory before
  `BrokerService` protected activation; this adapter alone cannot supply them.

## Allocation and dependency proposal

Create a new PF-20 follow-up sprint, not a PF-30-S01 extension. Prerequisites
PF-20-S02, PF-41-S03 and PF-27-S03 are archived; the new adapter must **not depend
on unfinished PF-27-S04**, whose broker consumes it. Native launcher qualification
is a separate PF-27-owned integration gate so the map does not form a cycle.
Root determines ID, exact worktree/base and readiness before any source edit.

| Proposed exact surface | Owner / scope |
| --- | --- |
| New `codex-rs/protected-state/src/{lib.rs,linux.rs,checkpoint.rs,enrollment.rs,bootstrap.rs}` and focused tests | PF-20 adapter lane; private native service/namespace capability and typed audit adapter. Small new leaf crate avoids Core -> broker -> Vault/Core cycles. No generic arbitrary-file privilege API. |
| New `codex-rs/protected-state/src/bin/corbanu-protected-state.rs` | Same lane only after explicit launcher/service contract allocation. Normal start is open-existing; enrollment is separate authorized mode, never worker dispatch. |
| `codex-rs/core/src/security/authoritative_state.rs` plus new `authoritative_state_anchor.rs` / tests | Coordinator-assigned PF-20 narrow wrapper around existing private anchor trait. Preserve config types and anchor-first policy protocol; no Core client/session edits. |
| `codex-rs/security-audit/src/journal_types.rs` | Read-only initial target. Reuse trait; any new ambiguity variant needs coordinator-owned exact API allocation and consumer tests. |
| `codex-rs/secret-broker-service/src/linux.rs`, new native launcher integration files | PF-27 broker owner, not concurrent PF-20 edits. Consume constructed capability to build `ReferenceJournal`/`JournalBrokerAudit`; retain exit-78 gate until complete bootstrap exists. |
| `codex-rs/secret-broker/src/platform_contract.rs`, platform probes and service/install assets | PF-27 owner with root coordination. No weakening required capabilities or treating a shape-valid report as evidence. |
| Workspace Cargo/Bazel manifests/exports/locks, active plan, new sprint/index | Integration owner only; register leaf dependencies after checking actual DAG. Suggested leaf depends on security-audit/config/policy and existing crypto/OS crates, never Core or Vault. |
| New `qa/security-levels/sprints/<allocated-PF20-follow-up>/` | New lane evidence, separate review ledger authorized by root; old S01 ledger never resets. |

The crate name/path is a proposed exact allocation, not an instruction to create
it before approval. Reuse existing SHA/crypto/OS dependencies and AbsolutePathBuf
where appropriate after reading the path-types skill; avoid speculative schema
exports solely to make private Core types public.

## Qualification matrix and setup approvals

Unprivileged synthetic tests can prove algorithms, not protection. Required tests:

- Genesis enrollment success, interrupted enrollment at each durability boundary,
  missing registry/key/head after install, foreign/wrong-key state and reinit refusal.
- Exact full-checkpoint CAS, sequence overflow, stale/cross-owner/key/namespace
  attempts, concurrent threads/processes, duplicate service lock contention.
- Every short write, ENOSPC, permission and fsync failure; crashes before/after
  rename and acknowledgment; torn temp/head; reply loss; no permit on ambiguity.
- Restart recovers every acknowledged root. Roll back/delete/truncate the
  separately stored journal/policy state and verify root mismatch denies. Explicit
  negative demonstration that restoring the entire trusted authority snapshot is
  not detected by a local-only implementation, rather than a false green test.
- Real PF-41 adapter append/recover/reconcile with synthetic events and existing
  PF-20 interrupted-anchor recovery, without editing their protocols.
- Native post-exec peer, wrong PID/UID, stale generation, disconnected channel,
  inherited socketpair negative control, malicious bounded frames and replay.

Privileged qualification must use a real separate worker principal and the real
launcher. At least trusted-versus-untrusted two-principal proof is required;
recommended deployment uses three roles (controller, broker, worker), exercising
each boundary. The current interactive `travis` account is not proof of a
restricted worker, particularly if it can elevate.

Request explicit approval for only these new, synthetic targets, after checking
none already exists: `corbanu-controller-test`, `corbanu-broker-test`, and
`corbanu-worker-test` non-login principals; `/etc/corbanu-protected-test` enrollment
manifest; `/var/lib/corbanu-controller-test` anchor/key; separate
`/var/lib/corbanu-broker-test` synthetic journal/Vault; `/run/corbanu-protected-test`
native sockets; clearly test-named systemd units and exact pinned test binaries.
If broker's approved setup already supplies a target, inspect/reuse it with its
owner rather than creating duplicate accounts or changing permissions broadly.
Do not add users to sudoers, persist elevation passwords, alter existing services,
copy real credentials, change production network rules, or disable host-wide
security. Proposed namespace/cgroup/seccomp/network/debug restrictions need exact
unit/launcher review and approval alongside these paths.

Prove genuine worker denial of read/write/delete/rename/relink/restore, config and
key access, `/proc`/debug/fd theft, unauthorized IPC and retained handles. Include
positive authorized operations to discriminate broken probes from protection.
Run service death/restart, ack loss and generation revocation. Process-kill tests
are not power-loss durability proof; abrupt VM/device power-cut testing needs an
additional disposable target and explicit approval. Preserve secret-free evidence,
stop/disable only test units afterward, and obtain separate approval before deleting
material setup. User TUI remains visibly unavailable until actual composed PF-27
gates pass; supporting actual-key TMUX smoke is not protected activation proof.

## Read-only RTX discovery and stronger-anchor options

Inspection on the allocated RTX host used the existing SSH master as `travis`,
without sudo, device opens, TPM commands, module loading or installation:

| Observation | Result |
| --- | --- |
| Kernel | `Linux 7.0.0-30-generic` |
| Service tooling | `/usr/bin/systemctl`, systemd `259 (259.5-0ubuntu3.4)`; `systemd-run` and `unshare` present |
| `/sys/class/tpm` | Root-owned; `tpm0` symlink present under ACPI `MSFT0101:00` |
| `/dev/tpm0` | Character device, owner `tss`, group `root`, mode `0660` |
| `/dev/tpmrm0` | Character device, owner/group `tss:tss`, mode `0660`; current `travis` has neither read nor write access |
| `tpm2_getcap`, `tpm2_nvread` | Not found on this SSH session's PATH; neither was executed |
| `/var/lib` filesystem magic | `stat -f` reports `ext2/ext3`; this does not establish exact filesystem options or crash durability |

These are availability observations only. No TPM public/private state, keys, NV
indices, owner authorizations or secrets were read. No capability claim follows.

| Option | New decisions and evidence before qualification |
| --- | --- |
| Controller-local durable CAS | Approve the limited trusted-host/worker threat model explicitly; create isolated principals and enforce actual containment. Prove journal/policy rollback denial, durability, startup and independent enrollment. Cannot pass a whole-anchor snapshot restoration test. |
| TPM-bound monotonic witness | Approve narrowly scoped TPM tooling/setup and a specifically allocated new NV resource only after a public-capability audit. Do not clear or reprovision TPM or reuse an unknown index. Specify counter/checkpoint-digest binding and crash/commit protocol so restored disk state cannot match an advanced hardware witness; fail closed on counter/key mismatch, TPM loss/clear or unavailable auth. Measure allowed write rate, endurance, latency and cross-store ambiguity against per-dispatch root writes. Device presence supplies none of this proof; software-emulated TPM snapshots may share the disk rollback domain. |
| Independent off-host anchor | Select a separately administered host/service outside the backup/rollback domain, authentication/signing-key ownership, durable CAS and backup/restore policy, latency/availability budget and human recovery authority. Prove local full-snapshot restore is rejected by that authority and remote replacement/rollback cannot reset it. Network partitions must withhold dispatch, not use a local fallback. No new remote service or credential setup is authorized by this document. |

The existing API permits an independent implementation without changing the
journal's root-last protocol. A TPM counter alone is not a checkpoint store, and
ordinary Keychain/DPAPI encryption alone is not a monotonic witness. Select the
full primitive and recovery protocol before claiming the stronger guarantee.

## Other platforms and honest endpoint

First implementation is explicitly Linux-only. macOS needs a separately identified
launchd helper, audit-token/code-requirement-bound XPC, helper-only key/store and
qualified durability/rollback primitive; ordinary login Keychain or same-user
files alone do not implement this service contract. Windows needs a service-SID
owned store and key, native pipe/token binding, restrictive handle sharing and
reparse/ACL checks plus qualified durable replacement. Existing Core non-Unix
persistence intentionally rejects, so this adapter cannot enable Windows by
accident. These remain candidate mechanisms from the repository platform design,
not freshly qualified OS implementations.

Expected deliverable is a real native durable-root capability for the approved
Linux threat model, tested as a broker dependency, not “protected mode enabled.”
Remaining network/worker/bootstrap/platform/screening gates stay visible. No new
source, builds, reviews, setup or production acceptance resulted from this design.
