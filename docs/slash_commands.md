# Slash commands

Corbanu Terminal retains the Codex-derived local command surface and adds
product-specific wallet, vault, provider, GPU, pane, spawn, Telegram, and Task
Node workflows. The table below is the release-facing PF inventory, not an
exhaustive replacement for the in-app command picker.

## Corbanu Terminal Commands

| Command                    | Purpose                                                       |
| -------------------------- | ------------------------------------------------------------- |
| `/model`                   | Select model/provider and effort mode                         |
| `/vault`                   | Open the encrypted credential vault action menu               |
| `/vault list`              | List credential labels and metadata without revealing secrets |
| `/vault show <label>`      | Inspect one credential's metadata                             |
| `/vault credential add`    | Add a credential through the masked secure-entry flow         |
| `/providers`               | Manage provider credentials and OpenAI Codex account login    |
| `/wallet`                  | Open wallet, balance, backup/restore, and plan workflows      |
| `/wallet status`           | Show wallet lock, identity, balance, and plan state           |
| `/gpu`                     | Browse qualified/experimental GPU routes with charge review   |
| `/gpu status`              | Reconcile local rental and endpoint state                     |
| `/gpu stop <id>`           | Stop serving for a supported rental without implying deletion |
| `/gpu terminate <id>`      | Terminate the selected provider rental after confirmation     |
| `/panes`                   | Switch user panes and create Claude Code headless panes       |
| `/spawn`                   | Open managed Nazgul/Troll/Orc orchestration                   |
| `/spawn status`            | Show the current spawn hierarchy and worker status            |
| `/spawn nazgul`            | Bind an existing user pane as the Nazgul root                 |
| `/spawn troll`             | Create a Troll under an existing parent                       |
| `/spawn orc`               | Create an Orc under an existing parent                        |
| `/tasknode`                | Open the Task Node terminal menu                              |
| `/tasknode link`           | Link a GitHub-backed Task Node account                        |
| `/tasknode status`         | Show Task Node account/session status                         |
| `/tasknode tasks [tab]`    | Show Task Node tasks; defaults to outstanding                 |
| `/tasknode outstanding`    | Show outstanding Task Node tasks                              |
| `/tasknode verification`   | Show tasks waiting on verification                            |
| `/tasknode refused`        | Show refused tasks                                            |
| `/tasknode rewarded`       | Show rewarded tasks                                           |
| `/tasknode task <id>`      | Open actions for one Task Node task                           |
| `/tasknode request`        | Open the Task Node request prompt                             |
| `/tasknode request <text>` | Submit a Task Node request from inline text                   |
| `/tasknode context`        | Open Task Node context                                        |
| `/tasknode chat`           | Open Task Node chat                                           |
| `/tasknode chat <text>`    | Submit a new Task Node chat message                           |
| `/tasknode requests`       | Show active Task Node requests                                |
| `/tasknode balance`        | Show read-only Task Node balance                              |
| `/tasknode rewards`        | Show recent Task Node rewards                                 |
| `/tasknode logout`         | Log out of the Task Node terminal session                     |
| `/telegram`                | Open Telegram connector management                            |
| `/telegram status`         | Show connector identity, authorization, and running state     |
| `/telegram connect`        | Securely configure bot and allowed-user/chat policy           |
| `/telegram start`          | Start the configured connector                                |
| `/telegram stop`           | Stop the configured connector                                 |
| `/telegram disconnect`     | Remove connector authorization after confirmation             |
| `/docs [page]`             | Open packaged documentation in the terminal                   |
| `/goal [objective]`        | Create or inspect a durable long-running goal                 |
| `/memories`                | Configure durable memory use and generation                   |
| `/side [prompt]`           | Run an ephemeral side conversation and return                 |
| `/btw [prompt]`            | Alias for an ephemeral side conversation                      |
| `/skills`                  | Browse bundled, repo, user, and plugin skills                 |

For inherited Codex CLI slash commands, see:

<https://developers.openai.com/codex/cli/slash-commands>
