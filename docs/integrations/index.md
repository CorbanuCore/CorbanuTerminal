# Core Integrations

This is the technical reference behind the live
[Models and providers](../features/model-providers.md) feature. It records
implementation boundaries and does not define separate product status.

Corbanu Terminal is a Codex CLI fork with product-specific model provider,
onboarding, packaging, and branding changes.

The important boundary: Corbanu Terminal still uses the Codex execution engine, tool
system, approval flows, sandboxing, and session mechanics, while adding OpenAI
Codex, Claude Plan, Anthropic, Ambient, Kimi Code, Z.AI, DeepSeek, OpenRouter,
Meta, Baseten, Vercel, Bedrock, Ollama, LM Studio, Corbanu API, and custom
providers as first-class choices.

## What Exists Now

| Area | Current state | Primary paths |
| --- | --- | --- |
| OpenAI Codex account | Built-in provider named `openai`, using Codex account auth and exposing the release-visible GPT-5.5/5.6 catalog. | `codex-rs/model-provider-info/src/lib.rs`, `codex-rs/tui/src/chatwidget/provider_credentials.rs` |
| Claude routes | Claude Plan account models and direct Anthropic Opus/Fable API-key models remain distinct billing/auth routes. | `codex-rs/model-provider-info/src/lib.rs` |
| Ambient provider | Built-in provider named `ambient`, using `AMBIENT_API_KEY` and the Chat Completions wire shape. | `codex-rs/model-provider-info/src/lib.rs` |
| Ambient default model | Bundled model `z-ai/glm-5.2`, displayed as `Ambient GLM 5.2`, is the only Ambient model option. | `codex-rs/models-manager/models.json` |
| Kimi Code provider | Built-in provider `kimi-code` using `KIMI_API_KEY` and current model `k3`. | `codex-rs/model-provider-info/src/lib.rs` |
| Z.AI provider | Built-in provider named `zai`, using `ZAI_API_KEY` and the Z.AI coding plan API base URL. | `codex-rs/model-provider-info/src/lib.rs` |
| DeepSeek provider | Direct Responses routes `deepseek-v4-flash` and `deepseek-v4-pro`, backed by `DEEPSEEK_API_KEY`. | `codex-rs/model-provider-info/src/lib.rs` |
| OpenRouter provider | Built-in metered provider including pinned `deepseek/deepseek-v4-flash-0731`, DeepSeek Pro, Kimi K3, Grok, MiniMax, Gemini, Owl, and Tencent routes. | `codex-rs/model-provider-info/src/lib.rs`, `codex-rs/models-manager/models.json` |
| Meta provider | Built-in `meta` route using `MODEL_API_KEY` and Muse Spark 1.1. | `codex-rs/model-provider-info/src/lib.rs` |
| Baseten provider | Built-in provider named `baseten`, using `BASETEN_API_KEY` and Baseten GLM 5.2. | `codex-rs/model-provider-info/src/lib.rs` |
| Vercel provider | Built-in provider named `vercel`, using `AI_GATEWAY_API_KEY` for GLM 5.3 Flash, GLM 5.3, Kimi K3, DeepSeek V4 Pro, and legacy GLM 5.2 routes. | `codex-rs/model-provider-info/src/lib.rs` |
| Amazon Bedrock | Cloud-authenticated Bedrock models through the inherited provider route. | `codex-rs/model-provider-info/src/lib.rs`, `docs/config.md` |
| Ollama and LM Studio | Local model-server routes configured against the operator's endpoint. | `codex-rs/model-provider-info/src/lib.rs`, `docs/config.md` |
| Corbanu API | Wallet-linked dollar balance, API-key lifecycle, explicit model prices, and model routes. | `codex-rs/tui/src/chatwidget/wallet_api.rs`, `codex-rs/tui/src/chatwidget/model_popups.rs` |
| Custom providers | Operator-defined endpoints and wire configuration in `config.toml`. | `codex-rs/core/src/config/`, `docs/config.md` |
| Provider key vault | Provider keys are stored in encrypted vault labels such as `provider/zai_api_key`. | `codex-rs/login/src/auth/provider_key_vault.rs`, `codex-rs/vault/` |
| GLM request shaping | Ambient and Z.AI requests map Corbanu Terminal reasoning levels to provider-specific `reasoning_effort`, `enable_thinking`, and `emit_usage` fields. | `codex-rs/core/src/client.rs` |
| Ambient/Z.AI input conversion | Responses-style turn items are flattened for Ambient/Z.AI string input, while hidden reasoning is not replayed. | `codex-rs/codex-api/src/common.rs` |
| Onboarding | Provider picker supports the account/plan and API-key routes listed above, with keys stored through the masked vault boundary. | `codex-rs/tui/src/onboarding/auth.rs`, `codex-rs/tui/src/chatwidget/provider_credentials.rs` |
| Model picker | The Corbanu Terminal model picker groups current routes by provider and preserves exact provider/model/effort identity. | `codex-rs/tui/src/chatwidget/model_popups.rs` |
| Product branding | TUI, login prompts, installer messages, package names, and status surfaces use Corbanu Terminal branding. | `codex-rs/tui/`, `codex-rs/login/`, `codex-cli/`, `scripts/install/` |

## Design line

Provider-specific compatibility should stay small and explicit:

- Provider constants and built-in provider definitions belong in `codex-rs/model-provider-info`.
- Model metadata belongs in `codex-rs/models-manager/models.json`.
- Request serialization differences belong in `codex-rs/core/src/client.rs` or `codex-rs/codex-api`.
- UI selection behavior belongs in the TUI model and onboarding modules.

Avoid spreading provider assumptions through prompts or docs-only instructions. If the agent needs a capability, model, or provider behavior, it should be represented in configuration, model metadata, or typed code.

## Reading Path

1. [Ambient](ambient.md) for the default provider path.
2. [Z.AI GLM 5.2](zai-glm-52.md) for the direct Z.AI coding-plan path.
3. [OpenRouter](openrouter.md) for metered OpenRouter models.
4. [Baseten](baseten.md) for metered Baseten GLM 5.2.
5. [Vercel](vercel.md) for metered Vercel AI Gateway models.
6. [Codex Fork](codex-fork.md) for product changes around command names,
   packaging, status surfaces, and model picker behavior.
