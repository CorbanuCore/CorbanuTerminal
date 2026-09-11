# Candidate identity and proof limits

Reconciled source checkpoint: `da77f7c03827d56284d62a3dadb477aed36bb6ce`.
Main included: `0d1c3240d752335cc89faeb2149c5030063e5d69`; published release 0.1.41.
Subsequent commits add QA/process artifacts only. The rebase changed no Rust
source or packaging scripts relative to the pre-rebase checkpoint.

macOS arm64, version 0.1.41:

- Actual package: `/Volumes/CorbanuDrive/Corbanu/.codex-work/provider-reauth-health/macos-candidate-picker-final-20260910`.
- Actual stable launcher target: `/Volumes/CorbanuDrive/Corbanu/.codex-work/corbanu-terminal/bin/corbanu`.
- Executable SHA256: `8275923c4ee9c0bfc7e52109f742564c5a429631d765e15b6f0643b4cb7d3669`.
- Host SHA256: `dc9d05cff406d18f1f6720b777a776d74f949b49c6df9d8a7c0bd654187b1fcf`.

Linux x86_64, version 0.1.41:

- Actual package: `/home/travis/security-round5/evidence/picker-20260910/candidate-final`.
- Executable SHA256: `5ecd62f90574e1b6208625220abf93e431d347c1f2a91c9111c3512e81448062`.
- Host SHA256: `b580c89d1300d891bacd2f8fe8118dc859e944e2f755594cae90138f80c8a4fd`.
- Test source mirror: `/home/travis/security-round5/picker-repair-20260910`.
  Read-only rsync checksum comparison of `codex-rs` against the allocated Mac
  worktree found no regular-file differences (target and .git excluded); the
  symlink `vendor/bubblewrap/LICENSE` was skipped, not verified by that command.

These are the existing exact packages, not rebuilt binaries with a new commit
embedded. Source equality supports reuse but does not turn historical tests
into new passes. The test result JSON and per-scenario binary SHA files identify
the binaries actually exercised in the new run. The isolated tests use fake
model/OAuth endpoints and synthetic credentials. Only `mac-existing` reads the
existing real profile, without a model call, credential edits or global tracing.

No native GUI automation, Keychain denial/allow manipulation, new live account
login, live inference billing proof or human acceptance is claimed. Existing
macOS menu tests execute the shortcut's target through tmux, not the native
Applications shortcut itself. Historical screenshots in the designer packet
are not screenshots of these new runs; new artifacts are actual PTY text captures.
