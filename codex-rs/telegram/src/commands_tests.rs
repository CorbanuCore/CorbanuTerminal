use pretty_assertions::assert_eq;

use codex_telegram::commands::Command;
use codex_telegram::commands::IncomingCommand;
use codex_telegram::commands::parse_incoming;

#[test]
fn known_commands_parse() {
    assert_eq!(
        parse_incoming("/new", Some("pfterminal_bot")),
        IncomingCommand::Known(Command::New)
    );
    assert_eq!(
        parse_incoming("/cancel@pfterminal_bot", Some("pfterminal_bot")),
        IncomingCommand::Known(Command::Cancel)
    );
    assert_eq!(
        parse_incoming("/status now", Some("pfterminal_bot")),
        IncomingCommand::Known(Command::Status)
    );
}

#[test]
fn matching_bot_mentions_are_stripped() {
    assert_eq!(
        parse_incoming("/new@pfterminal_bot", Some("pfterminal_bot")),
        IncomingCommand::Known(Command::New)
    );
    assert_eq!(
        parse_incoming("/new@PFTerminal_Bot", Some("pfterminal_bot")),
        IncomingCommand::Known(Command::New)
    );
}

#[test]
fn other_bot_mentions_are_not_treated_as_commands() {
    assert_eq!(
        parse_incoming("/new@other_bot", Some("pfterminal_bot")),
        IncomingCommand::AgentInput("/new@other_bot".to_string())
    );
}

#[test]
fn tasknode_and_unknown_slash_commands_pass_to_agent() {
    assert_eq!(
        parse_incoming("/tasknode submit wallet", Some("pfterminal_bot")),
        IncomingCommand::AgentInput("/tasknode submit wallet".to_string())
    );
    assert_eq!(
        parse_incoming("/custom thing", Some("pfterminal_bot")),
        IncomingCommand::AgentInput("/custom thing".to_string())
    );
}

#[test]
fn regular_text_passes_to_agent_unchanged() {
    assert_eq!(
        parse_incoming("  explain the repo\n", Some("pfterminal_bot")),
        IncomingCommand::AgentInput("  explain the repo\n".to_string())
    );
    assert_eq!(
        parse_incoming("zoz@example.com sent this", Some("pfterminal_bot")),
        IncomingCommand::AgentInput("zoz@example.com sent this".to_string())
    );
}
