use std::sync::atomic::Ordering;
use teloxide::prelude::*;
use teloxide::{ApiError, RequestError};
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, Me, MessageId, ParseMode, User};
use crate::bot::make_callback_kb;
use crate::site::form::Form;
use crate::types::AppConfig;


pub async fn handle_shit(
    app_config: &AppConfig,
    form: Form,
    user: &User,
) -> Result<MessageId, RequestError> {
    let group_id = ChatId(app_config.group_id.load(Ordering::Relaxed));

    // todo move check earlier
    if !is_member(&app_config.bot, group_id, user.id).await? {
        println!("ha some looser tried to post without being part of the group");
        return Err(RequestError::Api(ApiError::NotEnoughRightsToPostMessages)); // todo return some kind of Err
    }

    let group_msg = post_ad(app_config, &form, user).await?;

    report_ad(user, group_id, &group_msg, app_config).await?;

    Ok(group_msg.id)
}

async fn is_member(
    bot: &Bot,
    group_id: ChatId,
    user_id: UserId,
) -> Result<bool, RequestError> {
    let chat_member = bot.get_chat_member(group_id, user_id).await?;

    Ok(chat_member.is_present())
}

async fn post_ad(
    app_config: &AppConfig,
    form: &Form,
    user: &User,
) -> Result<Message, RequestError> {
    let bot = &app_config.bot;

    let msg = format!(
        "<a href=\"{}\">{}</a> (⭐️14):\n\n{}",
        user.url(),
        user.full_name().trim(),
        form,
    );

    let group_id = ChatId(app_config.group_id.load(Ordering::Relaxed));
    bot.send_message(group_id, msg)
        // .reply_markup(make_ad_kb(&user.id))
        .parse_mode(ParseMode::Html)
        .await
}

async fn report_ad(
    user: &User,
    group_id: ChatId,
    msg: &Message,
    app_config: &AppConfig,
) -> Result<(), RequestError> {
    use crate::bot::commands::CallbackQueryCommand::*;
    let bot = &app_config.bot;

    let butts = vec![
        vec![
            ("Редактировать ✏️".into(), Edit(msg.id)),
            ("Снять 🗑️".into(), Delete(msg.id)),
        ],
    ];

    bot.copy_message(user.id, group_id, msg.id)
        .reply_markup(make_callback_kb(butts))
        .await.map(|_| ())
}

fn make_ad_kb(user_id: &UserId) -> InlineKeyboardMarkup {
    let mut kb: Vec<Vec<InlineKeyboardButton>> = vec![];
    let url = format!("tg://user?id={}", user_id).parse().unwrap();
    let write = InlineKeyboardButton::url(
        "Написать  💬".to_string(),
        url,
        //"https://t.me/reina_bailando".parse().unwrap()
    );
    kb.push(vec![write]);

    // let a = InlineKeyboardButton::callback("⭐️", "xx");
    //let b = InlineKeyboardButton::callback("👹", "yy");
    // kb.push(vec![write, a]);

    InlineKeyboardMarkup::new(kb)
}