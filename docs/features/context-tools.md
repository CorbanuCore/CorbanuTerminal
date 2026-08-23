# Goals, memories, side work, skills, and docs

## The pain

An agent needs durable direction for long work, selective memory across
sessions, and a safe way to answer tangential questions without polluting the
main task. The user also needs product guidance to remain available inside the
terminal.

## Product contract

| Field | Value |
| --- | --- |
| Status | **LIVE** |
| Exact product-spec heading | **Shipping MVP — LIVE** |
| Requirement excerpt | “Remote and context: Allowlisted Telegram; durable `/goal` and `/memories`; ephemeral `/side` and `/btw`; `/skills` and `/docs`.” |

## Context surfaces

| Surface | Persistence | Purpose |
| --- | --- | --- |
| `/goal [objective]` | Durable | Set or inspect the objective for a long-running task |
| `/memories` | Durable | Configure memory use and generation |
| `/side [prompt]` | Ephemeral fork | Ask a side question and return to the main conversation |
| `/btw [prompt]` | Ephemeral fork | Alias for the same side-conversation workflow |
| `/skills` | Installed state | Browse skills available to the current session |
| `/docs [page]` | Packaged docs | Browse this MkDocs documentation inside the terminal |

## Use durable context deliberately

A goal is for a concrete long-running objective. Memories are for information
that should be available across appropriate sessions. Neither should be used to
store credentials, wallet recovery material, or protected financial data.

## Keep side work ephemeral

`/side` and `/btw` fork the current context for a tangential exchange and
then return to the main work. They do not replace a durable goal and should not
be treated as a hidden authorization channel.

## Skills and documentation

Skills supply task-specific operating instructions. `/skills` shows bundled,
repository, user, and plugin-provided skills. `/docs` opens the packaged
documentation so users and agents can inspect current product behavior without
leaving the TUI.

See [Skills](../skills.md) and the
[Development Mandate](../development-mandate.md).
