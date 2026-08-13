# PFTerminal

<p align="center">
  <img src="docs/assets/images/pfterminal-logo.png" alt="PFTerminal - Post Fiat Terminal" width="720">
</p>

PFTerminal is an open-source, multi-provider coding terminal built on the
Codex CLI. It keeps Codex's local coding-agent workflow while adding native
routes for OpenAI, Anthropic, Kimi, GLM, Grok, and other models across direct
providers, model gateways, prepaid plans, and local inference. It also includes
encrypted credentials, model-aware agent orchestration, Telegram control, and
a local Solana wallet for SOL, USDC, and PFTerminal inference plans.

## Install

### Linux

```bash
curl -fsSL https://github.com/agtico/PfTerminal/releases/latest/download/install.sh | sh
```

### Windows

```shell
powershell -ExecutionPolicy ByPass -c "irm https://chatgpt.com/codex/install.ps1 | iex"
```

The standalone installers download from `https://releases.openai.com/codex` by default and fall back to GitHub Releases if a metadata or asset download is unavailable. To force GitHub Releases, set `CODEX_INSTALLER_USE_RELEASES_OPENAI_COM` to `false` (`0` and `no` are also accepted):

```shell
curl -fsSL https://chatgpt.com/codex/install.sh | CODEX_INSTALLER_USE_RELEASES_OPENAI_COM=false sh
```

```powershell
$env:CODEX_INSTALLER_USE_RELEASES_OPENAI_COM='false'; irm https://chatgpt.com/codex/install.ps1 | iex
```

Codex CLI can also be installed via the following package managers:

### macOS

Download the latest DMG from
[GitHub Releases](https://github.com/agtico/PfTerminal/releases/latest):

- `PFTerminal-aarch64-apple-darwin.dmg` for Apple Silicon
- `PFTerminal-x86_64-apple-darwin.dmg` for Intel Macs

Terminal install also works on macOS:

```bash
curl -fsSL https://github.com/agtico/PfTerminal/releases/latest/download/install.sh | sh
```

The installer creates a `pfterminal` command, leaves any stock `codex` command
alone, and stores PFTerminal state in `$HOME/.pfterminal` by default.

## Remove Local Installations

To remove the standalone Linux/macOS install while keeping local credentials,
sessions, and settings:

```bash
rm -f "${PFTERMINAL_INSTALL_DIR:-$HOME/.local/bin}/pfterminal"
rm -rf "${PFTERMINAL_HOME:-$HOME/.pfterminal}/packages/standalone"
```

If you installed the npm package instead:

```bash
npm uninstall -g @agticorp/pfterminal
```

If you installed it with Bun:

```bash
bun remove -g @agticorp/pfterminal
```

To delete all PFTerminal local state as well, including vault credentials,
login state, session history, pane artifacts, and installed packages:

```bash
rm -rf "${PFTERMINAL_HOME:-$HOME/.pfterminal}"
```

## Key Features

- Multi-provider model routing across OpenAI, Anthropic, Kimi Code, Z.AI,
  OpenRouter, Ambient, Meta, Baseten, Vercel AI Gateway, Amazon Bedrock, and
  local Ollama or LM Studio.
- OpenAI Codex account auth, Claude Code plan auth, PFTerminal prepaid plans,
  direct API keys, cloud credentials, and configurable custom providers.
- Models including OpenAI GPT, Anthropic Claude, Kimi K3, GLM 5.2, and Grok,
  selected with `/model` and available to model-aware agent orchestration.
- A first-class `/wallet` workflow for creating or restoring a local Solana
  wallet, viewing SOL and USDC balances, controlling signing access, backing
  up recovery material, and buying or recovering PFTerminal inference plans.
- Encrypted `/vault` storage for provider API keys and user credentials.
- `pfterminal telegram` connector for allowlisted Telegram chats.
- Codex-level coding workflows in a local terminal.
- Native pane orchestration for Sauron -> Nazgul -> Troll -> Orc agent workflows.
- Separate PFTerminal home at `$HOME/.pfterminal`, so it does not collide with
  a stock Codex install.

## Model Providers

PFTerminal ships provider adapters for the following routes:

| Provider or route    | Access                                                                     |
| -------------------- | -------------------------------------------------------------------------- |
| OpenAI               | Codex account authentication or an API-backed configuration                |
| Anthropic            | Direct Anthropic API keys and Claude Code plan-backed panes                |
| PFTerminal Plan      | Prepaid inference plans purchased with USDC through `/wallet`              |
| Kimi Code            | Direct Kimi Code access, including Kimi K3                                 |
| Z.AI                 | Direct Z.AI access, including GLM models and an Anthropic-compatible route |
| OpenRouter           | OpenRouter's model catalogue, including Kimi, GLM, Grok, and other models  |
| Ambient              | Hosted GLM and Kimi routes                                                 |
| Meta                 | Meta-hosted model access                                                   |
| Baseten              | Baseten routes, including an Anthropic-compatible adapter                  |
| Vercel AI Gateway    | Responses, Anthropic-compatible, and fast Anthropic routes                 |
| Amazon Bedrock       | AWS-authenticated Bedrock models                                           |
| Ollama and LM Studio | Local model servers                                                        |
| Custom providers     | Additional provider endpoints configured in `config.toml`                  |

Use `/providers` for interactive account and API-key setup. Amazon Bedrock,
local servers, and custom providers use their normal environment or
`config.toml` configuration.

## First Run

Launch PFTerminal from the workspace you want it to inspect:

```bash
cd ~/repos
pfterminal
```

For a local workspace where you want PFTerminal to run commands without
approval prompts, launch it with `--yolo`:

```bash
cd ~/repos
pfterminal --yolo
```

`--yolo` bypasses command approvals and sandbox prompts, so use it only in a
workspace where you are comfortable letting the agent read and write files and
run shell commands.

Use:

- `/providers` to sign into OpenAI Codex, Claude Code Plan, or PFTerminal Plan,
  and to add Anthropic, Ambient, Kimi Code, Z.AI, OpenRouter, Meta, Baseten, or
  Vercel credentials.
- `/vault` to manage encrypted credentials.
- `/wallet` to create or restore a wallet, view SOL and USDC, control signing,
  and manage PFTerminal inference plans.
- `/model` or `pfterminal -m <model>` to choose a model.
- `/spawn` to create and route multi-agent work.

## Core Slash Commands

Slash commands are typed inside the interactive `pfterminal` chat. PFTerminal
inherits the normal Codex slash commands and adds a few commands for providers,
credentials, wallets, panes, spawned agents, and Task Node.

### `/providers`

Use `/providers` when a model or provider needs credentials. It opens the
provider setup menu and stores API keys or account-backed auth in the encrypted
PFTerminal vault.

Common uses:

- Add an Anthropic, Ambient, Kimi Code, Z.AI, OpenRouter, Meta, Baseten, or
  Vercel API key.
- Add or refresh OpenAI Codex account auth.
- Connect Claude Code Plan or an existing PFTerminal Plan.
- Check which provider key a selected model expects.

### `/wallet`

Use `/wallet` to manage the local Solana wallet and PFTerminal inference plans.
Wallet secrets stay outside normal chat history and model context. Signing
access is handled by the local wallet daemon and can be unlocked for one action
or for a bounded period.

The wallet menu can:

- Create a new wallet or restore one from recovery material.
- Show the receive address and current SOL and USDC balances.
- Lock signing globally or unlock it for one action, 5 minutes, 15 minutes,
  1 hour, or a custom duration.
- Back up recovery material in a secure view or remove the wallet from the
  device after recovery material is saved.
- Buy or upgrade a PFTerminal inference plan with an exact USDC payment.
- Recover access to an existing paid plan, inspect usage and limits, and view
  the latest plan receipt.

Useful forms:

- `/wallet` or `/wallet status` opens the wallet menu.
- `/wallet create` starts wallet creation.
- `/wallet restore` restores a wallet.
- `/wallet unlock` unlocks signing for 15 minutes.
- `/wallet lock` revokes signing from every PFTerminal process.

### `/vault`

Use `/vault` to inspect and manage encrypted local credentials. Secrets are
stored under `$HOME/.pfterminal` and are not typed into normal chat history.

Useful forms:

- `/vault` opens the vault action menu.
- `/vault list` lists stored credential labels.
- `/vault show <label>` shows credential metadata without revealing the raw
  secret.
- `/vault credential add` opens a masked entry flow for adding a new secret.

### `/panes`

Use `/panes` to switch between the main Codex conversation, native Codex agent
threads, and Claude Code panes. A pane has its own visible transcript and
running state, so long-running work can continue in one pane while you inspect
another.

The main pane is `Codex Main`. Other panes may be native Codex agent panes or
Claude panes created from the pane picker or through `/spawn`.

### `/spawn`

Use `/spawn` for managed multi-agent work. It can create or bind the hierarchy
PFTerminal uses for larger tasks:

- **Nazgul**: the supervising/root pane.
- **Troll**: a coordinating implementation or review pane.
- **Orc**: a focused worker pane.

Useful forms:

- `/spawn` opens the role picker.
- `/spawn status` shows the current hierarchy, running state, and recent
  dispatches.
- `/spawn nazgul`, `/spawn troll`, and `/spawn orc` create or bind specific
  roles.

Use `/spawn` when you want work split across persistent panes instead of asking
the current chat to do everything in one thread.

## Telegram Connector

`pfterminal telegram` starts a long-polling Telegram bot that sends allowlisted
chat messages into PFTerminal agent threads and returns streamed replies.
Configure it in `$CODEX_HOME/config.toml`:

```toml
[telegram]
enabled = true
bot_token_env = "PFTERMINAL_TELEGRAM_TOKEN"
allowed_chat_ids = [21000038, -1001941234987]
allowed_user_ids = [21000038]
mode = "polling"
approval_policy = "on-request"
sandbox_mode = "workspace-write"
```

Store the bot token in the named environment variable or in the encrypted vault
label `telegram/bot_token`; do not put raw bot tokens in `config.toml`.
Approvals are confirmed with Telegram inline buttons.
Run `/telegram` in PFTerminal for the normal masked-token, chat-discovery, and
connector-management flow. `pfterminal telegram --setup` remains available for
unattended host configuration.

## Codex Sessions vs Claude Panes

The default PFTerminal experience is a native Codex session. In that mode,
PFTerminal runs the Codex harness directly: it manages the active model,
tooling, permissions, local context, and command execution in the main terminal
session.

A Claude pane is different. It is a managed Claude Code subprocess wrapped by
PFTerminal through the local exec/pane runner. PFTerminal starts the Claude Code
process, feeds it the task, tracks its output, stores pane artifacts under
`$HOME/.pfterminal/panes`, and shows the result inside the PFTerminal pane UI.

In practice:

- Use native Codex when you want the normal PFTerminal/Codex harness.
- Use a Claude pane when you specifically want Claude Code behavior inside a
  separate, inspectable pane.
- Switching to a Claude pane with `/panes` does not turn the whole terminal into
  Claude. It opens that pane's own subprocess-backed transcript.
- Claude Plan panes use Claude Code's own plan auth. API-key Claude routes use
  keys stored through `/providers` or `/vault`.

## Task Node Quick Guide

`/tasknode` connects PFTerminal to Task Node tasks, rewards, context, and chat.
To use it, you need an account registered on
[tasknode.postfiat.org](https://tasknode.postfiat.org). If you are not
registered there, PFTerminal can open the menu but cannot show your tasks or
submit Task Node actions.

Useful forms:

- `/tasknode` opens the Task Node menu.
- `/tasknode link` starts or refreshes the GitHub-backed Task Node link flow.
- `/tasknode status` shows account, wallet, and task status.
- `/tasknode tasks` lists outstanding Task Node work.
- `/tasknode task <task-id>` opens one task.
- `/tasknode request` starts a personal task request.
- `/tasknode requests` shows active task requests.
- `/tasknode balance` and `/tasknode rewards` show PFT reward state.
- `/tasknode chat` and `/tasknode context` open Task Node chat and context
  surfaces.

Terminal session tokens are stored locally through the encrypted vault. If
linking fails, register or sign in at `https://tasknode.postfiat.org`, then run
`/tasknode link` again.

More setup detail:

- [Install And First Run](docs/install.md)
- [Getting Started](docs/getting-started.md)
- [Authentication And Vault](docs/authentication.md)
- [Configuration](docs/config.md)

## Source Build

```bash
git clone https://github.com/agtico/PfTerminal.git
cd PfTerminal/codex-rs
CARGO_NET_GIT_FETCH_WITH_CLI=true cargo build -p codex-cli --bin pfterminal
```

Then run:

```bash
./target/debug/pfterminal
```

## Upstream

PFTerminal is based on the open-source Codex CLI project. Keep upstream changes
isolated through the `upstream` remote and land PFTerminal changes through this
repository.

This repository is licensed under the [Apache-2.0 License](LICENSE).
