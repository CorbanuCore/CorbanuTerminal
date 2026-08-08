# PfTerminal 0.1.28 acceptance ledger

Date: 2026-08-08. Operator: goodalexander. Preparer: release agent
(session-supervised). Scope decision recorded by the operator: 0.1.28
ships PR #93 only. PR #94 (upstream Codex 0.147.0 sync) is explicitly
excluded and deferred to the 0.1.29 cycle.

## Candidate identity

- Branch: `release/0.1.28`
- Base: `main` at `abfc1daac1` (merge of PR #93)
- Version: `0.1.28` in `codex-rs/Cargo.toml` (lockfile refreshed via
  `cargo check -p codex-cli`; version-only changes, no dependency
  selection changes)
- Previous release: `0.1.27` (`rust-v0.1.27` = `827ad686`)

## Release content

PR #93 `fix(tasknode): non-destructive auth-state machine with headless link`:

- `5d623904c` non-destructive auth-state machine with headless link
- `b6d546060` expiry-aware session resolution
- `5bbb5b636` tasknode-usage skill resolves the entrypoint matching the
  session home
- `611993d3e` rustfmt fixes; `801f9a13f` clippy fixes (redundant clone,
  unused import); `598f5059a`/`d474ffd3f` codespell CI greening (config
  plus one real typo fix in `psuedocon.rs`)

Field evidence: the auth-state machine was exercised end-to-end in
production on 2026-08-07 (expired-session shadowed link recovered via
poll→validate→promote; operator's live session relinked).

## Feature-manifest gate

- Baseline: `pf-feature-manifest-0.1.27.json` from `rust-v0.1.27`
- Candidate: `pf-feature-manifest-candidate.json` from the release
  commit
- Comparison: `pf-feature-comparison.json` —
  `difference_count 0`, `unresolved_difference_count 0`,
  `invalid_allowlist_entry_count 0`, empty allowlist.
- Interpretation: #93 changes tasknode internals and adds the
  `codex-tasknode-session` crate without altering any inventoried
  product surface (binaries, CLI subcommands, slash commands,
  configuration, model catalog, app-server methods, persistence,
  platform artifacts, protected integrations).

## Test evidence (Linux x86_64, this host)

Release-facing local suites, all exit 0:

- `scripts/dev/test_check_worktree.sh`
- `python3 -m unittest scripts/release/test_pf_feature_manifest.py`
- `python3 scripts/install/test_pfterminal_release_contract.py`
- `python3 -m unittest discover -s scripts/codex_package`
- `python3 -m unittest discover -s scripts/install`

Focused Rust suites on the release tree, all green:

- `cargo test -p codex-tasknode-session`: 10 passed, 0 failed
- `cargo test -p codex-tui --lib tasknode`: 2 passed (3803 filtered)
- `cargo test -p codex-cli --bin pfterminal tasknode`: 6 passed
  (243 filtered)
- `cargo fmt --check`: clean; `cargo clippy -p codex-cli --no-deps`:
  no errors (remaining warnings pre-exist on main in untouched crates)
- Local `codespell --count`: exit 0 after config completion

## CI state

On the #93 head before merge: `rust-ci / Format` pass,
`Codespell` pass (first green after config fix). Failing jobs
(`cargo-deny`, `cargo shear`, `repo-checks/build-test`, Bazel suite)
fail identically on `main` and are pre-existing infrastructure debt,
not introduced by this release:

- cargo-deny: advisories on existing dependencies (deps added by #93
  are workspace-internal only)
- cargo shear: unlinked test files in `codex-core` / `codex-telegram`
- Bazel: build failures present on main's own runs

## Windows evidence (postfiat1, Windows 11 26200, x64, 2026-08-08)

From-source verification on real Windows (MSVC 14.44, stable-msvc
toolchain, clean clone of `release/0.1.28` on D:):

- `cargo test -p codex-tasknode-session`: 10 passed, 0 failed
  (native Windows paths; includes migration, expiry, promotion cases)
- `cargo test -p codex-cli --bin pfterminal tasknode`: 6 passed
- `cargo build -p codex-cli --bins`: success;
  `pfterminal.exe --version` reports `pfterminal 0.1.28`
- Unlinked `tasknode status` returns the state machine's correct
  guidance ("not linked → run pfterminal tasknode link") with proper
  Windows path rendering
- `codex-utils-pty` (Windows pseudo-console, includes this release's
  string fix) compiles into the shipped binaries
- Cross-check from Linux: `cargo check --target x86_64-pc-windows-msvc
  -p codex-utils-pty` clean

Observations (neither introduced by this release):

- `pfterminal-debug` does not auto-create its state home on first run;
  it errors clearly until the directory exists. First-run UX nit.
- OS-keyring vault operations fail under SSH sessions
  (`ERROR_NO_SUCH_LOGON_SESSION`) because Windows secure storage
  requires an interactive logon. Interactive-session linking was
  therefore not exercised remotely.

## Open cells (operator acceptance required)

1. Hands-on matrix: macOS NOT run; Windows covered from source as
   above, but packaged-ZIP install and interactive TUI/link on Windows
   remain unexercised (SSH keyring limitation). Linux packaged-binary
   behavior is exercised daily on this host, including the debug-home
   isolation this release fixes.
2. Full workspace `cargo test`: NOT run (multi-hour; focused suites and
   production field evidence stand in). 
3. Pre-existing red CI jobs on main, listed above, ride along
   unchanged.

Per the runbook these residual risks require explicit operator
acceptance in the release PR before merge.

## Authorization checkpoints

- [ ] Operator merges `release/0.1.28` PR (accepts open cells above)
- [ ] Unpublished workflow run (`publish_release=false`) hashes
      reconciled against this ledger
- [ ] Operator authorizes tag push and publication
## Post-release verification (2026-08-08)

Release 0.1.28 published from tag rust-v0.1.28 = 0f01128d77 (merge of
PR #95), marked Latest, not draft, not prerelease. All 11 assets
present. Both SHA256 manifests (copied into this directory) verified
against downloaded assets from the published release, and match the
qualification-run artifact bundle from run 31264209533. The packaged
Windows ZIP from qualification was additionally smoke-tested on
postfiat1 (reports pfterminal 0.1.28).
