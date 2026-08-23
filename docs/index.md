# Corbanu Terminal

Corbanu Terminal is a trader-first, wallet-native agentic terminal built on the
open-source Codex CLI. It keeps the local coding-agent runtime and makes model
providers, retained orchestration, wallet and Plan, GPU compute, Telegram,
Task Node identity, encrypted credentials, and persistent workspaces one
coherent product.

This site is the engineering front door. It is not a dump of internal notes. It
points to the current code paths, docs, and packaging surfaces that define what
has been integrated.

## What exists now

The [Feature Catalog](features/index.md) is the canonical inventory of finished
product behavior.

| Area | Pain solved | Current capability | Read |
| --- | --- | --- | --- |
| Runtime | A local agent needs visible tools and authorization | Cross-platform Rust TUI, permissions, sandbox, review, extensions, and the `corbanu` command | [Runtime and extensions](features/runtime-extensions.md) |
| Models | Provider access is fragmented | Unified provider authentication, model selection, usage, and status across hosted, cloud, local, Plan, and custom routes | [Models and providers](features/model-providers.md) |
| Vault | Secrets should never be pasted into chat | Encrypted storage, masked entry, metadata-only inspection, and operational credential use | [`/vault` and credentials](features/vault.md) |
| Orchestration | Delegated work needs hierarchy and supervision | Nazgul, Troll, and Orc roles; subagents; durable mailboxes; persistent assignments; resume and recovery | [`/spawn` hierarchy](features/spawn-orchestration.md) · [`/orchestrate` supervision](features/orchestrate.md) |
| Workspaces | Long-running sessions are hard to track | User, Claude, and agent panes; retained state; approvals; background terminals | [`/panes` and workspaces](features/workspaces.md) |
| Wallet | Custody and payment need explicit control | Local Solana wallet, SOL/USDC, scoped signing, backup/restore, and Plan ownership | [Wallet and Corbanu Plan](features/wallet-plan.md) |
| Corbanu Plan | Inference payment and entitlement should be native | Wallet-purchased monthly x402 Plan, tier allowance, receipt, recovery, usage, and model routing | [Wallet and Corbanu Plan](features/wallet-plan.md) |
| Compute | GPU rentals can overspend or continue billing | Vast.ai and RunPod budgets, readiness, stop, and provider-confirmed termination | [GPU rentals](features/gpu-rentals.md) |
| Task Node | Tasks and identity should follow the agent | Tasks, evidence, requests, context, chat, rewards, balances, and Task Node-linked Nostr identity | [Task Node and identity](features/tasknode.md) |
| Remote control | Users need bounded access away from the keyboard | Allowlisted Telegram connector with explicit workspace and authorization | [Telegram](features/telegram.md) |
| Context | Long work and tangents need different persistence | Durable goals and memories; ephemeral side conversations; in-terminal skills and docs | [Context tools](features/context-tools.md) |

## Fast reading path

1. Start with the [Feature Catalog](features/index.md) for everything that is
   live now.
2. Read the [Product Specification](corbanu-product-spec.md) for product
   boundaries, ownership, and explicitly labeled roadmap work.
3. Use [Install and First Run](install.md), then
   [Authentication and account setup](authentication.md) and
   [`/vault` and credentials](features/vault.md).
4. Keep the [Slash Command Reference](slash_commands.md) nearby for repeatable
   TUI actions.
5. Contributors and coding agents must read the
   [Development Mandate](development-mandate.md).

## Core Claim

Corbanu Terminal is a Codex-derived terminal agent with all of the product surfaces
listed above kept first-class. The implementation changes include provider and
model routing, native retained agents, pane/session persistence, wallet and GPU
operations, remote-control and Task Node bridges, encrypted credential storage,
packaging, and branding—not just prompt text.

## Development mandate

Contributors and coding agents follow the same classified development and
release workflow. Read the [Development Mandate](development-mandate.md) for
the explicitly marked human-readable mirror of the repository's development
skill.

## Repository layout

The main implementation is under `codex-rs/`, inherited from the open-source Codex CLI Rust workspace.

Product-facing packaging lives in:

- `codex-cli/` for the npm CLI package.
- `scripts/install/` for standalone installers.
- `sdk/` for SDK surfaces.

User-facing docs live in this `docs/` directory and are built by MkDocs from the repository root:

```bash
mkdocs serve
mkdocs build
```

## Self-Hosted URL

The docs can be built and served as a standalone site:

```bash
python3 -m venv .venv-docs
. .venv-docs/bin/activate
pip install -r requirements-docs.txt
scripts/docs-site-build
scripts/docs-site-serve --host 127.0.0.1 --port 8089
```

On the current shared docs host, the static Corbanu Terminal build is also published
under the existing authenticated L1 docs server:

```text
http://5.223.45.94:8088/terminal/
```
