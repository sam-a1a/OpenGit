use crate::{
    db::queries::{tokens, users},
    error::AppError,
    models::user::User,
};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

// JWT

const ACCESS_TOKEN_EXPIRY_SECS: i64 = 15 * 60;       // 15 minutes
const REFRESH_TOKEN_EXPIRY_SECS: i64 = 7 * 24 * 3600; // 7 days

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub:  String, // user_id
    pub role: String,
    pub exp:  usize,
    pub iat:  usize,
}

pub fn generate_access_token(user_id: Uuid, role: &str, secret: &str) -> Result<String, AppError> {
    let now = Utc::now().timestamp();
    let claims = Claims {
        sub:  user_id.to_string(),
        role: role.to_string(),
        exp:  (now + ACCESS_TOKEN_EXPIRY_SECS) as usize,
        iat:  now as usize,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
        .map_err(|e| AppError::Internal(anyhow::anyhow!("JWT encode failed: {}", e)))
}

pub fn validate_access_token(token: &str, secret: &str) -> Result<Claims, AppError> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
        .map(|d| d.claims)
        .map_err(|_| AppError::Unauthorized)
}

// Password

pub fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Hash failed: {}", e)))
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    let parsed = PasswordHash::new(hash)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Invalid hash: {}", e)))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}

// Token helpers

fn generate_opaque_token() -> String {
    format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple())
}

fn hash_token(token: &str) -> String {
    blake3::hash(token.as_bytes()).to_hex().to_string()
}

// Auth output

#[derive(Debug, Serialize)]
pub struct AuthOutput {
    pub access_token:   String,
    pub refresh_token:  String,
    pub token_type:     &'static str,
    pub expires_in:     i64,
    pub user:           User,
}

// Register

#[derive(Debug, Deserialize)]
pub struct RegisterInput {
    pub username: String,
    pub email:    String,
    pub password: String,
}

pub async fn register(
    pool:   &PgPool,
    input:  RegisterInput,
    secret: &str,
    ip:     Option<&str>,
    ua:     Option<&str>,
) -> Result<AuthOutput, AppError> {
    // validate
    if input.username.len() < 3 {
        return Err(AppError::BadRequest("Username must be at least 3 characters".into()));
    }
    if input.password.len() < 8 {
        return Err(AppError::BadRequest("Password must be at least 8 characters".into()));
    }
    if !input.email.contains('@') {
        return Err(AppError::BadRequest("Invalid email address".into()));
    }

    // uniqueness checks
    if users::username_exists(pool, &input.username).await? {
        return Err(AppError::Conflict("Username".into()));
    }
    if users::email_exists(pool, &input.email).await? {
        return Err(AppError::Conflict("Email".into()));
    }

    // create user
    let password_hash = hash_password(&input.password)?;
    let user = users::create_user(pool, &input.username, &password_hash).await?;
    users::create_user_email(pool, user.id, &input.email).await?;

    // issue tokens
    issue_tokens(pool, user, secret, ip, ua).await
}

// Login

#[derive(Debug, Deserialize)]
pub struct LoginInput {
    pub email:    String,
    pub password: String,
}

pub async fn login(
    pool:   &PgPool,
    input:  LoginInput,
    secret: &str,
    ip:     Option<&str>,
    ua:     Option<&str>,
) -> Result<serde_json::Value, AppError> {
    let user = users::find_by_email(pool, &input.email)
        .await?
        .ok_or(AppError::Unauthorized)?;

    let hash = user.password_hash.as_deref()
        .ok_or(AppError::BadRequest("Use OAuth to sign in".into()))?;

    if !verify_password(&input.password, hash)? {
        return Err(AppError::Unauthorized);
    }

    // if 2FA enabled return pending token
    if user.two_factor_enabled {
        let pending_token = generate_access_token(user.id, "pending_2fa", secret)?;
        return Ok(serde_json::json!({
            "two_factor_required": true,
            "pending_token":       pending_token,
        }));
    }

    let output = issue_tokens(pool, user, secret, ip, ua).await?;
    Ok(serde_json::to_value(output)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("{}", e)))?)
}

// Refresh

#[derive(Debug, Deserialize)]
pub struct RefreshInput {
    pub refresh_token: String,
}

pub async fn refresh_tokens(
    pool:   &PgPool,
    input:  RefreshInput,
    secret: &str,
    ip:     Option<&str>,
    ua:     Option<&str>,
) -> Result<AuthOutput, AppError> {
    let token_hash = hash_token(&input.refresh_token);

    let stored = tokens::find_refresh_token(pool, &token_hash)
        .await?
        .ok_or(AppError::Unauthorized)?;

    // family invalidation — if token was already used, someone is replaying it
    if stored.used {
        tokens::invalidate_family(pool, stored.family_id).await?;
        return Err(AppError::Unauthorized);
    }

    tokens::mark_token_used(pool, stored.id).await?;

    let user = users::find_by_id(pool, stored.user_id)
        .await?
        .ok_or(AppError::Unauthorized)?;

    // new tokens, same family
    let access_token   = generate_access_token(user.id, &format!("{:?}", user.role), secret)?;
    let raw_refresh    = generate_opaque_token();
    let new_hash       = hash_token(&raw_refresh);

    let session_id = stored.session_id.unwrap_or(Uuid::new_v4());

    tokens::create_refresh_token(
        pool, user.id, &new_hash, stored.family_id, session_id, ip,
    ).await?;

    Ok(AuthOutput {
        access_token,
        refresh_token: raw_refresh,
        token_type: "Bearer",
        expires_in: ACCESS_TOKEN_EXPIRY_SECS,
        user,
    })
}

// Logout

pub async fn logout(pool: &PgPool, session_id: Uuid) -> Result<(), AppError> {
    tokens::invalidate_family(pool, session_id).await?;
    tokens::delete_session(pool, session_id).await?;
    Ok(())
}

// Internal helpers

async fn issue_tokens(
    pool:   &PgPool,
    user:   User,
    secret: &str,
    ip:     Option<&str>,
    ua:     Option<&str>,
) -> Result<AuthOutput, AppError> {
    let access_token  = generate_access_token(user.id, &format!("{:?}", user.role), secret)?;
    let raw_refresh   = generate_opaque_token();
    let token_hash    = hash_token(&raw_refresh);
    let family_id     = Uuid::new_v4();

    let session = tokens::create_session(pool, user.id, ip, ua).await?;
    tokens::create_refresh_token(pool, user.id, &token_hash, family_id, session.id, ip).await?;

    Ok(AuthOutput {
        access_token,
        refresh_token: raw_refresh,
        token_type: "Bearer",
        expires_in: ACCESS_TOKEN_EXPIRY_SECS,
        user,
    })
}