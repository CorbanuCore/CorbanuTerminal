# Candidate manifest

| Field | Value |
| --- | --- |
| Source | `origin/main` at `9ead7a57558f8f0639511cc5cbcc8ec726756ed3` (merge of #413, `broker_model_auth` default-on under Aggressive), clean tree |
| Version | 0.1.48 (`codex-rs/Cargo.toml` workspace version; same number as the installed release) |
| Build | `scripts/build_codex_package.py --variant corbanu --cargo-profile release` ([build.sh](build.sh)); Rust 1.95.0 pinned toolchain, thin LTO, `CARGO_BUILD_JOBS=8`, own target dir; finished 2026-10-10 22:29Z ([build-head.txt](build-head.txt)) |
| Platform | macOS arm64 (`aarch64-apple-darwin`) |
| Entry point | `bin/corbanu`, SHA-256 `181ad0299d238accbf936512531c90988e8d15cb02a50ea8241f8566b2372a4a` |
| Package | every file, mode and digest in [package-files.txt](package-files.txt); archive `corbanu-aarch64-apple-darwin.tar.gz` SHA-256 `7da035720857249a9fa704fe9a437b7627e577e588c36263c8b3182aeffd8a67` (kept outside the repository) |
| Signing | ad-hoc linker signature only; not Developer ID signed or notarized (release CI does that) |
| Staged on guest | `/opt/agms/pkg`, root-owned, files 0444/0555, dirs 0555; digests re-checked by every preflight and by every executor |

## Windows and Linux

Not covered by this run. The guest is macOS only. Windows broker and launch were qualified separately on a real
Windows host (PF-27-S06 to S09, archived with known limits); Linux by the PF-13-S07 canary (rerun on the current
candidate is still open in the plan). No code-blind milestone execution exists for either platform; that is a known
limit of this milestone record, not a pass.
