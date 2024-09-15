use std::fmt::{format, Display};
use std::sync::Arc;
use std::sync::atomic::Ordering;
use redis::Commands;
use teloxide::{Bot, RequestError};
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::{CallbackQuery, ChatId, Message, Requester};
use teloxide::types::{ChatKind, InlineKeyboardButton, InlineKeyboardMarkup, KeyboardButton, KeyboardMarkup, MessageId, ParseMode};
use teloxide::utils::command::BotCommands;
use teloxide::payloads::AnswerCallbackQuerySetters;
use teloxide::dispatching::dialogue::GetChatId;
use teloxide::payloads::EditMessageTextSetters;

use super::commands::*;
use super::TARGET_GROUP_ID_KEY;
use crate::types::AppConfig;

pub async fn handle_added_to_group(
    bot: Bot,
    config: Arc<AppConfig>,
    message: Message
)  -> Result<(), RequestError>
{
    let grp_title = match message.chat.kind {
        ChatKind::Public(chat) => { chat.title }
        ChatKind::Private(_) => None
    }.unwrap_or_default();

    let user = if let Some(user) = message.from {
        format!("[{} {}](tg://user?id={})",
                user.first_name,
                user.last_name.unwrap_or_default(),
                user.id
        )
    } else { String::from("Somebody") };

    let grp_id = message.chat.id;
    let text = format!("{user} added me to {grp_title} \\(`{grp_id}`\\)");
    bot.send_message(config.bot_maintainer, text).parse_mode(ParseMode::MarkdownV2)
        .await?;

    Ok(())
}

pub async fn handle_callback_query(
    bot: Bot,
    callback_query: CallbackQuery,
    config: Arc<AppConfig>,
) -> Result<(), RequestError> {
    if let Some(ref data) = callback_query.data {
        use CallbackQueryCommand::{Delete, Edit, Repost};
        let cmd = CallbackQueryCommand::parse(data);
        if cmd.is_none() { todo!("return some kinda error") }

        let group_id = ChatId(config.group_id.load(Ordering::Relaxed));
        match cmd.unwrap() {
            Delete(msg_id) => {
                let res = bot.delete_message(group_id, msg_id).await;

                if res.is_err() {
                    return bot.answer_callback_query(callback_query.id)
                        .text("Something went wrong")
                        .await.map(|_| ());
                }

                // assuming some filtering has been done previously
                let msg = callback_query.regular_message().unwrap();
                let chat_id = callback_query.chat_id().unwrap();
                let new_text = format!("{}\n\nВы сняли это объявление.",
                                       msg.text().unwrap_or_default());
                bot.edit_message_text(chat_id, msg.id, new_text)
                    .parse_mode(ParseMode::Html)
                    .await?;
            }
            Edit(id) => {
                // открыть форму с двумя параметрами: айди сообщения в группе и айди сообщения в чате
                // желательно зашифрованными

                // юзер делает изменения в форме и отправляет

                // здесь больше нечего делать
                // этой ветки вообще не существует
                bot.edit_message_text(group_id, id, "kekw").await?;
            }
            Repost(_) => {}
        }
    }

    bot.answer_callback_query(callback_query.id).await.map(|_| ())
}

pub async fn handle_simple_command(
    config: Arc<AppConfig>,
    bot: Bot,
    msg: Message,
    command: SimpleCommand,
) -> Result<(), RequestError> {
    let mut kb: Option<KeyboardMarkup> = None;
    let text = match command {
        SimpleCommand::Start => {
            kb = Some(make_start_kb());
            "Привет! Чтобы узнать список моих комманд, используй /help.".to_string()
        }
        SimpleCommand::Help => {
            if msg.from.unwrap().id == config.bot_maintainer {
                format!(
                    "{}\n\n{}",
                    SimpleCommand::descriptions(),
                    MaintainerCommand::descriptions(),
                )
            } else {
                SimpleCommand::descriptions().to_string()
            }
        }
        SimpleCommand::MyId => {
            msg.from.unwrap().id.to_string()
        }
        SimpleCommand::Maintainer => {
            config.bot_maintainer.to_string()
        }
    };

    bot.send_message(msg.chat.id, text)
        .reply_markup(kb.unwrap_or_default())
        .await?;

    Ok(())
}

pub async fn handle_maintainer_command(
    bot: Bot,
    message: Message,
    config: Arc<AppConfig>,
    command: MaintainerCommand,
) -> Result<(), RequestError> {
    use MaintainerCommand::*;
    match command {
        GetGroup => {
            let msg = config.group_id.load(Ordering::Relaxed).to_string();
            bot.send_message(message.chat.id, msg).await.map(|_| ())
        }
        SetGroup(gid) => {
            let res =
                match config.redis_client.get_connection().unwrap()
                    .set::<&str, i64, ()>(TARGET_GROUP_ID_KEY, gid)
                {
                    Ok(_) => String::from("Successfully set"),
                    Err(e) => e.to_string()
                };
            config.group_id.store(gid, Ordering::Relaxed);
            bot.send_message(message.chat.id, res).await.map(|_| ())
        }
        TestMsg => {
            let gid = ChatId(config.group_id.load(Ordering::Relaxed));
            send_test_msg(bot, gid, message.chat.id).await.map(|_| ())
        }
    }
}

async fn send_test_msg(bot: Bot, dst_chat_id: ChatId, requester_chat_id: ChatId) -> Result<(), RequestError> {
    let sent_msg = bot.send_message(dst_chat_id, "Hi, this is a test message").await?;

    let report = format!("sent\nid: `{}`\ngroup_id: `{}`", sent_msg.id, dst_chat_id);
    bot.send_message(requester_chat_id, report)
        // .parse_mode(ParseMode::MarkdownV2)
        .reply_markup(make_kb(vec![("Delete".into(), sent_msg.id.to_string())]))
        .await.map(|_| ())
}

pub fn make_callback_kb(butts: Vec<Vec<(String, impl Display)>>) -> InlineKeyboardMarkup {
    let mut kb: Vec<Vec<InlineKeyboardButton>> = vec![];

    butts.into_iter().for_each(|row| {
        kb.push(
            row.into_iter().map(|butt| InlineKeyboardButton::callback(butt.0, butt.1.to_string())).collect()
        );
    });

    InlineKeyboardMarkup::new(kb)
}

pub fn make_kb(butts: Vec<(String, String)>) -> InlineKeyboardMarkup
{
    let mut kb: Vec<Vec<InlineKeyboardButton>> = vec![];

    let row = butts.into_iter().map(
        |butt| InlineKeyboardButton::callback(butt.0, butt.1)
    )
        .collect();


    kb.push(row);

    InlineKeyboardMarkup::new(kb)
}

fn make_start_kb() -> KeyboardMarkup {
    let mut kb: Vec<Vec<KeyboardButton>> = vec![];

    kb.push(
        ["Вручить ⭐️", "Мои ⭐️"].map(
            |s| KeyboardButton::new(s)
        ).into_iter().collect()
    );

    KeyboardMarkup::new(kb).resize_keyboard().persistent()
}