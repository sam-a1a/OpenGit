use crate::{error::AppError, models::repo::Repository};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Repository>, AppError> {
    sqlx::query_as("SELECT * FROM repositories WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)
}

pub async fn find_by_owner_and_name(
    pool:     &PgPool,
    owner_id: Uuid,
    name:     &str,
) -> Result<Option<Repository>, AppError> {
    sqlx::query_as(
        "SELECT * FROM repositories WHERE owner_id = $1 AND name = $2"
    )
        .bind(owner_id)
        .bind(name)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)
}

pub async fn name_exists(pool: &PgPool, owner_id: Uuid, name: &str) -> Result<bool, AppError> {
    let row: (bool,) = sqlx::query_as(
        "SELECT EXISTS(SELECT 1 FROM repositories WHERE owner_id = $1 AND name = $2)"
    )
        .bind(owner_id)
        .bind(name)
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)?;
    Ok(row.0)
}