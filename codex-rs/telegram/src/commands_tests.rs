use pretty_assertions::assert_eq;

use codex_telegram::commands::Command;
use codex_telegram::commands::IncomingCommand;
use codex_telegram::commands::parse_incoming;

#[test]
fn known_commands_parse() {
    assert_eq!(parse_incoming("/new"), IncomingCommand::Known(Command::New));
    assert_eq!(
        parse_incoming("/cancel@pfterminal_bot"),
        IncomingCommand::Known(Command::Cancel)
    );
    assert_eq!(
        parse_incoming("/status now"),
        IncomingCommand::Known(Command::Status)
    );
}

#[test]
fn tasknode_and_unknown_slash_commands_pass_to_agent() {
    assert_eq!(
        parse_incoming("/tasknode submit wallet"),
        IncomingCommand::AgentInput("/tasknode submit wallet".to_string())
    );
    assert_eq!(
        parse_incoming("/custom thing"),
        IncomingCommand::AgentInput("/custom thing".to_string())
    );
}

#[test]
fn regular_text_passes_to_agent_unchanged() {
    assert_eq!(
        parse_incoming("  explain the repo\n"),
        IncomingCommand::AgentInput("  explain the repo\n".to_string())
    );
    assert_eq!(
        parse_incoming("zoz@example.com sent this"),
        IncomingCommand::AgentInput("zoz@example.com sent this".to_string())
    );
}
