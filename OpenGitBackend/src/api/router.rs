use crate::{api::routes::health::health_check, state::AppState};
use axum::{routing::get, Router};

pub fn build(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .with_state(state)
}