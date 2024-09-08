use std::sync::Arc;
use axum::extract::{RawForm, State};
use axum::http;
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::IntoResponse;
use tokio::time::Instant;
use url::Url;
use super::tg;
use crate::types::AppConfig;
use super::init_data;
use std::borrow::Borrow;
use teloxide::types::User;
use crate::site::form::{Form, FormParseError};
use crate::site::init_data::Error;

pub async fn handle_posting(
    headers: HeaderMap,
    State(app_config): State<Arc<AppConfig>>,
    RawForm(bytes): RawForm,
) -> impl IntoResponse {
    let mut resp_headers = HeaderMap::new();
    add_access_control_headers(&mut resp_headers, &app_config.app_url);

    let user;
    let now = Instant::now();
    if let Some(data) = headers.get("X-Telegram-Init-Data") {
        user = match init_data::validate(data.as_bytes(), app_config.bot_token.as_bytes(), false) {
            Ok(u) => u,
            Err(e) => {
                println!("Some shit with init data: {e:?}");
                return (
                    StatusCode::BAD_REQUEST,
                    resp_headers,
                    String::default(),
                );
            }
        }
    } else {
        return (
            StatusCode::BAD_REQUEST,
            resp_headers,
            String::default(),
        );
    }

    let elapsed = now.elapsed();
    println!("Validating init data took {}ns", elapsed.as_nanos());

    let form_data = Form::try_from(bytes.borrow());
    println!("{form_data:?}");
    if let Err(e) = form_data {
        let msg = if let FormParseError::Invalid(s) = e {
            s.to_string()
        } else {
            "Что-то пошло не так".to_string()
        };

        return (
            StatusCode::BAD_REQUEST,
            resp_headers,
            msg,
        );
    }

    let form_data = form_data.unwrap();

    tg::handle_shit(
        app_config.borrow(),
        form_data,
        &user
    ).await.expect("TODO: panic message");

    let elapsed = now.elapsed();
    println!("Handling shit took {}ns", elapsed.as_nanos());

    (
        StatusCode::OK,
        resp_headers,
        String::default(),
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