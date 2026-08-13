use std::path::PathBuf;
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
use crate::bridge::UserInputReceipt;
use crate::commands::Command;
use crate::commands::IncomingCommand;
use crate::commands::help_text;
use crate::commands::parse_incoming;
use crate::conversation::ConversationKey;
use crate::dedup::BeginUpdate;
use crate::dedup::UpdateDeduplicator;
use crate::media::DocumentFetch;
use crate::media::FetchedImages;
use crate::media::MediaStore;
use crate::media::fetch_message_document;
use crate::media::fetch_message_images;
use crate::model_selection::ModelPickerCallback;
use crate::outbound::CallSafety;
use crate::outbound::DEFAULT_API_TIMEOUT;
use crate::outbound::call_with_policy;

type HandlerResult = anyhow::Result<()>;
const CALLBACK_ANSWER_TEXT_LIMIT: usize = 200;

/// Send a plain-text notice with an explicit timeout. Mutating, so never
/// auto-retried — duplicate protection is the update-level dedup gate.
async fn send_notice(
    bot: &Bot,
    conversation: ConversationKey,
    text: impl Into<String>,
) -> HandlerResult {
    let bot = bot.clone();
    let text = text.into();
    call_with_policy(
        CallSafety::Mutating,
        DEFAULT_API_TIMEOUT,
        "telegram notice",
        move || {
            let bot = bot.clone();
            let text = text.clone();
            async move {
                let mut request = bot.send_message(conversation.chat_id, text);
                if let Some(thread_id) = conversation.thread_id {
                    request = request.message_thread_id(thread_id);
                }
                request.await
            }
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
/// What an inbound Telegram message contributes as turn input.
///
/// A plain text message is used as-is. Photos and image documents are fetched
/// by the media layer before this classifier runs — this fallback only handles
/// what that layer did not take: unsupported media (video, audio, archives,
/// executables, …) with a
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
struct BotIdentity {
    id: UserId,
    username: String,
}

pub async fn run_bot(
    bot: Bot,
    polling: Bot,
    bridge: BridgeHandle,
    allowlist: ChatAllowlist,
    media: MediaStore,
    codex_home: PathBuf,
    max_consecutive_polling_failures: u32,
) -> anyhow::Result<()> {
    bot.set_my_commands(Command::bot_commands())
        .await
        .context("failed to register Telegram bot commands")?;
    info!("Telegram bot commands registered");
    let me = bot
        .get_me()
        .await
        .context("failed to fetch Telegram bot identity")?;
    let bot_identity = BotIdentity {
        id: me.id,
        username: me.username().to_string(),
    };
    info!(username = %bot_identity.username, bot_id = bot_identity.id.0, "Telegram bot identity loaded");

    let dedup = Arc::new(UpdateDeduplicator::load(&codex_home, bot_identity.id.0).await);
    let pending_updates = dedup.pending_updates().await;
    if !pending_updates.is_empty() {
        info!(
            count = pending_updates.len(),
            "replaying pending Telegram updates"
        );
    }

    let mut listener =
        crate::polling::guarded_polling(polling, max_consecutive_polling_failures, pending_updates)
            .await;
    let fatal_polling = listener.fatal_flag();
    let listener_error_handler =
        crate::polling::listener_error_handler(listener.stop_token(), fatal_polling.clone());

    let mut dispatcher = Dispatcher::builder(bot, schema())
        .dependencies(dptree::deps![bridge, allowlist, bot_identity, media, dedup])
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

// Teloxide injects each handler dependency as a separate argument.
#[allow(clippy::too_many_arguments)]
async fn handle_message(
    bot: Bot,
    update: Update,
    message: Message,
    bridge: BridgeHandle,
    allowlist: ChatAllowlist,
    bot_identity: BotIdentity,
    media: MediaStore,
    dedup: Arc<UpdateDeduplicator>,
) -> HandlerResult {
    let chat_id = message.chat.id;
    let conversation = ConversationKey::from_message(&message);
    let actor_user_id = message.from.as_ref().map(|user| user.id);
    if !allowlist.reject_if_unauthorized_actor(
        chat_id,
        message.from.as_ref().map(|user| user.id),
        message.chat.is_private(),
    ) {
        return Ok(());
    }
    if !begin_or_explain(&bot, conversation, &dedup, &update).await? {
        return Ok(());
    }
    let update_id = u64::from(update.id.0);
    let dedup = Arc::clone(&dedup);
    let result = handle_message_inner(
        bot,
        update,
        message,
        bridge,
        bot_identity,
        media,
        actor_user_id,
    )
    .await;
    match result {
        Ok(UserInputReceipt::Applied) => {
            complete_update(&dedup, conversation, update_id).await;
            Ok(())
        }
        Ok(UserInputReceipt::Queued(completion_rx)) => {
            monitor_queued_update(dedup, conversation, update_id, completion_rx);
            Ok(())
        }
        Err(err) => {
            dedup.release_update(update_id).await;
            Err(err)
        }
    }
}

#[instrument(skip(bot, update, message, bridge, bot_identity, media))]
async fn handle_message_inner(
    bot: Bot,
    update: Update,
    message: Message,
    bridge: BridgeHandle,
    bot_identity: BotIdentity,
    media: MediaStore,
    actor_user_id: Option<UserId>,
) -> anyhow::Result<UserInputReceipt> {
    let conversation = ConversationKey::from_message(&message);

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
            return bridge
                .send_user_input(
                    conversation,
                    caption,
                    paths,
                    client_user_message_id(&bot_identity, &update),
                    actor_user_id,
                )
                .await;
        }
        Ok(FetchedImages::Rejected(reason)) => {
            send_notice(&bot, conversation, reason).await?;
            return Ok(UserInputReceipt::Applied);
        }
        Ok(FetchedImages::None) => {}
        Err(err) => {
            warn!("inbound Telegram image fetch failed: {err:#}");
            send_notice(
                &bot,
                conversation,
                "I couldn't download that image from Telegram — try again, or upload it somewhere and send the link.",
            )
            .await?;
            return Ok(UserInputReceipt::Applied);
        }
    }

    match fetch_message_document(&bot, &media, &message).await {
        Ok(DocumentFetch::Document(document)) => {
            let caption = message.caption().unwrap_or("").trim();
            let text = format!(
                "The user attached a local file that is available to your tools.\nPath: {}\nOriginal name: {}\nMIME: {}\nSize: {} bytes\nSHA-256: {}{}",
                document.path.display(),
                document.original_name,
                document.mime_type,
                document.size,
                document.sha256,
                if caption.is_empty() {
                    String::new()
                } else {
                    format!("\nUser caption: {caption}")
                }
            );
            return bridge
                .send_user_text(
                    conversation,
                    text,
                    client_user_message_id(&bot_identity, &update),
                    actor_user_id,
                )
                .await;
        }
        Ok(DocumentFetch::Rejected(reason)) => {
            send_notice(&bot, conversation, reason).await?;
            return Ok(UserInputReceipt::Applied);
        }
        Ok(DocumentFetch::None) => {}
        Err(err) => {
            warn!("inbound Telegram document fetch failed: {err:#}");
            send_notice(
                &bot,
                conversation,
                "I couldn't download that document from Telegram. Retry or send a repository/link instead.",
            )
            .await?;
            return Ok(UserInputReceipt::Applied);
        }
    }

    let text = match resolve_message_input(message.text(), message.caption()) {
        MessageInput::Text(text) => text,
        MessageInput::Unsupported => {
            send_notice(
                &bot,
                conversation,
                "I can read text, photos, images, source files, JSON, PDF, XML, and YAML. This message had none of those — send a link or paste the text and I'll act on it.",
            )
            .await?;
            return Ok(UserInputReceipt::Applied);
        }
    };

    match parse_incoming(&text, Some(&bot_identity.username)) {
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
                        let mut request = bot
                            .send_message(conversation.chat_id, help_text())
                            .parse_mode(ParseMode::Html);
                        if let Some(thread_id) = conversation.thread_id {
                            request = request.message_thread_id(thread_id);
                        }
                        request.await
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
            bridge.new_thread(conversation).await?;
        }
        IncomingCommand::Known {
            command: Command::Cancel | Command::Stop,
            ..
        } => {
            bridge.cancel(conversation).await?;
        }
        IncomingCommand::Known {
            command: Command::Status,
            ..
        } => {
            let status = bridge.status_text(conversation).await?;
            let bot2 = bot.clone();
            call_with_policy(
                CallSafety::Mutating,
                DEFAULT_API_TIMEOUT,
                "telegram status",
                move || {
                    let bot = bot2.clone();
                    let status = status.clone();
                    async move {
                        let mut request = bot
                            .send_message(conversation.chat_id, status)
                            .parse_mode(ParseMode::Html);
                        if let Some(thread_id) = conversation.thread_id {
                            request = request.message_thread_id(thread_id);
                        }
                        request.await
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
            bridge.model(conversation, args).await?;
        }
        IncomingCommand::Known {
            command: Command::Approvals,
            args,
        } => {
            bridge.approvals(conversation, args).await?;
        }
        IncomingCommand::Known {
            command: Command::Compact,
            ..
        } => {
            bridge.compact(conversation).await?;
        }
        IncomingCommand::Known {
            command: Command::Diff,
            ..
        } => {
            bridge.diff(conversation).await?;
        }
        IncomingCommand::Known {
            command: Command::Skills,
            ..
        } => {
            bridge.skills(conversation).await?;
        }
        IncomingCommand::AgentInput(input) => {
            return bridge
                .send_user_text(
                    conversation,
                    input,
                    client_user_message_id(&bot_identity, &update),
                    actor_user_id,
                )
                .await;
        }
    }
    Ok(UserInputReceipt::Applied)
}

async fn handle_callback(
    bot: Bot,
    update: Update,
    query: CallbackQuery,
    bridge: BridgeHandle,
    allowlist: ChatAllowlist,
    dedup: Arc<UpdateDeduplicator>,
) -> HandlerResult {
    let conversation = callback_conversation(&query);
    let authorized = conversation.is_some_and(|conversation| {
        allowlist.reject_if_unauthorized_actor(
            conversation.chat_id,
            Some(query.from.id),
            query
                .message
                .as_ref()
                .is_some_and(|message| message.chat().is_private()),
        )
    });
    if !authorized {
        return handle_callback_inner(bot, query, bridge, allowlist).await;
    }
    let Some(conversation) = conversation else {
        return Ok(());
    };
    if !begin_or_explain(&bot, conversation, &dedup, &update).await? {
        return Ok(());
    }
    let update_id = u64::from(update.id.0);
    let result = handle_callback_inner(bot, query, bridge, allowlist).await;
    finish_update(&dedup, conversation, update_id, &result).await;
    result
}

#[instrument(skip(bot, query, bridge, allowlist))]
async fn handle_callback_inner(
    bot: Bot,
    query: CallbackQuery,
    bridge: BridgeHandle,
    allowlist: ChatAllowlist,
) -> HandlerResult {
    let conversation = callback_conversation(&query);
    let response = match conversation {
        Some(conversation)
            if allowlist.reject_if_unauthorized_actor(
                conversation.chat_id,
                Some(query.from.id),
                query
                    .message
                    .as_ref()
                    .is_some_and(|message| message.chat().is_private()),
            ) =>
        {
            let data = query.data.as_deref();
            if let Some(callback) = data.and_then(ApprovalCallback::decode) {
                bridge
                    .handle_approval_callback(conversation, callback, query.from.id)
                    .await?
            } else if let Some(callback) = data.and_then(ModelPickerCallback::decode) {
                bridge
                    .handle_model_picker_callback(conversation, callback)
                    .await?
            } else {
                warn!("ignoring unknown Telegram callback data");
                "Unsupported callback.".to_string()
            }
        }
        Some(_) => "This chat is not authorized to use PFTerminal.".to_string(),
        None => "Approval callbacks must come from the original Telegram chat.".to_string(),
    };
    let answer_text = callback_answer_text(&response);
    if answer_text != response
        && let Some(conversation) = conversation
    {
        send_notice(&bot, conversation, response).await?;
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

async fn finish_update(
    dedup: &UpdateDeduplicator,
    conversation: ConversationKey,
    update_id: u64,
    result: &HandlerResult,
) {
    if result.is_ok() {
        complete_update(dedup, conversation, update_id).await;
    } else {
        dedup.release_update(update_id).await;
    }
}

async fn complete_update(
    dedup: &UpdateDeduplicator,
    conversation: ConversationKey,
    update_id: u64,
) {
    if let Err(err) = dedup.complete_update(update_id).await {
        warn!(
            update_id,
            "failed to commit Telegram inbox completion: {err}"
        );
    } else {
        info!(
            update_id,
            conversation = %conversation.redacted_id(),
            result = "applied",
            "Telegram update completed"
        );
    }
}

fn monitor_queued_update(
    dedup: Arc<UpdateDeduplicator>,
    conversation: ConversationKey,
    update_id: u64,
    completion_rx: tokio::sync::oneshot::Receiver<Result<(), String>>,
) {
    tokio::spawn(async move {
        match completion_rx.await {
            Ok(Ok(())) => complete_update(&dedup, conversation, update_id).await,
            Ok(Err(err)) => {
                dedup.release_update(update_id).await;
                warn!(
                    update_id,
                    "queued Telegram update failed before acceptance: {err}"
                );
            }
            Err(_) => {
                dedup.release_update(update_id).await;
                warn!(update_id, "queued Telegram update completion was dropped");
            }
        }
    });
}

async fn begin_or_explain(
    bot: &Bot,
    conversation: ConversationKey,
    dedup: &UpdateDeduplicator,
    update: &Update,
) -> anyhow::Result<bool> {
    match dedup.begin_update(update).await {
        BeginUpdate::Accepted => {
            info!(
                update_id = u64::from(update.id.0),
                conversation = %conversation.redacted_id(),
                result = "accepted",
                "Telegram update entered durable inbox"
            );
            Ok(true)
        }
        BeginUpdate::Duplicate => Ok(false),
        BeginUpdate::InboxFull => {
            send_notice(
                bot,
                conversation,
                "PFTerminal's Telegram inbox is full. Wait for queued work to finish, then retry.",
            )
            .await?;
            Ok(false)
        }
        BeginUpdate::PersistenceFailed => {
            send_notice(
                bot,
                conversation,
                "PFTerminal could not safely persist this update, so it was not started. Retry after checking host storage.",
            )
            .await?;
            Ok(false)
        }
    }
}

fn callback_conversation(query: &CallbackQuery) -> Option<ConversationKey> {
    query.message.as_ref().map(|message| {
        ConversationKey::new(
            message.chat().id,
            message
                .regular_message()
                .and_then(|message| message.thread_id),
        )
    })
}

fn client_user_message_id(identity: &BotIdentity, update: &Update) -> String {
    format!("telegram:{}:{}", identity.id.0, update.id.0)
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
    use teloxide::types::Update;
    use teloxide::types::UserId;

    use super::BotIdentity;
    use super::MessageInput;
    use super::client_user_message_id;
    use super::resolve_message_input;

    #[test]
    fn plain_text_is_used_verbatim() {
        assert_eq!(
            resolve_message_input(Some("hello there"), /*caption*/ None),
            MessageInput::Text("hello there".to_string())
        );
    }

    #[test]
    fn caption_on_non_image_media_is_accepted_but_flags_the_dropped_attachment() {
        let MessageInput::Text(text) = resolve_message_input(
            /*text*/ None,
            Some("posted: https://discord.com/channels/1/2/3"),
        ) else {
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
        assert_eq!(
            resolve_message_input(/*text*/ None, /*caption*/ None),
            MessageInput::Unsupported
        );
        // A whitespace-only caption is not real input.
        assert_eq!(
            resolve_message_input(/*text*/ None, Some("   ")),
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

    #[test]
    fn client_message_identity_is_stable_and_bot_scoped() {
        let update: Update = serde_json::from_value(serde_json::json!({
            "update_id": 42,
            "message": {
                "message_id": 7,
                "date": 1,
                "chat": {"id": 99, "type": "private"},
                "text": "hello"
            }
        }))
        .unwrap();
        let identity = BotIdentity {
            id: UserId(123),
            username: "pft".into(),
        };
        assert_eq!(
            client_user_message_id(&identity, &update),
            "telegram:123:42"
        );
    }
}
