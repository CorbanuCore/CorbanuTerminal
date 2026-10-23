# Candidate manifest (re-run)

| Field | Value |
| --- | --- |
| Source | `origin/main` at `6ebfa1f47c849c264c8b8c64beb64f8b6bf2c7b7` (merge of #520), clean tree |
| Version | 0.1.48 (`codex-rs/Cargo.toml` workspace version) |
| Build | `scripts/build_codex_package.py --variant corbanu --cargo-profile release` ([build.sh](build.sh)); Rust 1.95.0 pinned toolchain, `CARGO_BUILD_JOBS=8`, own target dir; finished 2026-10-12 04:37Z ([build-head.txt](build-head.txt)). A first build attempt was stopped and restarted; only the second produced this package |
| Platform | macOS arm64 (`aarch64-apple-darwin`) |
| Entry point | `bin/corbanu`, SHA-256 `9fe390351b2e2bd08fad3b210697f0e95156910baa829d1b09423008eb8be5d1` |
| Package | every file, mode and digest in [package-files.txt](package-files.txt); archive `corbanu-aarch64-apple-darwin.tar.gz` SHA-256 `0f9b4df88911ce2a0d64af6ab4b54185f43610874d4e470feefed27c8263e5d3` (kept outside the repository) |
| Signing | ad-hoc linker signature only; not Developer ID signed or notarized |
| Staged on guest | `/opt/agms/pkg`, root-owned, read-only; digests re-checked by every preflight and by every executor |

## Windows and Linux

Not covered. The guest is macOS only; no code-blind milestone execution exists for Windows or Linux. That is a
known limit of this record, not a pass.
