# PF27 proposal — one owned child launched from the inspected sealed image

**Proposal only; implementation and invocation not yet allocated.** Parser
checkpoint `406aa3c5f58af0f3422558545dc821a64f777e6b` is verified on main;
its window is released. Manager accepted Fable22's corrected evidence gap,
preserving the original exit1/P3. Runtime tree after test repair:
`2edd1107d9590195956a30cf2b6a84c6cfb7c0f0`.

PF-27-S04 remains `in_progress`, sole owner `/root`, worktree
`/Volumes/CorbanuDrive/Corbanu/worktrees/security-broker-resume-20260911`, branch
`feat/security-broker-resume-20260911`, original base
`d870c92dab2bf3fbb602dc3b8447fe9f3534aecb`; incremental baseline406aa3c5f.
Product heading **Non-negotiable controls**: “Permit agents to reference
credentials only by label; resolve them solely inside the trusted execution
boundary.” Existing PF27 initiative, no new product authority or release claim.

## Next useful construction increment

Connect one profile-inspected image to an owned child-launch result, without
exposing arbitrary commands or activating a service. The previous increments
inspect and hold bytes but never bind an actual process to them. This stage
must prove that binding and child ownership together, not merely add a public
Command factory. Keep the full broker pair, admission/listener wiring and
privileged service installation out of this allocation.

Proposed literal source boundary, all beneath
`codex-rs/secret-broker-service/src/launch/`:
`manifest/sealed.rs`, `manifest/mod.rs`, new `manifest/spawn.rs`, new
`manifest/spawn_tests.rs`, and `mod.rs` for a private recipe construction seam.
QA under `qa/security-levels/sprints/PF-27-S04/descriptor-launch-20260912/`,
this proposal and existing plan/sprint/review ledgers. No lib export, probe-mode,
Core/Vault/PF20/children.rs, graph/lock/default or dashboard change without an
exact new seam decision. Target under500 runtime and800 total nonmechanical.

## Engineering decision to agree before coding

Preserve `forbid(unsafe_code)` and an actual retained `std::process::Child`.
Recommended Linux-specific route to investigate within this allocation:
`Command` uses the held memfd through `/proc/self/fd/<owned-fd>`, keeping the
descriptor alive through spawn and CLOEXEC at successful exec. This is a
procfs-assisted descriptor binding, **not** an execveat syscall claim. Do not
reopen the original deployment path, clear CLOEXEC, use a shell, or expose that
path/descriptor/Command to callers. Verify the pinned Rust spawn behavior and
trusted procfs assumptions before implementation; ambiguity returns to the
manager, not a fallback to the original path or unsafe pre_exec callback.

An alternative is a separately reviewed descriptor-exec adapter with a different
process-ownership abstraction. That changes the source/API boundary and is not
silently included. The manager should accept or amend the recommended route.

## Proposed ownership and proof contract

- Consume the opaque profile-inspected image; use only its validated role recipe,
  fixed non-secret argv, empty environment, `/` cwd and null stdio. No caller
  supplied executable, environment, arbitrary FD map or argument vector.
- Validate prerequisites before spawning. Preserve actual-root checks for the
  future privileged path. A private test seam supplies test-owned images only;
  it is not a public alternate authority constructor.
- On success retain the actual Child and stable pidfd; do not publish a PID as
  ownership. Define bounded polling/stop and cleanup ownership before returning
  an object. Any failure after spawn must retain/reap the child, not detach it.
  Reuse the existing cleanup principles; no new thread allocation from Drop.
- Agree launch deadline handling, reaper startup failure and ownership transfer
  explicitly; a timed-out caller is not proof the child never launched. If this
  cannot fit the proposed coherent boundary, report measured scope before coding.
- Freeze tests for successful binding, source-path replacement, sanitized spawn
  failure, missing procfs, invalid/lost descriptor, early child exit, pidfd failure,
  timeout/late spawn completion, cancellation/drop, and exact argv/env/stdio.
  Prove cleanup without PID reuse or secret-bearing output. Do not drop cases
  merely to fit the line target.

## Explicit invocation decision requested

Unlike the completed parser, meaningful proof now requires executing a test
artifact. Request **non-root, isolated synthetic invocation only**, using the
recorded static probe and existing modes after inspecting their exact behavior;
record full hash, argv, profile, environment and expected denial/observation
before running. No new probe mode, root-positive run or installed executable.
If long-lived positive cleanup requires another fixture, return its exact
source/command proposal first. Current permission remains no invocation.

After acceptance: RTX serialized fix/format, focused/default/synthetic/affected
checks, actual-key TMUX evidence, and separately allocated necessary reviews.
No reviews are started or reserved by this proposal. This internal stage has no
product UX or human-readiness claim. Native/all-OS containment, runtime dependency
or CVE suitability, aggregate supervisor budgets, full identity/channel mapping
and privileged installation remain open even if a non-root launch works.

## Read-only feasibility amendment — September 12, 2026

This amendment supersedes the recommendation to proceed with stock `Command`
without further qualification. Preparation is complete; the proposed implementation
is **not yet technically qualified**. No artifact was invoked, fixture written,
build started, review consumed, service installed or main push performed here.
This is an internal proposal amendment under the existing PF-27-S04 allocation,
not a product behavior change or human-test readiness claim.

### Pinned spawn implementation and limits

The repository pins Rust 1.95.0 with `rust-src`. Read the installed RTX source:

`/home/travis/.rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/std/src/sys/process/unix/unix.rs`

SHA-256, rechecked during this amendment:
`cabba94153bcdae7674f5743886d518db848710aee0ee20abfecf8bb159d3c4b`.
Toolchain commit: `59807616e1fa2540724bfbac14d7976d7e4a3860`.
[Pinned upstream source](https://github.com/rust-lang/rust/blob/59807616e1fa2540724bfbac14d7976d7e4a3860/library/std/src/sys/process/unix/unix.rs).
Line references below identify that installed source, not web-rendered lines.

- Lines 55–166: `spawn` first tries `posix_spawn`, then may fork. In the fork
  path the parent waits on an exec-error socket before returning the internal
  process handle. The read loop at 137–161 has no caller deadline or cancellation
  parameter. Moving this operation to a worker bounds caller waiting only;
  it does not bound the underlying spawn or expose the pending child to callers.
- Lines 278–410: child setup duplicates configured stdio onto 0/1/2, applies
  configured cwd and captured environment, then calls `libc::execvp` at 408.
  Lines 449–547 and 640–660 describe fast-path eligibility and fallback when
  required runtime support is unavailable. Lines 694–720 configure stdio/cwd
  file actions; the normal fast path calls `posix_spawnp` from 765 onward.
- No unrelated-FD close sweep was found in those setup paths. A held image FD
  above 2 should survive until exec, subject to libc/procfs assumptions below.
  This is source-based inference, **not execution proof**.
- `execvp` has an implicit `/bin/sh` fallback on `ENOEXEC`; an absolute filename
  avoids PATH search but does not provide a no-shell contract.
  [Linux exec manual](https://man7.org/linux/man-pages/man3/exec.3.html).
  Our bounded ELF parser is not proof the kernel can never reject an image.
  CLOEXEC may prevent the shell reading the memfd afterwards, but that does not
  undo the unauthorized shell launch. We cannot accept that as a safe fallback.
- The current fixed GNU recipe appears eligible for `posix_spawnp`; this does
  **not** establish that it takes the fallback on today's host. Conversely,
  stock `Command` does not expose an enforce-fast-path/no-shell option. Exact
  deployed glibc implementation qualification is still missing: the attempted
  upstream `glibc-2.43` source URL returned 404. Do not substitute a different
  version or assert that the fast path has the same shell behavior as `execvp`.
- Sibling `unix/common.rs:183` defaults `create_pidfd` to false. The pinned
  `std/src/os/linux/process.rs:5` marks `linux_pidfd` unstable. Stable Rust cannot
  simply enable its `CommandExt::create_pidfd` API within this proposal.
- `unix.rs:986–1002` uses a stored pidfd for signaling if present, otherwise
  numeric-PID `kill`. Keeping a `Child` is valuable ownership, but calling its
  `kill()` is not automatically proof of pidfd-only cleanup.

### Procfs, CLOEXEC and identity assumptions

Read-only RTX observations: `getconf GNU_LIBC_VERSION` returned `glibc 2.43`;
`stat -f -c '%T %t' /proc /proc/self/fd` returned `proc 9fa0` for both paths.
These are current observations, not a guarantee against mount-namespace changes.

For the proposed procfs route, `/proc/self` must resolve in the child's trusted
procfs namespace, and the owned sealed FD must remain open and unreused through
spawn. Require FD >= 3 so stdio replacement cannot overwrite it. CLOEXEC closes
the image descriptor at successful exec, rather than before pathname resolution.
No caller-supplied FD mapping, close/clear-CLOEXEC operation or deployment-path
reopening is allowed. Other inherited non-CLOEXEC descriptors are another explicit
precondition; `Command` does not make them disappear for us.
[Exec descriptor semantics](https://man7.org/linux/man-pages/man2/execve.2.html).

Proc FD dereference is subject to ptrace access and inode permissions; a procfs
type check alone neither establishes access nor prevents a privileged remount.
Missing/inaccessible procfs must fail without path fallback.
[Proc FD access rules](https://man7.org/linux/man-pages/man5/proc_pid_fd.5.html).

Post-spawn `pidfd_open` also requires a documented reaping contract. The kernel's
early-exit PID-retention guarantee assumes SIGCHLD is not explicitly ignored,
SA_NOCLDWAIT is not set and no other actor reaps the child. Otherwise atomic
CLONE_PIDFD-style ownership is needed. Do not silently assume those conditions
for an arbitrary embedding process.
[Pidfd ownership conditions](https://man7.org/linux/man-pages/man2/pidfd_open.2.html).

### Frozen probe input and expected non-root outcomes

Artifact, read and hashed only:

`/home/travis/security-round5/evidence/pf27-static-probe-20260912/linkage-retry/candidate/codex-protected-root-probe`

SHA-256: `f6ca8e3368dcbc8e5ba92bbbc7860ee825628b9470ac53116c6c3f6996bef3f0`.
Size: 310247352 bytes. Existing static-linkage evidence identifies an x86-64
ET_DYN NOW PIE without PT_INTERP/DT_NEEDED. It is a development artifact, not a
release build; static linkage is not a runtime CVE or containment qualification.

Proposed fixed argv[0]: `codex-protected-root-probe` (private `arg0`, not a caller
input); executable resolution would still use the owned descriptor, not argv[0].
Proposed environment: zero entries via `env_clear`, cwd `/`, stdin/stdout/stderr
all `/dev/null`, existing non-root `travis` UID/GID with no identity setters in
the parent and no real credential/profile loading. Arguments after argv[0]:

| Existing mode and exact arguments | Expected result, not yet executed |
| --- | --- |
| `--prepare-synthetic-child journal 101 201 204` | Exit 78; deny before identity/group/hardening setters because invoking identity is non-root. |
| `--prepare-synthetic-child policy 102 202 204` | Same non-root denial. |
| `--prepare-synthetic-child worker 103 203 none` | Same non-root denial. |
| `--inspect-post-exec` | Private test seam only: exit 0 with hardening observations if kernel operations succeed, otherwise sanitized exit 78. Not a production recipe. |

Source inspected at proposal base 5ad285be:
[`probe/main.rs`](../../../../codex-rs/secret-broker-service/src/probe/main.rs)
dispatches the two modes at lines 13–24, handles report/error at 29–37, and
rejects native/unrecognized modes at 42–45.
[`probe/prepare.rs`](../../../../codex-rs/secret-broker-service/src/probe/prepare.rs)
calls `root_and_single_thread` before any setter; that guard requires real,
effective and saved UID/GID all zero. The non-root denial is thus a source-based
expectation, not a claim to have exercised privileged preparation.
[`probe/hardening.rs`](../../../../codex-rs/secret-broker-service/src/probe/hardening.rs)
changes only its own process hardening and reports bounded observations. It must
never run in the parent/test runner. Its `Report::verified` does not include
`descriptor_allowlist`; a test must separately assert that reported boolean.

With null stdout there is no report to inspect. Observational testing therefore
needs an explicitly private bounded stdout pipe exception (proposed 4096-byte
cap, excess = failure), leaving stdin/stderr null. Record it as a test seam,
not proof of the all-null production recipe. No invocation is authorized by
this table and no existing mode keeps a child alive for cancellation testing.

### Concrete deadline and cleanup ownership proposal

1. Acquire one process-local launch permit before allocating a sealed image.
   Create the cleanup worker and control channels before calling spawn; worker
   creation failure returns without launching anything. Never allocate a thread
   from Drop. One outstanding job bounds late workers and held-image memory.
2. The pre-existing worker exclusively owns the image, pending spawn and eventual
   Child/pidfd. A caller receives only a cancellation/status handle. Record an
   `Instant` deadline and a sticky cancellation flag before starting spawn.
3. At deadline, return `CleanupPending`, not `Stopped` or `NeverLaunched`, if
   spawn has not resolved. Keep the permit and image quarantined; do not retry
   or admit another launch. Caller Drop requests cancellation but transfers no
   ownership away from the worker.
4. If spawn completes late, check cancellation/deadline before publishing a
   running state. The owner must acquire/retain stable process identity, stop
   the child and reap it. Only observed reaping releases the permit. Child exit
   before publication follows the same ownership path, not a detached success.
5. Failure to acquire a pidfd after obtaining Child is an unresolved API seam,
   not permission to abandon it or silently signal a numeric PID. Accepting
   numeric signaling under enforced exclusive-reaper/SIGCHLD assumptions would
   amend the proposed strict contract; atomic pidfd creation requires another
   adapter/API. Neither is implemented or implicitly selected here.
6. A worker stuck inside `spawn` has no public Child to kill. The caller can
   remain responsive, but a hard launch/termination deadline cannot be promised
   with this API. Worker panic/process failure also needs a durable supervisor
   contract; a dropped channel is not evidence that no child exists. Until that
   is designed, the contract is incomplete rather than a successful timeout fix.

State tests can deterministically gate a private fake spawn backend before and
after child publication, inject worker-creation/pidfd errors, cancel/drop, and
assert quarantine/no-relaunch/reap ordering. Such tests prove our state machine,
not interruptibility of libc or kernel spawn. Production-owner integration is
still necessary; a sleep-based race is not an adequate deterministic fixture.

### Exact remaining fixture/API decisions

Existing probe modes can exercise early exit and bounded inspection but cannot
reliably prove stopping a live child. Proposed separate QA-only fixture location:
`qa/security-levels/sprints/PF-27-S04/descriptor-launch-20260912/fixtures/hold.rs`.
It would have no arguments or alternate modes, emit a fixed readiness marker
through the explicitly approved private stdout pipe, then remain alive until
terminated; no files, credentials, networking or identity mutations. Compile it
with the already qualified static musl toolchain/recipe, profile-inspect and hash
the output, and propose that exact candidate before invocation. No source or
binary has been created, so no hash or successful compile is claimed. Approving
this fixture is a separate QA seam, not a new product probe mode.

**Manager decision requested:** do not allocate the stock-Command implementation
as if all five original source paths suffice. First choose whether to qualify a
fixed fast-path-only platform contract or allocate a descriptor-exec adapter
whose API guarantees no shell fallback and explicit child/pidfd ownership. Also
choose bounded caller response with honest cleanup-pending quarantine versus a
hard cleanup deadline requiring an independently controllable launch supervisor.
The latter is outside the currently proposed ownership API. No unsafe callback,
nightly feature, new dependency or external helper is silently proposed as an
already-approved solution. Once the API decision is recorded, the private live
fixture and deterministic fault seams can be allocated together with it.
