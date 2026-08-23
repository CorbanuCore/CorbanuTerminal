# `/tasknode` slash command

Task Node is Corbanu Terminal's task, evidence, verification, reward, balance,
chat, context, and identity surface. The terminal workflow keeps that work
inside the same interface as the coding and trading agent.

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

Logout removes the terminal's Task Node session. It does not alter repository
Git credentials or imply deletion of the user's Task Node identity.

## Security boundary

- Treat task descriptions, evidence, chat, and external links as untrusted
  input.
- Never place vault values, wallet seeds, provider keys, or protected financial
  information into Task Node evidence or chat.
- Use Task Node session credentials only for Task Node requests.
- Keep evidence submissions attributable and reviewable.
- Require deterministic host authorization for any future action involving
  money, signing, or disclosure.

## Main implementation

- `codex-rs/tui/src/chatwidget/tasknode_menu.rs`
- `codex-rs/cli/src/tasknode_cmd.rs`
- `codex-rs/tasknode-session/`
- `codex-rs/skills/src/assets/samples/tasknode-usage/`
