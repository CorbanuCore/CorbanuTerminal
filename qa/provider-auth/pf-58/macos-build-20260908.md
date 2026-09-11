# macOS and Linux candidate parity

User requested a native Mac build as well as the existing RTX Linux build before
testing the Mac launcher. This is build/staging QA for the existing PF-58 repairs,
not a product-scope change or release publication. Product: **Shipping MVP —
LIVE**, “Rust, Apache-2.0, Linux/macOS/Windows” and shipped MCP/workspace support.

The source remains `feat/provider-reauth-health` in the allocated worktree, with
the uncommitted repaired source manifest recorded in
`evidence/repairs-20260908/source.sha256`. Linux's existing 0.1.41 package and that
manifest were rechecked successfully. Its CLI SHA-256 is
`fa2b710733a57787cb0cfbd250be903ef9d2fe1f08b981dd49196697eae14fb8`.

Mac build output, Cargo/Rustup caches, package resources and temporary files stay
on CorbanuDrive. Preserve the old launcher installation until the new package
passes its checks. Desktop 3/grid placement, Keychain behavior and human
acceptance are not implied by successful compilation.

## Result

Native Mac release build succeeded in **10m 03s** (warm APFS-cloned build cache;
not an interference-controlled benchmark). Built `corbanu`,
`codex-code-mode-host`, `corbanu-walletd` and `corbanu-acp`. The canonical
`corbanu` package also includes its debug alias, pinned ripgrep/zsh and Telegram
resources. No Rust source changed during this build task. All 16 files in the
repaired source manifest matched locally and in the RTX source mirror.

Mac package:
`/Volumes/CorbanuDrive/Corbanu/.codex-work/provider-reauth-health/macos-candidate-repaired-20260908`

- Version: `corbanu 0.1.41`, native `aarch64-apple-darwin` / Mach-O arm64.
- CLI SHA-256: `22b78e2d5c26865b454103229220bbc922839208aee5eff7f5ea03f22aa033a0`.
- All four built binaries ad-hoc signed after packaging/stripping; strict
  signature verification passed. No app signature, Keychain ACL or TCC change.
- Code Mode and wallet helper help commands, bundled ripgrep and zsh ran.
- Canonical package layout validation passed.
- [Actual-package TMUX smoke](evidence/macos-20260908/result.json): **PASS**.
  Four actual Code Mode → shell/synthetic MCP round trips; three concurrent
  TUI processes; same-home restart; streaming cancellation/recovery; inert
  permission/security cancellation; Claude token guidance at 40 columns.
  Synthetic credentials and disposable state only; not live-account testing.
- [Package hashes](evidence/macos-20260908/package.sha256) and
  [metadata](evidence/macos-20260908/codex-package.json) archived alongside the
  TMUX captures/results.

Linux package remains
`/home/travis/security-round5/evidence/provider-reauth-health-final/candidate-repaired-qualified/bin/codex`.
Its version, SHA above and repaired-source manifest were rechecked over SSH in
this task; no redundant Linux rebuild was performed. Existing qualification is
45/45 TMUX suite tests plus package/tool and live Fable/MCP checks, as recorded
in `product-repairs-20260908.md`. Mac/Linux binaries necessarily have different
hashes. Mac smoke coverage is narrower than the Linux suite, not 45 Mac tests.

## Apps launcher installation

`/Applications/Corbanu Terminal Launcher.app` retains its existing bundle and
compiled script. That script launches
`/Volumes/CorbanuDrive/Corbanu/.codex-work/corbanu-terminal/bin/corbanu` after
sourcing the existing external-drive environment. This stable binary symlink
now resolves to the qualified Mac package above. Stable sibling links for
Code Mode, wallet and ACP also resolve to that package; tmux/Bazel links and the
separate debug installation were not changed. Running terminals were untouched.
Launch a **new** window from Applications to use the replacement.

The compiled app script SHA-256 is unchanged:
`68103509480e70f4d92019b6554e3253e71347cb32321d1c13af81eb5584862f`.
The old release files remain at
`/Volumes/CorbanuDrive/Corbanu/.codex-work/corbanu-terminal/target/release`.
Rollback targets were `bin/corbanu -> ../target/release/corbanu` and
`bin/codex-code-mode-host -> ../target/release/codex-code-mode-host`;
wallet/ACP links in `bin` were newly added. Copies of the old symlink entries
are in `macos-evidence/previous-launcher-links` (their relative targets are
documentary outside the original bin directory).

Build target and full build/package logs are under
`/Volumes/CorbanuDrive/Corbanu/.codex-work/provider-reauth-health/` in
`macos-target/` and `macos-evidence/`. Cargo/Rustup and temporary downloads use
the pre-existing external-drive toolchain environment. Old artifacts remain
available; no main-system-drive build cache was introduced.

## Remaining handoff limits

This satisfies the missing **Mac build/install** prerequisite, not Desktop 3
placement, grid behavior, real-account authentication or Keychain acceptance.
`humanTest.html` keeps all 11 partially unverified cards yellow and all 26
checkbox identities/saved acceptance state unchanged. Its presentation changed;
earlier archived gate-report HTML hashes remain historical. No product changes,
additional independent reviews, release, commit, merge or push in this task.
PF-58 remains in progress; no new human sign-off or benchmark claim.
