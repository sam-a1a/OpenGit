use crate::{
    db::queries::users,
    error::AppError,
    git::{pack, repository},
    state::AppState,
};
use axum::{
    body::Bytes,
    extract::{Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct InfoRefsQuery {
    pub service: String,
}

// GET /:owner/:repo.git/info/refs

pub async fn info_refs(
    State(state):   State<AppState>,
    Path((owner, repo)): Path<(String, String)>,
    Query(params):  Query<InfoRefsQuery>,
) -> Result<Response, AppError> {
    let repo_name = repo.trim_end_matches(".git");

    let owner = users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    let repo_record = crate::db::queries::repos::find_by_owner_and_name(
        &state.db, owner.id, repo_name
    )
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    let path = repository::repo_path(&state.config.git_base_dir, &repo_record.git_path);

    if !repository::exists(&path) {
        return Err(AppError::NotFound("Repository on disk".into()));
    }

    let (data, content_type) = match params.service.as_str() {
        "git-upload-pack" => {
            let raw  = pack::upload_pack_info_refs(&path).await?;
            let data = pack::prefix_info_refs("git-upload-pack", raw);
            (data, "application/x-git-upload-pack-advertisement")
        }
        "git-receive-pack" => {
            let raw  = pack::receive_pack_info_refs(&path).await?;
            let data = pack::prefix_info_refs("git-receive-pack", raw);
            (data, "application/x-git-receive-pack-advertisement")
        }
        _ => return Err(AppError::BadRequest("Unknown git service".into())),
    };

    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, content_type),
            (header::CACHE_CONTROL, "no-cache"),
        ],
        data,
    ).into_response())
}

// POST /:owner/:repo.git/git-upload-pack

pub async fn git_upload_pack(
    State(state):   State<AppState>,
    Path((owner, repo)): Path<(String, String)>,
    body:           Bytes,
) -> Result<Response, AppError> {
    let repo_name = repo.trim_end_matches(".git");

    let owner = users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    let repo_record = crate::db::queries::repos::find_by_owner_and_name(
        &state.db, owner.id, repo_name
    )
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    let path = repository::repo_path(&state.config.git_base_dir, &repo_record.git_path);
    let data = pack::upload_pack(&path, body).await?;

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/x-git-upload-pack-result")],
        data,
    ).into_response())
}

// POST /:owner/:repo.git/git-receive-pack

pub async fn git_receive_pack(
    State(state):   State<AppState>,
    Path((owner, repo)): Path<(String, String)>,
    body:           Bytes,
) -> Result<Response, AppError> {
    let repo_name = repo.trim_end_matches(".git");

    let owner = users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    let repo_record = crate::db::queries::repos::find_by_owner_and_name(
        &state.db, owner.id, repo_name
    )
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    let path = repository::repo_path(&state.config.git_base_dir, &repo_record.git_path);
    let data = pack::receive_pack(&path, body).await?;

    // update pushed_at timestamp
    sqlx::query("UPDATE repositories SET pushed_at = now() WHERE id = $1")
        .bind(repo_record.id)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/x-git-receive-pack-result")],
        data,
    ).into_response())
}