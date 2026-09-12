# PF27 static probe feasibility — initial build blocked

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

## Next decision and evidence review

The receiving owner has the exact error and a proposed bounded private Linux UAPI
header prerequisite. No headers were copied, secure-memory behavior disabled or
broad host include directory added. No graph/source change is justified yet.
One build/linkage evidence review is allocated from the old contingency but has
not yet been dispatched, pending that decision; reviews1–19 remain intact.
No new main window is granted.
