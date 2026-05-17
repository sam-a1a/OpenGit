use crate::{
    api::routes::{
        auth::{login_handler, logout_handler, me_handler, refresh_handler, register_handler},
        health::health_check,
    },
    state::AppState,
};
use axum::{
    routing::{get, post},
    Router,
};

pub fn build(state: AppState) -> Router {
    Router::new()
        // health
        .route("/health", get(health_check))
        // auth
        .route("/api/v1/auth/register", post(register_handler))
        .route("/api/v1/auth/login",    post(login_handler))
        .route("/api/v1/auth/refresh",  post(refresh_handler))
        .route("/api/v1/auth/logout",   post(logout_handler))
        .route("/api/v1/auth/me",       get(me_handler))
        .with_state(state)
}