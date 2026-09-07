# Vercel Integration

Vercel is a built-in metered provider path through Vercel AI Gateway.

## Current Provider

The built-in Vercel provider is defined in
`codex-rs/model-provider-info/src/lib.rs`:

| Field | Value |
| --- | --- |
| Provider id | `vercel` |
| Display name | `Vercel` |
| Base URL | `https://ai-gateway.vercel.sh/v1` |
| API key env var | `AI_GATEWAY_API_KEY` |
| Wire API | `responses` |
| OpenAI auth required | `false` |
| WebSockets | `false` |

Auth guidance shown to users:

```text
Set AI_GATEWAY_API_KEY to your Vercel AI Gateway API key.
```

## Current Models

The visible Vercel models are bundled in
`codex-rs/models-manager/models.json`:

| Slug | Display name | Description |
| --- | --- | --- |
| `zai/glm-5.2` | `Vercel GLM 5.2` | `Vercel: GLM 5.2 - $1.40/M input, $0.26/M cached input, $4.40/M output.` |
| `zai/glm-5.2-fast` | `Vercel GLM 5.2 Fast` | `Vercel: GLM 5.2 Fast - $2.10/M input, $0.21/M cached input, $6.60/M output.` |
| `zai/glm-5.3-flash` | `Vercel GLM 5.3 Flash` | Vercel GLM 5.3 Flash route with vision and reasoning. |
| `zai/glm-5.3` | `Vercel GLM 5.3` | Vercel GLM 5.3 reasoning route. |
| `vercel/moonshotai/kimi-k3` | `Vercel Kimi K3` | Provider-qualified local identity; sends `moonshotai/kimi-k3` to Vercel. |
| `vercel/deepseek/deepseek-v4-pro` | `Vercel DeepSeek V4 Pro` | Provider-qualified local identity; sends `deepseek/deepseek-v4-pro` to Vercel. |

All entries are listed in `/model`. Provider-qualified identities keep Kimi K3
and DeepSeek V4 Pro distinct from their OpenRouter catalog entries while the
wire request uses Vercel's official model slug.

## Model And Provider Selection

Corbanu Terminal maps every model above to provider `vercel`.

Examples:

```bash
corbanu -m zai/glm-5.2
corbanu -m zai/glm-5.2-fast
corbanu -m zai/glm-5.3-flash
corbanu -m zai/glm-5.3
corbanu -m vercel/moonshotai/kimi-k3
corbanu -m vercel/deepseek/deepseek-v4-pro
```

## Vault Behavior

Vercel keys saved through onboarding are stored in the encrypted vault at:

```text
provider/ai_gateway_api_key
```

The environment variable `AI_GATEWAY_API_KEY` is still supported for temporary
shells and automation.

## Claude Pane Behavior

Vercel AI Gateway also exposes an Anthropic-compatible Claude Code route at:

```text
https://ai-gateway.vercel.sh
```

The `/panes` menu exposes:

```text
Claude Code - GLM 5.2 Vercel
Claude Code - GLM 5.2 Fast Vercel
```

Both pane profiles use the same `provider/ai_gateway_api_key` vault credential.
The full GLM 5.2 pane uses `zai/glm-5.2` for Opus/Sonnet aliases and
`zai/glm-5.2-fast` for the Haiku/small-model aliases. The Fast pane uses
`zai/glm-5.2-fast` for all Claude Code aliases.

## Source

- `codex-rs/model-provider-info/src/lib.rs`
- `codex-rs/models-manager/models.json`
- `codex-rs/tui/src/onboarding/auth.rs`
- `codex-rs/tui/src/chatwidget/model_popups.rs`
- `codex-rs/login/src/auth/provider_key_vault.rs`
