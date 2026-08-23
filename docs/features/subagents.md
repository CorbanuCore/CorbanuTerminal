# Subagents

Corbanu Terminal can delegate bounded work to child agents, exchange messages,
wait for results, and inspect agent state. Subagents are a host-managed tool
surface, not a claim made only in prompt text.

> **Product specification — “Shipping MVP — LIVE”**
>
> “Agent orchestration: Sauron → Nazgul → Troll → Orc orchestration,
> model-aware delegation, durable mailboxes, supervision, resume, and recovery.”

## User flow

Ask the active agent to delegate a concrete task, for example:

```text
Spawn one explorer to inspect this repository's documentation structure and
return file-level findings.
```

The parent agent creates the child, sends the assignment, continues independent
work when useful, and waits for the result before claiming the delegated work
is complete.

Use `/agent` or `/subagents` to inspect available agent threads and switch
to a retained thread where supported.

## Bounded behavior

Agent creation is constrained by host configuration and the current hierarchy.
The host enforces thread and depth limits. An agent cannot widen its own
permissions merely by asking another agent to act.

Provider capability determines which collaboration tool representation is
available. If a provider cannot carry the active tool contract, Corbanu
Terminal must report that limitation instead of pretending delegation occurred.

## Relationship to `/spawn`

Plain subagents are useful for bounded exploration, review, and parallel
investigation. The `/spawn` workflow adds explicit Nazgul, Troll, and Orc
roles, parent validation, retained hierarchy, and product-specific supervision.

## Main implementation

- `codex-rs/core/src/tools/handlers/multi_agents.rs`
- `codex-rs/core/src/tools/handlers/multi_agents_spec.rs`
- `codex-rs/core/src/agent/`
- `codex-rs/tui/src/multi_agents.rs`
