# PF-83 executor package: the build lane, corrected

Harness increment 2 stopped because it could not produce a pinned candidate
package. The manager's first redirect — to `just build-for-release` — was wrong,
and the Bazel failure it hit was partly an environment defect on this host.
Both are recorded here so the next increment does not repeat either.

## What `just build-for-release` actually is

`just build-for-release` runs `bazel build //codex-rs/cli:release_binaries`,
which is a **multiplatform** filegroup over six targets (linux arm64/amd64 musl,
macOS amd64/arm64, windows amd64/arm64). It is the cross-platform *release
artifact* lane. It is not the lane for pinning one macOS executor package, and
asking for the whole filegroup is why analysis failed on `target_triple` — the
configurable attribute cannot resolve without a platform.

## Two real host defects, now fixed

`.bazelrc` points three caches at `~/.cache`, and two of those were **broken
symlinks** into a target directory that did not exist:

- `~/.cache/bazel-disk-cache` → `…/corbanu-terminal/bazel-cache/disk`
- `~/.cache/bazel-repo-cache` → `…/corbanu-terminal/bazel-cache/repository`

Bazel reported these only as `ERROR: /Users/Neo/.cache/bazel-disk-cache (File
exists)` and `Error initializing RemoteModule`, which reads like a permission
problem rather than a dangling link. Creating the two target directories cleared
both errors. Any Bazel work on this host was failing at startup before this.

## The remaining Bazel blocker

With the caches fixed, `//codex-rs/cli:codex_macos_arm64` still fails:
`no such package '@@llvm++llvm+llvm-project//compiler-rt'`. The external
`llvm-project` repo in the Bazel cache is a 30-entry, 12 KB stub with no
`compiler-rt`, so the fetch is incomplete or sparse. This is a toolchain-fetch
problem, not a code problem, and it is **not** on the critical path — see below.

## The correct lane for a pinned executor package

This repository's own release QA builds the shipped binary with Cargo, not
Bazel. For example `qa/release/0.1.31/PRE-PUBLISH-QA.md` records
`cargo build --release -p codex-cli --bin corbanu`, and 0.1.35 does the same.

The executor package therefore pins via:

```sh
cargo build --release -p codex-cli --bin corbanu
```

at the pinned source commit, with a dedicated `CARGO_TARGET_DIR`, recording the
exact command, the toolchain identity, the resulting binary path and its
SHA-256. That satisfies "pinned candidate package, refuse anything whose hash
does not match" without depending on the Bazel external toolchain.

Bazel cross-platform release artifacts remain a separate concern and the
`compiler-rt` fetch should still be repaired, but it does not block the gate.

## Verified on this host — September 15, 2026

The Cargo lane was run end to end by the manager to prove it works before the
next increment depends on it:

- Command: `cargo build --release -p codex-cli --bin corbanu`, run from
  `codex-rs/` with `CARGO_TARGET_DIR=/Volumes/CorbanuDrive/Corbanu/.codex-work/targets/pf83-gate-20260915`
  and `TMPDIR=/private/tmp`, home variables unset.
- Result: `Finished release profile [optimized + debuginfo] in 10m 27s`, exit 0.
- Source commit: `28fde97c447a4aebf0ea4a6affbd2cc9a2ed9bd0`.
- Toolchain: `rustc 1.95.0 (59807616e 2026-04-14)`, `cargo 1.95.0 (f2d3ce0bd 2026-03-21)`.
- Binary: `…/targets/pf83-gate-20260915/release/corbanu`, 322,912,296 bytes,
  reporting `corbanu 0.1.42`.
- SHA-256: `5ac84344bb74053fd8242fe42348a5dfd0615e813552e5028928ea349adfb0c6`.

This is a manager proof that the lane builds, not the frozen executor package:
the package must be rebuilt at whatever commit the gate pins, and its hash
recorded then. The harness already refuses any package whose manifest hash does
not match.
