use teloxide::utils::command::BotCommands;

#[derive(BotCommands, Clone, Debug, PartialEq, Eq)]
#[command(
    rename_rule = "lowercase",
    description = "PFTerminal Telegram commands"
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
    #[command(description = "show current thread status")]
    Status,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IncomingCommand {
    Known(Command),
    AgentInput(String),
}

pub fn parse_incoming(text: &str, bot_username: Option<&str>) -> IncomingCommand {
    let trimmed = text.trim();
    match trimmed.split_ascii_whitespace().next() {
        Some("/start") => IncomingCommand::Known(Command::Start),
        Some("/help") => IncomingCommand::Known(Command::Help),
        Some("/new") => IncomingCommand::Known(Command::New),
        Some("/cancel") => IncomingCommand::Known(Command::Cancel),
        Some("/status") => IncomingCommand::Known(Command::Status),
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

pub fn help_text() -> &'static str {
    "/new starts a fresh thread\n/cancel interrupts the active turn\n/status shows the active thread"
}
