# Models and providers

## The pain

Using several inference providers normally means juggling credentials, model
names, privacy boundaries, and separate usage dashboards. Corbanu Terminal puts
provider access and model selection behind one consistent TUI workflow.

## Product contract

| Field | Value |
| --- | --- |
| Status | **LIVE** |
| Exact product-spec heading | **Shipping MVP — LIVE** |
| Requirement excerpt | “Multi-provider inference: OpenAI, Anthropic/Claude Plan, Kimi, Z.AI, DeepSeek, OpenRouter, Ambient, Meta, Baseten, Vercel, Bedrock, Ollama, LM Studio, Corbanu Plan, and custom providers.” |

## User flow

1. Run `/providers`.
2. Sign in to an account-backed provider or add an API key through masked entry.
3. Run `/model`.
4. Select the provider, model, and reasoning effort.
5. Use `/usage` for allowance and reset information.
6. Use `/status` to confirm the active model, permissions, and session state.

Adding a credential does not select that provider. Authentication and model
selection are deliberately separate actions.

## Available provider routes

| Access class | Routes |
| --- | --- |
| Account and plan | OpenAI Codex account, Claude Plan, Corbanu Plan |
| Hosted provider | Anthropic, Kimi Code, Z.AI, DeepSeek, OpenRouter, Ambient, Meta, Baseten, Vercel AI Gateway |
| Cloud | Amazon Bedrock |
| Local | Ollama, LM Studio |
| Operator-defined | Custom providers configured in `config.toml` |

The authentication surface varies by provider. `/providers` handles supported
account and masked API-key flows. Cloud, local, and custom routes use their
documented environment or configuration inputs.

## Operational boundaries

- A listed provider is not necessarily authenticated on the current machine.
- A configured credential does not guarantee that every model is enabled for
  the current account or plan.
- Corbanu Terminal preserves the exact provider/model identity when routing a
  turn.
- Provider privacy, retention, billing, and jurisdictional rules still apply.
- Use the privacy label shown for Corbanu Plan models before sending sensitive
  strategy or financial context.

## Related documentation

- [Authentication and Vault](../authentication.md)
- [Configuration](../config.md)
- [Corbanu Plan](wallet-plan.md)
- [Provider integration references](../integrations/index.md)
