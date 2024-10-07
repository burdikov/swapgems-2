use std::fmt::{Display, Formatter};
use std::sync::atomic::Ordering;
use redis::{AsyncCommands, RedisResult};
use redis::aio::MultiplexedConnection;
use teloxide::{
    prelude::*,
    RequestError,
    types::{
        MessageId
    },
};
use crate::types::AppConfig;

pub trait ToSwappyUser<'a> {
    fn with_config(self, app_config: &'a AppConfig) -> impl std::future::Future<Output = SwappyUser> + Send;
}

impl<'a> ToSwappyUser<'a> for teloxide::types::User {
    async fn with_config(self, app_config: &'a AppConfig) -> SwappyUser {
        SwappyUser {
            group_id: ChatId(app_config.group_id.load(Ordering::Relaxed)),
            config: app_config,
            tg_user: self,
            redis_conn: app_config.redis_client.get_multiplexed_async_connection().await.unwrap()
        }
    }
}

pub struct SwappyUser<'a> {
    pub group_id: ChatId,
    pub config: &'a AppConfig,
    pub tg_user: teloxide::types::User,
    redis_conn: MultiplexedConnection,
}

impl<'a> SwappyUser<'a> {
    fn ads_key(&self) -> String {
        format!("{}:{}:ads", self.group_id.0, self.tg_user.id.0)
    }

    fn stars_key(&self) -> String {
        format!("{}:{}:stars", self.group_id.0, self.tg_user.id.0)
    }

    pub async fn is_group_member(&self) -> Result<bool, RequestError> {
        let o = self.config.bot.get_chat_member(self.group_id, self.tg_user.id).await?;

        Ok(o.kind.is_present())
    }

    pub async fn star_count(&mut self) -> RedisResult<usize> {
        self.redis_conn.scard(self.stars_key()).await
    }

    pub async fn set_author(&mut self, message_id: MessageId) -> RedisResult<()> {
        self.redis_conn.sadd(self.ads_key(), message_id.0).await
    }

    pub async fn is_author(&mut self, message_id: MessageId) -> RedisResult<bool> {
        self.redis_conn.sismember(self.ads_key(), message_id.0).await
    }
}

impl Display for SwappyUser<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "<a href=\"{}\">{}</a>",
               self.tg_user.url(),
               self.tg_user.full_name().trim()
        )
    }
}