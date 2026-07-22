use std::sync::Arc;

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
use crate::dedup::UpdateDeduplicator;
use crate::media::FetchedImages;
use crate::media::MediaStore;
use crate::media::fetch_message_images;
use crate::outbound::CallSafety;
use crate::outbound::DEFAULT_API_TIMEOUT;
use crate::outbound::call_with_policy;

type HandlerResult = anyhow::Result<()>;
const CALLBACK_ANSWER_TEXT_LIMIT: usize = 200;

/// Send a plain-text notice with an explicit timeout. Mutating, so never
/// auto-retried — duplicate protection is the update-level dedup gate.
async fn send_notice(bot: &Bot, chat_id: ChatId, text: impl Into<String>) -> HandlerResult {
    let bot = bot.clone();
    let text = text.into();
    call_with_policy(
        CallSafety::Mutating,
        DEFAULT_API_TIMEOUT,
        "telegram notice",
        move || {
            let bot = bot.clone();
            let text = text.clone();
            async move { bot.send_message(chat_id, text).await }
        },
    )
    .await
    .context("send Telegram notice")?;
    Ok(())
}

/// Dispatcher-side wrapper that deduplicates by Telegram `update_id` before
/// any handler runs. Telegram delivers updates at-least-once: a long-poll
/// reconnect or a restart before the offset is acknowledged replays the tail
/// of the stream. Without this gate a replayed message fires the same
/// mutating agent action twice (two identical turns, a double approval).
/// `Update::filter_message`/`filter_callback_query` forward the full update,
/// so the id is always available here.
async fn is_first_delivery(update: Update, dedup: Arc<UpdateDeduplicator>) -> bool {
    let update_id = u64::from(update.id.0);
    dedup.check_and_record(update_id).await
}

/// What an inbound Telegram message contributes as turn input.
///
/// A plain text message is used as-is. Photos and image documents are fetched
/// by the media layer before this classifier runs — this fallback only handles
/// what that layer did not take: NON-IMAGE media (video, audio, pdf, …) with a
/// caption contributes the caption prefixed with a note so the agent does not
/// answer as if it saw the attachment; without a caption it is unsupported.
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
            "[the user attached a non-image file; the connector cannot read it, only this caption text follows]\n{caption}"
        ));
    }
    MessageInput::Unsupported
}

#[derive(Clone, Debug)]
struct BotUsername(String);

pub async fn run_bot(
    bot: Bot,
    polling: Bot,
    bridge: BridgeHandle,
    allowlist: ChatAllowlist,
    media: MediaStore,
    dedup: UpdateDeduplicator,
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
        crate::polling::guarded_polling(polling, max_consecutive_polling_failures).await;
    let fatal_polling = listener.fatal_flag();
    let listener_error_handler =
        crate::polling::listener_error_handler(listener.stop_token(), fatal_polling.clone());

    let mut dispatcher = Dispatcher::builder(bot, schema())
        .dependencies(dptree::deps![
            bridge,
            allowlist,
            bot_username,
            media,
            Arc::new(dedup)
        ])
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
    with_dedup(
        dptree::entry()
            .branch(Update::filter_message().endpoint(handle_message))
            .branch(Update::filter_callback_query().endpoint(handle_callback)),
    )
}

fn with_dedup(handler: UpdateHandler<anyhow::Error>) -> UpdateHandler<anyhow::Error> {
    // A filter, unlike an endpoint, continues into the real handler branches
    // when it returns true. Replayed updates return false and fall through
    // without firing a mutating agent action twice.
    dptree::entry()
        .filter_async(is_first_delivery)
        .chain(handler)
}

#[instrument(skip(bot, message, bridge, allowlist, bot_username))]
async fn handle_message(
    bot: Bot,
    message: Message,
    bridge: BridgeHandle,
    allowlist: ChatAllowlist,
    bot_username: BotUsername,
    media: MediaStore,
) -> HandlerResult {
    let chat_id = message.chat.id;
    if !allowlist.reject_if_unauthorized(chat_id) {
        return Ok(());
    }

    // Images first: a screenshot (with or without caption) becomes real turn
    // input. A fetch *error* degrades to the caption-note path rather than
    // dropping the message — the agent still learns an attachment existed.
    match fetch_message_images(&bot, &media, &message).await {
        Ok(FetchedImages::Images(paths)) => {
            let caption = message
                .caption()
                .map(str::trim)
                .filter(|c| !c.is_empty())
                .unwrap_or("")
                .to_string();
            bridge.send_user_input(chat_id, caption, paths).await?;
            return Ok(());
        }
        Ok(FetchedImages::Rejected(reason)) => {
            send_notice(&bot, chat_id, reason).await?;
            return Ok(());
        }
        Ok(FetchedImages::None) => {}
        Err(err) => {
            warn!("inbound Telegram image fetch failed: {err:#}");
            send_notice(
                &bot,
                chat_id,
                "I couldn't download that image from Telegram — try again, or upload it somewhere and send the link.",
            )
            .await?;
            return Ok(());
        }
    }

    let text = match resolve_message_input(message.text(), message.caption()) {
        MessageInput::Text(text) => text,
        MessageInput::Unsupported => {
            send_notice(
                &bot,
                chat_id,
                "I can read text, photos, and image files. This message had none of those — send a link or paste the text and I'll act on it.",
            )
            .await?;
            return Ok(());
        }
    };

    match parse_incoming(&text, Some(&bot_username.0)) {
        IncomingCommand::Known {
            command: Command::Start | Command::Help,
            ..
        } => {
            let bot2 = bot.clone();
            call_with_policy(
                CallSafety::Mutating,
                DEFAULT_API_TIMEOUT,
                "telegram help",
                move || {
                    let bot = bot2.clone();
                    async move {
                        bot.send_message(chat_id, help_text())
                            .parse_mode(ParseMode::Html)
                            .await
                    }
                },
            )
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
            let bot2 = bot.clone();
            call_with_policy(
                CallSafety::Mutating,
                DEFAULT_API_TIMEOUT,
                "telegram status",
                move || {
                    let bot = bot2.clone();
                    let status = status.clone();
                    async move {
                        bot.send_message(chat_id, status)
                            .parse_mode(ParseMode::Html)
                            .await
                    }
                },
            )
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
        send_notice(&bot, chat_id, response).await?;
    }
    call_with_policy(
        CallSafety::Mutating,
        DEFAULT_API_TIMEOUT,
        "telegram answerCallbackQuery",
        move || {
            let bot = bot.clone();
            let query_id = query.id.clone();
            let answer_text = answer_text.clone();
            async move { bot.answer_callback_query(query_id).text(answer_text).await }
        },
    )
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
    use std::ops::ControlFlow;
    use std::sync::Arc;

    use teloxide::dptree;
    use teloxide::types::Update;

    use super::MessageInput;
    use super::resolve_message_input;
    use super::with_dedup;
    use crate::dedup::UpdateDeduplicator;

    #[test]
    fn plain_text_is_used_verbatim() {
        assert_eq!(
            resolve_message_input(Some("hello there"), None),
            MessageInput::Text("hello there".to_string())
        );
    }

    #[test]
    fn caption_on_non_image_media_is_accepted_but_flags_the_dropped_attachment() {
        let MessageInput::Text(text) =
            resolve_message_input(None, Some("posted: https://discord.com/channels/1/2/3"))
        else {
            panic!("a caption must produce text input");
        };
        // The caption reaches the agent...
        assert!(text.contains("https://discord.com/channels/1/2/3"));
        // ...but the agent is told a non-image attachment it cannot see was
        // dropped, so it never answers as if it saw the file. (Images never
        // reach this path — the media layer fetches them first.)
        assert!(text.contains("cannot read it"));
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

    #[tokio::test]
    async fn dedup_filter_routes_first_delivery_and_drops_replay() {
        let dedup = Arc::new(UpdateDeduplicator::new_for_test());
        let update: Update = serde_json::from_value(serde_json::json!({
            "update_id": 42,
            "message": {
                "message_id": 7,
                "date": 1,
                "chat": {"id": 99, "type": "private"},
                "text": "hello"
            }
        }))
        .expect("valid Telegram update");

        let handler = with_dedup(dptree::endpoint(|| async { Ok(()) }));

        let first = handler
            .dispatch(dptree::deps![update.clone(), Arc::clone(&dedup)])
            .await;
        assert!(matches!(first, ControlFlow::Break(Ok(()))));

        let replay = handler.dispatch(dptree::deps![update, dedup]).await;
        assert!(matches!(replay, ControlFlow::Continue(_)));
    }
}
