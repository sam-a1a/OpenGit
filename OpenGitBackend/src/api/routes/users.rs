use crate::{
    api::middleware::auth::AuthUser,
    db::queries::users,
    error::AppError,
    state::AppState,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

// Get user by username

pub async fn get_user(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let user = users::find_by_username(&state.db, &username)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;
    Ok(Json(user))
}

// Get current authenticated user

pub async fn get_me(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let user = users::find_by_id(&state.db, auth_user.user_id)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;
    Ok(Json(user))
}

// Update current user

#[derive(Debug, Deserialize)]
pub struct UpdateUserInput {
    pub display_name:        Option<String>,
    pub bio:                 Option<String>,
    pub website:             Option<String>,
    pub location:            Option<String>,
    pub pronouns:            Option<String>,
    pub company:             Option<String>,
    pub twitter_username:    Option<String>,
    pub profile_private:     Option<bool>,
}

pub async fn update_me(
    State(state): State<AppState>,
    auth_user:    AuthUser,
    Json(input):  Json<UpdateUserInput>,
) -> Result<impl IntoResponse, AppError> {
    let user: crate::models::user::User = sqlx::query_as(
        "UPDATE users SET
            display_name     = COALESCE($1, display_name),
            bio              = COALESCE($2, bio),
            website          = COALESCE($3, website),
            location         = COALESCE($4, location),
            pronouns         = COALESCE($5, pronouns),
            company          = COALESCE($6, company),
            twitter_username = COALESCE($7, twitter_username),
            profile_private  = COALESCE($8, profile_private),
            updated_at       = now()
         WHERE id = $9
         RETURNING *"
    )
        .bind(input.display_name)
        .bind(input.bio)
        .bind(input.website)
        .bind(input.location)
        .bind(input.pronouns)
        .bind(input.company)
        .bind(input.twitter_username)
        .bind(input.profile_private)
        .bind(auth_user.user_id)
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(user))
}

// Update status

#[derive(Debug, Deserialize)]
pub struct UpdateStatusInput {
    pub emoji:          Option<String>,
    pub message:        Option<String>,
    pub availability:   Option<String>,
    pub expires_at:     Option<chrono::DateTime<chrono::Utc>>,
}

pub async fn update_status(
    State(state): State<AppState>,
    auth_user:    AuthUser,
    Json(input):  Json<UpdateStatusInput>,
) -> Result<impl IntoResponse, AppError> {
    let user: crate::models::user::User = sqlx::query_as(
        "UPDATE users SET
            status_emoji        = $1,
            status_message      = $2,
            status_expires_at   = $3,
            updated_at          = now()
         WHERE id = $4
         RETURNING *"
    )
        .bind(input.emoji)
        .bind(input.message)
        .bind(input.expires_at)
        .bind(auth_user.user_id)
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(user))
}

// Follow / Unfollow

pub async fn follow_user(
    State(state): State<AppState>,
    auth_user:    AuthUser,
    Path(username): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let target = users::find_by_username(&state.db, &username)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    if target.id == auth_user.user_id {
        return Err(AppError::BadRequest("You cannot follow yourself".into()));
    }

    sqlx::query(
        "INSERT INTO user_follows (follower_id, following_id)
         VALUES ($1, $2)
         ON CONFLICT DO NOTHING"
    )
        .bind(auth_user.user_id)
        .bind(target.id)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn unfollow_user(
    State(state): State<AppState>,
    auth_user:    AuthUser,
    Path(username): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let target = users::find_by_username(&state.db, &username)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    sqlx::query(
        "DELETE FROM user_follows WHERE follower_id = $1 AND following_id = $2"
    )
        .bind(auth_user.user_id)
        .bind(target.id)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// Followers / Following

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub page:     Option<i64>,
    pub per_page: Option<i64>,
}

pub async fn get_followers(
    State(state):   State<AppState>,
    Path(username): Path<String>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<impl IntoResponse, AppError> {
    let target = users::find_by_username(&state.db, &username)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    let per_page = pagination.per_page.unwrap_or(30).min(100);
    let offset   = (pagination.page.unwrap_or(1) - 1) * per_page;

    let followers: Vec<crate::models::user::User> = sqlx::query_as(
        "SELECT u.* FROM users u
         JOIN user_follows f ON f.follower_id = u.id
         WHERE f.following_id = $1
         ORDER BY f.created_at DESC
         LIMIT $2 OFFSET $3"
    )
        .bind(target.id)
        .bind(per_page)
        .bind(offset)
        .fetch_all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(followers))
}

pub async fn get_following(
    State(state):   State<AppState>,
    Path(username): Path<String>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<impl IntoResponse, AppError> {
    let target = users::find_by_username(&state.db, &username)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    let per_page = pagination.per_page.unwrap_or(30).min(100);
    let offset   = (pagination.page.unwrap_or(1) - 1) * per_page;

    let following: Vec<crate::models::user::User> = sqlx::query_as(
        "SELECT u.* FROM users u
         JOIN user_follows f ON f.following_id = u.id
         WHERE f.follower_id = $1
         ORDER BY f.created_at DESC
         LIMIT $2 OFFSET $3"
    )
        .bind(target.id)
        .bind(per_page)
        .bind(offset)
        .fetch_all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(following))
}

// Block / Unblock

pub async fn block_user(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path(username): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let target = users::find_by_username(&state.db, &username)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    if target.id == auth_user.user_id {
        return Err(AppError::BadRequest("You cannot block yourself".into()));
    }

    sqlx::query(
        "INSERT INTO user_blocks (blocker_id, blocked_id)
         VALUES ($1, $2)
         ON CONFLICT DO NOTHING"
    )
        .bind(auth_user.user_id)
        .bind(target.id)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn unblock_user(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path(username): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let target = users::find_by_username(&state.db, &username)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    sqlx::query(
        "DELETE FROM user_blocks WHERE blocker_id = $1 AND blocked_id = $2"
    )
        .bind(auth_user.user_id)
        .bind(target.id)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// SSH Keys

#[derive(Debug, Deserialize)]
pub struct AddSshKeyInput {
    pub title:    String,
    pub key:      String,
}

#[derive(Debug, Serialize)]
pub struct SshKeyResponse {
    pub id:         Uuid,
    pub title:      String,
    pub key_type:   String,
    pub fingerprint: String,
    pub read_only:  bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub async fn list_ssh_keys(
    State(state): State<AppState>,
    auth_user:    AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let keys: Vec<crate::models::user::UserSshKey> = sqlx::query_as(
        "SELECT * FROM user_ssh_keys WHERE user_id = $1 ORDER BY created_at DESC"
    )
        .bind(auth_user.user_id)
        .fetch_all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(keys))
}

pub async fn add_ssh_key(
    State(state): State<AppState>,
    auth_user:    AuthUser,
    Json(input):  Json<AddSshKeyInput>,
) -> Result<impl IntoResponse, AppError> {
    // parse key type and fingerprint from the raw key
    let parts: Vec<&str> = input.key.trim().splitn(3, ' ').collect();
    if parts.len() < 2 {
        return Err(AppError::BadRequest("Invalid SSH key format".into()));
    }
    let key_type    = parts[0].to_string();
    let fingerprint = format!("SHA256:{}", parts[1].chars().take(16).collect::<String>());

    let key: crate::models::user::UserSshKey = sqlx::query_as(
        "INSERT INTO user_ssh_keys (user_id, title, key_type, key_data, fingerprint)
         VALUES ($1, $2, $3, $4, $5)
         RETURNING *"
    )
        .bind(auth_user.user_id)
        .bind(&input.title)
        .bind(&key_type)
        .bind(&input.key)
        .bind(&fingerprint)
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok((StatusCode::CREATED, Json(key)))
}

pub async fn delete_ssh_key(
    State(state): State<AppState>,
    auth_user:    AuthUser,
    Path(key_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let result = sqlx::query(
        "DELETE FROM user_ssh_keys WHERE id = $1 AND user_id = $2"
    )
        .bind(key_id)
        .bind(auth_user.user_id)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("SSH key".into()));
    }

    Ok(StatusCode::NO_CONTENT)
}

// User repos (public listing)

pub async fn get_user_repos(
    State(state):   State<AppState>,
    Path(username): Path<String>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<impl IntoResponse, AppError> {
    let target = users::find_by_username(&state.db, &username)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    let per_page = pagination.per_page.unwrap_or(30).min(100);
    let offset   = (pagination.page.unwrap_or(1) - 1) * per_page;

    let repos: Vec<crate::models::repo::Repository> = sqlx::query_as(
        "SELECT * FROM repositories
         WHERE owner_id = $1 AND visibility = 'public'
         ORDER BY updated_at DESC
         LIMIT $2 OFFSET $3"
    )
        .bind(target.id)
        .bind(per_page)
        .bind(offset)
        .fetch_all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(repos))
}

// Search users

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q:        String,
    pub page:     Option<i64>,
    pub per_page: Option<i64>,
}

pub async fn search_users(
    State(state):  State<AppState>,
    Query(params): Query<SearchQuery>,
) -> Result<impl IntoResponse, AppError> {
    let per_page = params.per_page.unwrap_or(30).min(100);
    let offset   = (params.page.unwrap_or(1) - 1) * per_page;
    let pattern  = format!("%{}%", params.q.to_lowercase());

    let users: Vec<crate::models::user::User> = sqlx::query_as(
        "SELECT * FROM users
         WHERE (LOWER(username) LIKE $1 OR LOWER(display_name) LIKE $1)
           AND is_active = true
           AND profile_private = false
         ORDER BY username
         LIMIT $2 OFFSET $3"
    )
        .bind(&pattern)
        .bind(per_page)
        .bind(offset)
        .fetch_all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(json!({
        "users": users,
        "page": params.page.unwrap_or(1),
        "per_page": per_page,
    })))
}