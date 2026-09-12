# Accepted pair-stage allocation

Manager task01a08522-76a1-7ad1-afe6-ad690d55c0d7 accepted this proposal at19:22UTC,
subsequently confirmed the receiving RTX gate and directed implementation.
Original coordinates/base remain in plan/sprint; incremental launch HEAD is
62956038e77d41dd4ee1a3130b8b8ab77558a83f. Sole owner /root, no child workers.
Remote detached implementation checkout will be
/home/travis/worktrees/security-broker-pair-20260912, separate target
/home/travis/security-round5/targets/pf27-pair-20260912. Existing build.lock lease.
No code in canonical receiving; no dependency/manifest/lock edits authorized.
Exactly eleven paths below, estimate766/hard800 changed code/test/fixture lines.
Two scoped review extensions28AstraHigh/29Fable5.1High through Corbanu/TMUX;
reserve at dispatch, retain all prior review usage. Not native/PF20 activation.
The following original proposal is frozen unedited; its proposal-only language
is historical, superseded solely by this explicit scoped allocation.

# PF27: bounded two-child ownership and admission proposal

Prepared September 12, 2026 by sole security owner /root for CorbanuManager.
Proposal only: no implementation, new reviews, native activation or privileged
installation is authorized by this document. Manager records the final literal
allocation before implementation. Keep PF-27-S04 in_progress in active
docs/plans/active/p0-security-levels.md; this is not a new sprint or initiative.

## Baseline and requirement

Accepted owner branch feat/security-broker-resume-20260911 is clean and pushed
at 62956038e77d41dd4ee1a3130b8b8ab77558a83f. Its qualified runtime is
4d830cbb31330da21cf84b6f0193e7d56dc30776 / Rust
7e488200b2cfc23ee9c99bf188ea9e76e9693558. Manager receiving integration is
c5e606fa2df329e40d731dce3fe24a846d3e2e2e / Rust
18cf961a79b4959baa005bd7f9577c04e9b798f6. The adapter and service directories
are identical between these commits; combined lock retains accounting changes.

Product heading **Non-negotiable controls**: “Permit agents to reference
credentials only by label; resolve them solely inside the trusted execution
boundary.” This next internal construction stage advances authenticated child
ownership, not credential resolution or production security eligibility.

## Existing interfaces, and the actual gap

All paths below are relative to codex-rs/.

| Existing surface | Current contract | Consequence |
| --- | --- | --- |
| secret-broker-service/src/children.rs: TrustedChildRun::capture | Takes generation, two actual std::process::Child values and their expected UIDs; returns both children on construction failure | Cannot accept adapter-owned children |
| TrustedChildRun::admit | UnixStream plus callback receiving ChildRole, stream and &mut Child; observed peer UID/PID, SO_PEERPIDFD, generation fencing and preallocated reaping | Do not replace this with numeric-PID-only authentication |
| secret-broker-service/src/root.rs: RootChildRun and private RootHandler | Delegates to ControllerRoot::serve_child(stream, &mut Child) | Root composition currently depends on the same concrete Child |
| protected-state/src/native.rs: ControllerRoot::serve_child | Checks real Child and its pidfd around authenticated bounded IPC; retains typed/ambiguous errors | Changing PF20 is a separately allocated public-interface step |
| linux-pidfd-spawn/src/spawn.rs: OwnedChild | Owns private pidfd; try_wait, terminate, eventual kill/reap on Drop; no exported PID/FD or identity predicate | Sole owner must remain inside the precreated worker |
| service src/launch/manifest/spawn.rs: Reservation, LaunchHandle | One process-wide permit, one image/backend, sticky cancellation/deadline; permit released only on positive cleanup; panic quarantined | Two independent reservations cannot form a pair; reserve once for the generation |

## Recommended smallest coherent stage

Add a **private, synthetic-only pair owner with authenticated admission receipts**.
Leave TrustedChildRun, RootChildRun and PF20 unchanged. This deliberately does
not yet service protected-root requests. It supplies the ownership/peer contract
needed for a subsequent explicit PF20 bridge, without fabricating std::Child,
exporting a raw child PID, or altering reviewed spawn/FFI semantics.

Proposed interfaces (names/signatures frozen by the eventual allocation):

1. Adapter safe method:
   `OwnedChild::is_same_process(&self, peer: BorrowedFd<'_>) -> io::Result<bool>`.
   Only compares stable kernel identities of retained pidfds. It neither exports
   ownership nor signals/reaps the supplied peer. A cached exited child cannot
   match. Identity alone is not liveness or readiness.
2. Private service `PairImages { journal, policy }`, containing two separately
   owned SyntheticProfileInspectedImage values; no raw-FD cloning escape hatch.
   `Reservation::launch_pair(images, generation, expected_uid, deadline)` returns
   a PairHandle, or both original images on pre-worker construction failure.
   The synthetic children execute as the real non-root test UID, not the fixed
   identity-preparation recipe's target IDs (101/102). Role comes from the owned
   image/slot, never a child-supplied role claim. No production UID configuration.
3. PairHandle exposes status/cancel and a bounded nonblocking
   `admit(stream) -> Result<AdmissionTicket, (io::Error, UnixStream)>`.
   Queued does not mean admitted. At most two pending requests are owned by a
   preallocated bounded queue. The sole worker validates requests between
   lifecycle polls; unsupported identity facilities fail closed.
4. AdmissionTicket has a nonblocking take-result operation: Pending, Denied,
   or AdmittedPeer. AdmittedPeer contains immutable role/generation and a private
   stream with a cancellation guard. Dropping a delivered peer or losing its
   delivery fences the pair. The worker retains a shutdown clone of every
   accepted stream. Do not expose this receipt as a PF20 authority token.

Reuse the existing reservation/worker and private Backend/Child seam, widening
visibility only to sibling manifest modules. Pass a narrow cancellation view
into pair spawning so cancellation is checked between first and second spawn.
No new public generic backend, second global permit, unbounded callbacks,
per-admission thread or caller-side kill/reap. If reuse requires a broader
refactor than the budget below, stop and return the literal delta to manager.

### Stable identity mechanism and limitations

For the initial qualified Linux x86_64 target, require both descriptors to be
pidfs descriptors and compare their stable inode identities while retained.
Use existing safe rustix fstat/fstatfs access and the platform's actual constant;
do not infer identity from fd numbers, /proc strings, numeric PIDs, or identical
anonymous-inode numbers. Older anonymous-inode pidfds and unsupported targets
return Unsupported. No 32-bit claim without its separate generation-aware path.

Primary design references:
[systemd pidfd inode identity contract](https://man7.org/linux/man-pages/man3/sd_pidfd_get_inode_id.3.html)
explains the boot-stable identity and 64-bit fstat versus 32-bit limitations;
[Linux pidfs implementation](https://kernel.googlesource.com/pub/scm/linux/kernel/git/torvalds/linux/+/0f23d56f17fdfc7db69d51f64c8b91bbab947aa9/fs/pidfs.c)
documents inode identity and poll semantics. These are design references, not
proof of the RTX kernel behavior. Positive/negative descriptor comparisons and
death races must be measured on that actual host before accepting the API.

Service admission obtains SO_PEERPIDFD and peer UID from the actual socket,
checks both child lifetimes before and after comparison, and permits one
connection per role. No numeric-PID fallback. In particular, a descriptor opened
for an unrelated process or a retained socket from a prior generation cannot
consume a replacement generation's slot.

## Required ownership invariants

- One reservation owns both pending images and every eventually published
  child, from before worker creation until both are positively reaped.
- First-spawn failure is no-launch; second-spawn failure retains, kills and
  reaps the first child before releasing the generation. Never propagate an
  error that drops its last child owner on the caller.
- Cancellation, deadline, one-child exit, failed accepted-peer delivery and
  accepted-channel teardown fence the whole pair and shut both retained streams.
- Every cleanup iteration attempts both children even if the first signal/wait
  errors. A reaped first child never releases the permit for an unreaped second.
- No mutex held across spawn/signal/wait/IPC. Caller Drop sets cancellation and
  returns; the existing worker performs cleanup. Pending tickets own no pidfd.
- A blocked spawn is not interrupted by the deadline: report CleanupPending,
  retain capacity, and kill/reap late children when the primitive returns.
- Permanent signal/wait errors retain ownership and retry. Worker panic remains
  quarantined; no timeout-based release, forced reset or readiness from exit.
- No process-crash durable supervision or native containment claim.

## Literal prospective files and changed-line ceilings

These are planning ceilings, not measured implementation counts. Addition and
deletion each count; do not compress tests to hit a target. Runtime total is
under 500 and all source/test/fixture changes must stay at or below 800.

| File (codex-rs/ unless marked QA) | Runtime/registration | Tests/fixture |
| --- | ---: | ---: |
| linux-pidfd-spawn/src/peer.rs (new safe identity helper) | 50 | 0 |
| linux-pidfd-spawn/src/peer_tests.rs (new) | 0 | 65 |
| linux-pidfd-spawn/src/spawn.rs | 10 | 0 |
| linux-pidfd-spawn/src/lib.rs | 3 | 0 |
| secret-broker-service/src/launch/manifest/spawn.rs | 45 | 0 |
| secret-broker-service/src/launch/manifest/spawn_tests.rs | 0 | 15 |
| secret-broker-service/src/launch/manifest/pair.rs (new) | 235 | 0 |
| secret-broker-service/src/launch/manifest/pair_tests.rs (new) | 0 | 260 |
| secret-broker-service/src/launch/manifest/mod.rs | 3 | 0 |
| QA: qa/security-levels/sprints/PF-27-S04/descriptor-pair-20260912/fixtures/connect.rs | 0 | 45 |
| QA: same directory/qualify-rtx.sh | 0 | 35 |
| Total estimate | 346 | 420 |

Total 766, leaving 34 changed lines contingency. No children.rs, root.rs,
protected-state, Core, Vault, login/http-client or native installer changes.
No dependency change anticipated: safe descriptor APIs and socket/poll support
already exist. If a manifest/feature/lock edge is needed, manager must allocate
that exact serialized change; do not borrow accounting's shared surfaces.
Manager separately owns canonical plan/sprint allocation and final integration.

## Actual-host test matrix proposed for this stage

These tests have NOT run; they describe the next allocation, not the accepted
single-child stage or current combined-tree proof.

| Layer | Required cases / observable proof |
| --- | --- |
| Deterministic lifecycle | Capacity reservation; worker failure returns both images; cancel before launch; first failure; second failure reaps first; second spawn delayed across cancel/drop/deadline; late publication reaped; asymmetric stop/wait errors; panic quarantine |
| Kernel identity | Two pidfds for same live process match; unrelated child fails; ordinary FD/unsupported pidfs fails; dead child cannot admit; old socket cannot match new generation |
| Pair admission | Both arrival orders; duplicate, parent, unrelated peer and wrong UID rejected; denied connection does not consume another role; generation binding; dropped pending ticket and lost accepted reply close/fence correctly |
| Pair teardown | Either child dies; either accepted peer drops; both sockets close; blocked stream read wakes; both actual children reaped before next reservation succeeds |
| Primary Linux | travis UID1001, RTX100.99.88.49, x86_64 Linux7.0.0-31, GNU2.43: real sealed static connector through existing spawn backend, both exact roles, actual socket admission plus cleanup |
| Unsupported Linux | development host GNU2.39: explicit existing unsupported-backend rejection only; not a positive pair qualification |
| macOS/Windows | Feature-OFF registration/compile only when separately assigned host/build leases are available; no Linux adapter/native/all-OS behavior claim |

Existing hold/inspect/probe fixtures cannot prove socket admission. Add the
named tiny static synthetic connector, accepting the existing fixed recipe
arguments without changing the adapter argv contract. Bind a test-only Linux
abstract socket name derived from the test parent's PID; connector reads its
PPid from bounded /proc/self/status and connects, then waits for EOF. A public
synthetic address grants no authority: owned-pidfd+UID checks remain required.
Record pinned compiler, source/artifact hashes and static ELF profile before
execution. No root operation, identity drop, credentials or protected endpoint.

Run new focused cases and the existing default3, focused-owner8, real-owner3,
static hold/inspect cases and full service suite. Capture actual resulting
counts and every skipped/ignored disposition; do not pre-label new counts pass.
Use actual TMUX keys for the lifecycle runner, with text and Enter separate.
This is internal runtime evidence, not product TUI, live-repository acceptance
or human sign-off. No new interactive surface means blind UX design is N/A here.

## Build/dependency lease and next handoff

After manager allocation, implement under the existing sole PF27 owner; no
extra agent lane. Proposed new remote target:
/home/travis/security-round5/targets/pf27-pair-20260912. Use a dedicated detached
checkout at the manager-accepted base. A target existing from receiving proof
may be reused only if exclusively leased to PF27. Never reuse accounting's
target or mutate the canonical Mac target. Builds are non-root RTX, pinned
Rust1.95.0, offline cached dependencies, initially four jobs, reduced if needed.
Use the existing remote build.lock convention for serial build execution;
manager serializes shared Cargo/Bazel registration with accounting. If new
prerequisites are missing, report them rather than modifying shared tooling.

Required completion: scoped strict feature-enabled Clippy; exact-tree tests and
actual-key TMUX; Cargo/Bazel parity preserving accounting SQLx; source/lock/tree
digests; one scoped independent review of this materially new stage under the
integrator's ledger. No repeat reviews of unchanged accepted adapter/owner code.

Next *separate* boundary after this stage: introduce a typed live-child proof
accepted by PF20's bounded root handler, preserving peer checks, death fencing
and RootError::Ambiguous; then route two-child receipts through root composition.
Do not treat the proposed role/stream receipt as that live proof. Manager must
allocate exact PF20/service files and line budgets before that work starts.
