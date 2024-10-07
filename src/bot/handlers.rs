use super::commands::*;
use super::TARGET_GROUP_ID_KEY;
use crate::store::{get_star_count, give_star};
use crate::types::{AppConfig, ToSwappyUser};
use redis::Commands;
use std::fmt::Display;
use std::sync::Arc;
use teloxide::dispatching::dialogue::GetChatId;
use teloxide::payloads::AnswerCallbackQuerySetters;
use teloxide::payloads::EditMessageTextSetters;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::{CallbackQuery, ChatId, Message, Requester};
use teloxide::types::{ButtonRequest, ChatKind, InlineKeyboardButton, InlineKeyboardMarkup, KeyboardButton, KeyboardButtonRequestUsers, KeyboardMarkup, MessageKind, ParseMode, RequestId};
use teloxide::utils::command::BotCommands;
use teloxide::{Bot, RequestError};
use ButtonRequest::RequestUsers;

pub async fn handle_added_to_group(
    bot: Bot,
    config: Arc<AppConfig>,
    message: Message,
) -> Result<(), RequestError>
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

        let group_id = config.group_id();
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
            Edit(_) => {
                // everything happens in webapp
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
            "Привет! С помощью этого бота ты можешь опубликовать своё объявление в нашей группе. \
            Также можно раздавать и получать ⭐.\n\n\
            Подробнее о публикации объявлений: /posting\n\
            Подробнее о звёздах: /stars\n\
            Советы о совершении сделок: /safety\n\
            О хранении данных: /personaldata".to_string()
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
        SimpleCommand::MyStars => {
            let sc = get_star_count(msg.from.unwrap().id, config.group_id(),
                           &config.redis_client).expect("").to_string();

            format!("У вас {}⭐", sc)
        }
        SimpleCommand::Stars => {
            "Чтобы вручить кому-нибудь звезду, воспользуйтесь кнопкой под полем ввода. Если кнопку не \
            видно, команда /start должна помочь. Телеграм предложит вам выбрать пользователей и \
            поделиться ими с ботом. Можно выбрать до 10 пользователей за раз. Каждый получит от вас \
            звезду.\n\n\
            Звёзды (если есть) отображаются рядом с вашим именем в публикуемых вами объявлениях. Ко\
            личество имеющихся у вас звёзд можно узнать командой /mystars.\n\n\
            Звёзды можно получать от и давать только другим участникам группы. Нельзя вручить звезду \
            себе (как бы ни хотелось). Для того, чтобы вручить кому-то звезду, не обязательно обмени\
            ваться с этим человеком. Если вы готовы в будущем обменяться с кем-то - это хороший повод \
            вручить звезду.\n\n\
            Одному человеку звезду дать можно только один раз. Забирать звёзды пока нельзя, но будет \
            можно в дальнейшем.\n\n\
            Звёзды призваны облегчить принятие решения о сделке в ситуациях, когда у вас нет общих \
            чатов с человеком, и когда вы с ним не знакомы. Тем не менее, они не ставят своей целью \
            заменить ваш здравый смысл, поэтому не полагайтесь только на них.".to_string()
        }
        SimpleCommand::Posting => {
            "Чтобы опубликовать объявление в группе, воспользуйтесь кнопкой слева от поля ввода. \
            Откроется мини-приложение с формой, которую нужно будет заполнить, после чего нажать \
            кнопку \"Опубликовать\". После вашего подтверждения бот опубликует сообщение в группе \
            и пришлёт в этот чат его копию с кнопками управления вашим объявлением. В сообщении \
            будет содержаться ссылка на чат с вами, количество ваших звезд и само объявление.\n\n\
            По умолчанию, бот не запоминает информацию об объявлениях, поэтому объявление можно будет \
            только удалить. Чтобы включить возможность редактировать объявления, вы можете включить \
            запоминание данных опубликованных объявлений. Для этого откройте мини-приложение и в его \
            настройках включите запоминание данных форм. Все опубликованные в дальнейшем сообщения \
            будет возможно редактировать. Также, если эта настройка включена, следующие публикации \
            будут предзаполнены данными последнего объявления.".to_string()
        }
        SimpleCommand::Safety => {
            "Этот раздел посвящается обменам с незнакомыми людьми. О том, как вытрясти долги с людей, \
            которых вы знаете, здесь информации не будет.\n\n\
            При онлайн сделках проверьте количество общих чатов с человеком: чем их больше, тем лучше. \
            Если общих чатов нет, позадавайте вопросы, узнайте, кто пригласил человека в группу, \
            откуда он, как познакомился с пригласившим. Можно написать пригласившему и сверить ответы. \
            Если всё ещё не уверены, не отправляйте всю сумму целиком, разбейте обмен на серию мелких \
            транзакций.\n\n\
            При обмене наличными главное, что нужно помнить, - кто-то будет знать, где и когда вы будете \
            и сколько именно денег будет у вас в кармане. Планируйте встречи соответствующе: лучше \
            в светлое время суток, не в поле и не в безлюдных местах, возьмите с собой на встречу \
            кого-нибудь ещё.".to_string()
        }
        SimpleCommand::PersonalData => {
            "Что хранится в базе бота?\n\nБот хранит информацию о том, какие объявления ваши, чтобы \
            никто кроме вас не смог их отредактировать. Эта информация не зашифрована, потому что \
            она является публичной. Также хранится информация о звёздах. Эта информация хранится в \
            зашифрованном (точнее, хешированном с секретной солью) виде.\n\n\
            Где будут храниться данные моих объявлений, если я включу соответствующую настройку?\n\n\
            Эта информация будет храниться в вашем персональном облачном хранилище для этого бота от \
            Telegram.".to_string()
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
            let msg = config.group_id().to_string();
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
            config.set_group_id(gid);
            bot.send_message(message.chat.id, res).await.map(|_| ())
        }
        TestMsg => {
            let gid = config.group_id();
            send_test_msg(bot, gid, message.chat.id).await.map(|_| ())
        }
    }
}

pub async fn handle_shared_users(
    bot: Bot,
    config: Arc<AppConfig>,
    message: Message,
) -> Result<(), RequestError> {
    let group_id = config.group_id();

    let giver = message.from.unwrap();
    let giver_id = giver.id;

    // silently ignore non-members
    if ! &config.bot.get_chat_member(group_id, giver_id).await?.is_present() {
        return Ok(());
    }

    let mut count = 0;
    match message.kind {
        MessageKind::UsersShared(mut users) => {
            while let Some(receiver_id) = users.users_shared.user_ids.pop() {
                // can't give stars to yourself
                if receiver_id == giver_id { continue; }

                // if receiver is group not member, don't give them star
                if let Some(member) = &config.bot.get_chat_member(group_id, receiver_id).await.ok() {
                    if ! member.is_present() {
                        continue;
                    }
                } else {
                    continue;
                }

                give_star(
                    giver_id,
                    receiver_id,
                    config.bot_token.as_bytes(),
                    &format!("{}:{}:stars", group_id, receiver_id.0),
                    &config.redis_client,
                ).unwrap();
                count += 1;
            }
        }
        _ => ()
    }

    bot.send_message(giver_id, format!("Успешно врученных звёзд: {}", count)).await.map(|_|())
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

    kb.push(vec![
        KeyboardButton::new("Вручить ⭐️").request(RequestUsers(KeyboardButtonRequestUsers {
            request_id: RequestId(1),
            user_is_bot: Some(false),
            user_is_premium: None,
            max_quantity: 10,
        }))
    ]);

    KeyboardMarkup::new(kb).resize_keyboard()
}