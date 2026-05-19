use crate::{
    api::middleware::auth::AuthUser,
    db::queries::{repos, users},
    error::AppError,
    models::release::{Release, ReleaseAsset},
    state::AppState,
};
use axum::{
    body::Bytes,
    extract::{Multipart, Path, Query, State},
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub page:     Option<i64>,
    pub per_page: Option<i64>,
}

// Create release

#[derive(Debug, Deserialize)]
pub struct CreateReleaseInput {
    pub tag_name:     String,
    pub name:         Option<String>,
    pub body:         Option<String>,
    pub draft:        Option<bool>,
    pub prerelease:   Option<bool>,
    pub target_sha:   Option<String>,
}

pub async fn create_release(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name)): Path<(String, String)>,
    Json(input):    Json<CreateReleaseInput>,
) -> Result<impl IntoResponse, AppError> {
    if input.tag_name.trim().is_empty() {
        return Err(AppError::BadRequest("tag_name is required".into()));
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

    // if marking as latest, unmark previous latest
    if input.draft.unwrap_or(false) == false && input.prerelease.unwrap_or(false) == false {
        sqlx::query(
            "UPDATE releases SET is_latest = false WHERE repo_id = $1 AND is_latest = true"
        )
            .bind(repo.id)
            .execute(&state.db)
            .await
            .map_err(AppError::Database)?;
    }

    let is_latest = !input.draft.unwrap_or(false) && !input.prerelease.unwrap_or(false);

    let release: Release = sqlx::query_as(
        "INSERT INTO releases
            (repo_id, author_id, tag_name, target_sha, name, body,
             is_draft, is_prerelease, is_latest, published_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9,
             CASE WHEN $7 = false THEN now() ELSE NULL END)
         RETURNING *"
    )
        .bind(repo.id)
        .bind(auth_user.user_id)
        .bind(&input.tag_name)
        .bind(&input.target_sha)
        .bind(&input.name)
        .bind(&input.body)
        .bind(input.draft.unwrap_or(false))
        .bind(input.prerelease.unwrap_or(false))
        .bind(is_latest)
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok((StatusCode::CREATED, Json(release)))
}

// List releases

pub async fn list_releases(
    State(state):   State<AppState>,
    Path((owner, repo_name)): Path<(String, String)>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<impl IntoResponse, AppError> {
    let owner = users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    let repo = repos::find_by_owner_and_name(&state.db, owner.id, &repo_name)
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    let per_page = pagination.per_page.unwrap_or(30).min(100);
    let offset   = (pagination.page.unwrap_or(1) - 1) * per_page;

    let releases: Vec<Release> = sqlx::query_as(
        "SELECT * FROM releases
         WHERE repo_id = $1 AND is_draft = false
         ORDER BY created_at DESC
         LIMIT $2 OFFSET $3"
    )
        .bind(repo.id)
        .bind(per_page)
        .bind(offset)
        .fetch_all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(releases))
}

// Get latest release

pub async fn get_latest_release(
    State(state):   State<AppState>,
    Path((owner, repo_name)): Path<(String, String)>,
) -> Result<impl IntoResponse, AppError> {
    let owner = users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    let repo = repos::find_by_owner_and_name(&state.db, owner.id, &repo_name)
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    let release: Release = sqlx::query_as(
        "SELECT * FROM releases
         WHERE repo_id = $1 AND is_latest = true AND is_draft = false"
    )
        .bind(repo.id)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound("Release".into()))?;

    Ok(Json(release))
}

// Get release by tag

pub async fn get_release_by_tag(
    State(state):   State<AppState>,
    Path((owner, repo_name, tag)): Path<(String, String, String)>,
) -> Result<impl IntoResponse, AppError> {
    let owner = users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    let repo = repos::find_by_owner_and_name(&state.db, owner.id, &repo_name)
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    let release: Release = sqlx::query_as(
        "SELECT * FROM releases WHERE repo_id = $1 AND tag_name = $2"
    )
        .bind(repo.id)
        .bind(&tag)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound("Release".into()))?;

    Ok(Json(release))
}

// Get release by id

pub async fn get_release(
    State(state):   State<AppState>,
    Path((owner, repo_name, release_id)): Path<(String, String, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let owner = users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    let repo = repos::find_by_owner_and_name(&state.db, owner.id, &repo_name)
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    let release: Release = sqlx::query_as(
        "SELECT * FROM releases WHERE id = $1 AND repo_id = $2"
    )
        .bind(release_id)
        .bind(repo.id)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound("Release".into()))?;

    Ok(Json(release))
}

// Update release

#[derive(Debug, Deserialize)]
pub struct UpdateReleaseInput {
    pub name:       Option<String>,
    pub body:       Option<String>,
    pub draft:      Option<bool>,
    pub prerelease: Option<bool>,
    pub tag_name:   Option<String>,
}

pub async fn update_release(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name, release_id)): Path<(String, String, Uuid)>,
    Json(input):    Json<UpdateReleaseInput>,
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

    let release: Release = sqlx::query_as(
        "UPDATE releases SET
            name        = COALESCE($1, name),
            body        = COALESCE($2, body),
            is_draft    = COALESCE($3, is_draft),
            is_prerelease = COALESCE($4, is_prerelease),
            tag_name    = COALESCE($5, tag_name),
            published_at = CASE
                WHEN $3 = false AND published_at IS NULL THEN now()
                ELSE published_at
            END,
            updated_at  = now()
         WHERE id = $6 AND repo_id = $7
         RETURNING *"
    )
        .bind(&input.name)
        .bind(&input.body)
        .bind(input.draft)
        .bind(input.prerelease)
        .bind(&input.tag_name)
        .bind(release_id)
        .bind(repo.id)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound("Release".into()))?;

    Ok(Json(release))
}

// Delete release

pub async fn delete_release(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name, release_id)): Path<(String, String, Uuid)>,
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

    // get assets to delete from MinIO
    let assets: Vec<ReleaseAsset> = sqlx::query_as(
        "SELECT * FROM release_assets WHERE release_id = $1"
    )
        .bind(release_id)
        .fetch_all(&state.db)
        .await
        .map_err(AppError::Database)?;

    // delete assets from MinIO
    for asset in &assets {
        let _ = delete_from_minio(&state, &asset.storage_key).await;
    }

    let result = sqlx::query(
        "DELETE FROM releases WHERE id = $1 AND repo_id = $2"
    )
        .bind(release_id)
        .bind(repo.id)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Release".into()));
    }

    Ok(StatusCode::NO_CONTENT)
}

// List assets

pub async fn list_assets(
    State(state):   State<AppState>,
    Path((owner, repo_name, release_id)): Path<(String, String, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let owner = users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    let repo = repos::find_by_owner_and_name(&state.db, owner.id, &repo_name)
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    let assets: Vec<ReleaseAsset> = sqlx::query_as(
        "SELECT * FROM release_assets WHERE release_id = $1 ORDER BY created_at ASC"
    )
        .bind(release_id)
        .fetch_all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(assets))
}

// Upload asset

pub async fn upload_asset(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name, release_id)): Path<(String, String, Uuid)>,
    mut multipart:  Multipart,
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

    // verify release exists
    let _: Release = sqlx::query_as(
        "SELECT * FROM releases WHERE id = $1 AND repo_id = $2"
    )
        .bind(release_id)
        .bind(repo.id)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound("Release".into()))?;

    // read multipart field
    let field = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("multipart error: {}", e)))?
        .ok_or(AppError::BadRequest("No file in request".into()))?;

    let file_name    = field.file_name()
        .unwrap_or("unknown")
        .to_string();
    let content_type = field.content_type()
        .unwrap_or("application/octet-stream")
        .to_string();
    let data: Bytes  = field.bytes().await
        .map_err(|e| AppError::BadRequest(format!("read error: {}", e)))?;

    let size_bytes   = data.len() as i64;
    let storage_key  = format!(
        "releases/{}/{}/{}",
        repo.id, release_id, file_name
    );

    // upload to MinIO
    upload_to_minio(&state, &storage_key, &content_type, data).await?;

    let asset: ReleaseAsset = sqlx::query_as(
        "INSERT INTO release_assets
            (release_id, uploader_id, name, content_type, size_bytes, storage_key)
         VALUES ($1, $2, $3, $4, $5, $6)
         RETURNING *"
    )
        .bind(release_id)
        .bind(auth_user.user_id)
        .bind(&file_name)
        .bind(&content_type)
        .bind(size_bytes)
        .bind(&storage_key)
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok((StatusCode::CREATED, Json(asset)))
}

// Download asset

pub async fn download_asset(
    State(state):   State<AppState>,
    Path((owner, repo_name, release_id, asset_id)): Path<(String, String, Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let owner = users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    let repo = repos::find_by_owner_and_name(&state.db, owner.id, &repo_name)
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    let asset: ReleaseAsset = sqlx::query_as(
        "SELECT ra.* FROM release_assets ra
         JOIN releases r ON r.id = ra.release_id
         WHERE ra.id = $1 AND r.repo_id = $2 AND ra.release_id = $3"
    )
        .bind(asset_id)
        .bind(repo.id)
        .bind(release_id)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound("Asset".into()))?;

    // increment download count
    sqlx::query(
        "UPDATE release_assets SET download_count = download_count + 1 WHERE id = $1"
    )
        .bind(asset.id)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    // get presigned URL from MinIO
    let url = presigned_url(&state, &asset.storage_key).await?;

    Ok((
        StatusCode::FOUND,
        [(header::LOCATION, url)],
    ))
}

// Delete asset

pub async fn delete_asset(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name, release_id, asset_id)): Path<(String, String, Uuid, Uuid)>,
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

    let asset: ReleaseAsset = sqlx::query_as(
        "SELECT ra.* FROM release_assets ra
         JOIN releases r ON r.id = ra.release_id
         WHERE ra.id = $1 AND r.repo_id = $2"
    )
        .bind(asset_id)
        .bind(repo.id)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound("Asset".into()))?;

    delete_from_minio(&state, &asset.storage_key).await?;

    sqlx::query("DELETE FROM release_assets WHERE id = $1")
        .bind(asset_id)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// MinIO helpers

const BUCKET: &str = "opengit-releases";

async fn upload_to_minio(
    state:        &AppState,
    key:          &str,
    content_type: &str,
    data:         Bytes,
) -> Result<(), AppError> {
    let client = build_s3_client(state).await;

    // ensure bucket exists
    let _ = client
        .create_bucket()
        .bucket(BUCKET)
        .send()
        .await;

    client
        .put_object()
        .bucket(BUCKET)
        .key(key)
        .content_type(content_type)
        .body(data.into())
        .send()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("MinIO upload failed: {}", e)))?;

    Ok(())
}

async fn delete_from_minio(state: &AppState, key: &str) -> Result<(), AppError> {
    let client = build_s3_client(state).await;

    client
        .delete_object()
        .bucket(BUCKET)
        .key(key)
        .send()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("MinIO delete failed: {}", e)))?;

    Ok(())
}

async fn presigned_url(state: &AppState, key: &str) -> Result<String, AppError> {
    use aws_sdk_s3::presigning::PresigningConfig;
    use std::time::Duration;

    let client = build_s3_client(state).await;

    let presigned = client
        .get_object()
        .bucket(BUCKET)
        .key(key)
        .presigned(
            PresigningConfig::expires_in(Duration::from_secs(3600))
                .map_err(|e| AppError::Internal(anyhow::anyhow!("presign config: {}", e)))?,
        )
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("presign failed: {}", e)))?;

    Ok(presigned.uri().to_string())
}

async fn build_s3_client(state: &AppState) -> aws_sdk_s3::Client {
    let creds = aws_sdk_s3::config::Credentials::new(
        &state.config.minio_access_key,
        &state.config.minio_secret_key,
        None,
        None,
        "opengit",
    );

    let s3_config = aws_sdk_s3::config::Builder::new()
        .endpoint_url(&state.config.minio_endpoint)
        .credentials_provider(creds)
        .region(aws_sdk_s3::config::Region::new("us-east-1"))
        .force_path_style(true)
        .behavior_version(aws_sdk_s3::config::BehaviorVersion::latest())
        .build();

    aws_sdk_s3::Client::from_conf(s3_config)
}