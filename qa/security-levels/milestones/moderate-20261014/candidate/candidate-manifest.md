# Candidate manifest (Moderate, first run)

| Field | Value |
| --- | --- |
| Source | `origin/main` at `899cf3e62a08e3469aba5712fc659b114d19fdc0` (merge of #791; includes the Moderate picker sprint #764 and #781, and #760 for #748), clean tree |
| Version | 0.1.48 (`codex-rs/Cargo.toml` workspace version) |
| Build | `scripts/build_codex_package.py --variant corbanu --cargo-profile release` ([build.sh](build.sh)); Rust 1.95.0 pinned toolchain, `CARGO_BUILD_JOBS=8`, a fresh target directory for this worktree; finished 2026-10-14 14:20Z ([build-head.txt](build-head.txt)). A first attempt was stopped when its launching shell closed, before any artifact; only the second produced this package |
| Platform | macOS arm64 (`aarch64-apple-darwin`) |
| Entry point | `bin/corbanu`, SHA-256 `379d421c34de833119d266eebec15b1d5c2dc0dbeaa4b1a0079050e051442692` |
| Package | every file, mode and digest in [package-files.txt](package-files.txt); archive `corbanu-aarch64-apple-darwin.tar.gz` SHA-256 `a27518ed00aa45c20bcb40c559c67653e3cc6cc7801bc328bd9366164e3a856e` (kept outside the repository) |
| Signing | ad-hoc linker signature only; not Developer ID signed or notarized |
| Staged on guest | `/opt/agms/pkg`, root-owned, read-only; digests re-checked by every preflight and by every executor |

## Windows and Linux

Not covered (D7). The guest is macOS only; no code-blind milestone execution exists for Windows or Linux. That is a
named limit of this record, not a pass.
