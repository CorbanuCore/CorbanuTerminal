# PF27 proposal — one owned child launched from the inspected sealed image

**Approved adapter-only allocation September 12, 17:29 UTC; implementation and
non-root synthetic invocation now allocated as specified below.** Historical
proposal/qualification decisions below remain preserved, not current blockers. Parser
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

### Accepted isolated adapter / OS-proof stage

Travis explicitly approved `pf27-linux-launch-adapter`: “Ok. So it seems like I
need to approve PF-27-S04? We have multiple linux boxes we can test with. I approve
the isolated adapter”. Integration manager task
`01a08522-76a1-7ad1-afe6-ad690d55c0d7` relayed this decision and directed resumption.
It permits the narrow separately reviewed FFI dependency and qualified immutable
procfs binding, not unsafe code inside the service or direct-execveat claims.

Stage one literal allocation: `codex-rs/linux-pidfd-spawn/` (Cargo.toml,
BUILD.bazel and private lib/ffi/spawn/test modules), workspace Cargo.toml,
Cargo.lock, MODULE.bazel.lock, this proposal, PF27 plan/sprint/review ledgers and
`qa/security-levels/sprints/PF-27-S04/descriptor-launch-20260912/` including the
already described hold fixture. Original owner/worktree/branch/base above remain;
incremental starting HEAD is `021f82e6c10b2f58e3b67ad5bfdcc1562a4bde0f`.
Preserve the integration owner's separately accepted SQLx lock delta87a9a81.
No service dependency/wiring, PF20 API or dashboard changes in stage one.

Build/test on `travis@100.99.88.49`; qualify platform/access on already authorized
`pfrpc@178.156.143.199` and record its actual supported/unsupported outcome.
Record OS/kernel/libc/architecture/artifact provenance. Synthetic non-root
execution is approved; no real profile, credential execution, privileged install,
protected activation, main push or release. Two necessary independent closeouts
(Astra High and Fable5.1High through TMUX) use the replenished review allowance.

Implement the real creation/exec/pidfd primitive and OS tests first. Full
asynchronous deadline/cancel/quarantine owner integration is the separate second
stage: stage one must not claim that integration or completed PF27. This internal
dependency has no new user-facing UI; blind UX design is not applicable. Actual
TMUX invocation and lifecycle receipts remain required supporting OS proof.

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

## Adapter API disposition — September 12, 2026

Manager chose a dedicated descriptor-exec/ownership adapter, preserving safe
first-party Rust and no shell/path fallback, with bounded caller response and
honest cleanup-pending quarantine (not hard termination). The hold fixture and
4096-byte private stdout cap are accepted in principle. Builds and invocation
remain unallocated. This section records the exact remaining API blocker.

### Available pinned APIs, inspected without compilation

RTX registry root:
`/home/travis/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/`.
Versions below match the existing Cargo.lock; no dependency was added.

| API | Actual contract | Why it does not complete this adapter |
| --- | --- | --- |
| `nix 0.30.1`, `unistd.rs:1214`, safe `execveat<Fd: AsFd, SA: AsRef<CStr>, SE: AsRef<CStr>>(dirfd, pathname, args, env, flags) -> Result<Infallible>` | Calls SYS_execveat directly; use empty C string plus `AtFlags::AT_EMPTY_PATH` to execute the held FD. Existing `process` feature enables it. | Replaces the **calling process**; no child creation or parent pidfd ownership. Must not call it in our supervisor. |
| `nix 0.30.1`, `unistd.rs:1187`, safe `fexecve` | Replaces current process through libc. | Same creation gap; direct execveat is preferable to libc's descriptor/path compatibility behavior. |
| `nix 0.30.1`, `spawn.rs:364`, safe `posix_spawn(...) -> Result<Pid>` | Direct libc posix_spawn, not std's fast-path selection. | Returns numeric PID, not pidfd/Child; still pathname-based. Does not fill the selected descriptor-exec plus stable-owner API. |
| `nix 0.30.1`, `unistd.rs:278`, `fork`; `sched.rs:107`, `clone` | Both public calls are `unsafe`. | Disallowed at the first-party call site. A safe execveat wrapper does not make post-fork execution safe. |
| `rustix 1.1.4`, `runtime.rs:419/456` | `kernel_fork` and runtime `execveat` are unsafe; fork returns PID variants. | Not an available safe create-and-own primitive. |
| `rustix 1.1.4`, `process/pidfd.rs:29/41`, `process/wait.rs:399/488` | Safe pidfd_open, pidfd_send_signal, and waitid with `WaitId::PidFd(BorrowedFd)`. | Monitoring/signaling/reaping are available **after** obtaining a pidfd; they do not eliminate spawn-to-pidfd failure. |
| `process-wrap 9.0.1`, `generic_wrap.rs:79–97` | Its spawn wrapper calls the wrapped `command.spawn()` at line 90. | Does not add an atomic descriptor-exec/pidfd primitive. |

Source hashes: nix `spawn.rs`
`4a3d2918ec210199fab392757cd4577c46b484a6c923a70474ae049b8bcfc1c9`;
nix `unistd.rs`
`26b45c0e0861ca82a9300eb952bc81bf626bbde6d3606fd780ad27fc86360ba7`.
These are installed source hashes, not crate archive checksums.

Targeted external candidate check did not identify an adoptable dependency:
[`clone3 0.2.3`](https://docs.rs/clone3/latest/clone3/struct.Clone3.html) exposes
unsafe `call`/`call_unchecked`; it does not solve the safe call-site requirement.
[`pidfd 0.2.4`](https://docs.rs/crate/pidfd/latest/source/README.md) describes
converting an already-spawned Command child, not descriptor spawning.
[`memfd-ng 0.1.1`](https://docs.rs/memfd-ng/latest/memfd_ng/) accepts image bytes
and documents temporary-file fallback; its public overview does not establish
adoption of our already-owned sealed FD or atomic pidfd ownership. Detailed
process/executable source retrieval failed, so this is **not** a completed audit
or an assertion that no configurable variant exists. None is recommended as
an already-qualified dependency. No download/install/build was performed.

### Exact missing capability and next allocation boundary

Missing: a supported **safe spawn-from-owned-FD operation that creates the child
and obtains its stable owned pidfd before any fallible post-spawn return can lose
ownership**, directly execs the FD without shell/path fallback, and retains
cleanup ownership on exec failure/cancellation. A numeric-PID wrapper, a new
trait with a mock backend, or calling execveat in the supervisor would not provide
that capability. There is no verified compilable end-to-end implementation using
the inspected dependencies. This is a concrete dependency/API gap, not a request
for another generic design pass or additional human authorization.

The eventual dependency must expose an owned process token (not necessarily
`std::process::Child`) with pidfd-based stop and wait, and distinguish failure
before creation from failure after creation with retained cleanup ownership.
The existing `TrustedChildRun`/PF20 consumers require actual `Child` today, so
adapting them is a **separate future API boundary**, not a hidden conversion from
PID. The single-child stage may remain private until that integration is allocated.

Conditional first-party allocation after a concrete dependency is qualified:
`launch/manifest/{sealed.rs,mod.rs,spawn.rs,spawn_tests.rs}`, `launch/mod.rs`,
new `launch/manifest/spawn_integration_tests.rs`, and the accepted QA `hold.rs`.
A new dependency additionally requires crate/workspace Cargo declarations,
Cargo.lock, affected BUILD.bazel and MODULE.bazel.lock allocation. Estimate
250–400 runtime lines and 350–550 test/fixture lines (600–950 nonmechanical),
excluding the missing dependency implementation/audit. The old 800-line target
cannot be promised without the actual backend. Required proof remains held-FD
binding, no fallback, live/early exit, creation/exec/pidfd failures, late spawn,
cancel/drop, quarantine/no-relaunch and observed reaping. Do not start a mock-only
implementation and call it this end-to-end stage.

Return to manager with this exact capability gap. A maintained safe library
extension or separately qualified launch service could supply it, but neither
currently has a verified version/API/source allocation. No unsafe shim, C helper,
nightly feature or numeric-PID fallback is being silently introduced to close it.

## Focused GNU pidfd_spawn qualification — September 12, 2026

**Conditional GO for a separately approved narrow FFI adapter; NO-GO for direct
use inside the service's `forbid(unsafe_code)` crate or for claiming runtime
qualification today.** This concrete libc primitive closes the creation-to-pidfd
gap at source level. It uses held-procfs-path `execve`, not direct `execveat`.
If a direct syscall is itself mandatory, this backend does not satisfy that
additional constraint. If immutable image binding/no fallback is the requirement,
the retained-FD/trusted-procfs construction is a viable qualification target.

### Exact deployed version and matching source, now located

RTX package queries identify Ubuntu 26.04.1, `libc6` and `libc6-dev`
`2.43-2ubuntu2.4`. Its `/lib/x86_64-linux-gnu/libc.so.6` SHA-256 is
`85e64f97e348786a8fb4d9f3d52fec289e2fb86bba20f0731dfe61990525e0f7`,
ELF build ID `066527e430a32768d82741e00b81eebb1a872294`.
Read-only `objdump -T` confirms both symbols at version GLIBC_2.39. Disassembly
of `pidfd_spawn` at 0x122970 shows xflags 4 passed to its shared implementation;
`pidfd_spawnp` passes 5. No call to either function was made.

The installed declaration is in `bits/spawn_ext.h`, not `sys/pidfd.h`:
`int pidfd_spawn(int *pidfd, const char *path, const posix_spawn_file_actions_t *,
const posix_spawnattr_t *, char *const argv[], char *const envp[])`.
Header SHA-256 `6bac139066f0c69b8ac9f228fd982db2078033067a6eb5a22dec98e2493a2424`
matches the header in the retrieved upstream source archive.

The failed GitHub tag fetch was not treated as source evidence. Instead the
[Ubuntu security publication](https://api.launchpad.net/1.0/ubuntu/+archive/primary/+sourcepub/18721986)
returned the exact version's source URLs. Retrieved, not installed or built:

| Source artifact | SHA-256 |
| --- | --- |
| `glibc_2.43.orig.tar.xz` | `d9c86c6b5dbddb43a3e08270c5844fc5177d19442cf5b8df4be7c07cd5fa3831` |
| `glibc_2.43-2ubuntu2.4.debian.tar.xz` | `28103a7cf808c29901c6053d89a4e8299880abfbb850c0d97a09c8df8533e306` |
| `glibc_2.43-2ubuntu2.4.dsc` | `da10d551ca51076bf7bf663de38369d04e267386b8211c283c2814e68e482ed3` |

Both archives match SHA-256 values in that `.dsc`. Download base:
`https://launchpad.net/ubuntu/+archive/primary/+sourcefiles/glibc/2.43-2ubuntu2.4/`.
Local preserved downloads (about 21 MB, all on the external drive):
`/Volumes/CorbanuDrive/Corbanu/.codex-work/pf27-glibc-source-Ev2rON/`.
The Debian patch series and patch payloads were inspected for these Linux spawn
files; no patch changes the examined pidfd_spawn/spawni/clone implementation.
Only an unrelated Hurd spawni patch matched the broader search. This establishes
matching package-version source with checksum consistency, not a reproducible
build match or an independently verified `.dsc` signing chain. `deb-src` is not
configured on RTX; no apt sources were modified.

### Concrete source/error ownership findings

References below are to the downloaded glibc-2.43 archive, not `master`:

- `sysdeps/unix/sysv/linux/pidfd_spawn.c:23–30` calls `__spawni` with only
  `SPAWN_XFLAGS_RET_PIDFD` (4). Source SHA-256:
  `0e0cea67b591e672f7693e019f3a73c4cac708ab2fac1d9d610c5716163e41a4`.
- `posix/spawn_int.h` defines USE_PATH=1, TRY_SHELL=2, RET_PIDFD=4.
  `sysdeps/unix/sysv/linux/spawni.c:78–98` permits shell compatibility only
  with TRY_SHELL, absent here; lines 488–496 select `__execve`, not execvp.
  This is a direct API selection, not an unenforceable std fast-path preference.
- `spawni.c:323–332,398–427` checks kernel pidfd/wait support before creation,
  supplies CLONE_PIDFD to clone3 or its clone fallback, and obtains the pidfd
  atomically into `args.pidfd`. No post-spawn pidfd_open or PID lookup is needed.
  Kernel clone fallback retains CLONE_PIDFD; it is not executable-path fallback.
- `spawni.c:443–480` waits with P_PIDFD and closes the pidfd on reported setup/
  exec failure. Success writes the owned pidfd through the mandatory non-null
  out-pointer. Premature child death can still return success: the caller must
  inspect/reap the returned handle, not infer a running/ready child from rc=0.
  The known historical exec-failure FD leak fix is present in this source.
  [Upstream fix and regression](https://sourceware.org/pipermail/glibc-cvs/2024q2/085497.html).
- `spawni.c` SHA-256:
  `c03cf7328a31d21af4a98b1ed840a414b4d916831fb55ce542b19d9bd8f696de`.
  Its internal waitid return is not checked; do not claim verified cleanup
  under arbitrary seccomp-denied waitid or hostile process-wide reaping state.
  Those are explicit platform/owner preconditions and failure-test obligations.
- The call disables pthread cancellation internally and restores it before
  return. The owning worker must never be externally pthread-cancelled; use
  cooperative cancellation only. The call remains potentially blocking.
  The manager-selected CleanupPending/quarantined-permit contract still applies.

### Exact trust-boundary proposal for manager decision

There is no safe wrapper in the inspected nix/rustix APIs. Cached pinned
`libc 0.2.186` also has no pidfd_spawn declaration. A narrow separate crate,
proposed `codex-linux-pidfd-spawn`, would therefore introduce a **new first-party
unsafe/FFI trust boundary**, not remove the service crate's forbid attribute.
Its only unsafe responsibilities would be the exact C ABI declaration/call,
spawn-action/attribute initialization and destruction, and immediate unique
conversion of the returned raw FD to OwnedFd. No Rust callback after clone,
pre_exec, custom fork/clone implementation or shell is necessary.

The safe interface should consume the held `OwnedFd` plus a bounded fixed recipe
and return one opaque pidfd-owned process token. Before the call, validate FD>=3,
seals/profile, procfs and all fixed strings/actions; keep image and pointer arrays
alive throughout. On rc=0 adopt the raw pidfd before any allocation/logging or
fallible operation, then hand it only to the existing cleanup owner. On nonzero
rc propagate the returned errno (not ambient errno) under qualified libc cleanup
semantics. Never convert through a PID or construct a fictitious std Child.
The wrapper must not accept arbitrary raw FDs, C pointers, callbacks, paths,
environment variables or uncontrolled file actions from service callers.

Required explicit allocation: new crate `Cargo.toml`, `BUILD.bazel`,
`src/{lib.rs,ffi.rs,spawn.rs,spawn_tests.rs}`; workspace/service Cargo dependency,
Cargo.lock and Bazel lock/registration; plus the previously scoped private
service owner and integration/hold tests. No such files have been created.
Estimate adapter 150–250 runtime plus 200–300 tests; service owner/wiring
200–300 runtime plus 300–450 tests: 850–1300 total nonmechanical. Prefer two
coherent reviewable stages (real adapter with non-root OS proof, then owner
integration), neither described as a completed broker service on its own.

Approval must explicitly permit unsafe only in the new reviewed dependency
boundary, retain `forbid(unsafe_code)` in the service, and accept qualified
procfs binding rather than require direct execveat. Proof must cover ENOEXEC
without shell, path replacement with held image, exec/setup failures without
FD/zombie leaks, pidfd support/resource failure, child early death, wait/signal
failure handling, stdio/env/cwd, and delayed completion/cancel/drop quarantine.
Use the accepted non-root fixtures only after source/build/hash allocation.
Source inspection is now sufficient to recommend this **specific** backend
boundary for consideration; it is not runtime or release qualification.
