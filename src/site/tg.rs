use std::fmt::format;
use crate::site::form::Form;
use crate::types::{AppConfig, SwappyUser};
use std::sync::atomic::Ordering;
use axum::http::StatusCode;
use redis::{Commands, RedisError};
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, MessageId, ParseMode, User, WebAppInfo};
use teloxide::RequestError;
use crate::site::handlers::PostParams;
use crate::store::get_star_count;
use crate::types::swappy_bot::ToSwappyBot;

pub async fn handle_shit(
    app_config: &AppConfig,
    post_params: PostParams,
    form: Form,
    mut sw_user: SwappyUser<'_>,
) -> Result<(MessageId, MessageId), (StatusCode, String)> {
    let mut delete_old_report = false;
    if let Some(edit_id) = post_params.edit_id {
        let edit_id = MessageId(edit_id);
        match sw_user.is_author(edit_id).await {
            Ok(true) => {
                delete_old_report = true;
            }
            Ok(false) => {
                return Err((StatusCode::FORBIDDEN, "Редактируемое сообщение не ваше".to_string()));
            }
            Err(e) => {
                log::error!("redis query failed: {}", e.to_string());
                return Err((StatusCode::INTERNAL_SERVER_ERROR,
                            "Что-то пошло не так, попробуйте позднее".to_string()));
            }
        }
    }


    // let sw_bot = app_config.bot.clone().to_swappy_bot(app_config.group_id());

    let group_msg = post_ad(
        post_params.edit_id,
        &app_config.bot,
        &form,
        &mut sw_user,
    ).await.map_err(|e| {
        log::error!("failed to post ad: {}", e.to_string());
        (StatusCode::INTERNAL_SERVER_ERROR, "Try later".to_string())
    })?;

    sw_user.set_author(group_msg.id).await.expect("kk");

    if delete_old_report {
        if let Err(e) = app_config.bot.delete_message(
            sw_user.tg_user.id, MessageId(post_params.report_id.unwrap_or_default()))
            .await {
            log::error!("failed to delete old report: {}", e.to_string());
        }
    }

    let report_id = report_ad(&sw_user.tg_user, sw_user.group_id, &group_msg, post_params, app_config).await
        .map_err(|e| {
            // if let Err(e) = app_config.bot.delete_message(group_id, group_msg.id).await {
            //     log::error!("failed to cleanup ad after failing to send report: {}", e.to_string());
            // };
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;

    Ok((group_msg.id, report_id))
}

async fn post_ad(
    edit_msg_id: Option<i32>,
    bot: &Bot,
    form: &Form,
    bot_user: &mut SwappyUser<'_>,
) -> Result<Message, RequestError> {
    let group_id = bot_user.group_id;
    let sc = bot_user.star_count().await.unwrap_or_default();
    let stars = if sc > 0 { format!(" (<i>⭐️</i>{})", sc) } else { String::default() };

    let msg = format!(
        "{}{}:\n\n{}",
        bot_user,
        stars,
        form,
    );

    if let Some(msg_id) = edit_msg_id {
        bot.edit_message_text(group_id, MessageId(msg_id), msg)
            .parse_mode(ParseMode::Html)
            .await
    } else {
        bot.send_message(group_id, msg)
            .parse_mode(ParseMode::Html)
            .await
    }
}

async fn report_ad(
    user: &User,
    group_id: ChatId,
    msg: &Message,
    post_params: PostParams,
    app_config: &AppConfig,
) -> Result<MessageId, RequestError> {
    use crate::bot::commands::CallbackQueryCommand::*;
    let bot = &app_config.bot;

    let mut edit_url = app_config.app_url.clone();
    // edit_url.set_path("/form");
    let query = format!("edit={}", msg.id.to_string());
    edit_url.set_query(Some(&query));

    let mut butts = vec![
        vec![
            InlineKeyboardButton::callback("Снять 🗑️".to_string(), Delete(msg.id).to_string()),
        ],
    ];
    if post_params.keeping {
        butts[0].insert(0,
            InlineKeyboardButton::web_app("Редактировать ✏️".to_string(), WebAppInfo { url: edit_url }),
        );
    }

    bot.copy_message(user.id, group_id, msg.id)
        .reply_markup(InlineKeyboardMarkup::new(butts))
        .await
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