# Configuration

PFTerminal inherits Codex configuration but ships PFTerminal-specific provider
defaults. Most users do not need to define model providers manually.

## Config Location

PFTerminal reads config from `CODEX_HOME/config.toml`.

Recommended PFTerminal home:

```bash
export CODEX_HOME="${PFTERMINAL_HOME:-$HOME/.pfterminal}"
```

If you use an installed `pfterminal` wrapper, it may set this automatically. If
you run the source-built binary directly, set it yourself to keep PFTerminal
state separate from stock Codex.

## Built-In Providers

These providers are compiled into PFTerminal:

| Provider id  | Display name | Base URL                              | Env key              | Wire API         |
| ------------ | ------------ | ------------------------------------- | -------------------- | ---------------- |
| `openai`     | OpenAI       | `https://chatgpt.com/backend-api/codex` | Account login      | Responses        |
| `ambient`    | Ambient      | `https://api.ambient.xyz/v1`          | `AMBIENT_API_KEY`    | Chat Completions |
| `zai`        | Z.AI         | `https://api.z.ai/api/coding/paas/v4` | `ZAI_API_KEY`        | Chat Completions |
| `openrouter` | OpenRouter   | `https://openrouter.ai/api/v1`        | `OPENROUTER_API_KEY` | Chat Completions |
| `baseten`    | Baseten      | `https://inference.baseten.co/v1`     | `BASETEN_API_KEY`    | Chat Completions |
| `vercel`     | Vercel       | `https://ai-gateway.vercel.sh/v1`     | `AI_GATEWAY_API_KEY` | Responses        |

OpenAI uses Codex account login from `/providers` or `pfterminal login`.
Provider API keys should normally be stored through onboarding, `/providers`,
or `/vault`. Environment variables are supported for temporary sessions and
automation.

## Common Model Configs

Set a default provider and model in `$CODEX_HOME/config.toml`.

Ambient:

```toml
model_provider = "ambient"
model = "zai-org/GLM-5.2-FP8"
```

OpenAI Codex account:

```toml
model_provider = "openai"
model = "gpt-5.5"
```

Z.AI:

```toml
model_provider = "zai"
model = "glm-5.2"
```

OpenRouter GLM:

```toml
model_provider = "openrouter"
model = "z-ai/glm-5.2"
```

OpenRouter MiniMax:

```toml
model_provider = "openrouter"
model = "minimax/minimax-m3"
```

Baseten GLM:

```toml
model_provider = "baseten"
model = "zai-org/GLM-5.2"
```

Vercel GLM:

```toml
model_provider = "vercel"
model = "zai/glm-5.2"
```

Vercel GLM Fast:

```toml
model_provider = "vercel"
model = "zai/glm-5.2-fast"
```

You can also select a model per run:

```bash
pfterminal -m glm-5.2
pfterminal -m gpt-5.5
pfterminal -m z-ai/glm-5.2
pfterminal -m zai-org/GLM-5.2
pfterminal -m zai/glm-5.2
pfterminal -m zai/glm-5.2-fast
```

The model picker maps these model slugs to the correct built-in provider.

## Vault And Secrets

Provider API keys saved by PFTerminal are stored in the encrypted vault, not in
`config.toml`.

Vault labels:

```text
provider/ambient_api_key
provider/zai_api_key
provider/openrouter_api_key
provider/baseten_api_key
provider/ai_gateway_api_key
```

Do not put long-lived provider keys in `experimental_bearer_token` unless you
are intentionally running an automation-only setup. For interactive use, use
onboarding or `/vault`.

## Telegram Connector

`pfterminal telegram` runs a Telegram long-polling connector that drives the
same in-process app-server harness as the terminal UI and `pfterminal exec`.
Telegram-specific configuration is read locally by the connector from the
`[telegram]` table; it is not part of the inherited Codex config schema.

```toml
[telegram]
enabled = true
bot_token_env = "PFTERMINAL_TELEGRAM_TOKEN"
allowed_chat_ids = [21000038, -1001941234987]
mode = "polling"
default_model = "glm-5.2"
approval_policy = "on-request"
webhook_url = ""
```

The bot token is never read from `config.toml`. Resolution order is:

1. The environment variable named by `bot_token_env`.
2. The encrypted vault label `telegram/bot_token`.
3. Startup error.

Chats are default-deny. Only numeric Telegram chat IDs in `allowed_chat_ids`
can start turns or answer approval prompts. The connector stores recovered
thread IDs in `$CODEX_HOME/telegram/state.json` so a restarted poller can resume
the same app-server threads.

Telegram messages use HTML parse mode, split outbound text at Telegram's
4096-character raw-text limit, and surface sensitive operations through inline
approval buttons rather than auto-approving them.

## Provider Overrides

Advanced users can still define custom providers under `[model_providers]`.
Prefer built-ins unless you need a different base URL, custom headers, or an
external bearer-token command.

Example custom OpenAI-compatible Chat provider:

```toml
model_provider = "custom-chat"
model = "some/model"

[model_providers.custom-chat]
name = "Custom Chat Provider"
base_url = "https://example.com/v1"
env_key = "CUSTOM_PROVIDER_API_KEY"
wire_api = "chat"
```

For inherited Codex configuration details, see:

- Basic configuration: <https://developers.openai.com/codex/config-basic>
- Advanced configuration: <https://developers.openai.com/codex/config-advanced>
- Full reference: <https://developers.openai.com/codex/config-reference>

## Lifecycle Hooks

Admins can set top-level `allow_managed_hooks_only = true` in
`requirements.toml` to ignore user, project, and session hook configs while
still allowing managed hooks from requirements and managed config layers. This
setting is only supported in `requirements.toml`; putting it in `config.toml`
does not enable managed-hooks-only mode.
