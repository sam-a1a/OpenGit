use crate::{error::AppError, models::auth::{RefreshToken, Session}};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create_session(
    pool:       &PgPool,
    user_id:    Uuid,
    ip_address: Option<&str>,
    user_agent: Option<&str>,
) -> Result<Session, AppError> {
    sqlx::query_as(
        "INSERT INTO sessions (user_id, ip_address, user_agent, expires_at)
         VALUES ($1, $2::inet, $3, now() + interval '30 days')
         RETURNING *"
    )
        .bind(user_id)
        .bind(ip_address)
        .bind(user_agent)
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)
}

pub async fn create_refresh_token(
    pool:       &PgPool,
    user_id:    Uuid,
    token_hash: &str,
    family_id:  Uuid,
    session_id: Uuid,
    ip_address: Option<&str>,
) -> Result<RefreshToken, AppError> {
    sqlx::query_as(
        "INSERT INTO refresh_tokens
             (user_id, token_hash, family_id, session_id, ip_address, expires_at)
         VALUES ($1, $2, $3, $4, $5::inet, now() + interval '7 days')
         RETURNING *"
    )
        .bind(user_id)
        .bind(token_hash)
        .bind(family_id)
        .bind(session_id)
        .bind(ip_address)
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)
}

pub async fn find_refresh_token(
    pool:       &PgPool,
    token_hash: &str,
) -> Result<Option<RefreshToken>, AppError> {
    sqlx::query_as(
        "SELECT * FROM refresh_tokens
         WHERE token_hash = $1 AND expires_at > now()"
    )
        .bind(token_hash)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)
}

pub async fn mark_token_used(pool: &PgPool, id: Uuid) -> Result<(), AppError> {
    sqlx::query("UPDATE refresh_tokens SET used = true WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await
        .map_err(AppError::Database)?;
    Ok(())
}

pub async fn invalidate_family(pool: &PgPool, family_id: Uuid) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE refresh_tokens SET used = true WHERE family_id = $1"
    )
        .bind(family_id)
        .execute(pool)
        .await
        .map_err(AppError::Database)?;
    Ok(())
}

pub async fn delete_session(pool: &PgPool, session_id: Uuid) -> Result<(), AppError> {
    sqlx::query("DELETE FROM sessions WHERE id = $1")
        .bind(session_id)
        .execute(pool)
        .await
        .map_err(AppError::Database)?;
    Ok(())
}