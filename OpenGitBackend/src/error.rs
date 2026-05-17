use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Authentication required")]
    Unauthorized,
    #[error("Access forbidden")]
    Forbidden,
    #[error("{0} not found")]
    NotFound(String),
    #[error("{0}")]
    BadRequest(String),
    #[error("{0} already exists")]
    Conflict(String),
    #[error("Internal server error")]
    Internal(#[from] anyhow::Error),
    #[error("Database error")]
    Database(#[from] sqlx::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::Unauthorized      => (StatusCode::UNAUTHORIZED, self.to_string()),
            AppError::Forbidden         => (StatusCode::FORBIDDEN, self.to_string()),
            AppError::NotFound(m)       => (StatusCode::NOT_FOUND, m.clone()),
            AppError::BadRequest(m)     => (StatusCode::BAD_REQUEST, m.clone()),
            AppError::Conflict(m)       => (StatusCode::CONFLICT, m.clone()),
            AppError::Internal(_) | AppError::Database(_) => {
                tracing::error!("Internal error: {:?}", self);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            }
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}