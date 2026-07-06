use anyhow::Context;
use teloxide::dispatching::UpdateHandler;
use teloxide::dptree;
use teloxide::prelude::Requester;
use teloxide::prelude::*;
use teloxide::types::ParseMode;
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

pub async fn run_bot(
    bot: Bot,
    bridge: BridgeHandle,
    allowlist: ChatAllowlist,
    max_consecutive_polling_failures: u32,
) -> anyhow::Result<()> {
    let me = bot
        .get_me()
        .await
        .context("failed to validate Telegram bot token")?;
    info!(bot_username = ?me.user.username, "Telegram bot token validated");

    let mut listener =
        crate::polling::guarded_polling(bot.clone(), max_consecutive_polling_failures).await;
    let listener_error_handler = crate::polling::listener_error_handler(listener.stop_token());

    let mut dispatcher = Dispatcher::builder(bot, schema())
        .dependencies(dptree::deps![bridge, allowlist])
        .enable_ctrlc_handler()
        .build();
    dispatcher
        .try_dispatch_with_listener(listener, listener_error_handler)
        .await
        .context("Telegram dispatcher failed")?;
    Ok(())
}

fn schema() -> UpdateHandler<anyhow::Error> {
    dptree::entry()
        .branch(Update::filter_message().endpoint(handle_message))
        .branch(Update::filter_callback_query().endpoint(handle_callback))
}

#[instrument(skip(bot, message, bridge, allowlist))]
async fn handle_message(
    bot: Bot,
    message: Message,
    bridge: BridgeHandle,
    allowlist: ChatAllowlist,
) -> HandlerResult {
    let chat_id = message.chat.id;
    if !allowlist.reject_if_unauthorized(chat_id) {
        return Ok(());
    }

    let Some(text) = message.text() else {
        bot.send_message(chat_id, "Send text messages to start PFTerminal turns.")
            .await
            .context("send Telegram unsupported message notice")?;
        return Ok(());
    };

    match parse_incoming(text) {
        IncomingCommand::Known(Command::Start | Command::Help) => {
            bot.send_message(chat_id, help_text())
                .parse_mode(ParseMode::Html)
                .await
                .context("send Telegram help")?;
        }
        IncomingCommand::Known(Command::New) => {
            bridge.new_thread(chat_id).await?;
        }
        IncomingCommand::Known(Command::Cancel) => {
            bridge.cancel(chat_id).await?;
        }
        IncomingCommand::Known(Command::Status) => {
            let status = bridge.status_text(chat_id).await?;
            bot.send_message(chat_id, status)
                .parse_mode(ParseMode::Html)
                .await
                .context("send Telegram status")?;
        }
        IncomingCommand::AgentInput(input) => {
            bridge.send_user_text(chat_id, input).await?;
        }
    }
    Ok(())
}

#[instrument(skip(bot, query, bridge))]
async fn handle_callback(bot: Bot, query: CallbackQuery, bridge: BridgeHandle) -> HandlerResult {
    let Some(data) = query.data.as_deref() else {
        return Ok(());
    };
    let Some(callback) = ApprovalCallback::decode(data) else {
        warn!("ignoring unknown Telegram callback data");
        return Ok(());
    };
    let response = bridge.handle_approval_callback(callback).await?;
    bot.answer_callback_query(query.id)
        .text(response)
        .await
        .context("answer Telegram callback query")?;
    Ok(())
}
