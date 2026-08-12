# OpenRouter Integration

OpenRouter is a built-in metered provider for PFTerminal.

## Current Provider

The built-in OpenRouter provider is defined in
`codex-rs/model-provider-info/src/lib.rs`:

| Field | Value |
| --- | --- |
| Provider id | `openrouter` |
| Display name | `OpenRouter` |
| Base URL | `https://openrouter.ai/api/v1` |
| API key env var | `OPENROUTER_API_KEY` |
| Wire API | `chat` |
| OpenAI auth required | `false` |
| WebSockets | `false` |

Auth guidance shown to users:

```text
Set OPENROUTER_API_KEY to your OpenRouter API key.
```

## Current Models

Visible OpenRouter models are bundled in
`codex-rs/models-manager/models.json`:

| Slug | Display name | Listed pricing text |
| --- | --- | --- |
| `x-ai/grok-4.6` | OpenRouter Grok 4.6 | `$2.00/M input`, `$0.50/M cached input`, `$6.00/M output` |
| `deepseek/deepseek-v4-flash-0731` | OpenRouter DeepSeek V4 Flash 0731 | `$0.14/M input`, `$0.0028/M cached input`, `$0.28/M output` |
| `deepseek/deepseek-v4-pro-0813` | OpenRouter DeepSeek V4 Pro 0813 | `$0.435/M input`, `$0.003625/M cached input`, `$0.87/M output` |
| `deepseek/deepseek-v4-pro` | OpenRouter DeepSeek V4 Pro | `$0.435/M input`, `$0.87/M output` |
| `moonshotai/kimi-k3` | OpenRouter Kimi K3 | `$3.00/M input`, `$0.30/M cached input`, `$15.00/M output` |
| `x-ai/grok-4.5` | OpenRouter Grok 4.5 | `$2.00/M input`, `$6.00/M output` |
| `minimax/minimax-m3` | OpenRouter MiniMax M3 | `$0.60/M input`, `$2.40/M output` |
| `openrouter/owl-alpha` | OpenRouter Owl Alpha | `$0/M input`, `$0/M output` |
| `google/gemini-3.5-flash` | OpenRouter Gemini 3.5 Flash | `$1.50/M input`, `$9.00/M output` |
| `tencent/hy3:free` | OpenRouter Tencent Hy3 Free | `$0/M input`, `$0/M output` |

Grok 4.6's listed rates double when the prompt exceeds 200,000 tokens.

DeepSeek V4 Flash is intentionally pinned to the exact
`deepseek/deepseek-v4-flash-0731` slug. Direct DeepSeek uses provider
`deepseek` and model `deepseek-v4-flash`; the two billing and routing paths are
distinct and must remain visible as such.

## Model And Provider Selection

PFTerminal maps these model slugs to provider `openrouter` in
`codex-rs/tui/src/chatwidget/model_popups.rs`.

Examples:

```bash
pfterminal -m deepseek/deepseek-v4-flash-0731
pfterminal -m deepseek/deepseek-v4-pro-0813
pfterminal -m deepseek/deepseek-v4-pro
pfterminal -m moonshotai/kimi-k3
pfterminal -m x-ai/grok-4.6
pfterminal -m x-ai/grok-4.5
pfterminal -m minimax/minimax-m3
pfterminal -m openrouter/owl-alpha
pfterminal -m google/gemini-3.5-flash
```

## Vault Behavior

OpenRouter keys saved through onboarding are stored in the encrypted vault at:

```text
provider/openrouter_api_key
```

The environment variable `OPENROUTER_API_KEY` is still supported for temporary
shells and automation.

## Source

- `codex-rs/model-provider-info/src/lib.rs`
- `codex-rs/models-manager/models.json`
- `codex-rs/tui/src/onboarding/auth.rs`
- `codex-rs/tui/src/chatwidget/model_popups.rs`
- `codex-rs/login/src/auth/provider_key_vault.rs`
