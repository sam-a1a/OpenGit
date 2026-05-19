use crate::{
    api::middleware::auth::AuthUser,
    db::queries::{repos, users},
    error::AppError,
    models::webhook::{Webhook, WebhookDelivery},
    services::webhook::{dispatch, WebhookDispatch},
    state::AppState,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub page:     Option<i64>,
    pub per_page: Option<i64>,
}

// Create webhook

#[derive(Debug, Deserialize)]
pub struct CreateWebhookInput {
    pub url:          String,
    pub content_type: Option<String>,
    pub secret:       Option<String>,
    pub events:       Vec<String>,
    pub active:       Option<bool>,
    pub insecure_ssl: Option<bool>,
}

pub async fn create_webhook(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name)): Path<(String, String)>,
    Json(input):    Json<CreateWebhookInput>,
) -> Result<impl IntoResponse, AppError> {
    if input.url.trim().is_empty() {
        return Err(AppError::BadRequest("URL is required".into()));
    }
    if !input.url.starts_with("http://") && !input.url.starts_with("https://") {
        return Err(AppError::BadRequest("URL must start with http:// or https://".into()));
    }
    if input.events.is_empty() {
        return Err(AppError::BadRequest("At least one event is required".into()));
    }

    let owner = users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    if owner.id != auth_user.user_id {
        return Err(AppError::Forbidden);
    }

    let repo = repos::find_by_owner_and_name(&state.db, owner.id, &repo_name)
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    // hash secret if provided
    let secret_hash = input.secret.as_ref().map(|s| {
        blake3::hash(s.as_bytes()).to_hex().to_string()
    });

    let webhook: Webhook = sqlx::query_as(
        "INSERT INTO webhooks
            (repo_id, creator_id, url, content_type, secret_hash,
             events, is_active, insecure_ssl)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
         RETURNING *"
    )
        .bind(repo.id)
        .bind(auth_user.user_id)
        .bind(&input.url)
        .bind(input.content_type.as_deref().unwrap_or("json"))
        .bind(&secret_hash)
        .bind(&input.events)
        .bind(input.active.unwrap_or(true))
        .bind(input.insecure_ssl.unwrap_or(false))
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok((StatusCode::CREATED, Json(webhook)))
}

// List webhooks

pub async fn list_webhooks(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name)): Path<(String, String)>,
) -> Result<impl IntoResponse, AppError> {
    let owner = users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    if owner.id != auth_user.user_id {
        return Err(AppError::Forbidden);
    }

    let repo = repos::find_by_owner_and_name(&state.db, owner.id, &repo_name)
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    let webhooks: Vec<Webhook> = sqlx::query_as(
        "SELECT * FROM webhooks WHERE repo_id = $1 ORDER BY created_at DESC"
    )
        .bind(repo.id)
        .fetch_all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(webhooks))
}

// Get webhook

pub async fn get_webhook(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name, hook_id)): Path<(String, String, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let owner = users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    if owner.id != auth_user.user_id {
        return Err(AppError::Forbidden);
    }

    let repo = repos::find_by_owner_and_name(&state.db, owner.id, &repo_name)
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    let webhook: Webhook = sqlx::query_as(
        "SELECT * FROM webhooks WHERE id = $1 AND repo_id = $2"
    )
        .bind(hook_id)
        .bind(repo.id)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound("Webhook".into()))?;

    Ok(Json(webhook))
}

// Update webhook

#[derive(Debug, Deserialize)]
pub struct UpdateWebhookInput {
    pub url:          Option<String>,
    pub content_type: Option<String>,
    pub secret:       Option<String>,
    pub events:       Option<Vec<String>>,
    pub active:       Option<bool>,
    pub insecure_ssl: Option<bool>,
}

pub async fn update_webhook(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name, hook_id)): Path<(String, String, Uuid)>,
    Json(input):    Json<UpdateWebhookInput>,
) -> Result<impl IntoResponse, AppError> {
    let owner = users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    if owner.id != auth_user.user_id {
        return Err(AppError::Forbidden);
    }

    let repo = repos::find_by_owner_and_name(&state.db, owner.id, &repo_name)
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    let new_secret_hash = input.secret.as_ref().map(|s| {
        blake3::hash(s.as_bytes()).to_hex().to_string()
    });

    let webhook: Webhook = sqlx::query_as(
        "UPDATE webhooks SET
            url          = COALESCE($1, url),
            content_type = COALESCE($2, content_type),
            secret_hash  = COALESCE($3, secret_hash),
            events       = COALESCE($4, events),
            is_active    = COALESCE($5, is_active),
            insecure_ssl = COALESCE($6, insecure_ssl),
            updated_at   = now()
         WHERE id = $7 AND repo_id = $8
         RETURNING *"
    )
        .bind(&input.url)
        .bind(&input.content_type)
        .bind(&new_secret_hash)
        .bind(&input.events)
        .bind(input.active)
        .bind(input.insecure_ssl)
        .bind(hook_id)
        .bind(repo.id)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound("Webhook".into()))?;

    Ok(Json(webhook))
}

// Delete webhook

pub async fn delete_webhook(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name, hook_id)): Path<(String, String, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let owner = users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    if owner.id != auth_user.user_id {
        return Err(AppError::Forbidden);
    }

    let repo = repos::find_by_owner_and_name(&state.db, owner.id, &repo_name)
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    let result = sqlx::query(
        "DELETE FROM webhooks WHERE id = $1 AND repo_id = $2"
    )
        .bind(hook_id)
        .bind(repo.id)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Webhook".into()));
    }

    Ok(StatusCode::NO_CONTENT)
}

// Ping / test webhook

pub async fn ping_webhook(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name, hook_id)): Path<(String, String, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let owner = users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    if owner.id != auth_user.user_id {
        return Err(AppError::Forbidden);
    }

    let repo = repos::find_by_owner_and_name(&state.db, owner.id, &repo_name)
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    let webhook: Webhook = sqlx::query_as(
        "SELECT * FROM webhooks WHERE id = $1 AND repo_id = $2"
    )
        .bind(hook_id)
        .bind(repo.id)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound("Webhook".into()))?;

    dispatch(state, WebhookDispatch {
        webhook_id: webhook.id,
        url:        webhook.url.clone(),
        secret:     webhook.secret_hash.clone(),
        event:      "ping".to_string(),
        payload:    serde_json::json!({
            "zen":    "Speak like a human.",
            "hook_id": webhook.id,
            "hook":   webhook,
        }),
    }).await;

    Ok(StatusCode::NO_CONTENT)
}

// List deliveries

pub async fn list_deliveries(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name, hook_id)): Path<(String, String, Uuid)>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<impl IntoResponse, AppError> {
    let owner = users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    if owner.id != auth_user.user_id {
        return Err(AppError::Forbidden);
    }

    let repo = repos::find_by_owner_and_name(&state.db, owner.id, &repo_name)
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    // verify webhook belongs to repo
    let _: Webhook = sqlx::query_as(
        "SELECT * FROM webhooks WHERE id = $1 AND repo_id = $2"
    )
        .bind(hook_id)
        .bind(repo.id)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound("Webhook".into()))?;

    let per_page = pagination.per_page.unwrap_or(30).min(100);
    let offset   = (pagination.page.unwrap_or(1) - 1) * per_page;

    let deliveries: Vec<WebhookDelivery> = sqlx::query_as(
        "SELECT * FROM webhook_deliveries
         WHERE webhook_id = $1
         ORDER BY created_at DESC
         LIMIT $2 OFFSET $3"
    )
        .bind(hook_id)
        .bind(per_page)
        .bind(offset)
        .fetch_all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(deliveries))
}

// Get delivery

pub async fn get_delivery(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name, hook_id, delivery_id)): Path<(String, String, Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let owner = users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    if owner.id != auth_user.user_id {
        return Err(AppError::Forbidden);
    }

    let repo = repos::find_by_owner_and_name(&state.db, owner.id, &repo_name)
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    let delivery: WebhookDelivery = sqlx::query_as(
        "SELECT wd.* FROM webhook_deliveries wd
         JOIN webhooks w ON w.id = wd.webhook_id
         WHERE wd.id = $1 AND wd.webhook_id = $2 AND w.repo_id = $3"
    )
        .bind(delivery_id)
        .bind(hook_id)
        .bind(repo.id)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound("Delivery".into()))?;

    Ok(Json(delivery))
}

// Redeliver

pub async fn redeliver(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name, hook_id, delivery_id)): Path<(String, String, Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let owner = users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    if owner.id != auth_user.user_id {
        return Err(AppError::Forbidden);
    }

    let repo = repos::find_by_owner_and_name(&state.db, owner.id, &repo_name)
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    let delivery: WebhookDelivery = sqlx::query_as(
        "SELECT wd.* FROM webhook_deliveries wd
         JOIN webhooks w ON w.id = wd.webhook_id
         WHERE wd.id = $1 AND wd.webhook_id = $2 AND w.repo_id = $3"
    )
        .bind(delivery_id)
        .bind(hook_id)
        .bind(repo.id)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound("Delivery".into()))?;

    let webhook: Webhook = sqlx::query_as(
        "SELECT * FROM webhooks WHERE id = $1"
    )
        .bind(hook_id)
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    // mark as redelivery
    sqlx::query(
        "UPDATE webhook_deliveries SET redelivery = true WHERE id = $1"
    )
        .bind(delivery_id)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    dispatch(state, WebhookDispatch {
        webhook_id: webhook.id,
        url:        webhook.url.clone(),
        secret:     webhook.secret_hash.clone(),
        event:      delivery.event.clone(),
        payload:    delivery.request_body.clone(),
    }).await;

    Ok(StatusCode::NO_CONTENT)
}