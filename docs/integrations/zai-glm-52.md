# Z.AI GLM Coding Plan Integration

Corbanu Terminal supports direct GLM access through the Z.AI Coding Plan API.

## Current Provider

The built-in Z.AI provider is defined in `codex-rs/model-provider-info/src/lib.rs`:

| Field | Value |
| --- | --- |
| Provider id | `zai` |
| Display name | `Z.AI` |
| Base URL | `https://api.z.ai/api/coding/paas/v4` |
| API key env var | `ZAI_API_KEY` |
| Wire API | `chat` |
| OpenAI auth required | `false` |
| WebSockets | `false` |

Auth guidance shown to users:

```text
Set ZAI_API_KEY to your Z.AI Plan API key.
```

## Current Models

The visible Z.AI models are bundled in `codex-rs/models-manager/models.json`:

| Slug | Display name | Input | Context | Default reasoning |
| --- | --- | --- | ---: | --- |
| `glm-5.3-flash` | Z.AI GLM 5.3 Flash | Text and images | 1,000,000 | `max` |
| `glm-5.3` | Z.AI GLM 5.3 | Text | 1,000,000 | `max` |
| `glm-5.2` | Z.AI GLM 5.2 | Text | 1,000,000 | `medium` |

All three are listed in `/model`, support parallel function-tool calls, and
route through the same `zai` credential. GLM-5.3-Flash is the recommended
Coding Plan choice when its lower quota burn and native image input fit the
task.

## Model And Provider Selection

Corbanu Terminal maps model selections to providers in `codex-rs/tui/src/chatwidget/model_popups.rs`:

- Every bare `glm-*` model resolves to provider `zai`.
- `zai-org/*` resolves to provider `ambient`.
- The all-models popup only shows Corbanu Terminal-relevant Ambient and Z.AI models by default.

Configuration normalization in `codex-rs/core/src/config/mod.rs` keeps Z.AI sessions on Z.AI-compatible models:

- If `model_provider = "zai"` and no compatible model is configured, Corbanu Terminal selects `glm-5.2`.
- If a configured Z.AI model does not start with `glm-`, Corbanu Terminal replaces it with `glm-5.2`.
- GLM-5.2 retains its `Standard`/`Deep` compatibility mapping. GLM-5.3 and
  GLM-5.3-Flash expose the provider's `low`, `high`, and `max` reasoning modes.

## Request Behavior

Z.AI shares the GLM request compatibility path with Ambient:

- Corbanu Terminal emits `enable_thinking=true`.
- Corbanu Terminal emits `emit_usage=true`.
- Corbanu Terminal sends the catalog-selected reasoning effort. Models that
  require thinking, including GLM-5.3 and GLM-5.3-Flash, always receive a
  supported enabled effort and default to `max`.
- Function-tool schemas omit the OpenAI `strict` wrapper bit because GLM chat streams tool calls correctly without it.

Z.AI has one extra guard in the Chat Completions path: when native `web_search` and function tools would be mixed, Corbanu Terminal preserves client-executed function tools and removes native `web_search`, because coding sessions need shell/file tools to continue.

## Onboarding

The onboarding flow can present Z.AI as a provider API-key account alongside Ambient. This is tested in `codex-rs/tui/src/onboarding/auth.rs`.

Z.AI keys saved through onboarding are stored in the encrypted vault at:

```text
provider/zai_api_key
```

The environment variable `ZAI_API_KEY` is still supported for temporary shells
and automation.

## Source

- [Z.AI GLM-5.3-Flash model overview](https://docs.z.ai/guides/vlm/glm-5.3-flash)
- [Z.AI Coding Plan overview](https://docs.z.ai/devpack/overview)
- [Z.AI GLM-5.3-Flash open weights](https://huggingface.co/zai-org/GLM-5.3-Flash)
- `codex-rs/model-provider-info/src/lib.rs`
- `codex-rs/models-manager/models.json`
- `codex-rs/core/src/config/mod.rs`
- `codex-rs/core/src/client.rs`
- `codex-rs/tui/src/onboarding/auth.rs`
- `codex-rs/tui/src/chatwidget/model_popups.rs`
- `codex-rs/login/src/auth/provider_key_vault.rs`
