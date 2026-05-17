use crate::{error::AppError, services::auth::validate_access_token, state::AppState};
use axum::{
    extract::{FromRequestParts, State},
    http::{request::Parts, HeaderMap},
};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub role:    String,
}

impl AuthUser {
    pub fn from_headers(headers: &HeaderMap, secret: &str) -> Result<Self, AppError> {
        let auth_header = headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or(AppError::Unauthorized)?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or(AppError::Unauthorized)?;

        let claims = validate_access_token(token, secret)?;

        let user_id = claims.sub.parse::<Uuid>()
            .map_err(|_| AppError::Unauthorized)?;

        Ok(AuthUser { user_id, role: claims.role })
    }
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        AuthUser::from_headers(&parts.headers, &state.config.jwt_secret)
    }
}