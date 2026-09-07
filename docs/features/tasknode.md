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

## Keep the tab and agent on the same account

In the accepted 0.1.40 candidate, agent Task Node commands inherit the active
tab's profile and home. Missing or conflicting scope stops the command before
it opens account credentials. Restart older terminal processes after upgrading
so their agent commands receive the repaired scope.

The Task Node menu shows the linked account. Direct human CLI commands can select
a named profile with `corbanu tasknode --profile <name> status --json`.

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

## Recover a request

The reliability changes in the current candidate save a request before sending it.
If the connection fails, reopen `/tasknode request`: the original text returns.
Submitting that text again recovers its receipt. Closing the menu or restarting
Corbanu preserves the pending request. Editing the text creates a distinct request.
These changes await candidate deployment; older installed builds may differ.

The JSON helper provides the same recovery path:

```text
corbanu tasknode requests pending --json
corbanu tasknode requests recover <saved-key> --json
corbanu tasknode requests list --cursor <next-cursor> --json
corbanu tasknode requests retry <request-id> --attempt <worker-attempt-count> --json
```

Use `--profile <name>` for the intended Corbanu profile. Saved requests and tokens
stay with that profile, Task Node account and server. Changing the server requires
linking it separately. The TUI request list also offers older pages and retry for
failed requests; background refreshes preserve the selected row and search text.
Chat supports large streamed replies and reports an interrupted stream as an error.

## Track a workspace

Product specification: **Campaign Tracker — LOCAL PILOT CANDIDATE**, “preserve attributable user prompts and compact GLM 5.3 Flash summaries in a bounded database”. The September 7 debug rollout connects this feature to the live Task Node backend.

Open `/tasknode`, choose **Campaign Tracker**, and enable the workspace. The **Tracker REC** indicator shows recording and pending uploads. **My activity** opens saved prompts, compact output summaries and observed actions; choose an action for session replay or **Read full action** for a scrollable view. **Pause this workspace** stops new capture.

Recording belongs to the linked Task Node account. Sharing requires an explicit Campaign Tracker grant in addition to an accepted collaborator or direct-report relationship. A manager relationship alone does not expose another person's prompts. Access audit shows reads and grant changes.

This records observed Corbanu TUI work. Runtime measures agent execution, and summaries are not proof of task completion. The Verglas pilot is enabled for `@secondfmaster`; an already running older terminal needs to restart into the updated debug build.

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

Logout also saves a durable local unlink marker. A matching legacy machine-wide
session will not be re-imported after a restart or a cancelled relink. Linking
explicitly and completing validation installs a new session. If credential
cleanup fails, the error is reported and the unlink marker keeps any residual
credential unusable by this release.

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

The September 5 reliability candidate is installed for the production Kimi
operator. The public release channel is unchanged. Its request recovery and
control-inbox checks passed in a real terminal; the scoped supervisor is live.
