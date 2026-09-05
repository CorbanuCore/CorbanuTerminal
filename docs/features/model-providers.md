# /providers, account login, and /model

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

On first run, configure as many providers as you want and then choose **Done**.
The first provider that completes successfully becomes the fresh installation's
initial provider; later successful setup does not silently replace it. Choosing
Corbanu API queues wallet and API-key setup until after **Done**, so you can configure the
other providers first. Cancelling or escaping the deferred API flow returns to
the terminal without changing an already usable provider.

Configured providers are active by default. Adding a credential does not select
that provider: authentication, eligibility, and the current model are separate.
Use `/providers` to deactivate or reactivate a provider. Deactivation preserves
its credential. Deactivating the current provider requires an explicit usable
replacement; cancelling that choice leaves both current-provider and
eligibility state unchanged.

## Choosing subagent models

Ask for the model you want, for example: “Use Luna and Kimi K3 as separate
subagents and wait for both results.” Providers still need working credentials,
and any explicit `agents.provider_allowlist` remains authoritative.

The reconciled debug candidate supports OpenAI Luna (`gpt-5.6-luna`) and Kimi
Code K3 (`k3`) under the same native V2 orchestration engine. A model's preferred
engine version does not exclude it from child selection. Exact runtime overrides
use the typed plaintext adapter; the native encrypted OpenAI interface remains
available for inherited runtimes. Kimi K3 supports low, high and max effort, not
medium. No fallback model is substituted when an exact request fails.

Both runtimes have passed real TUI repository work and follow-up on the same
child threads after a parent restart. [Candidate evidence](../../qa/release/0.1.38/subagent-runtime.md).
This implements **Shipping MVP — LIVE**, “model-aware delegation, durable
mailboxes, supervision, resume, and recovery.”

## Account-backed provider login

Account login is one access mode inside the provider feature. OpenAI Codex
Account and Claude Plan are peers in `/providers`; neither is a standalone
Corbanu Terminal feature.

| Account route | What `/providers` shows | Authentication owner |
| --- | --- | --- |
| OpenAI Codex Account | Sign-in status, email, and plan when available | Corbanu Terminal's inherited Codex account manager |
| Claude Plan | Explicit managed-token or Claude Code login source and health | Corbanu's encrypted managed-token path or Claude Code's platform-owned login, exactly as selected |

### OpenAI Codex Account

1. Run `/providers`.
2. Select **Provider: OpenAI Codex Account**.
3. Open the displayed verification URL and enter the one-time device code.
4. Return to Corbanu Terminal after sign-in completes.
5. Run `/model` and select an OpenAI model.

The same route is available from first-run onboarding or `corbanu login`.

#### GPT-6 Astra

Run `/model`, choose OpenAI, then **GPT-6 Astra** (`gpt-6-astra`). Choose Low,
Medium, High, Extra high, or **More reasoning… → Max**. Medium is Astra's picker
default; adding Astra does not replace your selected model or the existing Sol
default. Cancel before confirming the reasoning level to keep your selection.

Astra uses the native OpenAI Responses route. Availability still depends on
OpenAI's rollout and your account's permissions; a selector entry does not grant
access. It is offered for explicit selection, not automatic agent allocation,
whose account-usage economics are not yet configured. Corbanu uses Astra's native
Codex configuration: Code Mode tools, Responses Lite, and a 272,000-token
default context with an 872,000-token maximum. These native limits come from
the [upstream Codex catalog](https://github.com/openai/codex/blob/rust-v0.153.4/codex-rs/models-manager/models.json),
not the larger public API model configuration.

The reconciled candidate fixes the older-client rejection and has passed live
TUI file/tool, cancellation, restart and resume checks in both default test
repositories. [Qualification and build identity](../../qa/release/0.1.38/astra-runtime.md).

### Claude Plan

Claude Code must be installed for this account route.

1. On first run, select the default **Provider: Anthropic Claude Account** row.
   On an existing installation, run `/providers` and select **Provider: Claude
   Code Plan** instead.
2. Confirm the account route.
3. Choose **Long-lived subscription token (Recommended)** or **Claude Code
   login**. The first option is selected by default.
4. Complete the displayed setup. Token and authorization-code entry are masked.
5. Wait for Corbanu Terminal to report the exact method as selected.
6. Run `/model` and select a Claude Plan model.

Corbanu never falls back between the two methods after a choice is persisted.
OpenAI and Claude account state remain independent; signing in to one route
does not authenticate the other. See
[Reliable Claude Plan authentication](claude-plan-authentication.md) for token
eligibility, platform stores, replacement, and recovery.

## Included providers

Each included provider is listed explicitly below. A provider being included
means Corbanu Terminal has a route for it; the current machine still needs the
corresponding account, credential, entitlement, cloud configuration, or local
server.

| Provider | Route | Configure access |
| --- | --- | --- |
| OpenAI Codex Account | Built-in `openai` account route | Sign in with device code from `/providers` or `corbanu login`. |
| Anthropic | Built-in `anthropic` API route | Add `ANTHROPIC_API_KEY` through `/providers` or the encrypted vault. |
| Claude Plan | Built-in `claude-plan` account route | Choose the Anthropic Claude account on first run, or use `/providers` later; both offer the recommended long-lived subscription token and Claude Code login. |
| Ambient | Built-in `ambient` API route | Add `AMBIENT_API_KEY` through `/providers` or the encrypted vault. |
| Kimi Code | Built-in `kimi-code` API route | Add `KIMI_API_KEY` through `/providers` or the encrypted vault. |
| Z.AI | Built-in `zai` API route | Add `ZAI_API_KEY` through `/providers` or the encrypted vault. |
| DeepSeek | Built-in `deepseek` API route | Add `DEEPSEEK_API_KEY` through `/providers` or the encrypted vault. |
| OpenRouter | Built-in `openrouter` gateway route | Add `OPENROUTER_API_KEY` through `/providers` or the encrypted vault. |
| Meta | Built-in `meta` API route | Add `MODEL_API_KEY` through `/providers` or the encrypted vault. |
| Baseten | Built-in `baseten` gateway route | Add `BASETEN_API_KEY` through `/providers` or the encrypted vault. |
| Vercel AI Gateway | Built-in `vercel` gateway route | Add `AI_GATEWAY_API_KEY` through `/providers` or the encrypted vault. |
| Amazon Bedrock | Built-in `amazon-bedrock` cloud route | Configure AWS credentials/profile or a managed Bedrock API key and region. |
| Ollama | Built-in `ollama` local route | Run Ollama locally and select it as the local provider. |
| LM Studio | Built-in `lmstudio` local route | Run LM Studio locally and select it as the local provider. |
| Corbanu API | Wallet-linked `corbanu-plan` compatibility route | Fund a dollar balance and manage keys in `/wallet`; inspect status in `/providers`. |
| Custom provider | Operator-defined compatible route | Add a `[model_providers.<id>]` entry in `config.toml`. |

Some provider families have additional internal wire-protocol routes so the
same provider can serve both Responses-, Chat Completions-, and
Anthropic-compatible models. Those transport details do not create additional
user-facing providers.

## Which command to use

| Command | Purpose |
| --- | --- |
| `/providers` | Authenticate, replace a supported provider key, and inspect availability. |
| `/vault` | Manage encrypted service credentials by label. |
| `/model` | Select the active provider, model, and reasoning effort. |
| `/usage` | Inspect allowance and reset information where the route reports it. |
| `/status` | Confirm the provider/model active for the current session. |

The authentication surface varies by provider. `/providers` handles supported
account and masked API-key flows. Cloud, local, and custom routes use their
documented environment or configuration inputs.

## Operational boundaries

- A listed provider is not necessarily authenticated on the current machine.
- A configured credential does not guarantee that every model is enabled for
  the current account or plan.
- Corbanu Terminal preserves the exact provider/model identity across normal
  turns, native child-agent requests, process restart, and resumed work.
- If the current provider becomes inactive, unavailable, or disappears from the
  active profile, Corbanu blocks requests and child spawns until you recover it
  or explicitly choose a replacement. It never silently switches providers.
- A custom command-auth provider is visible and selectable without an invented
  enrollment screen. Corbanu executes and validates the configured command only
  when a request or native child-agent launch actually uses that provider; a
  failed command blocks that operation with the provider error.
- Provider privacy, retention, billing, and jurisdictional rules still apply.
- Use the privacy label shown for Corbanu API models before sending sensitive
  strategy or financial context.

## Related documentation

- [`/vault` and credentials](vault.md)
- [Authentication and account setup](../authentication.md)
- [Configuration](../config.md)
- [Corbanu API](wallet-plan.md)
- [Provider integration references](../integrations/index.md)
