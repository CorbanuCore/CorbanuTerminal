# Orchestration roles

## The pain

A hierarchy is only useful when every participant understands who owns product
direction, supervision, execution, and evidence. These roles make that chain
explicit.

## Product contract

Exact product-spec heading: **Shipping MVP — LIVE**

Requirement excerpt: “Agent orchestration: Sauron → Nazgul → Troll → Orc
orchestration, model-aware delegation, durable mailboxes, supervision, resume,
and recovery.”

## Hierarchy

Corbanu Terminal `/spawn` uses a fixed role hierarchy:

```text
Nazgul -> Troll -> Orc
```

The hierarchy is enforced by the host runtime, not only by model prompts.
Nazgul is the root orchestrator pane. Trolls supervise and review execution.
Orcs execute assigned work and report evidence.

## Role Docs

| Role | Purpose | Source |
| ---- | ------- | ------ |
| [Nazgul](nazgul.md) | Root orchestrator and user-facing CTO pane | Generated runtime context in `codex-rs/tui/src/spawn_orchestration.rs` |
| [Troll](troll.md) | Supervisor, reviewer, and foreman over Orcs | Built-in role file plus runtime pane context |
| [Orc](orc.md) | IC executor that performs assigned work | Built-in role file plus runtime pane context |

See [Spawn Orchestration](../features/spawn-orchestration.md) for the finished
user workflow.
