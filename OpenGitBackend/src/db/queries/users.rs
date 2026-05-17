use crate::{error::AppError, models::user::{User, UserEmail}};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<User>, AppError> {
    sqlx::query_as(
        "SELECT * FROM users WHERE id = $1 AND is_active = true"
    )
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)
}

pub async fn find_by_username(pool: &PgPool, username: &str) -> Result<Option<User>, AppError> {
    sqlx::query_as(
        "SELECT * FROM users WHERE username = $1 AND is_active = true"
    )
        .bind(username)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)
}

pub async fn find_by_email(pool: &PgPool, email: &str) -> Result<Option<User>, AppError> {
    sqlx::query_as(
        "SELECT u.* FROM users u
         JOIN user_emails e ON e.user_id = u.id
         WHERE e.email = $1 AND u.is_active = true"
    )
        .bind(email)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)
}

pub async fn username_exists(pool: &PgPool, username: &str) -> Result<bool, AppError> {
    let row: (bool,) = sqlx::query_as(
        "SELECT EXISTS(SELECT 1 FROM users WHERE username = $1)"
    )
        .bind(username)
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)?;
    Ok(row.0)
}

pub async fn email_exists(pool: &PgPool, email: &str) -> Result<bool, AppError> {
    let row: (bool,) = sqlx::query_as(
        "SELECT EXISTS(SELECT 1 FROM user_emails WHERE email = $1)"
    )
        .bind(email)
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)?;
    Ok(row.0)
}

pub async fn create_user(
    pool:          &PgPool,
    username:      &str,
    password_hash: &str,
) -> Result<User, AppError> {
    sqlx::query_as(
        "INSERT INTO users (username, password_hash)
         VALUES ($1, $2)
         RETURNING *"
    )
        .bind(username)
        .bind(password_hash)
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)
}

pub async fn create_user_email(
    pool:    &PgPool,
    user_id: Uuid,
    email:   &str,
) -> Result<UserEmail, AppError> {
    sqlx::query_as(
        "INSERT INTO user_emails (user_id, email, is_primary)
         VALUES ($1, $2, true)
         RETURNING *"
    )
        .bind(user_id)
        .bind(email)
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)
}