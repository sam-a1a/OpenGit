use crate::{
    error::AppError,
    api::middleware::auth::AuthUser,
    services::auth::{register, login, refresh_tokens, logout, RegisterInput, LoginInput, RefreshInput},
    state::AppState,
};
use axum::{extract::State, http::{HeaderMap, StatusCode}, response::IntoResponse, Json};
use serde_json::json;

pub async fn register_handler(
    State(state): State<AppState>,
    headers:      HeaderMap,
    Json(input):  Json<RegisterInput>,
) -> Result<impl IntoResponse, AppError> {
    let ip = headers.get("X-Forwarded-For")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let ua = headers.get("User-Agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let output = register(
        &state.db,
        input,
        &state.config.jwt_secret,
        ip.as_deref(),
        ua.as_deref(),
    ).await?;

    Ok((StatusCode::CREATED, Json(output)))
}

pub async fn login_handler(
    State(state): State<AppState>,
    headers:      HeaderMap,
    Json(input):  Json<LoginInput>,
) -> Result<impl IntoResponse, AppError> {
    let ip = headers.get("X-Forwarded-For")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let ua = headers.get("User-Agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let output = login(
        &state.db,
        input,
        &state.config.jwt_secret,
        ip.as_deref(),
        ua.as_deref(),
    ).await?;

    Ok(Json(output))
}

pub async fn refresh_handler(
    State(state): State<AppState>,
    headers:      HeaderMap,
    Json(input):  Json<RefreshInput>,
) -> Result<impl IntoResponse, AppError> {
    let ip = headers.get("X-Forwarded-For")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let ua = headers.get("User-Agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let output = refresh_tokens(
        &state.db,
        input,
        &state.config.jwt_secret,
        ip.as_deref(),
        ua.as_deref(),
    ).await?;

    Ok(Json(output))
}

pub async fn logout_handler(
    State(state): State<AppState>,
    auth_user:    AuthUser,
) -> Result<impl IntoResponse, AppError> {
    logout(&state.db, auth_user.user_id).await?;
    Ok(Json(json!({ "message": "Logged out successfully" })))
}

pub async fn me_handler(
    State(state): State<AppState>,
    auth_user:    AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let user = crate::db::queries::users::find_by_id(&state.db, auth_user.user_id)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;
    Ok(Json(user))
}