# Candidate (targeted re-run, 2026-10-14)

- Source: origin/main `4440d2d4ce3abce819603392278113993fd63374` (merge of #662; includes #752, #724, #742), clean tree
  (`build-head.txt`). Built with `build.sh` (pinned Rust 1.95.0, release profile, `scripts/build_codex_package.py --variant corbanu`).
- Package: macOS arm64, Corbanu Terminal 0.1.48. `bin/corbanu` SHA-256
  `dc61bd6403b99a892b3db4ed59fef22f30a4ff65f1cd2a9ee165e994c2a50483`. Archive `corbanu-aarch64-apple-darwin.tar.gz`
  SHA-256 `bb98993e76743e4888e0359a4bccfbfbfaa1efd7ec501156f9c37e75cb83c92a`.
- Every package file with mode and SHA-256: `package-files.txt`. The guest's `/opt/agms/pkg` matched it
  (`../common/provision-1.out`), and each executor's preflight checked the binary digest.
