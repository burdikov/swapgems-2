use teloxide::prelude::*;

pub struct SwappyBot {
    pub bot: Bot,
    group_id: ChatId,
}

pub trait ToSwappyBot {
    fn to_swappy_bot(self, group_id: ChatId) -> SwappyBot;
}

impl ToSwappyBot for Bot {
    fn to_swappy_bot(self, group_id: ChatId) -> SwappyBot {
        SwappyBot {
            bot: self,
            group_id,
        }
    }
}

impl SwappyBot {

}