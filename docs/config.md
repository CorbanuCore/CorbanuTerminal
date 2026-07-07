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

<a id="telegram"></a>

## Telegram Connector

`pfterminal telegram` runs a Telegram long-polling connector that drives the
same in-process app-server harness as the terminal UI and `pfterminal exec`.
Telegram-specific configuration is read locally by the connector from the
`[telegram]` table. Core accepts this table during strict config validation,
but the connector owns the individual settings.

```toml
[telegram]
enabled = true
bot_token_env = "PFTERMINAL_TELEGRAM_TOKEN"
allowed_chat_ids = [21000038, -1001941234987]
mode = "polling"
default_model = "glm-5.2"
approval_policy = "on-request"
default_cwd = "/home/alice"
webhook_url = ""
```

The bot token is never read from `config.toml`. Resolution order is:

1. The environment variable named by `bot_token_env`.
2. The encrypted vault label `telegram/bot_token`.
3. Startup error.

Chats are default-deny. Only numeric Telegram chat IDs in `allowed_chat_ids`
can start turns. Approval buttons are accepted only from the same authorized
chat that owns the pending request. The connector stores recovered
thread IDs in `$CODEX_HOME/telegram/state.json` so a restarted poller can resume
the same app-server threads.

Telegram messages use HTML parse mode, split outbound text at Telegram's
4096-character raw-text limit, and surface sensitive operations through inline
approval buttons rather than auto-approving them.

`default_cwd` is the workspace used for Telegram-created turns. Set it to the
directory where the agent should work, not to the PFTerminal source tree. Codex
automatically loads `AGENTS.md` from that workspace. The setup script seeds one
from `codex-rs/telegram/dist/AGENTS.md.template` when the workspace does not
already have one, giving the Telegram harness its identity and operating rules.

The recommended setup path is:

```bash
export PFTERMINAL_TELEGRAM_TOKEN="123456:telegram-token"
codex-rs/scripts/setup-telegram.sh --chat-id 21000038 --workspace "$HOME"
```

The script resolves `CODEX_HOME` the same way `pfterminal telegram` does,
writes the token to `~/.config/pfterminal/telegram.env`, writes or merges the
`[telegram]` block, sets `default_cwd`, and backs up an existing
`config.toml` before editing it. Use `--install-systemd` to copy the user
service template and print the `systemctl --user` enable/start commands.

On Linux, the connector emits one advisory startup warning when the resolved
sandbox policy is not `danger-full-access` and cheap host probes indicate the
sandbox is unlikely to launch: `bwrap` is missing from `PATH`,
`/proc/sys/user/max_user_namespaces` reads as `0`, or
`/proc/sys/kernel/unprivileged_userns_clone` exists and reads as `0`. In that
state, even simple shell commands can require manual Telegram approval because
the sandboxed launch fails before command execution.

On a trusted single-user host where unprivileged user namespaces are unavailable,
set top-level `sandbox_mode = "danger-full-access"` so the always-on connector
can execute commands without sandbox-launch approval churn. This disables the
filesystem sandbox, so do not use it on shared or untrusted hosts; install
`bwrap` and enable unprivileged user namespaces instead. The setup script makes
this decision automatically and prints the reason when it sets
`sandbox_mode = "danger-full-access"`.

To keep the poller always on, install the user service:

```bash
codex-rs/scripts/setup-telegram.sh --chat-id 21000038 --workspace "$HOME" --install-systemd
systemctl --user daemon-reload
systemctl --user enable --now pfterminal-telegram.service
```

The service runs `pfterminal telegram`, reads `CODEX_HOME` and
`PFTERMINAL_TELEGRAM_TOKEN` from
`~/.config/pfterminal/telegram.env`, and restarts automatically. Run only one
poller per Telegram bot token.

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
