# Corbanu Terminal 0.1.31 pre-publish QA

Date: 2026-08-14 UTC

Release decision: qualified for publication. No tag or release existed when this record was
finalized.

## Final canonical release binary

- Built `corbanu` with `cargo build --release -p codex-cli --bin corbanu` from commit
  `3cbcb3e097` plus the QA record update that follows it.
- Build completed successfully in 10m 16s.
- Output: `codex-rs/target/release/corbanu` (1,324,762,208 bytes before package stripping).
- SHA-256: `4ccf8d33c2931788cdb6309431913c47b52a1cec6677c8267ffcd80ab7e4bed0`.
- The exact binary reports `corbanu 0.1.31` and launches successfully. An isolated auth-empty
  `exec` smoke reached normal session initialization and then failed closed with HTTP 401; no
  authenticated model request or billable inference occurred.
- A fresh isolated home materialized all bundled system skills from the binary. Both
  `frontend-design` and `tasknode-usage` were present.

## Release-profile debug launcher

- Built `corbanu-debug` with `cargo build --profile release --bin corbanu-debug`.
- Cold build completed in 11m 04s.
- Output: `codex-rs/target/release/corbanu-debug` (1,321,643,088 bytes).
- SHA-256: `9ecae2eb42fdaa88d59c57b9adcfa8f53c93ca6aa26f5d31752898815ab3372a`.
- The local `~/.local/bin/corbanu-debug` wrapper now prefers this release-profile binary, retains isolated state under `~/.pfterminal-debug`, and emits two explicit warnings if it must fall back to `target/debug`.
- The wrapper no longer sets a worktree-specific `CODEX_MANAGED_PACKAGE_ROOT`.

## Persisted provider/model recovery

An isolated state home was created with this intentionally incompatible persisted pair:

```toml
model = "gpt-5.6-sol"
model_provider = "claude-plan"
```

The release-profile wrapper was launched with `--yolo --no-alt-screen`. It remained running until the eight-second QA timeout instead of failing configuration loading. The config SHA-256 was `e8eb1708bcb58ce6b0d63acf729bbfcc9a8a0428f4cc6d1d5eb211c205656a05` both before and after startup, confirming that recovery did not rewrite persisted configuration. Unit coverage separately asserts the startup warning and adjacent incompatible provider/model pairs.

## Installed release size debt

The installed `current` link still resolves to `0.1.27-local-claude-history-auth-fix`. Its `bin/pfterminal` is 1,325,153,288 bytes. Do not modify that installed release in place.

Before normal-launch performance results are treated as release evidence:

1. publish a qualified package built through the deduplication/strip packaging path;
2. reinstall that published release so `current` points to the stripped artifact;
3. confirm the installed binary and archive sizes, then rerun launch/performance smoke tests.

## Interactive startup and dispatcher findings

- TUI bootstrap previously called `model/list` with `OnlineIfUncached`, so a cold model cache could
  block first paint on the provider network. The request path now uses the bundled/on-disk catalog
  with `Offline`; the app server's existing refresh worker still updates the catalog online in the
  background. A five-second delayed `/models` regression fixture confirms bootstrap returns the
  bundled catalog within one second. All seven app-server `model_list` tests pass.
- `SubmitCodexUserPaneTask` previously awaited a thread-store mutex inside the main app-event
  dispatcher. It now uses `try_lock`; contention waits outside the dispatcher and re-enqueues the
  original task so ownership and session state are revalidated. The regression test holds the lock,
  requires the UI dispatcher to return within 250 ms, then confirms exactly one provider request
  after release. Both contended and uncontended operator-pane dispatch tests pass.

## Task Node skill and cross-provider fork findings

- The Task Node skill was embedded, but its instructions still hard-coded `pfterminal` and
  `pfterminal-debug`. When the compatibility debug command was absent, an agent attempted a large
  source build instead of using the installed Corbanu helper.
- The skill now resolves installed Corbanu entrypoints while preserving the active `CODEX_HOME`,
  prefers `corbanu-debug` for conventional debug homes, retains legacy names only as fallbacks, and
  explicitly forbids source builds to obtain the helper.
- A live debug-home smoke selected `~/.local/bin/corbanu-debug`, reported `corbanu 0.1.31`, and
  returned linked Task Node status from the same vault. The embedded-skills unit suite passed all
  four tests, including the new command-resolution contract.
- The remaining Ultra fork test exposed a real runtime boundary: `/fork` copied the active model
  but retained the app default provider. A resumed OpenAI thread under Ambient defaults therefore
  attempted the invalid pair `gpt-5.4` + `ambient`.
- Regular forks now derive their model, provider, cwd, permissions, and service tier from the
  active thread runtime snapshot. The strengthened cross-provider Ultra regression creates a new
  thread and preserves both `gpt-5.4` and Ultra.

## Aggregate verification

- `python3 -m pytest scripts/install scripts/codex_package -q`: 53 tests and 17 subtests passed.
- Core incompatible provider/model config coverage: fail-closed and interactive recovery tests
  passed again against the final source candidate.
- `codex-skills`: four tests passed, including embedded Task Node command resolution.
- Full `codex-tui` rerun after refreshing mechanical 0.1.31 version snapshots: 3,816 passed, one
  failed, nine skipped. `safety_retry_preserves_a_committed_steer_from_the_interrupted_turn` passed
  on retry and was reported flaky. The sole persistent failure,
  `fork_current_session_preserves_conversation_ultra`, was then diagnosed as a cross-provider fork
  bug, fixed, and passed in the final targeted library-only run. No known test failure remains.
- Generated incremental caches and enumerated giant test executables were removed after QA. The
  running release-profile `corbanu-debug` binary and wrapper remain intact and both report version
  0.1.31.
