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
default_cwd = "/home/alice/pfterminal-telegram"
webhook_url = ""
```

The bot token is never read from `config.toml`. Resolution order is:

1. The environment variable named by `bot_token_env`.
2. The encrypted vault label `telegram/bot_token`.
3. Startup error.

Chats are default-deny. Only numeric Telegram chat IDs in `allowed_chat_ids`
can start turns. Approval buttons are accepted only from the same authorized
chat that owns the pending request. Group and supergroup chat IDs are negative;
if a group/supergroup ID is allowlisted, every member of that Telegram chat can
drive turns and press approval buttons. For higher assurance, use a private chat
until per-user authorization is available. The connector emits a startup warning
when any allowlisted chat ID is negative.

The connector stores recovered thread IDs and delivered item markers in
`$CODEX_HOME/telegram/state.json` so a restarted poller can resume the same
app-server threads without replaying the full transcript after lag recovery.

Telegram messages use HTML parse mode, split outbound text at Telegram's
4096-character raw-text limit, and surface sensitive operations through inline
approval buttons rather than auto-approving them.

`default_cwd` is the workspace used for Telegram-created turns. Set it to the
directory where the agent should work, not to the PFTerminal source tree or all
of `$HOME`. Codex automatically loads `AGENTS.md` from that workspace. The setup
script defaults `--workspace` to `~/pfterminal-telegram`, creates it, and seeds
`AGENTS.md` there from `codex-rs/telegram/dist/AGENTS.md.template` when the
workspace does not already have one. Use `--workspace "$HOME"` only when you
intentionally want a home-rooted remote agent workspace.

The recommended setup path is:

```bash
export PFTERMINAL_TELEGRAM_TOKEN="123456:telegram-token"
codex-rs/scripts/setup-telegram.sh --chat-id 21000038
```

The script resolves `CODEX_HOME` the same way `pfterminal telegram` does,
writes the token to `~/.config/pfterminal/telegram.env`, writes or merges the
`[telegram]` block, sets `default_cwd`, and backs up an existing `config.toml`
before editing it. On reruns, the script only changes `[telegram]` settings that
were explicitly passed on that invocation or are missing from the existing file,
so operator tuning such as `approval_policy = "on-failure"` is preserved. Do not
pass the bot token on the command line; use `PFTERMINAL_TELEGRAM_TOKEN`, an
existing env-file entry, or the interactive prompt.

`pfterminal -c telegram.foo=... telegram` and profile overrides do not override
`[telegram]` connector settings today. The connector reads this table directly
from `CODEX_HOME/config.toml`; use the config file or setup script for Telegram
settings. Core settings such as model, cwd, approval policy, and sandbox posture
still resolve through the normal core config after the connector has loaded its
own table.

On Linux, the connector emits one advisory startup warning when the resolved
sandbox policy is not `danger-full-access` and cheap host probes indicate the
sandbox is unlikely to launch: `bwrap` is missing from `PATH`,
`/proc/sys/user/max_user_namespaces` reads as `0`, or
`/proc/sys/kernel/unprivileged_userns_clone` exists and reads as `0`, or
`/proc/sys/kernel/apparmor_restrict_unprivileged_userns` exists and reads as
`1`. The setup script also runs a live `bwrap --ro-bind / / true` probe when
`bwrap` exists. In a failing state, even simple shell commands can require
manual Telegram approval because the sandboxed launch fails before command
execution.

On a trusted single-user host where unprivileged user namespaces are unavailable,
set top-level `sandbox_mode = "danger-full-access"` so the always-on connector
can execute commands without sandbox-launch approval churn. This disables the
filesystem sandbox, so do not use it on shared or untrusted hosts; install
`bwrap` and enable unprivileged user namespaces instead. The setup script never
writes this setting silently: pass `--allow-danger-full-access` or, on an
interactive TTY, confirm the prompt that states this disables the sandbox
globally for all PFTerminal surfaces. In non-interactive mode without the flag,
preflight failure exits non-zero and does not write `sandbox_mode`.

At startup, the connector emits loud warnings when the effective approval policy
resolves to `never` or the effective sandbox resolves to `danger-full-access`.

To keep the poller always on, install the user service:

```bash
codex-rs/scripts/setup-telegram.sh --chat-id 21000038 --install-systemd
systemctl --user daemon-reload
systemctl --user enable --now pfterminal-telegram.service
```

The setup script installs a concrete user unit: `ExecStart` is rewritten to the
absolute path returned by `command -v pfterminal`, and `EnvironmentFile` is
rewritten to the actual `--env-file` path. The checked-in template remains
generic for review. The unit reads `CODEX_HOME` and `PFTERMINAL_TELEGRAM_TOKEN`
from `~/.config/pfterminal/telegram.env`, restarts automatically, and uses
`StartLimitIntervalSec=300` with `StartLimitBurst=5` so a persistent 409 conflict
from a second poller stops instead of restart-fighting forever. Run only one
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
