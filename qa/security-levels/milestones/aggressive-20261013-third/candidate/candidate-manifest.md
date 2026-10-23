# Candidate manifest (third run)

| Field | Value |
| --- | --- |
| Source | `origin/main` at `4b6dd55c85291dcf4458c285f0fc675a0121fd75` (merge of #661; includes #637 #638 #639 #640 #644 #655 #657 #664), clean tree |
| Version | 0.1.48 (`codex-rs/Cargo.toml` workspace version) |
| Build | `scripts/build_codex_package.py --variant corbanu --cargo-profile release` ([build.sh](build.sh)); Rust 1.95.0 pinned toolchain, `CARGO_BUILD_JOBS=8`, the second run's target directory reused incrementally by the same (re-checked-out) build worktree; finished 2026-10-13 14:37Z ([build-head.txt](build-head.txt)). A first build attempt was killed when its launching shell closed; only the second produced this package |
| Platform | macOS arm64 (`aarch64-apple-darwin`) |
| Entry point | `bin/corbanu`, SHA-256 `ad288be98d80d5eb164c2fb34a040565dcc7590a12c58ab2ac3aaf25f0675043` |
| Package | every file, mode and digest in [package-files.txt](package-files.txt); archive `corbanu-aarch64-apple-darwin.tar.gz` SHA-256 `596e285b5bc18aff5682979d0ee6a72578b07cf469cf9f4d3a4d49133d64e804` (kept outside the repository) |
| Signing | ad-hoc linker signature only; not Developer ID signed or notarized |
| Staged on guest | `/opt/agms/pkg`, root-owned, read-only; digests re-checked by every preflight and by every executor |

## Windows and Linux

Not covered. The guest is macOS only; no code-blind milestone execution exists for Windows or Linux. That is a
known limit of this record, not a pass.
