use std::fmt::{Display, Formatter};
use teloxide::macros::BotCommands;
use teloxide::types::MessageId;
use crate::bot::commands::CallbackQueryCommand::{Delete, Edit, Repost};

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
pub enum SimpleCommand {
    /// Начало работы
    Start,
    /// Узнать количество ваших ⭐️
    MyStars,
    /// Описание бота
    Help,
    /// О публикации объявлений
    Posting,
    /// О звёздах
    Stars,
    /// О безопасности сделок
    Safety,
    /// О хранимых данных
    PersonalData
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

        let id = MessageId(id.parse().ok()?);
        match cmd {
            "del" => Some(Delete(id)),
            "edit" => Some(Edit(id)) ,
            "repost" => Some(Repost(id)),
            _ => None,
        }
    }
}