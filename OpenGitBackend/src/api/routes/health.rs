use crate::state::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use redis::AsyncCommands;
use serde::Serialize;

#[derive(Serialize)]
pub struct HealthResponse {
    pub status:   &'static str,
    pub postgres: ServiceStatus,
    pub valkey:   ServiceStatus,
}

#[derive(Serialize)]
pub struct ServiceStatus {
    pub ok:      bool,
    pub latency: String,
}

pub async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    let pg_start = std::time::Instant::now();
    let pg_ok = sqlx::query("SELECT 1")
        .execute(&state.db)
        .await
        .is_ok();
    let pg_latency = format!("{}ms", pg_start.elapsed().as_millis());

    let valkey_start = std::time::Instant::now();
    let mut cache = state.cache.clone();
    let valkey_ok = cache.ping::<String>().await.is_ok();
    let valkey_latency = format!("{}ms", valkey_start.elapsed().as_millis());

    let all_ok = pg_ok && valkey_ok;

    let status_code = if all_ok {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    let body = Json(HealthResponse {
        status:   if all_ok { "ok" } else { "degraded" },
        postgres: ServiceStatus { ok: pg_ok,     latency: pg_latency },
        valkey:   ServiceStatus { ok: valkey_ok, latency: valkey_latency },
    });

    (status_code, body)
}