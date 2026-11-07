use strum::IntoEnumIterator;
use strum_macros::AsRefStr;
use strum_macros::EnumIter;
use strum_macros::EnumString;
use strum_macros::IntoStaticStr;

/// The forms `/cost` accepts; the full rules are in its usage message.
pub(crate) const COST_ARGUMENT_HINT: &str =
    "[YYYY-MM-DD]  or  START END hour|day|week|month  (UTC)";

/// Commands that can be invoked by starting a message with a leading slash.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, EnumString, EnumIter, AsRefStr, IntoStaticStr,
)]
#[strum(serialize_all = "kebab-case")]
pub enum SlashCommand {
    // DO NOT ALPHA-SORT! Enum order is presentation order in the popup, so
    // more frequently used commands should be listed first.
    Model,
<<<<<<< HEAD
    Gpu,
||||||| 413492cd6c
=======
    Presets,
    Daybreak,
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
    Ide,
    Permissions,
    Security,
    Keymap,
    Vim,
    #[strum(serialize = "setup-default-sandbox")]
    ElevateSandbox,
    Experimental,
    #[strum(to_string = "approve")]
    AutoReview,
    Memories,
    Skills,
    Import,
    Hooks,
    Review,
    Rename,
    New,
    Archive,
    Delete,
    Resume,
    Fork,
    Worktree,
    App,
    Init,
    Compact,
    Config,
    Recap,
    Plan,
    Voice,
    Goal,
<<<<<<< HEAD
    Agent,
    Spawn,
    Orchestrate,
    Tasknode,
    Panes,
||||||| 413492cd6c
    Agent,
=======
    Agents,
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
    Side,
    Btw,
    Copy,
    Export,
    Raw,
    Tui,
    Diff,
    Docs,
    Mention,
    Status,
    Daemon,
    Warnings,
    Cd,
    #[strum(to_string = "pwd", serialize = "cwd")]
    Pwd,
    Usage,
    Cost,
    DebugConfig,
    Title,
    Statusline,
    Theme,
    #[strum(to_string = "pets", serialize = "pet")]
    Pets,
    Mcp,
    Apps,
    Plugins,
    Providers,
    Telegram,
    Logout,
    Wallet,
    Vault,
    Quit,
    Exit,
    Feedback,
    Rollout,
    Ps,
    #[strum(to_string = "stop", serialize = "clean")]
    Stop,
    Clear,
    TestApproval,
    #[strum(serialize = "subagents")]
    MultiAgents,
    // Debugging commands.
    #[strum(serialize = "debug-m-drop")]
    MemoryDrop,
    #[strum(serialize = "debug-m-update")]
    MemoryUpdate,
}

impl SlashCommand {
    /// User-visible description shown in the popup.
    pub fn description(self) -> &'static str {
        match self {
            SlashCommand::Config => "configure Codex preferences",
            SlashCommand::Feedback => "send logs to maintainers",
            SlashCommand::New => "start a new chat during a conversation",
            SlashCommand::Init => "create an AGENTS.md file with instructions for Corbanu Terminal",
            SlashCommand::Compact => "summarize conversation to prevent hitting the context limit",
            SlashCommand::Recap => "summarize the current conversation now",
            SlashCommand::Review => "review my current changes and find issues",
            SlashCommand::Rename => "rename the current thread",
            SlashCommand::Resume => "resume a saved chat",
            SlashCommand::Archive => "archive this session",
            SlashCommand::Delete => "permanently delete this session",
            SlashCommand::Clear => "clear the terminal and start a new chat",
            SlashCommand::Fork => "fork the current chat",
<<<<<<< HEAD
            SlashCommand::App => "continue this session in Corbanu Terminal Desktop",
            SlashCommand::Quit | SlashCommand::Exit => "exit Corbanu Terminal",
            SlashCommand::Copy => "copy last response as markdown",
||||||| 413492cd6c
            SlashCommand::App => "continue this session in the Desktop app",
            SlashCommand::Quit | SlashCommand::Exit => "exit Codex",
            SlashCommand::Copy => "copy last response as markdown",
=======
            SlashCommand::Worktree => "start or continue a conversation in a new worktree",
            SlashCommand::App => "continue this session in the Desktop app",
            SlashCommand::Quit | SlashCommand::Exit => "exit Codex",
            SlashCommand::Copy => "copy the last response or part of it",
            SlashCommand::Export => "export the conversation as markdown",
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
            SlashCommand::Raw => "toggle raw scrollback mode for copy-friendly terminal selection",
            SlashCommand::Tui => "choose the TUI mode for the next launch",
            SlashCommand::Diff => "show git diff (including untracked files)",
            SlashCommand::Docs => "open MkDocs documentation in the terminal",
            SlashCommand::Mention => "mention a file",
            SlashCommand::Skills => {
                "use skills to improve how Corbanu Terminal performs specific tasks"
            }
            SlashCommand::Import => "import setup, this project, and recent chats from Claude Code",
            SlashCommand::Hooks => "view and manage lifecycle hooks",
            SlashCommand::Daemon => "Manage the local background server",
            SlashCommand::Warnings => "view retained warnings and diagnostic details",
            SlashCommand::Status => "show current session configuration and token usage",
<<<<<<< HEAD
            SlashCommand::Usage if cfg!(feature = "developer-accounting") => {
                "view recorded request cost, account usage, or use a rate-limit reset"
            }
            SlashCommand::Usage => "view account usage or use a rate-limit reset",
            SlashCommand::Cost => "what your turns cost, on whichever provider served them",
||||||| 413492cd6c
            SlashCommand::Usage => "view account usage or use a usage limit reset",
=======
            SlashCommand::Cd => "change the current working directory",
            SlashCommand::Pwd => "show the current working directory",
            SlashCommand::Usage => "view account usage or use a usage limit reset",
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
            SlashCommand::DebugConfig => "show config layers and requirement sources for debugging",
            SlashCommand::Title => "configure which items appear in the terminal title",
            SlashCommand::Statusline => "configure which items appear in the status line",
            SlashCommand::Theme => "choose a syntax highlighting theme",
            SlashCommand::Pets => "choose or hide the terminal pet",
            SlashCommand::Ps => "list background terminals",
            SlashCommand::Stop => "stop all background terminals",
            SlashCommand::MemoryDrop => "DO NOT USE",
            SlashCommand::MemoryUpdate => "DO NOT USE",
            SlashCommand::Model => "choose what model and reasoning effort to use",
<<<<<<< HEAD
            SlashCommand::Gpu => "rent, inspect, or terminate GPU capacity",
||||||| 413492cd6c
=======
            SlashCommand::Presets => "run or save reusable prompts and slash commands",
            SlashCommand::Daybreak => "turn Daybreak on or off",
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
            SlashCommand::Ide => {
                "include current selection, open files, and other context from your IDE"
            }
<<<<<<< HEAD
            SlashCommand::Personality => "choose a communication style for Corbanu Terminal",
||||||| 413492cd6c
            SlashCommand::Personality => "choose a communication style for Codex",
=======
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
            SlashCommand::Plan => "switch to Plan mode",
            SlashCommand::Voice => "start or stop voice; use /voice settings to choose a voice",
            SlashCommand::Goal => "set or view the goal for a long-running task",
<<<<<<< HEAD
            SlashCommand::Agent | SlashCommand::MultiAgents => "switch the active agent thread",
            SlashCommand::Spawn => "orchestrate Nazgul, Troll, and Orc agent roles",
            SlashCommand::Orchestrate => "attach and manage native pane whips",
            SlashCommand::Tasknode => "interact with Task Node tasks and rewards",
            SlashCommand::Panes => "switch Corbanu Terminal, Claude Code, or agent panes",
||||||| 413492cd6c
            SlashCommand::Agent | SlashCommand::MultiAgents => "switch the active agent thread",
=======
            SlashCommand::Agents => "open the agent command center",
            SlashCommand::MultiAgents => "switch between this session's subagents",
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
            SlashCommand::Side | SlashCommand::Btw => {
                "start a side conversation in an ephemeral fork"
            }
            SlashCommand::Permissions => "choose what Corbanu Terminal is allowed to do",
            SlashCommand::Security => security_description(crate::security::level::context()),
            SlashCommand::Keymap => "remap TUI shortcuts",
            SlashCommand::Vim => "toggle Vim mode for the composer",
            SlashCommand::ElevateSandbox => "set up elevated agent sandbox",
            SlashCommand::Experimental => "toggle experimental features",
            SlashCommand::AutoReview => "approve one retry of a recent auto-review denial",
            SlashCommand::Memories => "configure memory use and generation",
            SlashCommand::Mcp => "list MCP tools; use /mcp verbose or /mcp login <name>",
            SlashCommand::Apps => "manage apps",
            SlashCommand::Plugins => "browse plugins",
            SlashCommand::Providers => "configure, activate, or deactivate providers",
            SlashCommand::Telegram => "connect and manage Corbanu Terminal from Telegram",
            SlashCommand::Vault => "manage the encrypted credential vault (keys, tokens)",
            SlashCommand::Wallet => "manage SOL, USDC, and Corbanu API access",
            SlashCommand::Logout => "log out of Corbanu Terminal",
            SlashCommand::Rollout => "print the rollout file path",
            SlashCommand::TestApproval => "test approval request",
        }
    }

    /// Command string without the leading '/'. Provided for compatibility with
    /// existing code that expects a method named `command()`.
    pub fn command(self) -> &'static str {
        self.into()
    }

    /// Whether this command supports inline args (for example `/review ...`).
    pub fn supports_inline_args(self) -> bool {
        matches!(
            self,
            SlashCommand::Review
                | SlashCommand::Model
                | SlashCommand::Permissions
                | SlashCommand::Title
                | SlashCommand::Statusline
                | SlashCommand::Rename
                | SlashCommand::New
                | SlashCommand::Clear
                | SlashCommand::Fork
                | SlashCommand::Plan
                | SlashCommand::Goal
                | SlashCommand::Voice
                | SlashCommand::Ide
                | SlashCommand::Keymap
                | SlashCommand::Mcp
                | SlashCommand::Export
                | SlashCommand::Raw
<<<<<<< HEAD
                | SlashCommand::Docs
||||||| 413492cd6c
=======
                | SlashCommand::Cd
                | SlashCommand::Pwd
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
                | SlashCommand::Usage
                | SlashCommand::Cost
                | SlashCommand::Pets
                | SlashCommand::Side
                | SlashCommand::Btw
                | SlashCommand::Resume
<<<<<<< HEAD
                | SlashCommand::SandboxReadRoot
                | SlashCommand::Spawn
                | SlashCommand::Orchestrate
                | SlashCommand::Tasknode
                | SlashCommand::Vault
                | SlashCommand::Wallet
                | SlashCommand::Telegram
                | SlashCommand::Gpu
||||||| 413492cd6c
                | SlashCommand::SandboxReadRoot
=======
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        )
    }

    /// A short, dimmed hint of the accepted arguments, shown in the composer
    /// after the command and a space until the user types an argument.
    pub fn argument_hint(self) -> Option<&'static str> {
        match self {
            SlashCommand::Cost => Some(COST_ARGUMENT_HINT),
            _ => None,
        }
    }

    /// Whether this command remains available inside an active side conversation.
    pub fn available_in_side_conversation(self) -> bool {
        matches!(
            self,
            SlashCommand::Copy
                | SlashCommand::Agents
                | SlashCommand::Export
                | SlashCommand::Raw
                | SlashCommand::Diff
                | SlashCommand::Docs
                | SlashCommand::Mention
                | SlashCommand::Status
                | SlashCommand::Daemon
                | SlashCommand::Warnings
                | SlashCommand::Pwd
                | SlashCommand::Usage
                | SlashCommand::Cost
                | SlashCommand::Ide
                | SlashCommand::Providers
                | SlashCommand::Telegram
                | SlashCommand::Tasknode
                | SlashCommand::Orchestrate
                | SlashCommand::Vault
                | SlashCommand::Wallet
                | SlashCommand::Gpu
        )
    }

    /// Whether dispatch needs thread state to validate this command before consuming its draft.
    /// The composer must defer busy-state rejection and draft clearing for these commands.
    pub(crate) fn requires_dispatch_validation(self) -> bool {
        matches!(self, SlashCommand::Review)
    }

    /// Commands that do not require a writable current thread. The server must still be connected.
    pub(crate) fn available_when_thread_unavailable(self) -> bool {
        matches!(
            self,
            SlashCommand::New
                | SlashCommand::Clear
                | SlashCommand::Resume
                | SlashCommand::Agents
                | SlashCommand::MultiAgents
                | SlashCommand::Quit
                | SlashCommand::Exit
                | SlashCommand::Status
                | SlashCommand::Warnings
                | SlashCommand::DebugConfig
                | SlashCommand::Pwd
                | SlashCommand::Rollout
                | SlashCommand::Copy
                | SlashCommand::Raw
        )
    }

    /// Whether this command can be run while a task is in progress.
    pub fn available_during_task(self) -> bool {
        match self {
            SlashCommand::New
<<<<<<< HEAD
||||||| 413492cd6c
            | SlashCommand::Archive
            | SlashCommand::Delete
=======
            | SlashCommand::Delete
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
            | SlashCommand::Fork
            | SlashCommand::Worktree
            | SlashCommand::Init
            | SlashCommand::Compact
            | SlashCommand::Recap
            | SlashCommand::Export
            | SlashCommand::Keymap
            | SlashCommand::Tui
            | SlashCommand::Vim
            | SlashCommand::ElevateSandbox
            | SlashCommand::Experimental
            | SlashCommand::Memories
            | SlashCommand::Import
            | SlashCommand::Review
            | SlashCommand::Plan
            | SlashCommand::Cd
            | SlashCommand::Clear
            | SlashCommand::Logout
            | SlashCommand::MemoryDrop
            | SlashCommand::MemoryUpdate => false,
<<<<<<< HEAD
            SlashCommand::Cost
            | SlashCommand::Diff
            | SlashCommand::Archive
            | SlashCommand::Delete
            | SlashCommand::Docs
||||||| 413492cd6c
            SlashCommand::Diff
=======
            SlashCommand::Diff
            | SlashCommand::Archive
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
            | SlashCommand::Resume
            | SlashCommand::Model
<<<<<<< HEAD
            | SlashCommand::Gpu
            | SlashCommand::Personality
||||||| 413492cd6c
            | SlashCommand::Personality
=======
            | SlashCommand::Presets
            | SlashCommand::Daybreak
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
            | SlashCommand::Permissions
            | SlashCommand::Security
            | SlashCommand::Copy
            | SlashCommand::Raw
            | SlashCommand::Rename
            | SlashCommand::Mention
            | SlashCommand::Skills
            | SlashCommand::Hooks
            | SlashCommand::Status
            | SlashCommand::Daemon
            | SlashCommand::Warnings
            | SlashCommand::Pwd
            | SlashCommand::Usage
            | SlashCommand::DebugConfig
            | SlashCommand::Ps
            | SlashCommand::Stop
            | SlashCommand::App
            | SlashCommand::Goal
            | SlashCommand::Voice
            | SlashCommand::Mcp
            | SlashCommand::Apps
            | SlashCommand::Plugins
            | SlashCommand::Providers
            | SlashCommand::Telegram
            | SlashCommand::Panes
            | SlashCommand::Spawn
            | SlashCommand::Orchestrate
            | SlashCommand::Tasknode
            | SlashCommand::Title
            | SlashCommand::Statusline
            | SlashCommand::Vault
            | SlashCommand::Wallet
            | SlashCommand::AutoReview
            | SlashCommand::Feedback
            | SlashCommand::Ide
            | SlashCommand::Quit
            | SlashCommand::Exit
            | SlashCommand::Side
            | SlashCommand::Btw => true,
            SlashCommand::Rollout => true,
            SlashCommand::TestApproval => true,
            SlashCommand::Agents | SlashCommand::MultiAgents => true,
            SlashCommand::Config | SlashCommand::Theme | SlashCommand::Pets => false,
        }
    }

    fn is_visible(self) -> bool {
        match self {
            SlashCommand::Copy => !cfg!(target_os = "android"),
            SlashCommand::App => cfg!(any(target_os = "macos", target_os = "windows")),
            SlashCommand::Voice => true,
            SlashCommand::Rollout | SlashCommand::TestApproval => cfg!(debug_assertions),
            // The recorded-request view is held: collection cannot exist in a
            // shipped binary, and advertising the view there was never
            // authorised. It is gated on the same build that can collect: in
            // any other build typing it is an unrecognized command, and
            // `/usage requests` says the history is not part of the build.
            SlashCommand::Cost => cfg!(feature = "developer-accounting"),
            _ => true,
        }
    }
}

/// Return all built-in commands in a Vec paired with their command string.
pub fn built_in_slash_commands() -> Vec<(&'static str, SlashCommand)> {
    SlashCommand::iter()
        .filter(|command| command.is_visible())
        .map(|c| (c.command(), c))
        .collect()
}

/// `/security` opens the level picker when the `security_levels` flag is on
/// (or a non-Permissive level is stored); otherwise it is the read-only view.
fn security_description(context: Option<&crate::security::level::LevelContext>) -> &'static str {
    if context.is_some_and(|context| context.picker_enabled) {
        "choose a security level; takes effect when you restart"
    } else {
        "explore security profiles and protection readiness (read only)"
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use std::str::FromStr;

    use super::SlashCommand;
    use super::security_description;
    use crate::security::level::ChosenLevel;
    use crate::security::level::LevelContext;

    #[test]
    fn stop_command_is_canonical_name() {
        assert_eq!(SlashCommand::Stop.command(), "stop");
    }

    #[test]
    fn clean_alias_parses_to_stop_command() {
        assert_eq!(SlashCommand::from_str("clean"), Ok(SlashCommand::Stop));
    }

    #[test]
    fn pet_alias_parses_to_pets_command() {
        assert_eq!(SlashCommand::Pets.command(), "pets");
        assert_eq!(SlashCommand::from_str("pet"), Ok(SlashCommand::Pets));
    }

    #[test]
    fn certain_commands_are_available_during_task() {
        assert!(SlashCommand::Goal.available_during_task());
        assert!(SlashCommand::Ide.available_during_task());
        assert!(SlashCommand::Title.available_during_task());
        assert!(SlashCommand::Statusline.available_during_task());
        assert!(SlashCommand::Raw.available_during_task());
        assert!(SlashCommand::Raw.available_in_side_conversation());
        assert!(SlashCommand::Raw.supports_inline_args());
        assert!(SlashCommand::App.available_during_task());
        assert!(SlashCommand::Telegram.available_during_task());
        assert!(SlashCommand::Telegram.available_in_side_conversation());
        assert!(SlashCommand::Telegram.supports_inline_args());
    }

    #[test]
    fn auto_review_command_is_approve() {
        assert_eq!(SlashCommand::AutoReview.command(), "approve");
        assert_eq!(
            SlashCommand::from_str("approve"),
            Ok(SlashCommand::AutoReview)
        );
    }

    /// The description follows what `/security` opens: the picker when the
    /// launch context enables it, otherwise the read-only view.
    #[test]
    fn security_description_matches_what_the_command_opens() {
        let context = |picker_enabled| LevelContext {
            codex_home: std::path::PathBuf::from("/home"),
            picker_enabled,
            active: ChosenLevel::Permissive,
            preflight_enabled: false,
            boundary: None,
        };
        let read_only = "explore security profiles and protection readiness (read only)";
        assert_eq!(
            [
                security_description(Some(&context(true))),
                security_description(Some(&context(false))),
                security_description(/*context*/ None),
            ],
            [
                "choose a security level; takes effect when you restart",
                read_only,
                read_only,
            ]
        );
    }
}
