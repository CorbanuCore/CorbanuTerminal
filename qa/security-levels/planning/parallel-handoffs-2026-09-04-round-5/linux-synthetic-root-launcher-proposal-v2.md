# Linux synthetic root-launcher installation proposal v2

**Planning only. Not an install script or permission to run privileged commands.**
Target: RTX host `100.99.88.49`, accessed as `travis`. This new proposal supersedes
the topology, paths and units in `linux-synthetic-installation-proposal.md`; that
historical document is preserved. No accounts, permissions, services, source or
plan allocations were changed. No independent reviewer was invoked (broker
ledger remains 5/5).

## Authority and exact source state

This is internal planning for product-initiative PF-27-S04, not implementation
authority. Product citation: **Non-negotiable controls**, “Permit agents to
reference credentials only by label; resolve them solely inside the trusted
execution boundary.” The **Data-rollback scope decision — 2026-09-04** trusts the
administrator/kernel/controller storage and excludes whole-machine rollback;
it does not authorize privileged deployment or credential migration.

Inputs read from the integration checkout:

- `qa/security-levels/planning/parallel-handoffs-2026-09-04-round-5/pf20-pf27-consumer-contract.md`, including the accepted coordinator disposition.
- `qa/security-levels/sprints/PF-20-S03/pf27-consumer-handoff.md`, final PF20 source `ee07e0700`, merged at integration `b12e32db398c83854271e2e70f29e5290278af8b`.
- Existing typed service is staged on preserved broker branch `cd7457da743660fe36213816fcdb7bebd91ba1ce`; it is **not present in the inspected integration checkout**. Its normal executable exits 78; its inherited-socket fixture is not the native launch path.

PF20 supplies root-only fixed-path factories and live-child authenticated anchor
channels. It does not install or launch a service. PF20 combined qualification
and archival were pending at the author's handoff and are now complete on
runtime `b12e32db3` (coordinator archive `89ad317b9`). This closes only the
bounded dependency gate, not any privileged installation or broker review gate.

## Three separate gates

| Stage | Permitted outcome after its own approval | Not established by that stage |
| --- | --- | --- |
| Source staging | Allocate PF27 launcher/composition source, integrate dependencies, compile/test on RTX, freeze reviewed binaries and actual argv/FD contracts | No system installation; unit text or mocked identities are not containment proof |
| Synthetic native qualification | Explicitly approved installation below, synthetic-only enrollment/credentials, actual separate-UID denial and recovery probes | No real Vault transfer, live provider egress, production protection or cross-platform qualification |
| Actual deployment | Separate acceptance of operating lifecycle, all required platform/data-plane gates and real-data migration procedure | Cannot be inferred from successful synthetic CAS or a root-owned socket |

## Process/principal manifest

Use **one root anchor/launcher**, two separate `ControllerRoot` instances and
distinct trusted child roles/PIDs. No same-process dual-domain requirement and
no extra PF20 endpoint are proposed.

| Principal | Process role | Groups/access |
| --- | --- | --- |
| `root` | Minimal anchor/launcher only; owns both roots, fixed listener and actual spawned `Child` handles | Never runs model/tool requests; administrative authority remains outside threat boundary |
| `corbanu-broker-test` | Journal client and synthetic Vault/broker child | Own primary group; supplementary `corbanu-anchor-test` for anchor socket connect only |
| `corbanu-policy-test` | Policy anchor client and bounded Core policy controller child | Own primary group; supplementary `corbanu-anchor-test` for anchor socket connect only |
| `corbanu-worker-test` | Untrusted bounded synthetic tool/probe child | Own primary group; **not** in anchor, broker or policy groups |
| `corbanu-anchor-test` | Socket-connect group only, no account | Contains only broker and policy test accounts; no membership for `travis` or worker |

All three accounts: unused system UID/GID selected and recorded at installation,
locked password, `/usr/sbin/nologin`, `/nonexistent` home, no home creation, SSH
key, sudoers entry or interactive login. Do not create the historical
`corbanu-controller-test` account. Recheck every account/group name and numerical
ID immediately before installation; abort on collisions rather than modifying
or adopting existing accounts. Earlier name-absence observations are not fresh
evidence for this revised manifest.

The launcher—not separate systemd broker/policy services—must spawn and retain
the actual children required by `serve_child`. Route the accepted kernel peer
PID through a trusted PID→role→namespace registry before sending the channel key.
Validate the child's configured UID, pinned executable, post-exec readiness and
boot binding. PID strings, connection order, worker role claims and inherited
parent-created socketpairs cannot authorize a channel. The policy role is not an
interactive Corbanu process holding both root domains.

## Exact proposed filesystem manifest

Modes below are requested setup, not verified controls. No existing target may
be overwritten. Root retains administrative access throughout. Regular private
files must be non-symlink, single-link files; ancestry/type checks use opened
handles. No ACL is added to PF20 private stores.

| Exact path/object | Owner:group / mode | Intended access |
| --- | --- | --- |
| `/etc/corbanu-protected-state` | root:root 0700 | Fixed PF20 registry parent; root only |
| `/etc/corbanu-protected-state/journal` | root:root 0700 | Journal enrollment/binding records, root only |
| `/etc/corbanu-protected-state/policy` | root:root 0700 | Policy enrollment/binding records, root only |
| Registry regular files created by PF20 | root:root 0600 | API creates during explicit enrollment; installer never fabricates them |
| `/var/lib/corbanu-protected-state` | root:root 0700 | Fixed PF20 store parent; root only |
| `/var/lib/corbanu-protected-state/journal` | root:root 0700 | Journal `key`, `lock`, `head`, transient `pending`, root only |
| `/var/lib/corbanu-protected-state/policy` | root:root 0700 | Policy `key`, `lock`, `head`, transient `pending`, root only |
| Anchor regular files | root:root 0600 | Exact PF20 durable CAS; never copied into broker/worker data |
| `/etc/corbanu-protected-test` | root:root 0700 | Test launch/enrollment manifest and installation ownership ledger |
| `/etc/corbanu-protected-test/launch.json` | root:root 0600 | Proposed immutable launcher config; schema still to implement, contains no real secrets |
| `/var/lib/corbanu-broker-test` | corbanu-broker-test:corbanu-broker-test 0700 | Synthetic Vault and PF41 journal only; files 0600 |
| `/var/lib/corbanu-policy-test` | corbanu-policy-test:corbanu-policy-test 0700 | Synthetic policy data only; files 0600 |
| `/var/lib/corbanu-worker-test` | corbanu-worker-test:corbanu-worker-test 0700 | Disposable synthetic probe output only; files 0600 |
| `/run/corbanu-protected-state.sock` | root:corbanu-anchor-test 0660 | **Single fixed PF20 socket**; broker/policy connect; worker denied by DAC and peer/role gate |
| `/run/corbanu-protected-test` | root:root 0711 | Test runtime parent; child traversal only |
| `/run/corbanu-protected-test/broker` | root:corbanu-worker-test 0750 | Root-created proposed broker endpoint directory; worker cannot replace entries |
| `/run/corbanu-protected-test/broker/dispatch.sock` | root:corbanu-worker-test 0660 | Proposed root-bound listening FD passed to broker; only worker group connects; broker still authenticates live worker/run |
| `/opt/corbanu-protected-test` | root:root 0755 | Immutable test deployment parent |
| `/opt/corbanu-protected-test/<frozen-full-commit>` | root:root 0755 | **Unassigned until source freeze**; no `latest` symlink or writable binary parent |
| Deployed executable files | root:root 0755 | Exact reviewed SHA-256 verified after protected copy; not executed from writable `travis` checkout |
| `/etc/systemd/system/corbanu-root-anchor-test.service` | root:root 0644 | One manual-start supervising service; complete unit remains to author |

`/etc`, `/var/lib`, `/run` and `/opt` themselves are existing system parents:
inspect, do not recursively chown/chmod them. PF20's fixed paths mean this test
**cannot coexist at these paths with another protected-state installation**.
If any fixed target exists, stop for a fresh authority decision; do not use a
test symlink, alternate path, mount redirection or permission relaxation.
Supported filesystem/kernel checks remain mandatory; the earlier ext4 observation
is availability information, not durability or containment evidence.

## Proposed unit and child hardening contract

Only one unit is proposed: `corbanu-root-anchor-test.service`. No socket-activation
unit, separate broker/policy service, automatic restart or enable-at-boot action.
The root launcher binds the fixed anchor listener itself, preserving the native
root-peer contract, and retains child handles across session supervision.

Proposed service settings: `User=root`, `Group=root`, `UMask=0077`, `Restart=no`,
`KillMode=control-group`, bounded `TimeoutStopSec=30s`, `ProtectSystem=strict`,
`ProtectHome=yes`, `PrivateTmp=yes`, `PrivateDevices=yes`, `PrivateNetwork=yes`,
`RestrictAddressFamilies=AF_UNIX`, `NoNewPrivileges=yes`,
`ProtectKernelTunables=yes`, `ProtectKernelModules=yes`,
`ProtectControlGroups=yes`, `RestrictSUIDSGID=yes`, and proposed proc restrictions
`ProtectProc=invisible`, `ProcSubset=pid`. Confirm support and actual effect on
this host; a parsed unit is not qualification.

Normal-run writable exceptions are only the two anchor storage directories,
three synthetic child-state directories, fixed socket and test runtime directory.
`/etc` registry/config stays read-only during normal run. Creating the fixed
socket under a read-only `/run` view needs an explicit, validated path exception
strategy in the final unit; do not widen all `/run` merely to make launch pass.
This is one reason no purportedly runnable unit is attached yet.

The root launcher requires narrowly justified capability handling for dropping
child identities/groups, assigning socket groups and terminating its test child
UIDs: proposed bounding list `CAP_SETUID CAP_SETGID CAP_CHOWN CAP_KILL` (no ambient
capabilities). Verify exact requirements with the implemented launcher before
freezing the unit; no `CAP_SYS_ADMIN`, global ptrace change or broad capability
set is proposed. All children must have empty effective/permitted/inheritable/
ambient capabilities, locked intended UID/GIDs, no-new-privileges and a closed
FD allowlist. Post-exec children must set and attest nondumpable state before any
sensitive bootstrap; a pre-exec setting alone is insufficient proof. Broker and
policy root-channel MACs never reach the worker. Controls still need source and
real UID/ptrace/fd-denial measurements.

## Pinned execution and privileged actions requiring approval

No real launcher binary/argv/FD schema exists yet; **ExecStart is deliberately
unassigned**. Do not substitute the existing fixture. Existing staged denial
control `/home/travis/security-round5/evidence/broker/service-6bdc84195/codex-secret-broker-service`
has recorded SHA-256
`83a47d31342df3b37c82bd409ed0db9f48af7cf80f5fdedabe364fad6db23abf`
and no-argument exit 78; that is not a launcher or a reason to start a root unit.

After source allocation/tests/reviews, final approval must name actual complete
source commit, every deployed digest, exact executable paths and supported argv,
child/control/listening FD map and synthetic owner/key identifiers. Enrollment
argv must be distinct from ordinary existing-only startup. All credential-like
test values are newly synthetic, supplied through approved private handles and
never real Vault material or argv/environment secrets.

These are **proposed privileged command classes, not commands run here**:

1. `groupadd --system` for the four exact groups above, then `useradd --system
   --no-create-home --home-dir /nonexistent --shell /usr/sbin/nologin --gid
   <matching-group>` for the three exact accounts; broker/policy creation also
   assigns only `corbanu-anchor-test` supplementary membership. Final transaction
   replaces every placeholder with validated names and records assigned IDs.
   Verify locked password status; no interactive password setting.
2. `install -d -o <exact-owner> -g <exact-group> -m <table-mode> <exact-path>`
   only for absent manifest directories. Apply no broad recursive command and
   never invoke `install -d` to normalize an existing unknown path.
3. Protected copy/install of frozen executables, `launch.json` and the one unit
   file with table modes, using a validated opened source and post-copy hash.
   Actual hashes and exact commands cannot be approved until source exists.
4. Explicit one-time invocation of the future pinned enrollment executable for
   **both** synthetic namespaces. This is a separate approval item from starting
   the service; partial enrollment stops and requires human reconciliation.
5. `systemctl daemon-reload`, then manual `systemctl start
   corbanu-root-anchor-test.service` only after exact unit/ExecStart approval.
   No `enable`, unrelated service stops, sysctl, firewall, TPM or sudoers edits.
6. Bounded root-launched probes under the three exact test UIDs and signals to
   retained test PIDs only. Fault-injection changes to synthetic anchor files
   require a separately enumerated case; routine installation does not authorize
   deleting an enrolled key, replacing a lock inode or restoring old root state.

## Idle expiry, death and recovery protocol to implement

The native ten-second frame deadline includes idle wait. It is not a lease that
may be renewed implicitly. Timeout, EOF, malformed channel or either trusted
child death must fence the affected broker/policy generation and deny dispatch.
The root handler reports expiry to supervision; consumers do not keep a stale
healthy flag until the next request. Conservatively fence the whole synthetic run
when either namespace channel fails, so a journal failure cannot leave policy
authorization apparently live, or vice versa.

Stop new worker work, revoke run/grants, settle only operations with definite
receipts and classify ambiguous operations as unknown. The current staged broker
does not prove cancellation of a future upload/stream. Drain/terminate only test
children under bounded supervision; discard old keys and native clients. Explicit
rebootstrap creates new role PIDs, child handles, channel keys and run generation,
opens existing roots and performs durable recovery before granting work. Never
re-enroll to recover, retry an uncertain CAS, automatically replay a financial
operation or hide reconnect as an unchanged session. An idle run ending after ten
seconds is an expected limitation for this stage, not a seamless-service claim.

## Qualification and evidence requirements (future approved runs)

- Positive journal and policy CAS through their distinct post-exec child PIDs,
  plus the bounded authorized worker→broker operation and exact audit settlement.
- Worker denied anchor connect and private store access; broker/policy denied
  direct root files; unauthorized same-group child denied by trusted PID/role gate.
- Wrong UID/executable/PID, wrong namespace, inherited socketpair, reused/stale
  generation, replay, malformed/oversized frame and uncertain CAS all fail closed.
- Worker `ptrace`, `process_vm_readv/writev`, `/proc/<test-pid>/mem` and FD paths,
  `pidfd_getfd` attempts target only synthetic test canaries; positive disposable
  controls demonstrate probe validity. Never inspect unrelated process memory.
- Worker filesystem read/write/delete/rename/link/symlink attempts target only
  installed synthetic objects; verify no sensitive FD leakage into worker.
- Native idle expiry >10 seconds without requests fences dispatch; controller,
  broker and policy death/restart each invalidate old capabilities. No silent
  reenrollment or retry after lost acknowledgment.
- Concurrent root start, missing/malformed/FIFO/symlink files, partial enrollment,
  durable restart and restoring older ordinary policy/journal while retaining the
  newer root need explicit disposable fault cases. Kill tests are not physical
  power-loss proof; no whole-machine rollback claim.

Record host/kernel/filesystem, full binary commit/hash, real numerical IDs and
pidfds, inputs, positive/negative expectations, actual errnos/results and sanitized
artifacts. A failed forbidden operation with an unexplained error is not a pass.
Current same-user fixture tests remain supporting construction evidence only.

## Reversible stop / cleanup / recovery

Before installation, inventory exact targets, ensure absence, record transaction
ownership, IDs/digests and disabled state. No automatic cleanup of unknown objects.
At stop: fence the run, stop new worker dispatch, settle bounded definite results,
terminate only retained children, then close both roots and the fixed listener.
`systemctl stop corbanu-root-anchor-test.service` is the named administrative stop;
inspect its exact cgroup/PIDs before escalating. Do not kill broad name patterns.

After all those PIDs are gone, unlink only recorded test socket inodes after
ownership/type checks. Preserve private enrollment/key/lock/head and synthetic
Vault/journal evidence; never restore an older root to make a test green. Normal
restart opens existing namespaces. Missing/corrupt state remains unavailable.

Unit removal, account/group deletion and persistent-state deletion need separate
exact-list approval. Prefer moving exact installed artifacts to a restricted
root-owned quarantine after quiescence; do not delete directories recursively or
reuse IDs while owned objects remain. Record daemon reload and any accidental
enablement deviation. Removing enrollment is not harmless reversibility and is
never an implicit cleanup step. Fixed paths cannot be reused for another test
until their retained authority is explicitly dispositioned.

## Exact next source scope requested, not allocated here

| Owner / proposed literal scope | Required source work |
| --- | --- |
| PF27 `codex-rs/secret-broker-service/` | Integrate preserved typed composition after review disposition; add Linux root launcher, exact synthetic enrollment/start separation, pinned child launch/role registry, fixed listener ownership, trusted bootstrap FD contract, journal NativeAnchorClient→PF41 recovery→Vault/Broker composition, supervisor fencing/idle/death/rebootstrap, and subprocess qualification helpers. Keep normal unavailable default until required gates pass. |
| PF27 `qa/security-levels/sprints/PF-27-S04/` | Versioned exact install/uninstall manifest, binary/UID/FD evidence, separate-principal denial and idle/recovery scripts, stage limitations; not an archive claim. |
| Coordinator `codex-rs/Cargo.toml`, `codex-rs/Cargo.lock`, `codex-rs/MODULE.bazel.lock`, required workspace Bazel registrations | Dependency registration/lock parity for service consumer of `codex-protected-state`; integrate preserved service source without a broker↔Vault cycle. |
| Core owner `codex-rs/core/src/security/authoritative_state_anchor.rs` plus its exact module/factory registration | Narrow policy-child factory consuming policy-connected client and full durable recovery. The existing adapter is private and no native client hookup exists; allocate any further Core files explicitly before edits. |
| PF27/PF20 coordination only | Preserve PF20 public API and checkpoint ordering. No extra endpoint, alternate filesystem/UID constructor or same-process dual-namespace extension allocated. |

Future provider request/response streaming, PF28 output protection, actual provider
egress, real Vault migration and macOS/Windows containment are separate work.
This proposal does not manufacture a passing platform report or authorize full
PF27 completion. Next approval should allocate source first; the final privileged
installation approval must wait for actual executable/argv/FD/unit pins.
