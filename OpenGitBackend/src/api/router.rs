use axum::Router;
use crate::state::AppState;

pub fn build(state: AppState) -> Router {
    Router::new()
        .with_state(state)
}