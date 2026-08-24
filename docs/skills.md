# Skills

## The pain

A general agent should not make up specialized workflows for wallets, Task
Node, releases, design, or unfamiliar repositories. Skills provide
task-specific instructions that load only when their trigger matches the work.

## Product contract

| Field | Value |
| --- | --- |
| Status | **LIVE** |
| Exact product-spec heading | **Shipping MVP — LIVE** |
| Requirement excerpt | “Remote and context: Allowlisted Telegram; durable `/goal` and `/memories`; ephemeral `/side` and `/btw`; `/skills` and `/docs`.” |
| Related excerpt | “Workspaces: `/panes`, `/agent`, approvals, existing general sandboxing, review, MCP, skills, plugins, apps, connectors, and background terminals.” |

## Browse skills

Run:

```text
/skills
```

The browser shows system, repository, user, and plugin-provided skills available
to the current session. A skill is selected by its trigger description or by an
explicit user reference such as `$skill-name`.

## Skill locations

| Layer | Location | Scope |
| --- | --- | --- |
| Bundled system | `$CODEX_HOME/skills/.system/` | Installed with Corbanu Terminal |
| User | `$HOME/.agents/skills/` | Available across repositories |
| Repository | `<repo>/.agents/skills/` | Portable across compatible agents in that project |
| Plugin | Installed plugin package | Available while the plugin is installed |

A fresh Corbanu home normally uses `$HOME/.corbanu/skills/.system/`. An
upgraded installation may continue using the legacy state home selected by the
documented home-resolution rules.

## Bundled system skills

| Skill | Purpose |
| --- | --- |
| `corbanu-terminal-help` | Operate Corbanu providers, vault, wallet, Plan, GPU, panes, orchestration, and Task Node |
| `frontend-design` | Build production browser interfaces |
| `imagegen` | Generate or edit raster images |
| `openai-docs` | Use official OpenAI and Codex documentation |
| `plugin-creator` | Create and update plugins |
| `postfiat-l1-development` | Develop and review the Post Fiat L1 workspace |
| `review-agent` | Perform structured review work |
| `skill-creator` | Create and improve skills |
| `skill-installer` | Install curated or repository-hosted skills |
| `tasknode-usage` | Use Task Node as an agent-side context and work ledger |

## Portable repository bundle

This repository checks every contributor skill into two agent-facing locations:

| Location | Contract |
| --- | --- |
| `.codex/skills/` | Native Corbanu/Codex repository skill tree and editing source |
| `.agents/skills/` | Complete agent-neutral portability mirror committed with the repository |

The mirror contains every tracked contributor skill, including its scripts,
references, assets, executable modes, and agent metadata. After changing a
repository skill, update both paths and run:

```bash
python3 scripts/check_portable_skills.py
```

CI rejects a missing, extra, stale, or mode-mismatched portable file. Bundled
system skills are not duplicated here because their portable source is already
tracked under `codex-rs/skills/src/assets/samples/` and installed with Corbanu
Terminal.

The
[`corbanu-terminal-development` skill](corbanu-terminal-development-skill.md) is rendered in
this documentation for contributors; `test-tui` and `remote-tests` remain
repository-loaded QA skills.

## Safety boundary

A skill supplies instructions; it does not widen filesystem, network, vault,
wallet, financial, or approval permissions. Skills must direct secret entry to
masked host views and must never request recovery material or raw credentials
in chat.
