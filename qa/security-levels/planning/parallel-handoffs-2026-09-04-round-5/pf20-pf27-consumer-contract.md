# PF20 → PF27 actual API consumer contract

Author/integrator composition coordination only; not an independent review or a
sixth broker review invocation. No installation, service startup, source changes,
principal changes, or native deployment qualification was performed for this
document. Proposed composition below is **not implemented**.

Inspected PF20 source: `security-local-anchor` at
`8939d50412d7798402e988afe07571fddf5867f9`, including its
`qa/security-levels/sprints/PF-20-S03/pf27-consumer-handoff.md`.
PF27 consumer: `security-round5-broker/codex-rs/secret-broker-service/`.

## Coordinator disposition — 2026-09-04 local time

Accepted as the staged native API composition: a minimal root anchor/launcher,
separate journal/policy roots and distinct trusted child roles/PIDs. This
supersedes the earlier nonlogin-controller installation proposal for this
candidate; it does not authorize running or installing a root service. Broker
and untrusted worker must not gain root. Same-process dual-domain support is
not required for this stage; no speculative extra endpoints are allocated.
The ten-second idle expiry must trigger explicit generation fencing/rebootstrap
and durable recovery, never transparent retry of an ambiguous write. Revised
exact deployment manifests and privileged action approval remain mandatory.

## Actual public boundary

| API / source | Consumer requirement |
| --- | --- |
| `store.rs:110` `ControllerRoot::enroll_system(Enrollment)` | Explicit first-install administrative action; actual euid 0; fixed root-controlled ancestry and preinstalled private namespace directories. Does not install directories or reset existing/lost enrollment. |
| `store.rs:121,129` `open_journal_system`, `open_policy_system` | Normal startup opens existing enrollment. Registry `/etc/corbanu-protected-state/{journal,policy}`; key/lock/head `/var/lib/corbanu-protected-state/{journal,policy}`. One root instance binds one namespace. |
| `native.rs:212` `serve_child(UnixStream, &mut Child)` | Launcher retains the actual spawned child and selects its namespace. PF20 checks live Child, pidfd and kernel peer PID. Launcher still owns executable/principal authorization and containment. |
| `native.rs:300` `NativeAnchorClient::connect_system()` | Child connects after exec to fixed `/run/corbanu-protected-state.sock`; socket owner and kernel server peer must be root. No public alternate path, inherited-stream, UID, or namespace selector. |
| `NativeAnchorClient` trait implementations | Supplies `IntegrityRootStore` or `PolicyRootStore` over the chosen connection, not permission to use both on a single namespace-bound channel. |
| `core/src/security/authoritative_state_anchor.rs:12` | Private Core `NativeAuthoritativeStateAnchor(Arc<dyn PolicyRootStore>)` translates the full policy checkpoint. It does not establish a launcher/client or grant policy authority. |

All source locations in the table are under `codex-rs/protected-state/src/`
unless another path is given. Storage private-directory checks reject group/other
permission bits; root-checks cannot be solved with a group ACL. Supported storage
remains the PF20 Linux ext-family/XFS prototype, not every filesystem.

## Smallest compatible proposed launcher

1. Obtain explicit authority for a minimal **root anchor/launcher** and fixed
   private storage/socket installation. The root process must not execute model
   requests or tools. Enrollment is a separate approved one-time action, not an
   automatic fallback after normal-open failure.
2. Normal root startup opens one journal root and one policy root. It binds one
   fixed root-owned listener with explicitly approved child-access permissions;
   it does not attempt to bind separate servers at the same socket path.
3. Spawn distinct approved broker and policy-controller child roles, retaining
   their `Child` handles. Establish the intended nonlogin UID, immutable binary
   identity, environment/descriptor restrictions, containment and boot binding
   in the trusted launcher. These controls need implementation and measurement;
   `serve_child` does not supply them.
4. Record **trusted spawned PID → role → namespace**. Each child connects after
   exec. Accept a connection, match it against the launch registry using kernel
   identity, and pass it plus the matching actual `&mut Child` to the selected
   root's `serve_child`. No routing from worker-provided namespace claims or
   connection arrival order. Keep exclusive ownership of the child handle while
   serving; supervise exits without replacing it with an arbitrary numeric PID.
5. In the broker child, construct its journal-connected `NativeAnchorClient`,
   wrap it as `Arc<dyn IntegrityRootStore>`, construct/recover the PF41 journal,
   then construct `JournalBrokerAudit` and the existing typed `BrokerService`.
   Keep PF41 record-first/root-last recovery unchanged.
6. In the separate policy child, construct its policy-connected client and wire
   it through the Core-owned private anchor factory. Keep policy anchor-first
   pending-state recovery unchanged. Native anchor availability alone must not
   create `ProtectedModeAuthorization` or an affirmative containment report.
7. Treat channel failure as consumed authority. The native ten-second absolute
   frame deadline includes idle waiting: long-idle channels expire. A proposed
   lifecycle manager must explicitly fence the affected service generation,
   recover durable state, and bootstrap a fresh authorized generation/channel;
   it must not transparently replay an uncertain compare-and-store.

## Concrete incompatibilities / required decisions

- The earlier `linux-synthetic-installation-proposal.md` uses a nonlogin
  `corbanu-controller-test` principal and test-specific paths. Those cannot host
  these public PF20 system constructors. Preserve separate nonlogin broker and
  worker roles, but either authorize the minimal root anchor/launcher above or
  allocate an explicit PF20 ownership/API change. Do not silently substitute
  paths, relax modes, or grant root to the broker/worker.
- A single fixed listener **can** serve both namespaces when distinct child roles
  identify them. A **single child PID needing both namespaces** cannot select
  them through two indistinguishable `connect_system()` calls with the current
  public API. Choose separate roles or explicitly allocate a trusted namespace/
  channel-routing API. Peeking at private protocol requests is not a public seam.
- The existing synthetic fixture uses an inherited parent-created socketpair;
  it is not the required post-exec native-client path and cannot demonstrate this
  composition or separate-principal containment.
- Ten-second idle expiry makes a permanently idle broker/policy client unsuitable
  without an explicit lifecycle design. No keepalive, reconnect protocol, or
  two-namespace selector is present in the public API.

## Still absent; next source work versus system approval

PF27's normal executable still exits **78**. Missing source includes the root
listener/launcher, trusted child-role registry and identity gate, authorized
bootstrap channel into each child, journal client factory, Core policy client
hook, explicit channel-generation recovery, and measured containment-report
construction. `BrokerService::serve(TrustedSession)` already composes typed
Vault/PF41/runtime dependencies but cannot derive those trusted inputs itself.
Real Vault material migration and provider response data-plane isolation remain
outside this staged composition; the fixture does not substitute for either.

The next source allocation can implement the launcher/role-routing and bootstrap
contract with synthetic credentials and fail-closed production defaults after
the process-role decision. Workspace registrations and Core factory changes need
their owning scopes. Actual root installation, fixed directory creation, socket
permissions, account/ACL/unit changes and enrollment require separate approval.

Qualification after that approval must demonstrate both authorized namespaces,
wrong-child/UID/executable denial, inherited-socket and descriptor denial,
worker storage and process-memory/ptrace denial, idle expiry, death/restart,
ambiguous publication and exact durable recovery. No such system-level evidence
is claimed by this contract. Whole-controller-store rollback remains outside
PF20's stated resolved threat model; restoring only ordinary Corbanu state must
still be checked against the retained newer root.
