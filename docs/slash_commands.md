# Slash commands

Corbanu Terminal keeps the inherited Codex command surface and adds native
provider, wallet, compute, orchestration, Task Node, remote-control, and context
workflows. Use the command picker in the TUI for the complete active list.

## Session and inference

| Command        | Purpose                                                         |
| -------------- | --------------------------------------------------------------- |
| `/model`       | Choose provider, model, and reasoning effort                    |
| `/providers`   | Manage account sign-ins and provider credentials                |
| `/status`      | Show session configuration, model, permissions, and token state |
| `/usage`       | Show account or Plan allowance and reset information            |
| `/permissions` | Choose what the active session may do                           |
| `/review`      | Review current workspace changes                                |
| `/diff`        | Show the workspace diff, including untracked files              |

## Credentials, wallet, and compute

| Command                 | Purpose                                                           |
| ----------------------- | ----------------------------------------------------------------- |
| `/vault`                | Open encrypted credential actions                                 |
| `/vault list`           | List credential labels and metadata without values                |
| `/vault show <label>`   | Inspect metadata for one credential                               |
| `/vault credential add` | Open masked credential entry                                      |
| `/wallet`               | Manage wallet custody, balances, backup/restore, and Corbanu API  |
| `/wallet status`        | Show wallet lock, address, balance, and Plan state                |
| `/gpu`                  | Rent, inspect, stop, or terminate Vast.ai and RunPod capacity     |
| `/gpu status`           | Reconcile rental, endpoint, readiness, and spend state            |
| `/gpu stop <id>`        | Stop serving without claiming provider billing stopped            |
| `/gpu terminate <id>`   | Request provider termination after confirmation                   |

## Workspaces and agents

| Command                                                                          | Purpose                                                                |
| -------------------------------------------------------------------------------- | ---------------------------------------------------------------------- |
| [`/panes`](features/workspaces.md)                                               | Create or switch Corbanu Terminal, Claude Code, and managed crew panes |
| `/agent` or `/subagents`                                                         | Inspect and switch native agent threads                                |
| `/spawn`                                                                         | Create or bind Nazgul, Troll, and Orc roles                            |
| `/spawn status`                                                                  | Show hierarchy and worker state                                        |
| `/spawn nazgul`                                                                  | Bind an existing pane as the hierarchy root                            |
| `/spawn troll`                                                                   | Create a Troll under a valid parent                                    |
| `/spawn orc`                                                                     | Create an Orc under a valid parent                                     |
| [`/orchestrate`](features/orchestrate.md)                                        | Attach and manage persistent Manager → Worker supervision              |
| [`/orchestrate status`](features/orchestrate.md#inspect-and-control-assignments) | Show and control active supervisory assignments                        |
| `/ps`                                                                            | List background terminals                                              |
| `/stop`                                                                          | Stop all background terminals                                          |

## Task Node and identity

| Command                    | Purpose                                 |
| -------------------------- | --------------------------------------- |
| `/tasknode`                | Open the Task Node menu                 |
| `/tasknode link`           | Link a Task Node account                |
| `/tasknode status`         | Show terminal session and account state |
| `/tasknode tasks [tab]`    | Show tasks, defaulting to outstanding   |
| `/tasknode task <id>`      | Open one task and its available actions |
| `/tasknode request [text]` | Create a Task Node request              |
| `/tasknode requests`       | Show active requests                    |
| `/tasknode context`        | Open durable Task Node context          |
| `/tasknode chat [text]`    | Open or send Task Node chat             |
| `/tasknode verification`   | Show tasks waiting on verification      |
| `/tasknode refused`        | Show refused tasks                      |
| `/tasknode rewarded`       | Show rewarded tasks                     |
| `/tasknode balance`        | Show read-only balance                  |
| `/tasknode rewards`        | Show recent rewards                     |
| `/tasknode logout`         | Remove the terminal Task Node session   |

## Remote control and context

| Command                | Purpose                                            |
| ---------------------- | -------------------------------------------------- |
| `/telegram`            | Manage the allowlisted Telegram connector          |
| `/telegram status`     | Show connector identity, authorization, and state  |
| `/telegram connect`    | Configure token and allowlist through secure entry |
| `/telegram start`      | Start the configured connector                     |
| `/telegram stop`       | Stop polling without deleting authorization        |
| `/telegram disconnect` | Remove connector authorization after confirmation  |
| `/goal [objective]`    | Create or inspect a durable long-running goal      |
| `/memories`            | Configure durable memory use and generation        |
| `/side [prompt]`       | Run an ephemeral side conversation                 |
| `/btw [prompt]`        | Alias for the ephemeral side workflow              |
| `/docs [page]`         | Browse packaged documentation in the terminal      |
| `/skills`              | Browse available skills                            |

## Extensions

| Command    | Purpose                                        |
| ---------- | ---------------------------------------------- |
| `/mcp`     | Inspect configured MCP tools                   |
| `/apps`    | Manage connected app surfaces                  |
| `/plugins` | Browse installed plugins                       |
| `/hooks`   | Inspect and manage lifecycle hooks             |
| `/ide`     | Include editor selection and open-file context |

For inherited Codex commands not listed here, use the in-product command picker
or see the [upstream slash-command reference](https://developers.openai.com/codex/cli/slash-commands).
