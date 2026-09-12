# PF27 next proposal — qualify a self-contained synthetic probe artifact

**Accepted September12 by receiving owner task
`01a08522-76a1-7ad1-afe6-ad690d55c0d7` for build-only feasibility.**
This does not select a new release artifact strategy or permit image invocation.
Sealed-image checkpoint `140e094adc71da6e74e9017fce5d9b85b92ce956` is on main;
its main window is released. Source `ee1ac023c` / Rust
`0aa65bd04f5e30f3a21e1309aac7aa6b83336859` is unchanged.

PF-27-S04 remains the sole security allocation: owner `/root`, branch
`feat/security-broker-resume-20260911`, worktree
`/Volumes/CorbanuDrive/Corbanu/worktrees/security-broker-resume-20260911`, original
allocation base `d870c92dab2bf3fbb602dc3b8447fe9f3534aecb` unchanged. Product
heading **Non-negotiable controls**: “Permit agents to reference credentials
only by label; resolve them solely inside the trusted execution boundary.”

## Accepted feasibility decision

Recommend qualifying a statically linked version of the existing synthetic probe
before designing a dynamic-loader deployment closure. This is an explicit change
to the proposed artifact strategy, not a claim that the current probe is static.
The alternative is retaining GNU dynamic linking and separately pinning and
constraining the interpreter, transitive libraries, search paths and filesystem
view before root invocation; sealing only the main file does not solve that.

Read-only RTX inspection on September12 confirms the current candidate is ET_DYN,
has PT_INTERP `/lib64/ld-linux-x86-64.so.2` and PT_DYNAMIC. Only the
`x86_64-unknown-linux-gnu` Rust target is currently installed; no `/usr/bin/*musl*`
tool was found. The first login-shell rustup command missed PATH; the explicit
`/home/travis/.cargo/bin/rustup` check succeeded. No toolchain was installed,
binary rebuilt or image invoked for this proposal.

## Smallest coherent allocation

After acceptance, attempt a build-only qualification of the **existing**
`codex-protected-root-probe` for `x86_64-unknown-linux-musl` on RTX, using the
recorded source tree and serialized build lock. Toolchain prerequisites must
stay in the user's remote workspace; no root package installation or global
toolchain change. Pin and record toolchain/compiler/artifact versions and hashes.
Do not change ordinary Corbanu release targets, launchers or installed packages.

Scope is a reproducible script and evidence under
`qa/security-levels/sprints/PF-27-S04/static-probe-20260912/`, plus this proposal
and existing plan/sprint ledgers. No Rust, dependency, Cargo/Bazel/lock or service
configuration changes are included in this initial allocation. If the dependency
graph cannot produce that artifact without changes, stop with exact build errors
and propose a bounded correction; do not silently split/rewrite the probe crate.

Acceptance evidence: a successful build from the frozen source, bounded artifact
size, ELF64 little-endian x86-64 identity, program/dynamic table inspection proving
no PT_INTERP and no external DT_NEEDED entries, and exact digest/provenance. Static
PIE may still have PT_DYNAMIC for self-relocation; do not equate that header alone
with external dependencies. Preserve negative comparison against the current
dynamic probe. Use readelf or equivalent read-only inspection, not ldd or executing
the artifact. Reproduce the command/checks through actual-key private TMUX.

These checks prove build/linkage properties, not absence of runtime file access,
dlopen behavior, secret exposure, syscall containment or safe root execution.
No executable is invoked in this allocation, including a non-root smoke run.
No new UX; blind UX design and live-repository benchmarks are not applicable.

## Subsequent gates and accounting

September12 prerequisite amendment: the first Cargo build failed101 in
openssl-sys0.9.111 because no musl-target OpenSSL installation was available.
The receiving owner accepted ONE retry with a pinned, authenticated upstream
OpenSSL static build entirely under the remote evidence root. Use target-scoped
OPENSSL_DIR/STATIC only; preserve initial evidence byte-for-byte and use a
separate retry directory. No graph/source/feature changes or target invocation.
Select OpenSSL3.5.8, the supported3.5 LTS release listed by the official downloads
page when checked; record source signature/checksum and exact compiler/flags.
Any further contract/graph expansion returns for a fresh decision.

Second explicit prerequisite amendment: OpenSSL compilation stopped2 on missing
linux/mman.h before Cargo. The owner accepted one coherent private x86-64 Linux
userspace UAPI set and retry of the same OpenSSL/unchanged Cargo graph. Pin Ubuntu
linux-libc-dev7.0.0-31.31 amd64 through signed cached repository metadata and
package SHA256, extract only linux/asm-generic/target asm, and record inventory
and compiler include resolution. No raw internal kernel headers, glibc leakage,
musl header overwrite, security-feature suppression or root/global install.
Preserve both failures and use separate retry paths. Further source/graph or
security-behavior changes still require a new decision.

Only after artifact feasibility is proven, propose the corresponding runtime
ELF-profile validator, descriptor-bound exec and retained-child supervision as
separate bounded implementation. A static artifact must not be relabeled as a
native-qualified package. Exact privileged installation/execution approval,
identity/group/FD mapping, launch deadlines, runtime restrictions and all-OS
qualification remain open. No main window or additional review is assumed here.
Reviews1–19 remain preserved. The receiving owner allocated the one old unused
contingency to one build/linkage evidence review; reserve before dispatch, then
record its outcome. No duplicate unchanged-Rust code review is authorized.
