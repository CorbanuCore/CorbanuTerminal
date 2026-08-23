# Workspaces and panes

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

## Navigate panes

Run `/panes` to switch among:

- the main Corbanu Terminal pane;
- retained user panes;
- Claude Code headless panes; and
- panes created by agent orchestration.

Creating a Claude pane starts from the same picker. See
[Claude Code headless panes](claude-headless-panes.md) for provider and
credential behavior.

## Navigate native agents

Use `/agent` or `/subagents` to inspect and switch native agent threads.
Use `/spawn status` for the explicit Nazgul, Troll, and Orc hierarchy.

A compacted conversation is not a dead pane. Inspect the authoritative pane and
agent views before replacing work.

## Background terminals

```text
/ps
/stop
```

`/ps` lists background terminals. `/stop` stops all of them, so use it only
when that broad cleanup is intended.

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
