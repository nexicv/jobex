//! Jobex assistant proxy. Created by Claude Opus 5.5, 2026-11-06 Europe/Warsaw.

pub mod chat;
pub mod config;
pub mod deepseek;
pub mod limits;
pub mod prompt;
pub mod quiet;
pub mod seal;
pub mod team;

use axum::Router;
use axum::http::{HeaderName, HeaderValue, Method};
use axum::routing::{get, post};
use std::sync::Arc;
use tower_http::cors::{AllowOrigin, CorsLayer};

pub fn router(app: Arc<chat::App>) -> Router {
    let origins: Vec<HeaderValue> = app.cfg.allowed_origins.iter().filter_map(|o| HeaderValue::from_str(o).ok()).collect();
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([axum::http::header::CONTENT_TYPE, HeaderName::from_static("x-jobex-team")])
        .max_age(std::time::Duration::from_secs(3600));
    Router::new()
        .route("/healthz", get(chat::healthz))
        .route("/v1/status", get(chat::status))
        .route("/v1/chat", post(chat::chat))
        .layer(axum::extract::DefaultBodyLimit::max(256 * 1024))
        .layer(cors)
        .with_state(app)
}
