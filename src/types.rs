pub mod swappy_user;
pub mod swappy_bot;

pub use swappy_user::{SwappyUser, ToSwappyUser};


use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};
use teloxide::Bot;
use teloxide::prelude::UserId;
use teloxide::types::ChatId;
use url::Url;

// #[derive(Clone)]
pub struct AppConfig {
    pub app_url: Url,
    pub bot: Bot,
    pub redis_client: redis::Client,
    pub bot_maintainer: UserId,
    pub group_id: Arc<AtomicI64>,
    pub bot_token: String,
}

impl AppConfig {
    pub fn group_id(&self) -> ChatId {
        ChatId(self.group_id.load(Ordering::Relaxed))
    }

    pub fn set_group_id(&self, gid: i64) {
        self.group_id.store(gid, Ordering::Relaxed)
    }
}