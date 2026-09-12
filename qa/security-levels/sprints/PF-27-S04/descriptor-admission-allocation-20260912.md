# PF27 Stage B allocation: bounded private socket admission

Manager-authorized September 12 after literal combined Stage A proof acceptance.
Same sole owner /root, PF-27-S04 in progress, security-broker-resume-20260911
worktree and feat/security-broker-resume-20260911 branch. Original plan/worktree
base d870c92dab2bf3fbb602dc3b8447fe9f3534aecb remains unchanged; incremental B
size/review baseline is clean qualified owner6d770938c21f002dc572b084a1b428ad4a4379b0.
No rebase is required: the combined receipt verifies these exact A crate bytes.
Owner must record this allocation in its plan/sprint and pass both governance
checks before source edits. No GitHub push, native install or activation.

Manager read the full private proposal and verified combined receiving63cbce998:
all15 actual exit files zero, source/Rust identity, ten test summaries, positive
TMUX completion and unchanged lock hashes. Strict scoped lint and Bazel parity
pass; original setup failure and Bazel warnings remain. No additional A review.

Authorize exactly two new-source reviews30 Astra High and31 Fable5.1 High via
Corbanu/private TMUX, after final B proof. Record as explicit extension preserving
reviews1–29, not reset. Hard800 changed code/test/fixture lines; estimate760,
not measured. Scope breaks return to manager before additional edits; do not
remove proof to fit. Only this internal synthetic increment receives reasoned
functional N/A; later protected-user/PF26 gates retain policy1.7 isolation.

Manager receiving owner serializes combined integration and dependency edges.
No new writer to shared registrations/locks or fourth initiative is allocated.

## Baseline and preserved work

Owner qualified A:6d770938c21f002dc572b084a1b428ad4a4379b0;
sourcee0eb9eac4/Rustf64820d0. Receiving63cbce998c036aec65bd3bed82b2ba429ac47b51,
Rust205e89cd6a2c4739f76950da15367f30e636d26d, now passes combined proof
in pf27-stage-a-receiving-63cbce998.md. Count proposed B additions AND deletions
against the actual accepted A baseline, not an intermediate implementation.
If manager requires rebase to receiving first, preserve its accounting locks
and freeze the resulting exact baseline before writing code.

Read old db971e9b8 pair.rs/pair_tests.rs and current A pair/spawn/identity seams.
The archived combined WIP supplies an unqualified starting idea, not accepted
code/tests. It had no working connector and wrongly treated invalid expected-UID
construction as a substitute for real kernel-peer UID mismatch. Its tests omit
queue-full, lost deliveries, late cancellation and blocked-read wakeup. Preserve
that history; do not import it wholesale or count old unexecuted tests as proof.

## Proposed private surface

Keep A's `Reservation::launch_pair(images, deadline) -> Result<LaunchHandle,
(io::Error, PairImages)>` and all A tests intact. Add an admitting variant using
the same reservation, worker, child owners and pair cleanup, not a second owner:

`Reservation::launch_admitting_pair(images: PairImages, spec: AdmissionSpec,
deadline: Instant) -> Result<PairHandle, (io::Error, PairImages)>`.

`AdmissionSpec { generation: NonZeroU64, expected_uid: u32 }` is trusted private
input for fixed non-root synthetic probes. `PairHandle` delegates status/cancel
to its existing LaunchHandle and offers
`admit(UnixStream) -> Result<AdmissionTicket, (io::Error, UnixStream)>`.
Requests use bounded sync_channel(2) and try_send: full/disconnected returns
the caller's stream, never waits or creates an unbounded task. AdmissionTicket
has nonblocking `take_result`; dropping a still-pending ticket cancels the pair.
No generic stream getter on the process owner, no exported pidfd/numeric PID.

Successful `AdmittedPeer` contains private role, generation, stream and shared
cancellation. It is only a synthetic admission receipt, not PF20 authorization.
Before delivering a queued success, check sticky cancellation; a lost receiver
or dropped accepted receipt fences the pair. A subsequent cancellation shuts
down held stream clones, including receipts already delivered. No claim that
identity or a once-returned receipt proves perpetual liveness or tool authority.

## Admission and teardown

The same supervisor polls at most two queued requests per iteration so producers
cannot starve cleanup. Read real SO_PEERCRED UID and SO_PEERPIDFD; fail on missing
kernel support, wrong UID, dead pidfd, no unique match, duplicate role or stale
pair. Compare retained pidfd identities through accepted OwnedChild predicate;
derive role from which owned child matched, never connection order/peer bytes.
Check child state and cancellation before and after socket/identity work.

Retain at most one shutdown clone per admitted role. Pair death, cancellation,
deadline, caller/ticket/result loss, peer EOF or socket errors shut down both
accepted sockets and fence further admissions, while both child owners remain
held until reaped. Probe socket EOF using nonblocking MSG_PEEK without consuming
application bytes. All A signal/wait retry and panic-quarantine invariants remain.
No detached request worker, new permit, new unbounded queue or blocking handshake.

## Proposed exact paths and size envelope

All Rust paths under codex-rs/secret-broker-service/src/launch/manifest/:

1. pair.rs: integrate optional admission processing with existing lifecycle;
   private construction bridge/re-exports only, no duplicate cleanup algorithm.
2. pair_tests.rs: minimal test-seam conformance and A regression retention.
3. pair_admission.rs (new): pair-nested private module, queue/ticket/receipt,
   socket admission and teardown; can access pair-private state without widening
   crate API. Private identity seam implemented only by OwnedChild/test doubles.
4. pair_admission_tests.rs (new): deterministic failure/race plus real socket tests.
5. spawn.rs: narrow private cancellation-handle access for the admitting handle;
   no change to supervisor/permit/lifecycle contracts.
6. qa/security-levels/sprints/PF-27-S04/descriptor-admission-20260912/connector.c:
   new synthetic static connector fixture, no embedded secret or external endpoint.
7. qa/security-levels/sprints/PF-27-S04/descriptor-admission-20260912/qualify-rtx.sh:
   hash-bound fixture build/profile check and final real-key qualification runner.

Requested hard800 incremental changed lines. Planning envelope: runtime/wiring
250, tests350, connector110, runner50 =760,40 contingency. This is an estimate,
not a measured diff. Stop and return concrete split options before exceeding800
or deleting tests to fit. QA/plan/sprint evidence writes stay in existing reserved
owner boundaries. No Cargo/Bazel dependencies, manifests/locks, adapter unsafe/
FFI changes, children.rs/root.rs/PF20/Core/Vault/HTTP-client/installer edits.

## Real connector and proof cases

Use a separately compiled, static, profile-accepted synthetic x86_64 Linux
fixture with fixed recipe argv. A parent-owned abstract Unix endpoint can be
derived from parent PID for rendezvous only; naming is not authentication and
guessing it cannot pass descriptor admission. Fixture emits two independent
connections per actual owned child to test duplicate roles, then waits for
bounded synthetic control/closure. Role marker is observation only. Build and
record compiler invocation, source/artifact digests and accepted ELF profile
before executing through existing sealed-image and pidfd-owned launch path.
No root/setuid or production listener. Fixture build/profile failure blocks B
real qualification instead of falling back to a dynamic/unowned subprocess.

Required distinct cases, mapped to final logs with original failures preserved:

- Both actual owned roles admitted once; reverse arrival has same role mapping.
- Duplicate connection from a real owned child rejected; same-UID parent and
  unrelated real process rejected; retained old-generation connection rejected
  after cleanup/replacement; actual dead peer pidfd rejected.
- Real SO_PEERCRED UID mismatch is exercised by the private verifier with an
  intentionally different expected UID and an actual kernel-produced socket
  credential; constructor invalid-input alone does not satisfy this case.
- Queue exactly full returns the third stream; worker held behind a deterministic
  barrier proves bounded enqueue and cleanup after release.
- Ticket dropped pending, receiver lost after result formation, accepted receipt
  dropped, caller dropped: sticky cancellation, no stale success claim, both
  children reaped, capacity not reused early.
- Cancel/deadline during queued admission and just before result delivery;
  late delivery cannot revive a cancelled generation. Preserve separate attempts.
- Either admitted peer EOF or process death fences both roles; actual socket
  shutdown wakes a blocked reader with bounded test wait. Never count only a
  coordinator-local receipt Drop as proof of remote EOF handling.
- Socket clone/credential/identity/poll failure paths do not leak ownership or
  admit peers; asymmetric child signal/wait failures retain A cleanup guarantees.
- Full A pair/owner/adapter suites, strict scoped Clippy, read-only lock parity,
  final actual-key TMUX after formatting. Unsupported-kernel negative remains
  explicit. No generic capability/PF20 or whole-product acceptance inferred.

## Review and functional applicability

Request the usual scoped AstraHigh + Fable5.1High Corbanu/TMUX extension for
new B source only after final proof; A's clean reviews28/29 are not rerun or reset.
Further source changes follow the normal finding classification/convergence rule.
This remains an internal synthetic boundary; request exact increment-only N/A
under accepted policy1.7, with later protected-user/PF26 independent isolated
functional execution still mandatory. Manager owns receiving acceptance and
portable functional harness. No new product workstream or human decision asked.
