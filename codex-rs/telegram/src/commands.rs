use teloxide::utils::command::BotCommands;

#[derive(BotCommands, Clone, Debug, PartialEq, Eq)]
#[command(
    rename_rule = "lowercase",
    description = "Corbanu Terminal Telegram commands"
)]
pub enum Command {
    #[command(description = "start the connector")]
    Start,
    #[command(description = "show help")]
    Help,
    #[command(description = "start a fresh thread")]
    New,
    #[command(description = "cancel the active turn")]
    Cancel,
    #[command(description = "cancel the active turn")]
    Stop,
    #[command(description = "show current thread status")]
    Status,
    #[command(description = "show or change the active model")]
    Model,
    #[command(description = "show or change approval policy")]
    Approvals,
    #[command(description = "compact the active thread")]
    Compact,
    #[command(description = "show git diff to remote")]
    Diff,
    #[command(description = "list discovered skills")]
    Skills,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IncomingCommand {
    Known { command: Command, args: String },
    AgentInput(String),
}

pub fn parse_incoming(text: &str, bot_username: Option<&str>) -> IncomingCommand {
    let trimmed = text.trim();
    match trimmed.split_ascii_whitespace().next() {
        Some("/start") => known(Command::Start, ""),
        Some("/help") => known(Command::Help, ""),
        Some("/new") => known(Command::New, ""),
        Some("/cancel") => known(Command::Cancel, ""),
        Some("/stop") => known(Command::Stop, ""),
        Some("/status") => known(Command::Status, args_after_command(trimmed)),
        Some("/model") => known(Command::Model, args_after_command(trimmed)),
        Some("/approvals") => known(Command::Approvals, args_after_command(trimmed)),
        Some("/compact") => known(Command::Compact, args_after_command(trimmed)),
        Some("/diff") => known(Command::Diff, args_after_command(trimmed)),
        Some("/skills") => known(Command::Skills, args_after_command(trimmed)),
        Some(command) if command.starts_with('/') && command.contains('@') => {
            let Some((without_bot, mentioned_bot)) = command.split_once('@') else {
                return IncomingCommand::AgentInput(trimmed.into());
            };
            let Some(bot_username) = bot_username else {
                return IncomingCommand::AgentInput(trimmed.into());
            };
            if !mentioned_bot.eq_ignore_ascii_case(bot_username.trim_start_matches('@')) {
                return IncomingCommand::AgentInput(trimmed.into());
            }
            let rest = trimmed.get(command.len()..).unwrap_or_default();
            parse_incoming(&format!("{without_bot}{rest}"), Some(bot_username))
        }
        Some(command) if command.starts_with('/') => IncomingCommand::AgentInput(trimmed.into()),
        _ => IncomingCommand::AgentInput(text.into()),
    }
}

fn known(command: Command, args: &str) -> IncomingCommand {
    IncomingCommand::Known {
        command,
        args: args.to_string(),
    }
}

fn args_after_command(trimmed: &str) -> &str {
    trimmed
        .split_once(char::is_whitespace)
        .map(|(_, args)| args.trim())
        .unwrap_or_default()
}

pub fn help_text() -> &'static str {
    "/new starts a fresh thread\n\
     /cancel or /stop interrupts the active turn\n\
     /status shows the active thread\n\
     /model [alias-or-slug] shows or changes the active model\n\
     /approvals [untrusted|on-failure|on-request|never] shows or changes approval policy\n\
     /compact compacts the active thread\n\
     /diff shows the git diff to the remote branch\n\
     /skills lists discovered skills"
}
