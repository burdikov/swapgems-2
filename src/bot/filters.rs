use std::sync::Arc;
use teloxide::prelude::Message;
use teloxide::types::{Me, MessageKind};
use crate::types::AppConfig;

pub fn me_added_to_group(message: Message, me: Me) -> bool {
    if let Some(new_members) = message.new_chat_members() {
        new_members.iter().any(|member| member.id == me.id)
    } else if let Some(_) = message.group_chat_created() {
        true
    } else {
        false
    }
}

pub fn msg_from_maintainer(config: Arc<AppConfig>, message: Message) -> bool {
    message.from.map(|user| user.id == config.bot_maintainer).unwrap_or_default()
}

pub fn has_shared_users(message: Message) -> bool {
    if let MessageKind::UsersShared(_) = message.kind { true } else { false }
}