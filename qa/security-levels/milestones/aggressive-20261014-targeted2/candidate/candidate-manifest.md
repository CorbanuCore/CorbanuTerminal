# Candidate (second targeted re-run, 2026-10-14)

- Source: origin/main `df3d8c5d1c521228a8dbb1035831af572afe1284` (merge of #806; includes #797, #798, #799 and #801), clean
  tree (`build-head.txt`). Built with `build.sh` (pinned Rust 1.95.0, release profile, `scripts/build_codex_package.py --variant corbanu`).
- Package: macOS arm64, Corbanu Terminal 0.1.48. `bin/corbanu` SHA-256
  `0deeb58c7882cc9bb246fe2b423b6f258d2eb39561917b3884221cab7e1049b3`. Archive `corbanu-aarch64-apple-darwin.tar.gz`
  SHA-256 `0de7aa5c3a3b879034713ca205f5be2cfcab5edea05feb979fb21ee1ee2e6556`.
- Every package file with mode and SHA-256: `package-files.txt`. The guest's `/opt/agms/pkg` matched it
  (`../common/provision-1.out`), and each executor's preflight checked the binary digest.
