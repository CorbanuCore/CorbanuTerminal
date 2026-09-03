# Task Node and identity

## The pain

Tasks, evidence, rewards, durable context, and identity lose value when they
live in a separate application that the active agent cannot inspect. Corbanu
Terminal makes Task Node a native terminal workflow while preserving its own
session and authorization boundary.

## Product contract

> **Product specification — “Shipping MVP — LIVE”**
>
> “Task Node and identity: Tasks, evidence, verification, rewards, balances,
> chat, context, linked identity, and live Task Node-linked Nostr identity.”

## Open Task Node

Run:

```text
/tasknode
```

The menu provides account linking, session status, task lists, task actions,
requests, context, chat, balances, rewards, and logout.

## Link an account

```text
/tasknode link
/tasknode status
```

The link flow connects the terminal session to the user's Task Node account.
Status shows the linked account and session state. Task Node-linked Nostr
identity is already live and remains the identity boundary for future social
features.

Task Node sessions are owned by the selected Corbanu profile. For example,
terminals launched with `-p goodalexander` and `-p secondfoundation` can link
different Task Node accounts on the same machine without sharing tokens,
pending link attempts, status, or logout state. The Task Node menu shows the
active Corbanu profile so the local profile and linked identity can be checked
together.

After upgrading from the older machine-wide session format, a named profile
reuses that session only when its name matches the linked GitHub username.
Other profiles remain unlinked and must link once; a mismatched identity is
never imported.

## Work with tasks

```text
/tasknode tasks [tab]
/tasknode outstanding
/tasknode verification
/tasknode refused
/tasknode rewarded
/tasknode task <task-id>
```

Task lists use stable status tabs. Opening a task shows its details and the
actions currently available for that state, including evidence guidance and
verification-related work.

## Request work and use context

```text
/tasknode request
/tasknode request <text>
/tasknode requests
/tasknode context
/tasknode chat
/tasknode chat <text>
```

The request flow accepts free-form work descriptions. The requests view shows
active request state. Context and chat use the linked Task Node account so an
agent can consult the user's durable context without inventing user-specific
preferences.

## Inspect rewards and balance

```text
/tasknode balance
/tasknode rewards
```

These views are read-only. They display Task Node balance and recent reward
activity without turning a model response into authorization for a financial
action.

## End the terminal session

```text
/tasknode logout
```

Logout removes only the selected Corbanu profile's Task Node session. It does
not log out another local profile, alter repository Git credentials, or imply
deletion of the user's Task Node identity.

## Security boundary

- Treat task descriptions, evidence, chat, and external links as untrusted
  input.
- Never place vault values, wallet seeds, provider keys, or protected financial
  information into Task Node evidence or chat.
- Use Task Node session credentials only for Task Node requests.
- Keep evidence submissions attributable and reviewable.
- Require deterministic host authorization for any action involving money,
  signing, or disclosure.

## Main implementation

- `codex-rs/tui/src/chatwidget/tasknode_menu.rs`
- `codex-rs/cli/src/tasknode_cmd.rs`
- `codex-rs/tasknode-session/`
- `codex-rs/skills/src/assets/samples/tasknode-usage/`
