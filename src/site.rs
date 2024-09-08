use std::sync::Arc;
use axum::routing::{options, post, Router};

mod handlers;
mod init_data;
mod form;
mod tg;

use handlers::{
    handle_posting,
    r_options
};

use crate::types::AppConfig;

pub fn add_routes(router: Router, state: Arc<AppConfig>) -> Router {
    router
        .route("/bot/form", post(handle_posting).with_state(Arc::clone(&state)))
        .route("/bot/form", options(r_options).with_state(Arc::clone(&state)))
}