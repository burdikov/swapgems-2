use std::sync::atomic::Ordering;
use redis::{Commands, RedisResult};
use teloxide::prelude::UserId;
use teloxide::RequestError;
use teloxide::types::{ChatId, MessageId};
use crate::types::AppConfig;

pub mod types;
pub mod bot;
pub mod site;
pub mod store;

