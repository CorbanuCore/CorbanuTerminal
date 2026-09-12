# PF27 static probe feasibility — build and linkage passed

This is the accepted [build-only allocation](../static-probe-next-20260912.md),
not a new release target or native execution claim. No Rust, dependency graph,
Cargo/Bazel lock, service configuration or installed application was changed.
Build tooling may execute; neither probe artifact is invoked and ldd is not used.

## Frozen source and environment

- Source `140e094adc71da6e74e9017fce5d9b85b92ce956`; Rust subtree
  `0aa65bd04f5e30f3a21e1309aac7aa6b83336859`.
- RTX worktree `/home/travis/worktrees/security-broker-static-20260912`.
- Runtime/evidence `/home/travis/security-round5/evidence/pf27-static-probe-20260912`.
- Isolated Rust1.95.0 (59807616e), musl target and Cargo output under that root.
  The installer's “default toolchain set” refers to its private RUSTUP_HOME;
  ordinary `/home/travis/.rustup` still defaults to stable, with GNU targets only.
- musl1.2.6 C toolchain built into the same private root. Release signature
  verified against published primary fingerprint
  `836489290BB6B70F99FFDA0556BCDB593020450F`; original verification and SHA256s
  preserved. The GPG trust-database warning is not a failed signature check.
- Tool/compiler and Rust-target archive hashes are recorded. The separately
  built C toolchain does not establish the libc version bundled in Rust's target
  archives; do not claim the eventual entire runtime uses musl1.2.6.
- Official provenance: [musl release page](https://musl.libc.org/releases.html).
  Its published advisories are not waived by a build-only feasibility check.
  No production suitability or runtime security qualification is claimed.

## Actual first attempt

The original 159 handwritten lines across build-only.sh, inspect-linkage.py and
tmux-build.py launch build/inspection tooling with separate literal keys and Enter.
The first attempt installed private prerequisites and reached Cargo compilation.
`build.exit`, `run.exit` and TMUX all record **101**, not a pass. No candidate
was produced, so static linkage inspection and static-vs-dynamic comparison did
not run. Exact source status is clean and Cargo.lock before/after hashes match.

`openssl-sys0.9.111` reports that target OpenSSL could not be found and pkg-config
is not configured for cross-compilation. The frozen graph includes
native-tls0.2.14/hyper-tls/reqwest0.12.28 through codex-http-client and configuration
dependencies of protected-state/service (also Vault/configuration paths).
See [full inverse dependency tree](rtx/initial/openssl-dependency.txt) and
[original build log](rtx/initial/build.log). This is a target-native prerequisite
failure, not proof that a Rust graph change is necessary.

All original logs/captures are retained byte-for-byte under rtx/; raw copies
remain on RTX. Initial-integrity hashes are recorded; two trailing-whitespace
diagnostics in raw gcc/TMUX artifacts are retained deliberately. No failed
attempt is replaced by a later pass. There is no user-facing TUI or blind UX
design applicability in this internal artifact experiment.

## Approved prerequisite retry — also blocked

The receiving owner approved one private static OpenSSL prerequisite and an
unchanged-graph retry. OpenSSL3.5.8 is listed in the supported3.5 LTS series on the
[official downloads page](https://openssl-library.org/source/). Download SHA256
`a8f84a39918ec6415ce765d9b429d313ba97b8143169c172e734b9514464f5b2` matched the
official checksum; signature status matched the published primary trust anchor
`B146647E45A7B33947AB226B2A2C87D161692D40`. Official HTML remains private on RTX
with its hash recorded; release keys/archive provenance and verification logs
are retained. Exact compiler and Configure flags are in openssl-retry.sh and
rtx/openssl-retry/configuration.txt; no host OpenSSL setting was changed.

This attempt stopped with **exit2** while compiling OpenSSL, before Cargo was
invoked. `crypto/mem_sec.c:60` includes `linux/mman.h` when SYS_mlock2 exists;
the private musl sysroot lacks that Linux UAPI header. See
[original native build log](rtx/openssl-retry/openssl-build.log). There is no
second Cargo exit code, installed OpenSSL package, static probe, linkage pass or
probe invocation to report. Fresh source status remains clean and its lock hash
still matches the original. The two attempts are in separate directories.

## UAPI completion and linker correction

The owner accepted private userspace UAPI headers. Ubuntu's installed archive
keyring verified InRelease; its SHA256 verified the exact Packages index, whose
linux-libc-dev7.0.0-31.31 amd64 stanza matched downloaded package SHA256
`b26e3493c7180b0cc8b5e7c2bf819323deca8b7662f34ddffef936b08ffb1456`.
Only linux/asm-generic/x86-64 asm were extracted into private uapi/include. The
inventory, signature, metadata and include trace are in rtx/uapi-retry/; no musl
header overwrite, glibc include leakage or disabled secure-memory check occurred.

OpenSSL then built and unchanged Cargo completed **exit0 in49.18s**. The linkage
check correctly failed1: the 298,954,552-byte PIE still requested the musl
interpreter, despite no DT_NEEDED. Its SHA256 is
`37fb0d7c5fd7e09ca23e9a5b1901b77786b87dfe8a869e191452c571bda7081d`.
This rejected candidate remains on RTX and its readelf output is retained here.
Build success alone did not satisfy static acceptance.

The C compiler wrapper's musl-gcc.specs injects a dynamic interpreter when used
as the final static-PIE linker. The owner accepted using the pinned Rust rust-lld
as the command-scoped final linker while retaining musl-gcc for C compilation.
The [pinned Rust target](https://raw.githubusercontent.com/rust-lang/rust/1.95.0/compiler/rustc_target/src/spec/targets/x86_64_unknown_linux_musl.rs)
defaults to static CRT and supports static PIE; no relocation-model change or
post-link editing is used. The corrected attempt has separate linkage-retry/
and linkage-control/ evidence. Cargo completed0 in35.62s; real-key TMUX tooling
exit0 and read-only static/negative-control inspection passed. The new ELF is
ET_DYN, FLAGS_1 NOW PIE, without PT_INTERP or DT_NEEDED, size310,247,352 bytes,
SHA256 `f6ca8e3368dcbc8e5ba92bbbc7860ee825628b9470ac53116c6c3f6996bef3f0`.
It differs from the rejected wrapper-linked artifact; original failures remain.
The dynamic GNU control retains both interpreter and external library entries.
Neither artifact was invoked. This is a dev-profile feasibility candidate,
not a release build, installed image or native containment proof.

Final linker is Rust1.95.0's LLD22.1.2, SHA256
`60f305b55da767671e895b231e0c78e87f4ccbf790d7181842039c53bb60281c`;
the recorded Cargo fingerprint confirms `-C linker-flavor=ld.lld`. Per-target
linker path and native C compiler separation are in linkage-retry.sh. Rust source
and Cargo.lock remain unchanged; final source status is empty. All build and
TMUX sessions finished. No test suite was rerun or claimed by this build-only
allocation; the original sealed-stage tests remain tied to their GNU candidate.

One build/linkage evidence review is allocated from the old contingency but has
been reserved as Fable20 before dispatch; reviews1–19 remain intact. It reviews
this artifact evidence and QA tooling, not unchanged Rust code. No main window
is granted. Runtime ELF validation, static-dependency/CVE qualification, descriptor
exec, supervision and exact privileged installation remain subsequent gates.

The first review-helper attempt failed while decoding the Git diff, before model
dispatch. Raw GPG status includes non-UTF8 bytes; it remains byte-for-byte intact
and is now classified as a binary Git artifact. An ASCII-only VALIDSIG receipt
is provided separately. Original helper failure/exit is retained; retrying this
pre-dispatch infrastructure failure uses the same reserved review20, not another
model opinion or another Rust review. Candidate/source and test evidence do not
change with this artifact-classification correction.
