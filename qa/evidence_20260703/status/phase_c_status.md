# Phase C Status — PFTerminal 0.1.2 Release

## 2026-07-03 09:08 UTC — release complete / stop before Phase D

Status: COMPLETE — Phase C tri-platform release artifacts verified; STOP before Phase D.

Merge:

- PR: https://github.com/agtico/PfTerminal/pull/22 (`QA release fixes 2026-07-03`)
- PR merge SHA: `26eaec0e09f16c2f73c89d2c3ba06c41d3f331e9`
- Version bump commit: `9956f0336f921c12901cfd8fef895f7cb465274f` (`Prepare PFTerminal 0.1.2 release`)
- PR #21 / Fable: untouched.

Release:

- Tag: `rust-v0.1.2`
- Tag target: `9956f0336f921c12901cfd8fef895f7cb465274f`
- Release URL: https://github.com/agtico/PfTerminal/releases/tag/rust-v0.1.2
- Workflow run: https://github.com/agtico/PfTerminal/actions/runs/28645682785
- Workflow result: SUCCESS
- Workflow head SHA: `9956f0336f921c12901cfd8fef895f7cb465274f`

Workflow jobs:

- PASS: Validate release inputs — https://github.com/agtico/PfTerminal/actions/runs/28645682785/job/84951603995
- PASS: Build Windows package - `x86_64-pc-windows-msvc` — https://github.com/agtico/PfTerminal/actions/runs/28645682785/job/84951631082
- PASS: Build macOS package - `aarch64-apple-darwin` — https://github.com/agtico/PfTerminal/actions/runs/28645682785/job/84951631088
- PASS: Build Linux package - `x86_64-unknown-linux-gnu` — https://github.com/agtico/PfTerminal/actions/runs/28645682785/job/84951631090
- PASS: Build macOS package - `x86_64-apple-darwin` — https://github.com/agtico/PfTerminal/actions/runs/28645682785/job/84951631094
- PASS: Build Linux package - `aarch64-unknown-linux-musl` — https://github.com/agtico/PfTerminal/actions/runs/28645682785/job/84951631104
- PASS: Assemble release assets — https://github.com/agtico/PfTerminal/actions/runs/28645682785/job/84965861595

Release assets and checksums:

- `pfterminal-package-x86_64-unknown-linux-gnu.tar.gz`
  - URL: https://github.com/agtico/PfTerminal/releases/download/rust-v0.1.2/pfterminal-package-x86_64-unknown-linux-gnu.tar.gz
  - Size: `219479996`
  - SHA256: `22e639fd0d3c4d2889a25bb0285b7d5d30005dee552a80bb6ad8929f284263d9`
- `pfterminal-package-aarch64-unknown-linux-musl.tar.gz`
  - URL: https://github.com/agtico/PfTerminal/releases/download/rust-v0.1.2/pfterminal-package-aarch64-unknown-linux-musl.tar.gz
  - Size: `208255070`
  - SHA256: `842148058dcfb7c7a7dd71e54493451c8b0eb50df2049c32f8240e011b5682a6`
- `pfterminal-package-aarch64-apple-darwin.tar.gz`
  - URL: https://github.com/agtico/PfTerminal/releases/download/rust-v0.1.2/pfterminal-package-aarch64-apple-darwin.tar.gz
  - Size: `200656076`
  - SHA256: `3c5cf6ddc41d842b25fd8ae9d7cabdeceb20191f5ea431f64db64985d881cdd0`
- `pfterminal-package-x86_64-apple-darwin.tar.gz`
  - URL: https://github.com/agtico/PfTerminal/releases/download/rust-v0.1.2/pfterminal-package-x86_64-apple-darwin.tar.gz
  - Size: `211583767`
  - SHA256: `110e06ce8da23f0870560e81ece34e335b53d1be08c3b12241e361522c44a9b5`
- `pfterminal-package-x86_64-pc-windows-msvc.zip`
  - URL: https://github.com/agtico/PfTerminal/releases/download/rust-v0.1.2/pfterminal-package-x86_64-pc-windows-msvc.zip
  - Size: `236569901`
  - SHA256: `2edc9cd92b793ff4a2823e3edca435da8d34e05bd337ed62b87a5ceb474da1be`
- `PFTerminal-aarch64-apple-darwin.dmg`
  - URL: https://github.com/agtico/PfTerminal/releases/download/rust-v0.1.2/PFTerminal-aarch64-apple-darwin.dmg
  - Size: `202140320`
  - SHA256: `bd9fe05a7bd2284ee33a3cb3980597f9655641f017a4a88a16e0cdbe358b3f3d`
- `PFTerminal-x86_64-apple-darwin.dmg`
  - URL: https://github.com/agtico/PfTerminal/releases/download/rust-v0.1.2/PFTerminal-x86_64-apple-darwin.dmg
  - Size: `213294925`
  - SHA256: `e2fae9a4be68188fe7f964447b207554cc37f23a7ca969c9403be2cfc0395be9`
- `install.sh`: https://github.com/agtico/PfTerminal/releases/download/rust-v0.1.2/install.sh
- `install.ps1`: https://github.com/agtico/PfTerminal/releases/download/rust-v0.1.2/install.ps1
- Checksum manifests:
  - https://github.com/agtico/PfTerminal/releases/download/rust-v0.1.2/pfterminal-package_SHA256SUMS
  - https://github.com/agtico/PfTerminal/releases/download/rust-v0.1.2/pfterminal-dmg_SHA256SUMS

Installability evidence:

- Evidence directory: `/home/pfrpc/repos/pfterminal_qa_20260703/phase_c_release_0.1.2`
- Downloaded Linux artifact: `pfterminal-package-x86_64-unknown-linux-gnu.tar.gz`
- Checksum verification: `pfterminal-package-x86_64-unknown-linux-gnu.tar.gz: OK`
- Extracted binary command: `linux_extract/bin/pfterminal --version`
- Output: `codex-cli 0.1.2`
- Compatibility wrapper command: `linux_extract/bin/codex --version`
- Output: `codex-cli 0.1.2`

macOS / Windows execution note:

- macOS artifacts are present and checksums are recorded above, including both package archives and DMG installers. They were not executed on this Linux host.
- Windows artifact is present and checksum is recorded above. It was not executed on this Linux host.

Local release-prep verification:

- Local version validation matched workflow rule: `codex-rs/Cargo.toml` version `0.1.2`; tag `rust-v0.1.2`.
- Local clean rebuild after freeing generated target artifacts: PASS, `cargo build -p codex-cli --bin pfterminal`.
- Local debug binary version before tagging: `codex-cli 0.1.2`.

STOP:

- Phase C is complete.
- No Phase D local install reset/QA was performed.
- No additional pushes after tag/release workflow dispatch.
