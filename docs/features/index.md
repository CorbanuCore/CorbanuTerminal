# Feature catalog

## The pain

A feature list is useless when it mixes roadmap items, implementation journals,
and scattered command references. This catalog maps every capability marked
**LIVE** in the product specification to one canonical user-facing page.

## Shipping MVP inventory

Exact product-spec heading: **Shipping MVP — LIVE**

| Area | Status | Requirement excerpt | Canonical documentation |
| --- | --- | --- | --- |
| Runtime | **LIVE** | “Rust, Apache-2.0, Linux/macOS/Windows, the `corbanu` command, and legacy `pfterminal` command and state compatibility.” | [Runtime, permissions, and extensions](runtime-extensions.md) |
| Multi-provider inference | **LIVE** | “OpenAI, Anthropic/Claude Plan, Kimi, Z.AI, DeepSeek, OpenRouter, Ambient, Meta, Baseten, Vercel, Bedrock, Ollama, LM Studio, Corbanu Plan, and custom providers.” | [Models and providers](model-providers.md) |
| Vault and credentials | **LIVE** | “Encrypted `/vault`, masked entry, metadata-only inspection, and operational credential use without placing raw values in chat.” | [Authentication and Vault](../authentication.md) |
| Agent orchestration | **LIVE** | “Sauron → Nazgul → Troll → Orc orchestration, model-aware delegation, durable mailboxes, supervision, resume, and recovery.” | [Agent orchestration](spawn-orchestration.md) |
| Workspaces | **LIVE** | “`/panes`, `/agent`, approvals, existing general sandboxing, review, MCP, skills, plugins, apps, connectors, and background terminals.” | [Workspaces and panes](workspaces.md) |
| Wallet and payments | **LIVE** | “Local Solana wallet, SOL and canonical USDC support, scoped signing, backup/restore, and Corbanu Plan purchase/recovery.” | [Wallet and Corbanu Plan](wallet-plan.md) |
| Compute | **LIVE** | “Vast.ai and RunPod rental workflows with price, spend, duration, readiness, stop, and termination controls.” | [GPU rentals](gpu-rentals.md) |
| Task Node and identity | **LIVE** | “Tasks, evidence, verification, rewards, balances, chat, context, linked identity, and live Task Node-linked Nostr identity.” | [Task Node and identity](tasknode.md) |
| Remote and context | **LIVE** | “Allowlisted Telegram; durable `/goal` and `/memories`; ephemeral `/side` and `/btw`; `/skills` and `/docs`.” | [Telegram](telegram.md) · [Goals, memories, side work, skills, and docs](context-tools.md) |

## Corbanu Plan inventory

Exact product-spec heading: **Corbanu Plan — LIVE**

| Capability | Status | Requirement excerpt | Canonical documentation |
| --- | --- | --- | --- |
| Wallet-native prepaid inference | **LIVE** | “Corbanu Plan is wallet-native, one-calendar-month prepaid inference purchased through x402, normally using canonical USDC on Solana.” | [Wallet and Corbanu Plan](wallet-plan.md) |
| Plan model routing | **LIVE** | “GLM 5.2, Kimi K2.7 Code” use Ambient; “DeepSeek V4 Pro, Claude Fable 5” use xAPI. | [Wallet and Corbanu Plan](wallet-plan.md) |
| Tier allowance | **LIVE** | “Every tier uses the same model catalog and differs by allowance.” | [Models and providers](model-providers.md) |

## Detailed feature references

These pages explain deeper parts of the live capability groups without creating
additional product status:

- [OpenAI Codex account login](codex-account-login.md)
- [Claude Code headless panes](claude-headless-panes.md)
- [Subagents](subagents.md)
- [Nazgul, Troll, and Orc roles](../orchestration/index.md)
- [Provider integration references](../integrations/index.md)

Anything marked **TO BUILD**, **TO INTEGRATE**, **TO ACQUIRE**, or
**BUILT NOT LIVE** belongs only in the
[product specification](../corbanu-product-spec.md) or an active plan. It is not
represented here as an existing feature.
