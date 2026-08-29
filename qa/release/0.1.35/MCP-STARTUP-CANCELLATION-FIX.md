# Corbanu Terminal 0.1.35 MCP startup cancellation fix

Date: 2026-08-29 UTC

Status: bounded implementation and focused qualification complete. This record does not qualify 0.1.35 for publication.

## Product contract

Product specification heading: `# Shipping MVP — LIVE`

Requirement excerpt: “`/panes`, `/agent`, approvals, existing general sandboxing, review, MCP, skills, plugins, apps, connectors, and background terminals.”

Normal MCP connection replacement, supersession, and shutdown must not be presented as a failed user-visible MCP startup.

## Failure and fix

`McpServerConnection::drop` cancelled the same token used by explicit startup interruption. The connection-manager startup task converted every cancelled token into `McpStartupStatus::Cancelled`, so a normal lifecycle replacement could produce `MCP startup interrupted` even when `codex_apps` initialized and its tools were available.

Each connection now shares a first-writer-wins startup-cancellation disposition with its startup task. Explicit `cancel_startup` remains reportable. Drop, shutdown, and unpublished/superseded candidates are silent, so they do not emit a cancelled update or enter the cancelled startup summary. Genuine cancellation behavior remains unchanged and snapshot-covered.

## Automated evidence

- `just fmt` passed.
- `git diff --check` passed.
- `just test -p codex-mcp` passed 144 tests, 0 failed. This includes `dropping_pending_connection_suppresses_lifecycle_cancellation_warning` and `explicit_startup_cancellation_remains_user_visible`.
- `just test -p codex-tui explicit_mcp_startup_cancellation_renders_warning_history` passed 1 test, 0 failed; 3,847 tests were filtered out.
- `cargo build --bin corbanu --bin codex-code-mode-host` completed successfully with all Cargo state and build artifacts rooted on CorbanuDrive.
- The stable launcher targets resolve to the rebuilt Mach-O arm64 executables. `corbanu --version` reports `corbanu 0.1.35`, and `codex-code-mode-host --help` exits successfully.

## True-TUI evidence

The repository `just codex` target was launched in a PTY with `TERM=xterm-256color`, `RUST_LOG=trace`, `--no-alt-screen`, and a CorbanuDrive-local `log_dir`.

1. Confirmed the TUI rendered `Booting MCP server: codex_apps` and then returned to the ready composer without an MCP interruption warning.
2. Submitted `Reply with exactly: verification-ready` with text and Enter as separate terminal events.
3. Confirmed the assistant rendered `verification-ready`.
4. Exited with `/exit` and confirmed clean shutdown.
5. The trace records `codex_apps` service initialization and contains zero occurrences of `MCP startup interrupted`. Lifecycle cancellation remains visible only as internal service teardown during refresh and shutdown.

Trace artifact: `/Volumes/CorbanuDrive/Corbanu/.codex-work/corbanu-terminal/evidence/mcp-startup-fix/codex-tui.log`

## Launcher repeatability evidence

The Corbanu Terminal Launcher was invoked ten times after the rebuild. All ten runs created a fresh TUI process and rollout. For every fresh process, the structured CorbanuDrive log database records one completed terminal startup probe, one 49-tool catalog with one available and zero unavailable MCP servers, at least one successful `codex_apps` service initialization, and zero MCP warnings or errors.

Computer Use initially could not inspect the resulting Desktop 3 windows because the Mac was locked. After unlock, it confirmed the active desktop was visible, but its targeted-app safety boundary prevented direct terminal-window inspection and global Mission Control/space switching. The launcher placement log records all ten starts and placements, and the process count increased from 12 to 22. The ten verification terminals were left running for inspection.

## Release blockers outside this fix

- Complete the mandatory benchmark bootstrap cycle and evidence package described in `benchmarks/README.md` before publishing 0.1.35.
- Resolve any outstanding full-suite release-version snapshot work through the normal release preparation flow.
- Obtain the required human release sign-off before publication.
