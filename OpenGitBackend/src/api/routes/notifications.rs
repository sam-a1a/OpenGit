use crate::{
    api::middleware::auth::AuthUser,
    db::queries::repos,
    error::AppError,
    models::notification::{Notification, NotificationSubscription},
    state::AppState,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct NotificationQuery {
    pub all:        Option<bool>,
    pub unread:     Option<bool>,
    pub saved:      Option<bool>,
    pub repo_id:    Option<Uuid>,
    pub page:       Option<i64>,
    pub per_page:   Option<i64>,
}

// List notifications

pub async fn list_notifications(
    State(state):  State<AppState>,
    auth_user:     AuthUser,
    Query(params): Query<NotificationQuery>,
) -> Result<impl IntoResponse, AppError> {
    let per_page = params.per_page.unwrap_or(30).min(100);
    let offset   = (params.page.unwrap_or(1) - 1) * per_page;
    let all      = params.all.unwrap_or(false);
    let saved    = params.saved.unwrap_or(false);

    let notifications: Vec<Notification> = if saved {
        sqlx::query_as(
            "SELECT * FROM notifications
             WHERE user_id = $1 AND is_saved = true
             ORDER BY updated_at DESC
             LIMIT $2 OFFSET $3"
        )
            .bind(auth_user.user_id)
            .bind(per_page)
            .bind(offset)
            .fetch_all(&state.db)
            .await
            .map_err(AppError::Database)?
    } else if let Some(repo_id) = params.repo_id {
        sqlx::query_as(
            "SELECT * FROM notifications
             WHERE user_id = $1
               AND repo_id = $2
               AND ($3 OR is_read = false)
             ORDER BY updated_at DESC
             LIMIT $4 OFFSET $5"
        )
            .bind(auth_user.user_id)
            .bind(repo_id)
            .bind(all)
            .bind(per_page)
            .bind(offset)
            .fetch_all(&state.db)
            .await
            .map_err(AppError::Database)?
    } else {
        sqlx::query_as(
            "SELECT * FROM notifications
             WHERE user_id = $1
               AND ($2 OR is_read = false)
             ORDER BY updated_at DESC
             LIMIT $3 OFFSET $4"
        )
            .bind(auth_user.user_id)
            .bind(all)
            .bind(per_page)
            .bind(offset)
            .fetch_all(&state.db)
            .await
            .map_err(AppError::Database)?
    };

    let unread_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM notifications WHERE user_id = $1 AND is_read = false"
    )
        .bind(auth_user.user_id)
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(json!({
        "notifications": notifications,
        "unread_count":  unread_count.0,
        "page":          params.page.unwrap_or(1),
        "per_page":      per_page,
    })))
}

// Get notification

pub async fn get_notification(
    State(state):        State<AppState>,
    auth_user:           AuthUser,
    Path(notification_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let notification: Notification = sqlx::query_as(
        "SELECT * FROM notifications WHERE id = $1 AND user_id = $2"
    )
        .bind(notification_id)
        .bind(auth_user.user_id)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound("Notification".into()))?;

    Ok(Json(notification))
}

// Mark notification as read

pub async fn mark_read(
    State(state):        State<AppState>,
    auth_user:           AuthUser,
    Path(notification_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let result = sqlx::query(
        "UPDATE notifications
         SET is_read = true, last_read_at = now(), updated_at = now()
         WHERE id = $1 AND user_id = $2"
    )
        .bind(notification_id)
        .bind(auth_user.user_id)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Notification".into()));
    }

    Ok(StatusCode::RESET_CONTENT)
}

// Mark all as read

#[derive(Debug, Deserialize)]
pub struct MarkAllReadInput {
    pub repo_id:    Option<Uuid>,
    pub last_read_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub async fn mark_all_read(
    State(state):  State<AppState>,
    auth_user:     AuthUser,
    Json(input):   Json<MarkAllReadInput>,
) -> Result<impl IntoResponse, AppError> {
    if let Some(repo_id) = input.repo_id {
        sqlx::query(
            "UPDATE notifications
             SET is_read = true, last_read_at = now(), updated_at = now()
             WHERE user_id = $1 AND repo_id = $2 AND is_read = false"
        )
            .bind(auth_user.user_id)
            .bind(repo_id)
            .execute(&state.db)
            .await
            .map_err(AppError::Database)?;
    } else {
        let cutoff = input.last_read_at
            .unwrap_or_else(|| chrono::Utc::now());

        sqlx::query(
            "UPDATE notifications
             SET is_read = true, last_read_at = now(), updated_at = now()
             WHERE user_id = $1
               AND is_read = false
               AND created_at <= $2"
        )
            .bind(auth_user.user_id)
            .bind(cutoff)
            .execute(&state.db)
            .await
            .map_err(AppError::Database)?;
    }

    Ok(StatusCode::RESET_CONTENT)
}

// Save / unsave notification

pub async fn save_notification(
    State(state):        State<AppState>,
    auth_user:           AuthUser,
    Path(notification_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let result = sqlx::query(
        "UPDATE notifications
         SET is_saved = true, updated_at = now()
         WHERE id = $1 AND user_id = $2"
    )
        .bind(notification_id)
        .bind(auth_user.user_id)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Notification".into()));
    }

    Ok(StatusCode::OK)
}

pub async fn unsave_notification(
    State(state):        State<AppState>,
    auth_user:           AuthUser,
    Path(notification_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let result = sqlx::query(
        "UPDATE notifications
         SET is_saved = false, updated_at = now()
         WHERE id = $1 AND user_id = $2"
    )
        .bind(notification_id)
        .bind(auth_user.user_id)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Notification".into()));
    }

    Ok(StatusCode::OK)
}

// Delete notification

pub async fn delete_notification(
    State(state):        State<AppState>,
    auth_user:           AuthUser,
    Path(notification_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let result = sqlx::query(
        "DELETE FROM notifications WHERE id = $1 AND user_id = $2"
    )
        .bind(notification_id)
        .bind(auth_user.user_id)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Notification".into()));
    }

    Ok(StatusCode::NO_CONTENT)
}

// Delete all read notifications

pub async fn delete_all_read(
    State(state): State<AppState>,
    auth_user:    AuthUser,
) -> Result<impl IntoResponse, AppError> {
    sqlx::query(
        "DELETE FROM notifications WHERE user_id = $1 AND is_read = true AND is_saved = false"
    )
        .bind(auth_user.user_id)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// Unread count

pub async fn unread_count(
    State(state): State<AppState>,
    auth_user:    AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM notifications WHERE user_id = $1 AND is_read = false"
    )
        .bind(auth_user.user_id)
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(json!({ "unread_count": row.0 })))
}

// Repo subscription

pub async fn get_repo_subscription(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name)): Path<(String, String)>,
) -> Result<impl IntoResponse, AppError> {
    let owner = crate::db::queries::users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    let repo = repos::find_by_owner_and_name(&state.db, owner.id, &repo_name)
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    let sub: Option<NotificationSubscription> = sqlx::query_as(
        "SELECT * FROM notification_subscriptions
         WHERE user_id = $1 AND repo_id = $2"
    )
        .bind(auth_user.user_id)
        .bind(repo.id)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?;

    match sub {
        Some(s) => Ok(Json(s).into_response()),
        None    => Ok(StatusCode::NOT_FOUND.into_response()),
    }
}

#[derive(Debug, Deserialize)]
pub struct SubscriptionInput {
    pub subscribed: Option<bool>,
    pub ignored:    Option<bool>,
}

pub async fn set_repo_subscription(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name)): Path<(String, String)>,
    Json(input):    Json<SubscriptionInput>,
) -> Result<impl IntoResponse, AppError> {
    let owner = crate::db::queries::users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    let repo = repos::find_by_owner_and_name(&state.db, owner.id, &repo_name)
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    let subscribed = input.subscribed.unwrap_or(true);
    let ignored    = input.ignored.unwrap_or(false);

    let sub: NotificationSubscription = sqlx::query_as(
        "INSERT INTO notification_subscriptions (user_id, repo_id, subscribed, ignored)
         VALUES ($1, $2, $3, $4)
         ON CONFLICT (id) DO UPDATE
         SET subscribed = $3, ignored = $4
         RETURNING *"
    )
        .bind(auth_user.user_id)
        .bind(repo.id)
        .bind(subscribed)
        .bind(ignored)
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(sub))
}

pub async fn delete_repo_subscription(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name)): Path<(String, String)>,
) -> Result<impl IntoResponse, AppError> {
    let owner = crate::db::queries::users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    let repo = repos::find_by_owner_and_name(&state.db, owner.id, &repo_name)
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    sqlx::query(
        "DELETE FROM notification_subscriptions WHERE user_id = $1 AND repo_id = $2"
    )
        .bind(auth_user.user_id)
        .bind(repo.id)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// Create notification (internal use)

pub async fn create_notification(
    state:         &AppState,
    user_id:       Uuid,
    repo_id:       Option<Uuid>,
    subject_type:  &str,
    subject_id:    Option<Uuid>,
    subject_title: &str,
    reason:        &str,
) -> Result<(), AppError> {
    // check if user has ignored this repo
    if let Some(rid) = repo_id {
        let ignored: Option<(bool,)> = sqlx::query_as(
            "SELECT ignored FROM notification_subscriptions
             WHERE user_id = $1 AND repo_id = $2"
        )
            .bind(user_id)
            .bind(rid)
            .fetch_optional(&state.db)
            .await
            .map_err(AppError::Database)?;

        if ignored.map(|(i,)| i).unwrap_or(false) {
            return Ok(());
        }
    }

    // upsert — if notification for same subject exists, update it
    sqlx::query(
        "INSERT INTO notifications
            (user_id, repo_id, subject_type, subject_id,
             subject_title, reason, is_read, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6::notification_reason, false, now())
         ON CONFLICT DO NOTHING"
    )
        .bind(user_id)
        .bind(repo_id)
        .bind(subject_type)
        .bind(subject_id)
        .bind(subject_title)
        .bind(reason)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(())
}

// List repo notifications

pub async fn list_repo_notifications(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name)): Path<(String, String)>,
    Query(params):  Query<NotificationQuery>,
) -> Result<impl IntoResponse, AppError> {
    let owner = crate::db::queries::users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    let repo = repos::find_by_owner_and_name(&state.db, owner.id, &repo_name)
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    let per_page = params.per_page.unwrap_or(30).min(100);
    let offset   = (params.page.unwrap_or(1) - 1) * per_page;
    let all      = params.all.unwrap_or(false);

    let notifications: Vec<Notification> = sqlx::query_as(
        "SELECT * FROM notifications
         WHERE user_id = $1
           AND repo_id = $2
           AND ($3 OR is_read = false)
         ORDER BY updated_at DESC
         LIMIT $4 OFFSET $5"
    )
        .bind(auth_user.user_id)
        .bind(repo.id)
        .bind(all)
        .bind(per_page)
        .bind(offset)
        .fetch_all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(json!({
        "notifications": notifications,
        "page":          params.page.unwrap_or(1),
        "per_page":      per_page,
    })))
}

// Mark repo notifications as read

pub async fn mark_repo_read(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name)): Path<(String, String)>,
) -> Result<impl IntoResponse, AppError> {
    let owner = crate::db::queries::users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    let repo = repos::find_by_owner_and_name(&state.db, owner.id, &repo_name)
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    sqlx::query(
        "UPDATE notifications
         SET is_read = true, last_read_at = now(), updated_at = now()
         WHERE user_id = $1 AND repo_id = $2 AND is_read = false"
    )
        .bind(auth_user.user_id)
        .bind(repo.id)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(StatusCode::RESET_CONTENT)
}