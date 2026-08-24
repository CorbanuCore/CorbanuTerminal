# Corbanu Terminal

Corbanu Terminal is an open-source, multi-provider coding terminal built on the
Codex CLI. It combines local coding-agent workflows with model routing,
encrypted credentials, multi-agent orchestration, Telegram control, a local
Solana wallet, and Post Fiat Task Node integration.

Corbanu Terminal integrates Post Fiat Task Node and Ambient Inference. Post
Fiat remains the Layer 1 blockchain and Task Node remains its task, wallet, and
rewards product. Ambient Inference remains a separate inference network and
token. Corbanu Newsletter is a separate publication, not a terminal component.

## Install

### Linux and macOS

```bash
curl -fsSL https://github.com/CorbanuCore/CorbanuTerminal/releases/latest/download/install.sh | sh
```

### Windows

```powershell
irm https://github.com/CorbanuCore/CorbanuTerminal/releases/latest/download/install.ps1 | iex
```

The standalone installer creates the `corbanu` command. It does not replace a
stock `codex` command. macOS users can alternatively
download the latest Corbanu Terminal DMG from
[GitHub Releases](https://github.com/CorbanuCore/CorbanuTerminal/releases/latest).

## Local state and compatibility

Corbanu Terminal resolves its home directory without copying, merging, or
deleting user data:

1. `CORBANU_HOME`, when set.
2. `PFTERMINAL_HOME`, when set and `CORBANU_HOME` is not set.
3. `CODEX_HOME`, when set and neither product-specific override is set.
4. An existing `$HOME/.corbanu` directory.
5. An existing `$HOME/.pfterminal` directory, with a migration notice.
6. `$HOME/.corbanu` for a fresh installation.

If both product directories exist, `.corbanu` wins and Corbanu Terminal warns
about the unused legacy directory. Version 0.1.30 does not automatically move
state, so rollback to PFTerminal 0.1.29 remains safe.

To remove the standalone binaries while retaining credentials, sessions, and
settings:

```bash
rm -f "${CORBANU_INSTALL_DIR:-$HOME/.local/bin}/corbanu"
rm -rf "${CORBANU_HOME:-$HOME/.corbanu}/packages/standalone"
```

Do not delete `.corbanu` or `.pfterminal` unless you intend to erase vault
credentials, login state, session history, pane artifacts, wallet metadata, and
installed packages.

## Key features

- Model routing across OpenAI, Anthropic, Kimi Code, Z.AI, OpenRouter, Ambient,
  Meta, Baseten, Vercel AI Gateway, Amazon Bedrock, Ollama, LM Studio, and
  custom providers.
- OpenAI Codex account auth, Claude Code plan auth, Corbanu Terminal prepaid
  plans, direct API keys, and cloud credentials.
- Models including OpenAI GPT, Anthropic Claude, Kimi, GLM, Grok, and other
  OpenRouter catalogue models.
- A local Solana wallet for SOL and USDC, signing controls, backups, and plan
  purchases.
- Encrypted `/vault` credential storage.
- `corbanu telegram` for allowlisted Telegram chats.
- Native pane orchestration and managed multi-agent workflows.
- Post Fiat Task Node tasks, rewards, context, and chat through `/tasknode`.

For the complete shipping inventory, see the
[feature catalog](docs/features/index.md).

## Product and release governance

Corbanu development is governed by:

- the [product specification](docs/corbanu-product-spec.md);
- the repository [development policy](AGENTS.md);
- the [active-plan process and template](docs/plans/index.md);
- the [single-feature sprint execution process](docs/sprints/index.md); and
- the public [benchmark and performance tracker](benchmarks/README.md).

## First run

Launch Corbanu Terminal in the workspace you want it to inspect:

```bash
cd ~/repos
corbanu
```

To run commands without approval or sandbox prompts:

```bash
corbanu --yolo
```

`--yolo` gives the agent broad command and file access. Use it only in a
workspace you trust.

Useful commands include:

- `/providers` for account and API-key setup.
- `/vault` for encrypted credentials.
- `/wallet` for the local wallet and Corbanu Plan.
- `/model` or `corbanu -m <model>` to select a model.
- `/spawn` and `/panes` for managed agent work.
- `/tasknode` for Post Fiat Task Node.

## Providers

| Provider or route         | Access                                                   |
| ------------------------- | -------------------------------------------------------- |
| OpenAI                    | Codex account authentication or API-backed configuration |
| Anthropic                 | Direct API keys and Claude Code plan-backed panes        |
| Corbanu Plan              | Prepaid inference purchased with USDC through `/wallet`  |
| Kimi Code                 | Direct Kimi Code access                                  |
| Z.AI                      | Direct Z.AI access, including GLM models                 |
| OpenRouter                | OpenRouter's catalogue, including Kimi, GLM, and Grok    |
| Ambient Inference         | Hosted inference routes                                  |
| Meta, Baseten, and Vercel | Hosted provider and gateway routes                       |
| Amazon Bedrock            | AWS-authenticated Bedrock models                         |
| Ollama and LM Studio      | Local model servers                                      |
| Custom providers          | Additional endpoints configured in `config.toml`         |

Use `/providers` for interactive setup. The public provider alias is
`corbanu-plan`; the earlier `corbanu-terminal-plan` alias and legacy
`pfterminal-plan` identifier remain accepted so existing configuration and
receipts continue to work.

Corbanu Terminal contains the public client integration for Corbanu Plan. The
hosted plan service, billing operations, and internal methodology are maintained
privately by CorbanuCore.

Corbanu Terminal contains the public client integration for Corbanu Plan. The
hosted plan service, billing operations, and internal methodology are maintained
privately by CorbanuCore.

## Wallet and Task Node

`/wallet` creates or restores a local wallet, shows SOL and USDC balances,
controls signing access, backs up recovery material, and manages Corbanu Plans.
Wallet secrets stay outside normal chat history and
model context.

`/tasknode` connects Corbanu Terminal to Post Fiat Task Node tasks, rewards,
context, and chat. A Task Node account registered at
[tasknode.postfiat.org](https://tasknode.postfiat.org) is required for linked
actions. Corbanu branding does not rename or replace Task Node.

Common Task Node forms:

- `/tasknode link` links or refreshes the account.
- `/tasknode tasks` lists outstanding work.
- `/tasknode request` requests a personal task.
- `/tasknode balance` and `/tasknode rewards` show PFT reward state.
- `/tasknode chat` and `/tasknode context` open contextual surfaces.

## Telegram

`corbanu telegram` starts a long-polling Telegram connector. Configure it in
the active Corbanu home `config.toml`:

```toml
[telegram]
enabled = true
bot_token_env = "CORBANU_TELEGRAM_TOKEN"
allowed_chat_ids = [21000038, -1001941234987]
allowed_user_ids = [21000038]
mode = "polling"
approval_policy = "on-request"
sandbox_mode = "workspace-write"
```

Store the token in the named environment variable or encrypted vault label
`telegram/bot_token`; do not put a raw token in `config.toml`. The legacy
`PFTERMINAL_TELEGRAM_TOKEN` environment variable remains supported.

## Documentation

- [Install and first run](docs/install.md)
- [Getting started](docs/getting-started.md)
- [Authentication and vault](docs/authentication.md)
- [Configuration](docs/config.md)

## Source build

```bash
git clone https://github.com/CorbanuCore/CorbanuTerminal.git
cd CorbanuTerminal/codex-rs
CARGO_NET_GIT_FETCH_WITH_CLI=true cargo build -p codex-cli --bin corbanu
./target/debug/corbanu
```

## Upstream and license

Corbanu Terminal is based on the open-source Codex CLI project. Upstream
changes are isolated through the `upstream` remote and Corbanu changes land
through this repository.

This repository is licensed under the [Apache-2.0 License](LICENSE).
