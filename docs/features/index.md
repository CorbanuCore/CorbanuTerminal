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
| Multi-provider inference | **LIVE** | “OpenAI, Anthropic/Claude Plan, Kimi, Z.AI, DeepSeek, OpenRouter, Ambient, Meta, Baseten, Vercel, Bedrock, Ollama, LM Studio, Corbanu Plan, and custom providers.” | [`/providers`, account login, and `/model`](model-providers.md) |
| Vault and credentials | **LIVE** | “Encrypted `/vault`, masked entry, metadata-only inspection, and operational credential use without placing raw values in chat.” | [`/vault` and credentials](vault.md) |
| Agent orchestration | **LIVE** | “Sauron → Nazgul → Troll → Orc orchestration, model-aware delegation, durable mailboxes, supervision, resume, and recovery.” | [`/spawn` hierarchy](spawn-orchestration.md) · [`/orchestrate` supervision](orchestrate.md) |
| Workspaces | **LIVE** | “`/panes`, `/agent`, approvals, existing general sandboxing, review, MCP, skills, plugins, apps, connectors, and background terminals.” | [`/panes` and workspaces](workspaces.md) |
| Wallet and payments | **LIVE** | “Local Solana wallet, SOL and canonical USDC support, scoped signing, backup/restore, and Corbanu Plan purchase/recovery.” | [Wallet and Corbanu Plan](wallet-plan.md) |
| Compute | **LIVE** | “Vast.ai and RunPod rental workflows with price, spend, duration, readiness, stop, and termination controls.” | [GPU rentals](gpu-rentals.md) |
| Task Node and identity | **LIVE** | “Tasks, evidence, verification, rewards, balances, chat, context, linked identity, and live Task Node-linked Nostr identity.” | [Task Node and identity](tasknode.md) |
| Remote and context | **LIVE** | “Allowlisted Telegram; durable `/goal` and `/memories`; ephemeral `/side` and `/btw`; `/skills` and `/docs`.” | [Telegram](telegram.md) · [Goals, memories, side work, skills, and docs](context-tools.md) |

## Find a live feature by command

| Command surface | Finished user goal | Canonical documentation |
| --- | --- | --- |
| `/providers`, `/model`, `/usage`, `/status` | Authenticate account and credential routes, select the runtime, and inspect availability. | [`/providers`, account login, and `/model`](model-providers.md) |
| `/vault` | Store and use operational credentials without placing values in chat. | [`/vault` and credentials](vault.md) |
| `/panes`, `/agent`, `/subagents`, `/ps`, `/stop` | Create, switch, inspect, and safely distinguish panes, threads, and terminals. | [`/panes` and workspaces](workspaces.md) |
| `/spawn` | Create or bind the Nazgul, Troll, and Orc hierarchy. | [`/spawn` hierarchy](spawn-orchestration.md) |
| `/orchestrate` | Attach and control persistent Manager → Worker supervision. | [`/orchestrate` supervision](orchestrate.md) |
| `/wallet` | Manage custody, balances, backup/restore, and Corbanu API. | [`/wallet` and Corbanu API](wallet-plan.md) |
| `/gpu` | Rent, inspect, stop, and terminate third-party GPU capacity. | [`/gpu` rentals](gpu-rentals.md) |
| `/tasknode` | Use tasks, evidence, verification, rewards, context, and linked identity. | [`/tasknode` and identity](tasknode.md) |
| `/telegram` | Configure and operate bounded remote control. | [`/telegram` remote control](telegram.md) |
| `/goal`, `/memories`, `/side`, `/btw`, `/skills`, `/docs` | Separate durable work from ephemeral context and browse terminal guidance. | [Context tools](context-tools.md) |
| `/permissions`, `/review`, `/mcp`, `/apps`, `/plugins` | Inspect authorization, changes, and extension surfaces. | [Runtime, permissions, and extensions](runtime-extensions.md) |

The [slash command reference](../slash_commands.md) provides the complete
repeatable syntax.

## Wallet-funded inference transition

The 0.1.38 integration candidate implements the later **Corbanu API — TO
BUILD** decision. It removes new plan sales and legacy entitlement surfaces in
favor of an arbitrary dollar balance and wallet-owned API keys. Human acceptance
and release authorization are still pending; the older Plan heading remains in
the product specification as historical/deprecation context.

| Capability | Candidate status | Canonical documentation |
| --- | --- | --- |
| Wallet-funded dollar balance | **INTEGRATION CANDIDATE** | [Wallet and Corbanu API](wallet-plan.md) |
| One-time API-key reveal and key lifecycle | **INTEGRATION CANDIDATE** | [Wallet and Corbanu API](wallet-plan.md) |
| Explicit per-model prices and privacy labels | **INTEGRATION CANDIDATE** | [Models and providers](model-providers.md) |

## Detailed feature references

These pages explain deeper parts of the live capability groups without creating
additional product status:

- [Claude Code headless panes](claude-headless-panes.md)
- [Subagents](subagents.md)
- [Nazgul, Troll, and Orc roles](../orchestration/index.md)
- [Provider integration references](../integrations/index.md)

Anything marked **TO BUILD**, **TO INTEGRATE**, **TO ACQUIRE**, or
**BUILT NOT LIVE** belongs only in the
[product specification](../corbanu-product-spec.md) or an active plan. It is not
represented here as an existing feature.
