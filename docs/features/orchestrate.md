# /orchestrate persistent supervision

## The pain

A worker pane can exist and still stall, lose direction, or finish without
review. Corbanu Terminal can attach a durable Manager-to-Worker assignment so
progress checks, handoffs, pauses, and recovery remain visible instead of
depending on one chat turn.

## Product contract

| Field | Value |
| --- | --- |
| Status | **LIVE** |
| Exact product-spec heading | **Shipping MVP — LIVE** |
| Requirement excerpt | “Agent orchestration: Sauron → Nazgul → Troll → Orc orchestration, model-aware delegation, durable mailboxes, supervision, resume, and recovery.” |

## What /orchestrate does

`/orchestrate` attaches persistent supervision to an existing Corbanu
Terminal, Claude Code, or managed worker pane. It creates and manages the
assignment; it does not create the worker hierarchy.

Use the related commands for different jobs:

| Command | Responsibility |
| --- | --- |
| `/panes` | Create or switch user panes and inspect managed crew panes. |
| `/spawn` | Create or bind the Nazgul, Troll, and Orc hierarchy. |
| `/orchestrate` | Assign a Manager to keep driving an existing Worker. |
| `/agent` | Switch the active native agent thread. |

## Start an assignment

For the shortest guided flow, run:

```text
/orchestrate
```

Choose the Worker, choose or create its Manager, and confirm the visible
assignment. For the full guided flow—including duration and standing
instructions—run:

```text
/orchestrate attach
```

The full flow asks for:

1. the existing Worker pane;
2. a duration;
3. an assignment specification;
4. an existing or newly created Manager pane; and
5. final confirmation of the Manager → Worker relationship.

A Manager cannot also be its own Worker. Corbanu Terminal Main remains the
human control surface and is not used as an assignment Manager.

## Inspect and control assignments

Run:

```text
/orchestrate status
```

The status view shows each Manager → Worker relationship, assignment phase,
expiry, latest dispatch, and specification. Open an assignment to use the
guided controls, or use the repeatable commands below.

| Command | Effect |
| --- | --- |
| `/orchestrate pause <id>` | Stop automatic supervision without deleting the assignment. |
| `/orchestrate resume <id>` | Re-arm a paused assignment. |
| `/orchestrate extend <id> <duration>` | Add time to its current expiry. |
| `/orchestrate fire <id>` | Send the next Manager mandate immediately; it does not terminate the Worker. |
| `/orchestrate test <id>` | Preview the next mandate without sending it. |
| `/orchestrate detach <id|target>` | End the assignment while leaving its panes and crew intact. |

Advanced inline attachment is also available:

```text
/orchestrate attach <target> <whip-name> [--mode review|auto] \
  [--for 4h|--until HH:MM|--for unlimited] \
  [--max N] [--cooldown S] [--holder me|none]
```

Prefer the guided flow unless repeatable automation requires inline syntax.

## Persistence and recovery

Assignments are retained with the pane layout. After switching panes or
restarting Corbanu Terminal, use `/orchestrate status` to confirm the
authoritative assignment state before creating a replacement. If a Manager or
Worker becomes unreachable, preserve the panes and inspect the visible error;
do not treat context compaction as a dead worker.

Pausing, detaching, or expiring an assignment changes supervision only. It does
not archive a pane, delete an agent thread, stop every background terminal, or
remove the managed `/spawn` crew.

## Related documentation

- [`/panes` and workspaces](workspaces.md)
- [`/spawn` hierarchy](spawn-orchestration.md)
- [Slash command reference](../slash_commands.md)
