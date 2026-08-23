# Telegram remote control

## The pain

A long-running terminal agent is much less useful if the user must remain at
the same keyboard to check status, answer a bounded question, or send the next
instruction. Corbanu Terminal provides an allowlisted Telegram connector while
keeping local workspace and approval boundaries visible.

## Product contract

| Field | Value |
| --- | --- |
| Status | **LIVE** |
| Exact product-spec heading | **Shipping MVP — LIVE** |
| Requirement excerpt | “Remote and context: Allowlisted Telegram; durable `/goal` and `/memories`; ephemeral `/side` and `/btw`; `/skills` and `/docs`.” |

## Connect Telegram

Run:

```text
/telegram
```

The guided menu configures the bot credential through masked entry and records
the allowed user and chat policy.

Repeatable controls are:

```text
/telegram status
/telegram connect
/telegram start
/telegram stop
/telegram disconnect
```

- **Status** shows connector identity, authorization, and running state.
- **Start** begins the configured connector.
- **Stop** stops polling without deleting authorization.
- **Disconnect** removes connector authorization after confirmation.

## Remote workspace behavior

Telegram-created turns use the configured `default_cwd`. Set it to the
workspace the agent should operate in, not a broad home directory or the
Corbanu Terminal source tree by default. The connector loads that workspace's
`AGENTS.md`.

Supported images and documents are stored in the connector-owned media area and
passed to the agent by local path and metadata. Archives, executables, and
unrelated binary files are rejected.

## Authorization boundary

- Only configured users and chats may control the connector.
- The bot token must enter through the masked setup flow or protected
  environment file, never through chat.
- Remote input is untrusted and does not bypass local approval or sandbox
  policy.
- Run one poller per bot token.
- Use `/telegram status` before deleting state or replacing credentials.
