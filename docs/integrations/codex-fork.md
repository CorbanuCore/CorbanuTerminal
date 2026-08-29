# Corbanu Terminal Codex Fork

Corbanu Terminal is a product fork of the open-source Codex CLI. It preserves
Codex's local coding-agent runtime while changing product defaults, packaging,
branding, and provider integrations.

## Runtime Lineage

The Rust workspace remains under `codex-rs/` and keeps the major Codex subsystems:

- TUI and slash commands.
- Model/provider client runtime.
- Tool registry and execution.
- Sandbox and approval flows.
- MCP support.
- Session, rollout, and thread storage.
- Exec and review modes.

The repository README and `codex-rs/README.md` document this as a Codex-derived Rust workspace.

## Product Command Names

Corbanu Terminal keeps upstream-compatible internal `codex` paths while adding product-facing command names:

- `codex-rs/cli/Cargo.toml` defines `codex`, `corbanu`, and compatibility `pfterminal` binaries.
- `codex-rs/cli/src/pfterminal_main.rs` currently includes the same implementation as `main.rs`.
- `codex-cli/package.json` publishes `@corbanucore/terminal` with `corbanu` and compatibility `pfterminal` aliases; it does not claim the stock `codex` command.
- `codex-cli/bin/codex.js` resolves the compatibility platform packages, prefers the bundled `corbanu` binary, and follows the Corbanu and legacy-home precedence.

This keeps existing `pfterminal` automation usable while making `corbanu` the primary product command.

## Packaging And Installers

The npm packaging has been renamed around `@corbanucore/terminal`:

- Main package: `@corbanucore/terminal`.
- Platform packages: `@corbanucore/terminal-linux-x64`, `@corbanucore/terminal-darwin-arm64`, and related target variants.
- TypeScript SDK package: `@agticorp/pfterminal-sdk`.

Standalone installer scripts in `scripts/install/` install `corbanu` plus the
legacy alias, default fresh state to `$HOME/.corbanu`, reuse a lone
`$HOME/.pfterminal` in place, and avoid replacing a stock `codex` command.

## Branding Changes

The TUI and login surfaces have Corbanu Terminal branding:

- Device-code prompts welcome users to Corbanu Terminal.
- Session cards, composer placeholders, status surfaces, and guidance use Corbanu Terminal.
- Post Fiat Task Node and Ambient Inference retain their distinct product identities.
- Legacy command, state, provider, receipt, service, and protocol identifiers remain readable where compatibility requires them.

The status line can therefore show a session such as:

```text
zai-org/GLM-5.2-FP8 standard ... Corbanu Terminal
```

## Model Picker Changes

The model picker is intentionally narrowed for the current product:

- Ambient and Z.AI GLM models are shown by default.
- Hidden or non-product models can still be selected by config or command-line model override.
- GLM reasoning is presented as `Standard` and `Deep` instead of raw OpenAI-style effort labels.

Key path: `codex-rs/tui/src/chatwidget/model_popups.rs`.

## Prompt And Base Instructions

Bundled Ambient and Z.AI model metadata use Corbanu Terminal base instructions:

```text
You are Corbanu Terminal, a coding agent.
```

The instructions preserve the Codex engineering posture: inspect code first, keep edits scoped, use `rg`/`rg --files`, and verify work when practical.

## Upstream Isolation

The [upstream integration contract](../plans/upstream-integration.md) owns
adapter boundaries, upstream-touch records, parallel ownership, and upgrade
qualification. Plans record unfinished integration work and its evidence;
this lineage page does not certify a particular upstream update.

## Source

- `README.md`
- `codex-rs/README.md`
- `codex-rs/cli/Cargo.toml`
- `codex-rs/cli/src/pfterminal_main.rs`
- `codex-cli/package.json`
- `codex-cli/bin/codex.js`
- `scripts/install/`
- `codex-rs/tui/src/chatwidget/model_popups.rs`
- `codex-rs/models-manager/models.json`
