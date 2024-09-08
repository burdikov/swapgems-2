use std::fmt::{Display, Formatter};
use std::ptr::write;
use teloxide::macros::BotCommands;
use teloxide::types::MessageId;
use crate::bot::commands::CallbackQueryCommand::{Delete, Edit, Repost};

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
pub enum SimpleCommand {
    /// Начало работы
    Start,
    /// Эта подсказка
    Help,
    /// Узнать свой UserId
    MyId,
    /// Узнать UserId хозяина бота
    Maintainer,
}

#[derive(BotCommands, Clone, Debug)]
#[command(rename_rule = "lowercase")]
pub enum MaintainerCommand {
    /// Show chat id of currently assigned group
    GetGroup,
    /// Assign new group
    SetGroup(i64),
    /// Send a test message to a target group
    TestMsg,
}

#[derive(Clone, Debug)]
pub enum CallbackQueryCommand {
    Delete(MessageId),
    Edit(MessageId),
    Repost(MessageId)
}

impl Display for CallbackQueryCommand {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Delete(id) => write!(f, "del:{}", id),
            Edit(id) => write!(f, "edit:{}", id),
            Repost(id) => write!(f, "repost:{}", id),
        }
    }
}

impl CallbackQueryCommand {
    pub fn parse(s: &str) -> Option<Self> {
        let cmd = s.split_once(':')?;

        let (cmd, id) = cmd;
        let id = if let Ok(id) = id.parse::<i32>() {
            MessageId(id)
        } else {
            return None
        };

        match cmd {
            "del" => Some(Delete(id)),
            "edit" => Some(Edit(id)) ,
            "repost" => Some(Repost(id)),
            _ => None,
        }
    }
}