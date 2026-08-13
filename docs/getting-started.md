# Getting Started

Use this page after PFTerminal is installed. For a new-machine install, start
with [Install And First Run](install.md).

## First Run

Start PFTerminal from the workspace you want it to inspect:

```bash
cd ~/repos/my-project
pfterminal
```

If you built from source, the debug `pfterminal` binary also defaults
PFTerminal state to `$HOME/.pfterminal`, separate from stock Codex:

```bash
/path/to/PfTerminal/codex-rs/target/debug/pfterminal
```

On first run, choose a provider account and enter the API key. The key is stored
in the encrypted vault, not in the chat transcript.

## Provider Choices

PFTerminal currently ships these provider paths:

| Use case                  | Provider    | Current release-visible model examples                                                                           |
| ------------------------- | ----------- | ---------------------------------------------------------------------------------------------------------------- |
| OpenAI Codex account      | OpenAI      | `gpt-5.6-sol`, `gpt-5.6-luna`, `gpt-5.6-terra`                                                                   |
| Claude subscription       | Claude Plan | `claude-opus-5-plan`, `claude-fable-5-plan`                                                                      |
| Direct Claude API         | Anthropic   | `claude-opus-5`, `claude-fable-5`                                                                                |
| Ambient coding plan       | Ambient     | `z-ai/glm-5.2`, `moonshotai/kimi-k2.7-code`                                                                      |
| Kimi coding plan          | Kimi Code   | `k3`                                                                                                             |
| Z.AI coding plan          | Z.AI        | `glm-5.2`                                                                                                        |
| Direct DeepSeek Responses | DeepSeek    | `deepseek-v4-flash` (DeepSeek V4 Flash 0731)                                                                     |
| Metered model gateway     | OpenRouter  | `deepseek/deepseek-v4-flash-0731`, `deepseek/deepseek-v4-pro`, `moonshotai/kimi-k3`, and other catalogued routes |
| Meta API                  | Meta        | `muse-spark-1.1`                                                                                                 |
| Metered GLM               | Baseten     | `zai-org/GLM-5.2`                                                                                                |
| Metered GLM gateway       | Vercel      | `zai/glm-5.2`, `zai/glm-5.2-fast`                                                                                |

Open `/model` to switch models. You can also start with a specific model:

```bash
pfterminal -m glm-5.2
pfterminal -m deepseek-v4-flash
pfterminal -m deepseek/deepseek-v4-flash-0731
pfterminal -m gpt-5.6-luna
```

## Vault Basics

Open the vault menu:

```text
/vault
```

Common checks:

```text
/vault list
/vault show provider/ambient_api_key
/vault credential add
```

Provider API keys stored through onboarding use labels such as
`provider/anthropic_api_key`, `provider/ambient_api_key`,
`provider/kimi_api_key`, `provider/zai_api_key`,
`provider/deepseek_api_key`, `provider/openrouter_api_key`,
`provider/model_api_key`, `provider/baseten_api_key`, and
`provider/ai_gateway_api_key`.

## Useful Slash Commands

| Command      | Purpose                                                                  |
| ------------ | ------------------------------------------------------------------------ |
| `/model`     | Select provider model and reasoning/effort mode                          |
| `/providers` | Add or replace provider credentials and manage account routes            |
| `/vault`     | Add, inspect, or delete credentials without exposing raw secrets to chat |
| `/panes`     | Create or switch persistent PFTerminal and Claude headless panes         |
| `/spawn`     | Manage retained Nazgul/Troll/Orc agents                                  |
| `/wallet`    | Manage the local SOL/USDC wallet and PFTerminal plans                    |
| `/gpu`       | Inspect, rent, stop, or terminate supported GPU capacity                 |
| `/telegram`  | Configure and control the Telegram connector                             |
| `/tasknode`  | Link and use Task Node tasks, context, chat, balances, and rewards       |
| `/docs`      | Open these packaged docs inside any advertised pane                      |
| `/status`    | Inspect current model/provider/session state                             |

## Verify Setup

After adding a key:

1. Run `/vault` and confirm the provider credential exists.
2. Run `/model` and select the provider model you want.
3. Ask a small repo-local question, such as `summarize this repository`.

If a provider reports a missing environment variable, add the key through the
provider login/onboarding UI or export the relevant env var for that shell:
`ANTHROPIC_API_KEY`, `AMBIENT_API_KEY`, `KIMI_API_KEY`, `ZAI_API_KEY`,
`DEEPSEEK_API_KEY`, `OPENROUTER_API_KEY`, `MODEL_API_KEY`,
`BASETEN_API_KEY`, or `AI_GATEWAY_API_KEY`.
