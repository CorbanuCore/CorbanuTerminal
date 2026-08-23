# Agent orchestration

## The pain

Delegation becomes untrustworthy when workers have no visible parent, durable
mailbox, or supervising reviewer. Corbanu Terminal turns a flat set of agent
tabs into a visible chain of command and can attach continuing supervision to
existing work.

## Product contract

> **Product specification — “Shipping MVP — LIVE”**
>
> “Agent orchestration: Sauron → Nazgul → Troll → Orc orchestration,
> model-aware delegation, durable mailboxes, supervision, resume, and recovery.”

## Role hierarchy

```text
Sauron (human)
└── Nazgul (root orchestrator)
    └── Troll (supervisor and reviewer)
        └── Orc (executor)
```

- **Sauron** is the human operating Corbanu Terminal.
- **Nazgul** is an existing user pane bound as the hierarchy root.
- **Troll** plans, delegates, reviews evidence, and supervises Orcs.
- **Orc** performs one bounded implementation, investigation, or validation task.

The host enforces valid parent-child relationships. Role prompts do not carry
that security boundary by themselves.

## Start a hierarchy

Run:

```text
/spawn
```

The picker can bind an existing user pane as the Nazgul or create a Troll or
Orc under a valid parent. The creation flow records the role, harness, model,
effort, task, and parent.

Direct commands are also available:

```text
/spawn nazgul
/spawn troll
/spawn orc
```

Binding a Nazgul does not create a worker. It marks the pane through which the
human supervises the hierarchy.

## Attach persistent supervision

Use `/orchestrate` when the crew or pane already exists and needs a continuing
supervisory assignment:

```text
/orchestrate
/orchestrate status
```

The guided flow can attach, pause, resume, extend, test or fire, and detach an
assignment. `/orchestrate` manages the assignment; it does not create the crew.

## Inspect and resume work

Use either command:

```text
/spawn status
/panes
```

The status view renders the hierarchy instead of flattening agents into
unrelated rows. Completed members remain inspectable, durable mailbox messages
flow between related agents, and retained panes can be resumed after switching
or restarting.

## Operating rules

- Give every worker a concrete task and an explicit parent.
- Use Trolls to supervise and review Orc execution.
- Keep implementation work bounded to the recorded worktree and scope.
- Wait for child results before reporting the parent task complete.
- Treat status, messages, and artifacts as evidence; a prompt claiming success
  is not evidence.
- Use the repository's release policy for true-TUI and human acceptance gates.

## Main implementation

- `codex-rs/tui/src/spawn_orchestration.rs`
- `codex-rs/core/src/tools/handlers/multi_agents.rs`
- `codex-rs/core/src/tools/handlers/multi_agents_spec.rs`
- `codex-rs/core/src/agent/`
