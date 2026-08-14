# Corbanu Terminal 0.1.31 pre-publish QA

Date: 2026-08-14 UTC

Publishing remains a human decision. No tag or release was created by this QA run.

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
