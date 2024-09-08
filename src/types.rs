use std::sync::Arc;
use std::sync::atomic::AtomicI64;
use teloxide::Bot;
use teloxide::prelude::UserId;
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