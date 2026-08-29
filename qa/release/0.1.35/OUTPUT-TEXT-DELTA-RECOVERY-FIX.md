# Corbanu Terminal 0.1.35 output-text delta recovery fix

Date: 2026-08-29 UTC

Status: bounded implementation and focused qualification complete. This record does not qualify 0.1.35 for publication.

## Product contract

Product specification heading: `# Shipping MVP — LIVE`

Requirement excerpts: “Rust, Apache-2.0, Linux/macOS/Windows, the `corbanu` command, and legacy `pfterminal` command and state compatibility.” “Multi-provider inference: OpenAI Codex plan, Anthropic, OpenRouter, Gemini, DeepSeek, Kimi Code, Z.AI, Meta, Vercel AI Gateway, Baseten, and Ambient.”

A malformed or out-of-order provider stream must not crash a long-running Corbanu Terminal session.

## Failure and fix

The normalized `ResponseEvent::OutputTextDelta` discarded the Responses API `item_id`. The turn loop therefore associated text solely with one mutable `active_item` and called the debug-only `error_or_panic` path when a delta arrived before `response.output_item.added`. Corbanu's stable launcher pointed at a debug build, turning that invariant violation into the reported application panic.

Text deltas now retain their provider or synthesized item identity across Responses, Chat Completions, and Anthropic streams. The turn loop correlates deltas with the active item, buffers early text up to a hard 1 MiB limit, reconciles it when the matching item arrives, and fails the stream for retry when identity, content, or completion cannot be reconciled. A canonical completed item safely supersedes matching buffered text. No orphan text is attached to a different item.

The stable launcher target now resolves to the release build. A separate `Corbanu Terminal Debug Launcher.app` uses `corbanu-debug`, its matching debug code-mode host, and an isolated `CORBANU_DEBUG_HOME`, all on CorbanuDrive. The launcher source continues to use stable symlink paths, so rebuilding their targets does not require rebuilding the launcher app.

## Automated evidence

- `just fmt` passed.
- `git diff --check` passed.
- The three `PendingOutputText` unit tests passed, covering late identity binding, interleaved-item rejection, and the hard byte limit.
- `output_text_delta_before_item_added_is_recovered` passed and verified both streamed “Hello world” deltas and the canonical completed assistant item.
- All 199 `codex-api` tests passed, including preservation of item IDs for Responses, Chat Completions, and Anthropic normalization.
- `cargo build --release -p codex-cli --bin corbanu -p codex-code-mode-host --bin codex-code-mode-host` completed successfully with Cargo state and artifacts rooted on CorbanuDrive.
- `cargo build -p codex-cli --bin corbanu-debug -p codex-code-mode-host --bin codex-code-mode-host` completed successfully on CorbanuDrive.
- Stable launcher hashes match the release targets exactly: `corbanu` SHA-256 `39e64dafc50e7331090ce54e1c6820eb31d516a2e845434e749e3f450eb45d98`; code-mode host SHA-256 `0f282b1bde28ff21e92d2166adcf20f8496c61feb50fe4d5f49ff8f47bede2fc`.
- Both release and debug entry points report `corbanu 0.1.35`; both code-mode hosts accept `--help`; all four are arm64 Mach-O executables.
- The debug launcher passes strict deep code-signature verification and its compiled script points at the drive-local debug binary and home.

## True-TUI evidence

The mandatory tmux harness was run with `CORBANU_TMUX_REQUIRED=1`, retries disabled, a freshly built debug candidate, isolated `CODEX_HOME`, `RUST_LOG=trace`, and a CorbanuDrive artifact root.

1. A pre-fix debug candidate reproduced the reported `core/src/util.rs:95` panic after receiving `response.output_text.delta` before `response.output_item.added`.
2. The rebuilt candidate was launched in a 120x40 tmux PTY against a deterministic mocked Responses stream with that same event order.
3. The prompt was sent as literal text and Enter as a separate key event.
4. The TUI rendered `early delta recovery sentinel`, remained interactive, accepted `/exit`, and terminated cleanly.
5. The final harness result was 1 passed, 0 failed, with retries set to zero.

## Review

The repository autoreview helper reviewed the complete uncommitted diff with `gpt-5.5`. It reported no accepted or actionable findings, judged the patch correct with 0.83 confidence, and specifically confirmed the item-aware recovery scope, enum-consumer updates, and focused stream-order coverage.

## Release blockers outside this fix

- Complete the mandatory benchmark bootstrap cycle and evidence package described in `benchmarks/README.md` before publishing 0.1.35.
- Resolve outstanding broad-suite release-version snapshots and other known release qualification debt through the normal release preparation flow.
- Obtain the required human release sign-off before publication.
