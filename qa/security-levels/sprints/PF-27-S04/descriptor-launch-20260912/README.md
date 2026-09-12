# PF27 isolated GNU pidfd adapter — stage-one evidence

Owner /root; allocation and Travis's explicit approval are preserved in
[the accepted proposal](../descriptor-launch-next-20260912.md). Incremental base
c5b05d9d8; source30b471a47bce83605ee1f5ac385f62a12c17e56e, Rust tree
2d270c5c4cdde5e814055f2647b0a707b88497db. Original sprint/worktree/base unchanged.
This is an internal construction increment, not a completed broker, production
activation, native install or human-test-ready feature.

## Frozen review scope and contract

New `codex-linux-pidfd-spawn` crate, workspace membership/dependency and its
9-line Cargo.lock entry only; no package upgrades. Runtime314 lines, tests216.
The small coherent adapter/OS-proof stage is about590 code/build/fixture lines;
it remains below800 while retaining the required error-path tests. Service Rust
is unchanged and still forbids unsafe code. No production API is enabled by
default: the entire adapter is GNU/Linux plus synthetic-fixture gated.

Safe fixed non-root probe recipes consume an owned sealed CLOEXEC image;
the caller separately verifies static profile/provenance. Maximum image fd256
bounds close actions. Child cwd is /, environment empty, all stdio null; close
actions retain only the image until CLOEXEC. Versioned non-p pidfd_spawn creates
an atomic owned pidfd; no post-spawn PID lookup, shell or PATH fallback.
GLIBC2.43 only is presently source-qualified; other versions return Unsupported.
Trusted mount/procfs and exclusive reaping/permitted signal+wait syscalls are
explicit embedding preconditions, not claims about hostile host administrators.

OwnedChild retains its pidfd on signal/wait errors. Drop cooperatively retries
kill/wait without creating a thread and can block indefinitely on persistent
permission/kernel failure. ECHILD means no waitable child remains for this
process under the documented exclusive-reaper precondition. No caller deadline,
ready signal, panic-safe independent supervisor or asynchronous quarantine is
claimed here. Those belong to the separately allocated next owner-integration
stage; this dependency is not wired into the live service.

## Executed evidence

Primary host travis@100.99.88.49, non-root UID1001, Ubuntu26.04.1,
Linux7.0.0-31-generic x86_64, glibc2.43-2ubuntu2.4. Existing source-qualified
libc SHA85e64f97e348786a8fb4d9f3d52fec289e2fb86bba20f0731dfe61990525e0f7.
All builds are on RTX. [Reproducer](qualify-rtx.sh) uses the existing isolated
Rust1.95.0 musl target for QA-only hold/inspect binaries; inspect exits with a
bounded status rather than publishing host data. Exact artifacts in
[the hash ledger](rtx/artifacts.sha256); ELF metadata and validator receipts
are retained. Both fixtures pass the existing sealed-byte static profile check.

| Check | Observed result |
| --- | --- |
| Scoped just fix, just fmt, scoped Clippy | Pass, before final tests |
| Default adapter | No API/tests enabled; explicitly zero tests, not proof |
| Enabled focused tests | 3 pass: unsealed, non-CLOEXEC, bounded descriptor actions |
| GNU2.43 OS cases | 7 pass: three fixed non-root recipes; ENOEXEC/no-shell/leak; exact argv/env/cwd/stdio/FDs; exec failure/no-zombie; path replacement; live pidfd kill/drop reaping; fd-resource failure/no-detached-child |
| Existing service synthetic suite | 44 pass; one pre-existing artifact test excluded and separately run twice for new fixtures |
| New fixture ELF profile validation | 1 pass for hold,1 pass for inspect |
| Actual-key TMUX replay | 7 pass, PF27_TMUX_EXIT=0; [capture](tmux-os.txt); text and Enter sent separately |
| Cargo/Bazel lock parity | just bazel-lock-update exit0; MODULE blob unchanged5ecff077dbbbf7887a61c03a8e9aa354f002e1ed |
| Second host | pfrpc@178.156.143.199 UID1001, Linux6.8.0-49-generic x86_64, glibc2.39-0ubuntu8.9: explicit unsupported-libc case1 pass,10 excluded |

Second-host nextest run66de3c7d-f74c-4810-947d-64fd02ca1394 used the RTX-built
archive, isolated extraction and matching source/config remap, not a rebuild or
positive GLIBC2.39 qualification. No account/profile/credential files were read.
The archive remains outside git at external .codex-work/pf27-adapter-tests-20260912.tar.zst.

Remote final receipts retain initial HEADa57ff6573 plus formatted working-tree
diff; final Rust tree is matched to the local source commit rather than claiming
the unformatted HEAD alone was tested. Local and remote source hashes are
recorded in the final-tree receipt. Full logs under rtx/; no ignored test is
silently counted as passing.

## Failures retained and repaired before review

Initial compile failed because fcntl fd flags were imported from rustix::fs,
not rustix::io; corrected. Initial fixture build used the normal Rust home without
the musl target; corrected to the existing isolated qualified toolchain.
The first high-fd fixture requested1025 above the process soft limit1024; action
bound256 and fixture257 now exercise the adapter rejection on the measured host.
Second-host archive attempts failed for missing extraction/source/config paths;
final isolated remap/config run passed. Initial Bazel success left its own idle
server holding an inherited build-lock descriptor; shut down that exact server
normally, releasing the lock. No Rust process or unrelated service was killed.

## Remaining boundaries

Independent closeout complete: Astra23 and Fable24 both exit0, findings[],
patch correct. Fable ran through the existing Corbanu Fable5.1High wrapper in
private TMUX socket `pf27adapterreview20260912`, session `fable24`.
Original structured results and helper exits are retained beside this record;
no findings required code changes or disposition overrides. No main push.
Production owner integration must prove late spawn, cancellation/drop quarantine,
no relaunch until observed cleanup, persistent wait/signal denial, and supervisor
failure/recovery. Kernel pidfd-unavailability on a different kernel and privileged
containment are not proven by rejecting an old libc. Existing dynamic-loader,
full broker data-plane, credential migration and all-OS acceptance remain open.
The internal primitive has no new TUI surface: code-blind UX design is not
applicable, and TMUX OS invocation is supporting proof rather than a user-flow
substitute. Do not relabel this staged result as full PF27 completion.
