use crate::{
    api::middleware::auth::AuthUser,
    db::queries::{repos, users},
    error::AppError,
    models::repo::{RepoBranchProtection, RepoCollaborator, Repository},
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

// Pagination

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub page:     Option<i64>,
    pub per_page: Option<i64>,
}

// ── Create repo

#[derive(Debug, Deserialize)]
pub struct CreateRepoInput {
    pub name:           String,
    pub description:    Option<String>,
    pub visibility:     Option<String>,
    pub has_issues:     Option<bool>,
    pub has_wiki:       Option<bool>,
    pub auto_init:      Option<bool>,
}

pub async fn create_repo(
    State(state): State<AppState>,
    auth_user:    AuthUser,
    Json(input):  Json<CreateRepoInput>,
) -> Result<impl IntoResponse, AppError> {
    if input.name.len() < 1 {
        return Err(AppError::BadRequest("Repository name is required".into()));
    }
    if !input.name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.') {
        return Err(AppError::BadRequest(
            "Repository name can only contain letters, numbers, hyphens, underscores and dots".into()
        ));
    }

    if repos::name_exists(&state.db, auth_user.user_id, &input.name).await? {
        return Err(AppError::Conflict("Repository name".into()));
    }

    let visibility = input.visibility.as_deref().unwrap_or("public");
    let git_path   = format!("repos/{}/{}.git", auth_user.user_id, input.name);

    let repo: Repository = sqlx::query_as(
        "INSERT INTO repositories
            (owner_id, name, description, visibility, git_path, has_issues, has_wiki)
         VALUES ($1, $2, $3, $4::repo_visibility, $5, $6, $7)
         RETURNING *"
    )
        .bind(auth_user.user_id)
        .bind(&input.name)
        .bind(&input.description)
        .bind(visibility)
        .bind(&git_path)
        .bind(input.has_issues.unwrap_or(true))
        .bind(input.has_wiki.unwrap_or(true))
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok((StatusCode::CREATED, Json(repo)))
}

// ── Get repo

pub async fn get_repo(
    State(state):   State<AppState>,
    Path((owner, repo_name)): Path<(String, String)>,
) -> Result<impl IntoResponse, AppError> {
    let owner = users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    let repo = repos::find_by_owner_and_name(&state.db, owner.id, &repo_name)
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    Ok(Json(repo))
}

// List user repos

pub async fn list_my_repos(
    State(state):      State<AppState>,
    auth_user:         AuthUser,
    Query(pagination): Query<PaginationQuery>,
) -> Result<impl IntoResponse, AppError> {
    let per_page = pagination.per_page.unwrap_or(30).min(100);
    let offset   = (pagination.page.unwrap_or(1) - 1) * per_page;

    let repos: Vec<Repository> = sqlx::query_as(
        "SELECT * FROM repositories
         WHERE owner_id = $1
         ORDER BY updated_at DESC
         LIMIT $2 OFFSET $3"
    )
        .bind(auth_user.user_id)
        .bind(per_page)
        .bind(offset)
        .fetch_all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(repos))
}

// Update repo

#[derive(Debug, Deserialize)]
pub struct UpdateRepoInput {
    pub description:            Option<String>,
    pub visibility:             Option<String>,
    pub has_issues:             Option<bool>,
    pub has_wiki:               Option<bool>,
    pub has_projects:           Option<bool>,
    pub has_discussions:        Option<bool>,
    pub default_branch:         Option<String>,
    pub allow_merge_commit:     Option<bool>,
    pub allow_squash_merge:     Option<bool>,
    pub allow_rebase_merge:     Option<bool>,
    pub delete_branch_on_merge: Option<bool>,
}

pub async fn update_repo(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name)): Path<(String, String)>,
    Json(input):    Json<UpdateRepoInput>,
) -> Result<impl IntoResponse, AppError> {
    let owner = users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    if owner.id != auth_user.user_id {
        return Err(AppError::Forbidden);
    }

    let repo: Repository = sqlx::query_as(
        "UPDATE repositories SET
            description             = COALESCE($1,  description),
            has_issues              = COALESCE($2,  has_issues),
            has_wiki                = COALESCE($3,  has_wiki),
            has_projects            = COALESCE($4,  has_projects),
            has_discussions         = COALESCE($5,  has_discussions),
            default_branch          = COALESCE($6,  default_branch),
            allow_merge_commit      = COALESCE($7,  allow_merge_commit),
            allow_squash_merge      = COALESCE($8,  allow_squash_merge),
            allow_rebase_merge      = COALESCE($9,  allow_rebase_merge),
            delete_branch_on_merge  = COALESCE($10, delete_branch_on_merge),
            updated_at              = now()
         WHERE owner_id = $11 AND name = $12
         RETURNING *"
    )
        .bind(&input.description)
        .bind(input.has_issues)
        .bind(input.has_wiki)
        .bind(input.has_projects)
        .bind(input.has_discussions)
        .bind(&input.default_branch)
        .bind(input.allow_merge_commit)
        .bind(input.allow_squash_merge)
        .bind(input.allow_rebase_merge)
        .bind(input.delete_branch_on_merge)
        .bind(auth_user.user_id)
        .bind(&repo_name)
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(repo))
}

// Delete repo

pub async fn delete_repo(
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

    let result = sqlx::query(
        "DELETE FROM repositories WHERE owner_id = $1 AND name = $2"
    )
        .bind(auth_user.user_id)
        .bind(&repo_name)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Repository".into()));
    }

    Ok(StatusCode::NO_CONTENT)
}

// Fork repo

pub async fn fork_repo(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name)): Path<(String, String)>,
) -> Result<impl IntoResponse, AppError> {
    let owner = users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    let source = repos::find_by_owner_and_name(&state.db, owner.id, &repo_name)
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    if source.visibility.to_string() == "private" && owner.id != auth_user.user_id {
        return Err(AppError::Forbidden);
    }

    if repos::name_exists(&state.db, auth_user.user_id, &source.name).await? {
        return Err(AppError::Conflict(format!(
            "You already have a repository named '{}'", source.name
        )));
    }

    let git_path = format!("repos/{}/{}.git", auth_user.user_id, source.name);

    let fork: Repository = sqlx::query_as(
        "INSERT INTO repositories
            (owner_id, name, description, visibility, git_path,
             is_fork, forked_from_id, has_issues, has_wiki)
         VALUES ($1, $2, $3, $4::repo_visibility, $5, true, $6, $7, $8)
         RETURNING *"
    )
        .bind(auth_user.user_id)
        .bind(&source.name)
        .bind(&source.description)
        .bind(source.visibility.clone())
        .bind(&git_path)
        .bind(source.id)
        .bind(source.has_issues)
        .bind(source.has_wiki)
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    sqlx::query(
        "UPDATE repositories SET fork_count = fork_count + 1 WHERE id = $1"
    )
        .bind(source.id)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok((StatusCode::CREATED, Json(fork)))
}

// Star / Unstar

pub async fn star_repo(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name)): Path<(String, String)>,
) -> Result<impl IntoResponse, AppError> {
    let owner = users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    let repo = repos::find_by_owner_and_name(&state.db, owner.id, &repo_name)
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    sqlx::query(
        "INSERT INTO repo_stars (user_id, repo_id)
         VALUES ($1, $2) ON CONFLICT DO NOTHING"
    )
        .bind(auth_user.user_id)
        .bind(repo.id)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    sqlx::query(
        "UPDATE repositories SET star_count = star_count + 1 WHERE id = $1"
    )
        .bind(repo.id)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn unstar_repo(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name)): Path<(String, String)>,
) -> Result<impl IntoResponse, AppError> {
    let owner = users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    let repo = repos::find_by_owner_and_name(&state.db, owner.id, &repo_name)
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    sqlx::query(
        "DELETE FROM repo_stars WHERE user_id = $1 AND repo_id = $2"
    )
        .bind(auth_user.user_id)
        .bind(repo.id)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    sqlx::query(
        "UPDATE repositories SET star_count = GREATEST(star_count - 1, 0) WHERE id = $1"
    )
        .bind(repo.id)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// Watch / Unwatch

pub async fn watch_repo(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name)): Path<(String, String)>,
) -> Result<impl IntoResponse, AppError> {
    let owner = users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    let repo = repos::find_by_owner_and_name(&state.db, owner.id, &repo_name)
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    sqlx::query(
        "INSERT INTO repo_watches (user_id, repo_id, level)
         VALUES ($1, $2, 'watching')
         ON CONFLICT (user_id, repo_id) DO UPDATE SET level = 'watching'"
    )
        .bind(auth_user.user_id)
        .bind(repo.id)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    sqlx::query(
        "UPDATE repositories SET watcher_count = watcher_count + 1 WHERE id = $1"
    )
        .bind(repo.id)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn unwatch_repo(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name)): Path<(String, String)>,
) -> Result<impl IntoResponse, AppError> {
    let owner = users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    let repo = repos::find_by_owner_and_name(&state.db, owner.id, &repo_name)
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    sqlx::query(
        "DELETE FROM repo_watches WHERE user_id = $1 AND repo_id = $2"
    )
        .bind(auth_user.user_id)
        .bind(repo.id)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    sqlx::query(
        "UPDATE repositories SET watcher_count = GREATEST(watcher_count - 1, 0) WHERE id = $1"
    )
        .bind(repo.id)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// Topics

#[derive(Debug, Deserialize)]
pub struct UpdateTopicsInput {
    pub topics: Vec<String>,
}

pub async fn get_topics(
    State(state):   State<AppState>,
    Path((owner, repo_name)): Path<(String, String)>,
) -> Result<impl IntoResponse, AppError> {
    let owner = users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    let repo = repos::find_by_owner_and_name(&state.db, owner.id, &repo_name)
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    let topics: Vec<(String,)> = sqlx::query_as(
        "SELECT topic FROM repo_topics WHERE repo_id = $1 ORDER BY topic"
    )
        .bind(repo.id)
        .fetch_all(&state.db)
        .await
        .map_err(AppError::Database)?;

    let names: Vec<String> = topics.into_iter().map(|t| t.0).collect();
    Ok(Json(json!({ "topics": names })))
}

pub async fn update_topics(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name)): Path<(String, String)>,
    Json(input):    Json<UpdateTopicsInput>,
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

    sqlx::query("DELETE FROM repo_topics WHERE repo_id = $1")
        .bind(repo.id)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    for topic in &input.topics {
        let topic = topic.to_lowercase();
        sqlx::query(
            "INSERT INTO repo_topics (repo_id, topic) VALUES ($1, $2) ON CONFLICT DO NOTHING"
        )
            .bind(repo.id)
            .bind(&topic)
            .execute(&state.db)
            .await
            .map_err(AppError::Database)?;
    }

    Ok(Json(json!({ "topics": input.topics })))
}

// Collaborators

pub async fn list_collaborators(
    State(state):   State<AppState>,
    Path((owner, repo_name)): Path<(String, String)>,
) -> Result<impl IntoResponse, AppError> {
    let owner = users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    let repo = repos::find_by_owner_and_name(&state.db, owner.id, &repo_name)
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    let collaborators: Vec<crate::models::user::User> = sqlx::query_as(
        "SELECT u.* FROM users u
         JOIN repo_collaborators rc ON rc.user_id = u.id
         WHERE rc.repo_id = $1
         ORDER BY rc.created_at DESC"
    )
        .bind(repo.id)
        .fetch_all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(collaborators))
}

#[derive(Debug, Deserialize)]
pub struct AddCollaboratorInput {
    pub permission: Option<String>,
}

pub async fn add_collaborator(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name, username)): Path<(String, String, String)>,
    Json(input):    Json<AddCollaboratorInput>,
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

    let collaborator = users::find_by_username(&state.db, &username)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    let permission = input.permission.as_deref().unwrap_or("write");

    sqlx::query(
        "INSERT INTO repo_collaborators (repo_id, user_id, permission)
         VALUES ($1, $2, $3::collaborator_permission)
         ON CONFLICT (repo_id, user_id)
         DO UPDATE SET permission = $3::collaborator_permission"
    )
        .bind(repo.id)
        .bind(collaborator.id)
        .bind(permission)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn remove_collaborator(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name, username)): Path<(String, String, String)>,
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

    let collaborator = users::find_by_username(&state.db, &username)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    sqlx::query(
        "DELETE FROM repo_collaborators WHERE repo_id = $1 AND user_id = $2"
    )
        .bind(repo.id)
        .bind(collaborator.id)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// Branch protections

#[derive(Debug, Deserialize)]
pub struct BranchProtectionInput {
    pub pattern:                        String,
    pub require_pull_request:           Option<bool>,
    pub required_approving_review_count: Option<i32>,
    pub dismiss_stale_reviews:          Option<bool>,
    pub require_code_owner_reviews:     Option<bool>,
    pub require_status_checks:          Option<bool>,
    pub required_status_checks:         Option<Vec<String>>,
    pub require_up_to_date_branch:      Option<bool>,
    pub restrict_pushes:                Option<bool>,
    pub allow_force_pushes:             Option<bool>,
    pub allow_deletions:                Option<bool>,
}

pub async fn list_branch_protections(
    State(state):   State<AppState>,
    Path((owner, repo_name)): Path<(String, String)>,
) -> Result<impl IntoResponse, AppError> {
    let owner = users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    let repo = repos::find_by_owner_and_name(&state.db, owner.id, &repo_name)
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    let protections: Vec<RepoBranchProtection> = sqlx::query_as(
        "SELECT * FROM repo_branch_protections WHERE repo_id = $1"
    )
        .bind(repo.id)
        .fetch_all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(protections))
}

pub async fn create_branch_protection(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name)): Path<(String, String)>,
    Json(input):    Json<BranchProtectionInput>,
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

    let protection: RepoBranchProtection = sqlx::query_as(
        "INSERT INTO repo_branch_protections (
            repo_id, pattern, require_pull_request,
            required_approving_review_count, dismiss_stale_reviews,
            require_code_owner_reviews, require_status_checks,
            required_status_checks, require_up_to_date_branch,
            restrict_pushes, allow_force_pushes, allow_deletions
         ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)
         RETURNING *"
    )
        .bind(repo.id)
        .bind(&input.pattern)
        .bind(input.require_pull_request.unwrap_or(false))
        .bind(input.required_approving_review_count.unwrap_or(0))
        .bind(input.dismiss_stale_reviews.unwrap_or(false))
        .bind(input.require_code_owner_reviews.unwrap_or(false))
        .bind(input.require_status_checks.unwrap_or(false))
        .bind(input.required_status_checks.clone().unwrap_or_default())
        .bind(input.require_up_to_date_branch.unwrap_or(false))
        .bind(input.restrict_pushes.unwrap_or(false))
        .bind(input.allow_force_pushes.unwrap_or(false))
        .bind(input.allow_deletions.unwrap_or(false))
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok((StatusCode::CREATED, Json(protection)))
}

// Search repos

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q:        String,
    pub page:     Option<i64>,
    pub per_page: Option<i64>,
}

pub async fn search_repos(
    State(state):  State<AppState>,
    Query(params): Query<SearchQuery>,
) -> Result<impl IntoResponse, AppError> {
    let per_page = params.per_page.unwrap_or(30).min(100);
    let offset   = (params.page.unwrap_or(1) - 1) * per_page;
    let pattern  = format!("%{}%", params.q.to_lowercase());

    let repos: Vec<Repository> = sqlx::query_as(
        "SELECT * FROM repositories
         WHERE visibility = 'public'
           AND (LOWER(name) LIKE $1 OR LOWER(description) LIKE $1)
           AND is_archived = false
         ORDER BY star_count DESC, updated_at DESC
         LIMIT $2 OFFSET $3"
    )
        .bind(&pattern)
        .bind(per_page)
        .bind(offset)
        .fetch_all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(json!({
        "repositories": repos,
        "page": params.page.unwrap_or(1),
        "per_page": per_page,
    })))
}

// List stargazers

pub async fn list_stargazers(
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

    let stargazers: Vec<crate::models::user::User> = sqlx::query_as(
        "SELECT u.* FROM users u
         JOIN repo_stars s ON s.user_id = u.id
         WHERE s.repo_id = $1
         ORDER BY s.created_at DESC
         LIMIT $2 OFFSET $3"
    )
        .bind(repo.id)
        .bind(per_page)
        .bind(offset)
        .fetch_all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(stargazers))
}

// List forks

pub async fn list_forks(
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

    let forks: Vec<Repository> = sqlx::query_as(
        "SELECT * FROM repositories
         WHERE forked_from_id = $1
         ORDER BY created_at DESC
         LIMIT $2 OFFSET $3"
    )
        .bind(repo.id)
        .bind(per_page)
        .bind(offset)
        .fetch_all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(forks))
}

// Archive / Unarchive

pub async fn archive_repo(
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

    let repo: Repository = sqlx::query_as(
        "UPDATE repositories SET is_archived = true, updated_at = now()
         WHERE owner_id = $1 AND name = $2 RETURNING *"
    )
        .bind(auth_user.user_id)
        .bind(&repo_name)
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(repo))
}

pub async fn unarchive_repo(
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

    let repo: Repository = sqlx::query_as(
        "UPDATE repositories SET is_archived = false, updated_at = now()
         WHERE owner_id = $1 AND name = $2 RETURNING *"
    )
        .bind(auth_user.user_id)
        .bind(&repo_name)
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(repo))
}