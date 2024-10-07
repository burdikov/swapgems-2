use super::init_data;
use super::tg;
use crate::site::form::Form;
use crate::types::{AppConfig, SwappyUser, ToSwappyUser};
use axum::extract::{Query, State};
use axum::http;
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::IntoResponse;
use std::borrow::Borrow;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use serde::Deserialize;
use teloxide::{ApiError, RequestError};
use teloxide::types::ChatId;
use tokio::time::Instant;
use url::Url;
use init_data::validate;
use crate::site::init_data::Error;
use crate::types::swappy_bot::ToSwappyBot;

#[derive(Deserialize, Debug)]
pub struct PostParams {
    pub edit_id: Option<i32>,
    pub report_id: Option<i32>,
    pub keeping: bool,
}

pub async fn handle_posting(
    headers: HeaderMap,
    State(app_config): State<Arc<AppConfig>>,
    query: Query<PostParams>,
    bytes: axum::body::Bytes,
) -> impl IntoResponse {
    let now = Instant::now();

    let mut resp_headers = HeaderMap::new();
    add_access_control_headers(&mut resp_headers, &app_config.app_url);

    // validate init data
    let data = if let Some(data) = headers.get("X-Telegram-Init-Data") { data.as_bytes() } else {
        return (StatusCode::UNAUTHORIZED, resp_headers, String::default())
    };

    let tg_user =
        if let Some(user) = validate(data, app_config.bot_token.as_bytes(), false).ok() {
            user
        } else {
            return (StatusCode::UNAUTHORIZED, resp_headers, String::default())
        };

    // check if user is a part of the group
    let sw_user = tg_user.with_config(&app_config).await;
    match sw_user.is_group_member().await {
        Ok(true) => {} // continue
        Ok(false) => return (StatusCode::FORBIDDEN, resp_headers, String::default()),
        Err(e) => {
            log::error!("member check failed: {}", e.to_string());
            return (StatusCode::INTERNAL_SERVER_ERROR, resp_headers, "Try later".to_string())
        }
    }

    // parse form
    let form_data: serde_json::Result<Form> = serde_json::from_slice(&bytes);
    if form_data.is_err() {
        return (StatusCode::BAD_REQUEST, resp_headers, "Form error".to_string())
    }
    let form_data = form_data.unwrap();

    let (msg_id, report_id) = tg::handle_shit(
        app_config.borrow(),
        query.0,
        form_data,
        sw_user,
    ).await.expect("TODO: panic message");

    let elapsed = now.elapsed();
    println!("request took {}microsecs", elapsed.as_micros());

    (
        StatusCode::OK,
        resp_headers,
        format!("{},{}", msg_id, report_id),
    )
}

pub async fn r_options(
    State(app_config): State<Arc<AppConfig>>,
) -> impl IntoResponse {
    let mut resp_headers = HeaderMap::new();
    add_access_control_headers(&mut resp_headers, &app_config.app_url);

    (
        StatusCode::OK,
        resp_headers
    )
}

fn add_access_control_headers(resp_headers: &mut HeaderMap, app_url: &Url) {
    resp_headers.insert(header::ACCESS_CONTROL_ALLOW_HEADERS, "X-Telegram-Init-Data".parse().unwrap());
    resp_headers.insert(header::ACCESS_CONTROL_ALLOW_METHODS, http::Method::POST.as_str().parse().unwrap());

    let origin = app_url.origin().ascii_serialization();
    resp_headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, origin.parse().unwrap());
}