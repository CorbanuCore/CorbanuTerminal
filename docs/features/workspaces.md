# /panes and workspaces

## The pain

Long-running agent work becomes hard to trust when user sessions, Claude
sessions, spawned agents, and background terminals are scattered across
unrelated windows. Corbanu Terminal keeps them visible inside one retained
workspace.

## Product contract

| Field | Value |
| --- | --- |
| Status | **LIVE** |
| Exact product-spec heading | **Shipping MVP — LIVE** |
| Requirement excerpt | “Workspaces: `/panes`, `/agent`, approvals, existing general sandboxing, review, MCP, skills, plugins, apps, connectors, and background terminals.” |

## Open /panes

Run:

```text
/panes
```

The searchable picker makes all pane types visible in one place:

| Picker section | What it contains |
| --- | --- |
| **User Panes** | Main, retained native Corbanu Terminal panes, and Claude Code headless panes. |
| **Create User Pane** | A new native pane with its own model, or a new Claude pane with its provider route. |
| **Managed Crew (/spawn)** | The retained Nazgul, Troll, and Orc hierarchy for inspection and switching. |

Select a row to switch to that pane. Choose **+ Corbanu Terminal Pane** to pick
a model and name for a persistent native pane. Choose **+ Claude Pane** to pick
a Claude provider route and name. Use the visible rename shortcut to rename a
user pane; managed crew lifecycle remains under `/spawn`.

See [Claude Code headless panes](claude-headless-panes.md) for Claude provider
and credential behavior.

## Choose the right surface

| Command | Use it for |
| --- | --- |
| `/panes` | Create, find, rename, or switch user panes and inspect managed crew panes. |
| `/agent` or `/subagents` | Inspect and switch native agent threads. |
| `/spawn status` | Inspect the explicit Nazgul, Troll, and Orc hierarchy. |
| `/orchestrate status` | Inspect persistent Manager → Worker assignments attached to panes. |
| `/ps` | Inspect background terminals rather than conversational panes. |
| `/stop` | Stop all background terminals only when that broad cleanup is intended. |

A compacted conversation is not a dead pane. Check `/panes`, `/agent`, and
the relevant status view before replacing or recreating work.

## Workspace controls

The workspace retains the Codex-derived approval, sandbox, review, MCP, skill,
plugin, app, and connector surfaces. Use `/permissions` to inspect or change
what the active session may do, `/review` to review current changes, and
`/mcp`, `/skills`, `/plugins`, or `/apps` to inspect the corresponding
extension surface.

## State boundary

Pane switching must not silently widen permissions, expose vault secrets, or
discard retained work. A destructive session or terminal action requires the
same explicit user intent regardless of which pane is active.

## Related documentation

- [`/orchestrate` persistent supervision](orchestrate.md)
- [`/spawn` hierarchy](spawn-orchestration.md)
- [Slash command reference](../slash_commands.md)
