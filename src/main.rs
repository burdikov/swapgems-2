use std::env;
use std::sync::atomic::AtomicI64;
use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::update_listeners::UpdateListener;

use redis;
use redis::Commands;
use teloxide::types::{MenuButton, WebAppInfo};
use teloxide::update_listeners;
use teloxide::update_listeners::webhooks::Options;
use teloxide::utils::command::BotCommands;
use update_listeners::webhooks;
use webhooks::axum_to_router;
use swappy2::bot;
use swappy2::bot::TARGET_GROUP_ID_KEY;
use swappy2::site::add_routes;
use swappy2::types::AppConfig;
use url::Url;
use swappy2::bot::commands::SimpleCommand;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let bot_url = env::var("BOT_DOMAIN").expect("expected BOT_DOMAIN")
        .parse().expect("BOT_DOMAIN should be valid url");

    let redis_url = env::var("REDIS_URL").expect("expected REDIS_URL")
        .parse::<Url>().expect("REDIS_URL should be valid url");

    let client = redis::Client::open(redis_url).unwrap();
    let mut con = client.get_connection().expect("redis should be running");
    let group_id: i64 = con.get(TARGET_GROUP_ID_KEY).unwrap_or_default();

    let bot_token = env::var("BOT_TOKEN").expect("expected BOT_TOKEN");
    let maintainer_id = env::var("BOT_MAINTAINER").expect("expected BOT_MAINTAINER")
        .parse::<u64>().expect("BOT_MAINTAINER should be u64");
    let config = Arc::new(AppConfig {
        app_url: env::var("APP_DOMAIN").expect("expected APP_DOMAIN")
            .parse().expect("APP_DOMAIN should be valid url"),
        bot: Bot::new(&bot_token),
        bot_token,
        redis_client: client,
        bot_maintainer: UserId(maintainer_id),
        group_id: Arc::new(AtomicI64::new(group_id)),
    });

    let menu_button = MenuButton::WebApp {
        text: "Swappy".to_string(),
        web_app: WebAppInfo { url: config.app_url.clone() },
    };

    config.bot.set_chat_menu_button().menu_button(menu_button)
        .await.expect("should be able to change menu button");

    config.bot.set_my_commands(SimpleCommand::bot_commands()).await.expect("");

    let handler = bot::build_handler();
    let addr = ([0,0,0,0],8443).into();

    let (mut listener, stop_flag, router) = axum_to_router(
        config.bot.clone(),
        Options::new(addr, bot_url),
    ).await.expect("should be able to set webhook");


    let router = add_routes(router, Arc::clone(&config));
    let stop_token = listener.stop_token();

    tokio::spawn(async move {
        let tcp_listener = tokio::net::TcpListener::bind(addr)
            .await.map_err(|err|
            {
                stop_token.stop();
                err
            })
            .expect("should be able to bind");

        axum::serve(tcp_listener, router)
            .with_graceful_shutdown(stop_flag)
            .await.map_err(|e|
            {
                stop_token.stop();
                e
            })
            .expect("axum server error");
    });

    let error_handler =
        LoggingErrorHandler::with_custom_text("An error from the update listener");
    Dispatcher::builder(config.bot.clone(), handler)
        .dependencies(dptree::deps![config])
        .error_handler(LoggingErrorHandler::with_custom_text(
            "something went wrong",
        ))
        .default_handler(default)
        .enable_ctrlc_handler()
        .build()
        .dispatch_with_listener(listener, error_handler).await;
}

async fn default(update: Arc<Update>) {
    println!("{update:?}");
}