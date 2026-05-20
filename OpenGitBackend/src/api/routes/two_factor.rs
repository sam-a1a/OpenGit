// src/api/routes/two_factor.rs
use crate::{
    api::middleware::auth::AuthUser,
    error::AppError,
    state::AppState,
};
use axum::{
    extract::State,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use totp_rs::{Algorithm, Secret, TOTP};
use uuid::Uuid;

// Helper Functions

fn make_totp(secret_b32: &str, email: &str) -> Result<TOTP, AppError> {
    let secret = Secret::Encoded(secret_b32.to_string())
        .to_bytes()
        .map_err(|e| AppError::Internal(anyhow::anyhow!("TOTP secret error: {}", e)))?;

    TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        secret,
        Some("OpenGit".to_string()),
        email.to_string(),
    )
        .map_err(|e| AppError::Internal(anyhow::anyhow!("TOTP error: {}", e)))
}

fn generate_backup_code() -> String {
    let uuid = Uuid::new_v4().simple().to_string();
    format!("{}-{}", &uuid[..5], &uuid[5..10]).to_uppercase()
}

fn hash_code(code: &str) -> String {
    blake3::hash(code.as_bytes()).to_hex().to_string()
}

// Helper to fetch the primary email of a user, fallback to username if not found
async fn get_user_primary_email(
    db: &sqlx::PgPool,
    user_id: Uuid,
    username: &str,
) -> Result<String, AppError> {
    let email: Option<(String,)> = sqlx::query_as(
        "SELECT email FROM user_emails WHERE user_id = $1 AND is_primary = true"
    )
        .bind(user_id)
        .fetch_optional(db)
        .await
        .map_err(AppError::Database)?;

    Ok(email.map(|e| e.0).unwrap_or_else(|| username.to_string()))
}

// Helper to fetch the 2FA secret from the database
async fn get_two_factor_secret(
    db: &sqlx::PgPool,
    user_id: Uuid,
) -> Result<Option<String>, AppError> {
    sqlx::query_scalar("SELECT two_factor_secret FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(db)
        .await
        .map_err(AppError::Database)
}

// Route Handlers

#[derive(Serialize)]
pub struct SetupResponse {
    pub secret: String,
    pub otpauth_url: String,
    pub qr_url: String,
}

pub async fn setup_2fa(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let user = crate::db::queries::users::find_by_id(&state.db, auth_user.user_id)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    if user.two_factor_enabled {
        return Err(AppError::BadRequest("2FA is already enabled".into()));
    }

    let secret = Secret::generate_secret();
    let secret_b32 = secret.to_encoded().to_string();

    let email = get_user_primary_email(&state.db, user.id, &user.username).await?;

    sqlx::query("UPDATE users SET two_factor_secret = $1 WHERE id = $2")
        .bind(&secret_b32)
        .bind(user.id)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    let totp = make_totp(&secret_b32, &email)?;
    let otpauth = totp.get_url();
    let qr_url = format!(
        "https://api.qrserver.com/v1/create-qr-code/?size=200x200&data={}",
        urlencoding::encode(&otpauth)
    );

    Ok(Json(SetupResponse {
        secret: secret_b32,
        otpauth_url: otpauth,
        qr_url,
    }))
}

#[derive(Deserialize)]
pub struct VerifyInput {
    pub code: String,
}

pub async fn enable_2fa(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(input): Json<VerifyInput>,
) -> Result<impl IntoResponse, AppError> {
    let user = crate::db::queries::users::find_by_id(&state.db, auth_user.user_id)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    if user.two_factor_enabled {
        return Err(AppError::BadRequest("2FA is already enabled".into()));
    }

    let secret = get_two_factor_secret(&state.db, user.id)
        .await?
        .ok_or(AppError::BadRequest("Run setup first".into()))?;

    let email = get_user_primary_email(&state.db, user.id, &user.username).await?;
    let totp = make_totp(&secret, &email)?;

    if !totp.check_current(&input.code)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("TOTP check error: {}", e)))? {
        return Err(AppError::BadRequest("Invalid verification code".into()));
    }

    sqlx::query("UPDATE users SET two_factor_enabled = true, updated_at = now() WHERE id = $1")
        .bind(user.id)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    let codes = generate_backup_codes(&state, user.id).await?;

    Ok(Json(json!({
        "message":      "2FA enabled successfully",
        "backup_codes": codes,
        "warning":      "Save these backup codes — they will not be shown again",
    })))
}

#[derive(Deserialize)]
pub struct DisableInput {
    pub code: Option<String>,
    pub password: Option<String>,
}

pub async fn disable_2fa(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(input): Json<DisableInput>,
) -> Result<impl IntoResponse, AppError> {
    let user = crate::db::queries::users::find_by_id(&state.db, auth_user.user_id)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    if !user.two_factor_enabled {
        return Err(AppError::BadRequest("2FA is not enabled".into()));
    }

    if let Some(ref code) = input.code {
        let secret = get_two_factor_secret(&state.db, user.id)
            .await?
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("No secret stored")))?;

        let email = get_user_primary_email(&state.db, user.id, &user.username).await?;
        let totp = make_totp(&secret, &email)?;

        if !totp.check_current(code)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("{}", e)))? {
            return Err(AppError::BadRequest("Invalid verification code".into()));
        }
    } else if let Some(ref password) = input.password {
        let hash = user.password_hash.as_deref()
            .ok_or(AppError::BadRequest("No password set".into()))?;

        if !crate::services::auth::verify_password(password, hash)? {
            return Err(AppError::Unauthorized);
        }
    } else {
        return Err(AppError::BadRequest(
            "Provide either a TOTP code or your password".into()
        ));
    }

    sqlx::query(
        "UPDATE users SET
            two_factor_enabled = false,
            two_factor_secret  = NULL,
            updated_at         = now()
         WHERE id = $1"
    )
        .bind(user.id)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    sqlx::query("DELETE FROM user_backup_codes WHERE user_id = $1")
        .bind(user.id)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(json!({ "message": "2FA disabled successfully" })))
}

pub async fn get_2fa_status(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let user = crate::db::queries::users::find_by_id(&state.db, auth_user.user_id)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    let backup_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM user_backup_codes WHERE user_id = $1 AND used = false"
    )
        .bind(user.id)
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(json!({
        "enabled":              user.two_factor_enabled,
        "backup_codes_remaining": backup_count.0,
    })))
}

pub async fn regenerate_backup_codes(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(input): Json<VerifyInput>,
) -> Result<impl IntoResponse, AppError> {
    let user = crate::db::queries::users::find_by_id(&state.db, auth_user.user_id)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    if !user.two_factor_enabled {
        return Err(AppError::BadRequest("2FA is not enabled".into()));
    }

    let secret = get_two_factor_secret(&state.db, user.id)
        .await?
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("No secret stored")))?;

    let email = get_user_primary_email(&state.db, user.id, &user.username).await?;
    let totp = make_totp(&secret, &email)?;

    if !totp.check_current(&input.code)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("{}", e)))? {
        return Err(AppError::BadRequest("Invalid verification code".into()));
    }

    sqlx::query("DELETE FROM user_backup_codes WHERE user_id = $1")
        .bind(user.id)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    let codes = generate_backup_codes(&state, user.id).await?;

    Ok(Json(json!({
        "backup_codes": codes,
        "warning":      "Save these backup codes — they will not be shown again",
    })))
}

#[derive(Deserialize)]
pub struct TwoFactorLoginInput {
    pub pending_token: String,
    pub code: String,
}

pub async fn verify_2fa_login(
    State(state): State<AppState>,
    Json(input): Json<TwoFactorLoginInput>,
) -> Result<impl IntoResponse, AppError> {
    let claims = crate::services::auth::validate_access_token(
        &input.pending_token,
        &state.config.jwt_secret,
    )?;

    if claims.role != "pending_2fa" {
        return Err(AppError::Unauthorized);
    }

    let user_id = claims.sub.parse::<Uuid>()
        .map_err(|_| AppError::Unauthorized)?;

    let user = crate::db::queries::users::find_by_id(&state.db, user_id)
        .await?
        .ok_or(AppError::Unauthorized)?;

    let secret = get_two_factor_secret(&state.db, user.id)
        .await?
        .ok_or(AppError::Unauthorized)?;

    let email = get_user_primary_email(&state.db, user.id, &user.username).await?;
    let totp = make_totp(&secret, &email)?;

    let valid_totp = totp.check_current(&input.code).unwrap_or(false);

    if !valid_totp {
        let code_hash = hash_code(&input.code.to_uppercase());

        let backup: Option<(Uuid,)> = sqlx::query_as(
            "SELECT id FROM user_backup_codes
             WHERE user_id = $1 AND code_hash = $2 AND used = false"
        )
            .bind(user.id)
            .bind(&code_hash)
            .fetch_optional(&state.db)
            .await
            .map_err(AppError::Database)?;

        if let Some((code_id, )) = backup {
            sqlx::query("UPDATE user_backup_codes SET used = true, used_at = now() WHERE id = $1")
                .bind(code_id)
                .execute(&state.db)
                .await
                .map_err(AppError::Database)?;
        } else {
            return Err(AppError::BadRequest("Invalid code".into()));
        }
    }

    let access_token = crate::services::auth::generate_access_token(
        user.id,
        &format!("{:?}", user.role),
        &state.config.jwt_secret,
    )?;

    let raw_refresh = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
    let token_hash = blake3::hash(raw_refresh.as_bytes()).to_hex().to_string();
    let family_id = Uuid::new_v4();

    let session = crate::db::queries::tokens::create_session(
        &state.db, user.id, None, None,
    ).await?;

    crate::db::queries::tokens::create_refresh_token(
        &state.db, user.id, &token_hash, family_id, session.id, None,
    ).await?;

    Ok(Json(json!({
        "access_token":  access_token,
        "refresh_token": raw_refresh,
        "token_type":    "Bearer",
        "expires_in":    900,
        "user":          user,
    })))
}

async fn generate_backup_codes(state: &AppState, user_id: Uuid) -> Result<Vec<String>, AppError> {
    let mut plain_codes = Vec::new();

    for _ in 0..10 {
        let code = generate_backup_code();
        let code_hash = hash_code(&code);

        sqlx::query("INSERT INTO user_backup_codes (user_id, code_hash) VALUES ($1, $2)")
            .bind(user_id)
            .bind(&code_hash)
            .execute(&state.db)
            .await
            .map_err(AppError::Database)?;

        plain_codes.push(code);
    }

    Ok(plain_codes)
}