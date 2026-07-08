use anyhow::Context;
use teloxide::dispatching::UpdateHandler;
use teloxide::dptree;
use teloxide::prelude::Requester;
use teloxide::prelude::*;
use teloxide::types::ParseMode;
use teloxide::utils::command::BotCommands;
use tracing::info;
use tracing::instrument;
use tracing::warn;

use crate::approvals::ApprovalCallback;
use crate::auth::ChatAllowlist;
use crate::bridge::BridgeHandle;
use crate::commands::Command;
use crate::commands::IncomingCommand;
use crate::commands::help_text;
use crate::commands::parse_incoming;

type HandlerResult = anyhow::Result<()>;
const CALLBACK_ANSWER_TEXT_LIMIT: usize = 200;

/// What an inbound Telegram message contributes as turn input.
///
/// The connector is text-only inbound today. A plain text message is used as-is.
/// Media (photo/document) with a caption contributes the caption — but the
/// attachment itself cannot be read yet, so we prefix a note so the agent does
/// not answer as if it saw the image. Media with no caption is unsupported.
#[derive(Debug, PartialEq, Eq)]
enum MessageInput {
    Text(String),
    Unsupported,
}

fn resolve_message_input(text: Option<&str>, caption: Option<&str>) -> MessageInput {
    if let Some(text) = text {
        return MessageInput::Text(text.to_string());
    }
    if let Some(caption) = caption.map(str::trim).filter(|caption| !caption.is_empty()) {
        return MessageInput::Text(format!(
            "[the user attached an image or file; the connector cannot read attachments yet, only this caption text follows]\n{caption}"
        ));
    }
    MessageInput::Unsupported
}

#[derive(Clone, Debug)]
struct BotUsername(String);

pub async fn run_bot(
    bot: Bot,
    bridge: BridgeHandle,
    allowlist: ChatAllowlist,
    max_consecutive_polling_failures: u32,
) -> anyhow::Result<()> {
    bot.set_my_commands(Command::bot_commands())
        .await
        .context("failed to register Telegram bot commands")?;
    info!("Telegram bot commands registered");
    let bot_username = BotUsername(
        bot.get_me()
            .await
            .context("failed to fetch Telegram bot identity")?
            .username()
            .to_string(),
    );
    info!(username = %bot_username.0, "Telegram bot identity loaded");

    let mut listener =
        crate::polling::guarded_polling(bot.clone(), max_consecutive_polling_failures).await;
    let fatal_polling = listener.fatal_flag();
    let listener_error_handler =
        crate::polling::listener_error_handler(listener.stop_token(), fatal_polling.clone());

    let mut dispatcher = Dispatcher::builder(bot, schema())
        .dependencies(dptree::deps![bridge, allowlist, bot_username])
        .enable_ctrlc_handler()
        .build();
    dispatcher
        .try_dispatch_with_listener(listener, listener_error_handler)
        .await
        .context("Telegram dispatcher failed")?;
    if fatal_polling.is_fatal() {
        return Err(crate::error::TelegramError::PollingConflict.into());
    }
    Ok(())
}

fn schema() -> UpdateHandler<anyhow::Error> {
    dptree::entry()
        .branch(Update::filter_message().endpoint(handle_message))
        .branch(Update::filter_callback_query().endpoint(handle_callback))
}

#[instrument(skip(bot, message, bridge, allowlist, bot_username))]
async fn handle_message(
    bot: Bot,
    message: Message,
    bridge: BridgeHandle,
    allowlist: ChatAllowlist,
    bot_username: BotUsername,
) -> HandlerResult {
    let chat_id = message.chat.id;
    if !allowlist.reject_if_unauthorized(chat_id) {
        return Ok(());
    }

    let text = match resolve_message_input(message.text(), message.caption()) {
        MessageInput::Text(text) => text,
        MessageInput::Unsupported => {
            bot.send_message(
                chat_id,
                "I can only read text right now — images, screenshots, and files aren't supported yet. Send a link or paste the text and I'll act on it.",
            )
            .await
            .context("send Telegram unsupported message notice")?;
            return Ok(());
        }
    };

    match parse_incoming(&text, Some(&bot_username.0)) {
        IncomingCommand::Known {
            command: Command::Start | Command::Help,
            ..
        } => {
            bot.send_message(chat_id, help_text())
                .parse_mode(ParseMode::Html)
                .await
                .context("send Telegram help")?;
        }
        IncomingCommand::Known {
            command: Command::New,
            ..
        } => {
            bridge.new_thread(chat_id).await?;
        }
        IncomingCommand::Known {
            command: Command::Cancel | Command::Stop,
            ..
        } => {
            bridge.cancel(chat_id).await?;
        }
        IncomingCommand::Known {
            command: Command::Status,
            ..
        } => {
            let status = bridge.status_text(chat_id).await?;
            bot.send_message(chat_id, status)
                .parse_mode(ParseMode::Html)
                .await
                .context("send Telegram status")?;
        }
        IncomingCommand::Known {
            command: Command::Model,
            args,
        } => {
            bridge.model(chat_id, args).await?;
        }
        IncomingCommand::Known {
            command: Command::Approvals,
            args,
        } => {
            bridge.approvals(chat_id, args).await?;
        }
        IncomingCommand::Known {
            command: Command::Compact,
            ..
        } => {
            bridge.compact(chat_id).await?;
        }
        IncomingCommand::Known {
            command: Command::Diff,
            ..
        } => {
            bridge.diff(chat_id).await?;
        }
        IncomingCommand::Known {
            command: Command::Skills,
            ..
        } => {
            bridge.skills(chat_id).await?;
        }
        IncomingCommand::AgentInput(input) => {
            bridge.send_user_text(chat_id, input).await?;
        }
    }
    Ok(())
}

#[instrument(skip(bot, query, bridge, allowlist))]
async fn handle_callback(
    bot: Bot,
    query: CallbackQuery,
    bridge: BridgeHandle,
    allowlist: ChatAllowlist,
) -> HandlerResult {
    let chat_id = callback_chat_id(&query);
    let response = match chat_id {
        Some(chat_id) if allowlist.reject_if_unauthorized(chat_id) => {
            match query.data.as_deref().and_then(ApprovalCallback::decode) {
                Some(callback) => bridge.handle_approval_callback(chat_id, callback).await?,
                None => {
                    warn!("ignoring unknown Telegram callback data");
                    "Unsupported callback.".to_string()
                }
            }
        }
        Some(_) => "This chat is not authorized to use PFTerminal.".to_string(),
        None => "Approval callbacks must come from the original Telegram chat.".to_string(),
    };
    let answer_text = callback_answer_text(&response);
    if answer_text != response
        && let Some(chat_id) = chat_id
    {
        bot.send_message(chat_id, response)
            .await
            .context("send Telegram callback details")?;
    }
    bot.answer_callback_query(query.id)
        .text(answer_text)
        .await
        .context("answer Telegram callback query")?;
    Ok(())
}

fn callback_chat_id(query: &CallbackQuery) -> Option<ChatId> {
    query.message.as_ref().map(|message| message.chat().id)
}

fn callback_answer_text(text: &str) -> String {
    if text.chars().count() <= CALLBACK_ANSWER_TEXT_LIMIT {
        return text.to_string();
    }
    let mut truncated = text
        .chars()
        .take(CALLBACK_ANSWER_TEXT_LIMIT.saturating_sub(3))
        .collect::<String>();
    truncated.push_str("...");
    truncated
}

#[cfg(test)]
mod tests {
    use super::MessageInput;
    use super::resolve_message_input;

    #[test]
    fn plain_text_is_used_verbatim() {
        assert_eq!(
            resolve_message_input(Some("hello there"), None),
            MessageInput::Text("hello there".to_string())
        );
    }

    #[test]
    fn caption_on_media_is_accepted_but_flags_the_dropped_attachment() {
        let MessageInput::Text(text) =
            resolve_message_input(None, Some("posted: https://discord.com/channels/1/2/3"))
        else {
            panic!("a caption must produce text input");
        };
        // The caption reaches the agent...
        assert!(text.contains("https://discord.com/channels/1/2/3"));
        // ...but the agent is told an attachment it cannot see was dropped, so it
        // never answers as if it saw the image.
        assert!(text.contains("cannot read attachments"));
    }

    #[test]
    fn bare_media_without_caption_is_unsupported() {
        assert_eq!(resolve_message_input(None, None), MessageInput::Unsupported);
        // A whitespace-only caption is not real input.
        assert_eq!(
            resolve_message_input(None, Some("   ")),
            MessageInput::Unsupported
        );
    }

    #[test]
    fn text_wins_over_caption_when_both_present() {
        assert_eq!(
            resolve_message_input(Some("real text"), Some("caption")),
            MessageInput::Text("real text".to_string())
        );
    }
}
