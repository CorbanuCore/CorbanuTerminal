# Corbanu Terminal 0.1.35 model-header synchronization fix

Date: 2026-08-29 UTC

Status: bounded implementation and focused qualification complete. This record does not qualify 0.1.35 for publication.

## Product contract

Product specification heading: `# Shipping MVP — LIVE`

Requirement excerpt: “Multi-provider inference: OpenAI Codex plan, Anthropic, OpenRouter, Gemini, DeepSeek, Kimi Code, Z.AI, Meta, Vercel AI Gateway, Baseten, and Ambient.”

The model selected through `/model` must be represented consistently throughout the active terminal UI.

## Failure and fix

The footer read its model and reasoning effort from the live chat-widget state, while the boxed session header was committed to transcript history as a static history cell. Model-selection events refreshed the footer immediately but never updated the committed header, leaving the two indicators inconsistent until a new session. `Ctrl-L` made the same problem permanent by inserting only pre-rendered header lines.

The session-header history cell now owns updateable model details and renders from that source-backed state. Model, reasoning-effort, advanced-reasoning, and service-tier events refresh the committed header, rebuild an open transcript overlay, and reflow the transcript. The `Ctrl-L` path now commits a source-backed header cell rather than immutable lines, so subsequent model changes remain synchronized.

## Automated evidence

- Rust formatting passed with all Cargo and build state rooted on CorbanuDrive.
- `git diff --check` passed.
- `model_change_refreshes_committed_session_header_snapshot` passed and verifies that a committed session-info header changes from `gpt-before medium` to `gpt-after high`.
- `clear_header_remains_source_backed_for_model_refresh` passed and verifies that a header recreated by the clear path remains updateable.
- `cargo build --bin corbanu --bin codex-code-mode-host` completed successfully using the CorbanuDrive-local target directory.
- The stable launcher executable resolves to the rebuilt arm64 Mach-O `target/debug/corbanu`; its SHA-256 matched the target binary exactly.
- The broader `just test -p codex-tui` run executed 3,844 tests: 3,806 passed, 38 failed, and 7 were skipped. The 38 failures are existing broad-suite qualification debt outside this patch, dominated by release-version snapshot drift plus the daemon probe fixture. Both regressions added for this fix passed.

## True-TUI evidence

The repository `just codex` target was launched in a PTY with `TERM=xterm-256color`, `RUST_LOG=trace`, `--no-alt-screen`, and a CorbanuDrive-local `log_dir`.

1. Confirmed the initial boxed header showed `gpt-5.6-sol high` and the footer showed `GPT-5.6-Sol high`.
2. Invoked `/model`, selected GPT-5.6-Terra with medium reasoning, and confirmed the boxed header immediately showed `gpt-5.6-terra medium` while the footer showed `GPT-5.6-Terra medium`.
3. Pressed `Ctrl-L` and confirmed the recreated boxed header still showed the active Terra model.
4. Invoked `/model` again, selected GPT-5.6-Luna with medium reasoning, and confirmed the boxed header immediately showed `gpt-5.6-luna medium` while the footer showed `GPT-5.6-Luna medium`.
5. Exited with `/exit` and confirmed clean shutdown. The trace contains no panic or error-level event from the verification run.

Trace artifact: `/Volumes/CorbanuDrive/Corbanu/.codex-work/corbanu-terminal/evidence/model-header-sync/logs/codex-tui.log`

## Review

The structured Codex autoreview helper reviewed the complete uncommitted diff with `gpt-5.5`. It reported no accepted or actionable findings, judged the patch correct with 0.82 confidence, and specifically confirmed the source-backed clear-screen header, the relevant model/reasoning/service-tier refresh paths, and the mutable header cell's transcript-height behavior.

## Release blockers outside this fix

- Complete the mandatory benchmark bootstrap cycle and evidence package described in `benchmarks/README.md` before publishing 0.1.35.
- Resolve the outstanding broad-suite release-version snapshots and daemon probe fixture through the normal release preparation flow.
- Obtain the required human release sign-off before publication.
