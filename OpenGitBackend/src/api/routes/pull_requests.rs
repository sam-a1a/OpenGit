use crate::{
    api::middleware::auth::AuthUser,
    db::queries::{repos, users},
    error::AppError,
    models::pull_request::{PrReview, PrReviewComment, PrStatusCheck, PullRequest},
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
pub struct PrQuery {
    pub state:    Option<String>,
    pub page:     Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub page:     Option<i64>,
    pub per_page: Option<i64>,
}

// Create PR

#[derive(Debug, Deserialize)]
pub struct CreatePrInput {
    pub title:        String,
    pub body:         Option<String>,
    pub head:         String,
    pub base:         String,
    pub draft:        Option<bool>,
    pub milestone_id: Option<Uuid>,
}

pub async fn create_pr(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name)): Path<(String, String)>,
    Json(input):    Json<CreatePrInput>,
) -> Result<impl IntoResponse, AppError> {
    if input.title.trim().is_empty() {
        return Err(AppError::BadRequest("PR title is required".into()));
    }
    if input.head == input.base {
        return Err(AppError::BadRequest("Head and base branches must be different".into()));
    }

    let owner = users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    let repo = repos::find_by_owner_and_name(&state.db, owner.id, &repo_name)
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    let row: (i64,) = sqlx::query_as(
        "SELECT COALESCE(MAX(number), 0) + 1 FROM pull_requests WHERE repo_id = $1"
    )
        .bind(repo.id)
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    let number = row.0 as i32;

    let pr: PullRequest = sqlx::query_as(
        "INSERT INTO pull_requests
            (repo_id, author_id, number, title, body,
             head_branch, base_branch, is_draft, milestone_id)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
         RETURNING *"
    )
        .bind(repo.id)
        .bind(auth_user.user_id)
        .bind(number)
        .bind(&input.title)
        .bind(&input.body)
        .bind(&input.head)
        .bind(&input.base)
        .bind(input.draft.unwrap_or(false))
        .bind(input.milestone_id)
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok((StatusCode::CREATED, Json(pr)))
}

// List PRs

pub async fn list_prs(
    State(state):   State<AppState>,
    Path((owner, repo_name)): Path<(String, String)>,
    Query(params):  Query<PrQuery>,
) -> Result<impl IntoResponse, AppError> {
    let owner = users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    let repo = repos::find_by_owner_and_name(&state.db, owner.id, &repo_name)
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    let per_page    = params.per_page.unwrap_or(30).min(100);
    let offset      = (params.page.unwrap_or(1) - 1) * per_page;
    let state_filter = params.state.as_deref().unwrap_or("open");

    let prs: Vec<PullRequest> = sqlx::query_as(
        "SELECT * FROM pull_requests
         WHERE repo_id = $1 AND state = $2::pr_state
         ORDER BY created_at DESC
         LIMIT $3 OFFSET $4"
    )
        .bind(repo.id)
        .bind(state_filter)
        .bind(per_page)
        .bind(offset)
        .fetch_all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(json!({
        "pull_requests": prs,
        "page":          params.page.unwrap_or(1),
        "per_page":      per_page,
    })))
}

// Get PR

pub async fn get_pr(
    State(state):   State<AppState>,
    Path((owner, repo_name, number)): Path<(String, String, i32)>,
) -> Result<impl IntoResponse, AppError> {
    let owner = users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    let repo = repos::find_by_owner_and_name(&state.db, owner.id, &repo_name)
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    let pr: PullRequest = sqlx::query_as(
        "SELECT * FROM pull_requests WHERE repo_id = $1 AND number = $2"
    )
        .bind(repo.id)
        .bind(number)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound("Pull request".into()))?;

    Ok(Json(pr))
}

// Update PR

#[derive(Debug, Deserialize)]
pub struct UpdatePrInput {
    pub title:        Option<String>,
    pub body:         Option<String>,
    pub base:         Option<String>,
    pub milestone_id: Option<Uuid>,
    pub draft:        Option<bool>,
}

pub async fn update_pr(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name, number)): Path<(String, String, i32)>,
    Json(input):    Json<UpdatePrInput>,
) -> Result<impl IntoResponse, AppError> {
    let owner = users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    let repo = repos::find_by_owner_and_name(&state.db, owner.id, &repo_name)
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    let existing: PullRequest = sqlx::query_as(
        "SELECT * FROM pull_requests WHERE repo_id = $1 AND number = $2"
    )
        .bind(repo.id)
        .bind(number)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound("Pull request".into()))?;

    if existing.author_id != Some(auth_user.user_id) && owner.id != auth_user.user_id {
        return Err(AppError::Forbidden);
    }

    let pr: PullRequest = sqlx::query_as(
        "UPDATE pull_requests SET
            title        = COALESCE($1, title),
            body         = COALESCE($2, body),
            base_branch  = COALESCE($3, base_branch),
            milestone_id = COALESCE($4, milestone_id),
            is_draft     = COALESCE($5, is_draft),
            updated_at   = now()
         WHERE repo_id = $6 AND number = $7
         RETURNING *"
    )
        .bind(&input.title)
        .bind(&input.body)
        .bind(&input.base)
        .bind(input.milestone_id)
        .bind(input.draft)
        .bind(repo.id)
        .bind(number)
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(pr))
}

// Close PR

pub async fn close_pr(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name, number)): Path<(String, String, i32)>,
) -> Result<impl IntoResponse, AppError> {
    let owner = users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    let repo = repos::find_by_owner_and_name(&state.db, owner.id, &repo_name)
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    let existing: PullRequest = sqlx::query_as(
        "SELECT * FROM pull_requests WHERE repo_id = $1 AND number = $2"
    )
        .bind(repo.id)
        .bind(number)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound("Pull request".into()))?;

    if existing.author_id != Some(auth_user.user_id) && owner.id != auth_user.user_id {
        return Err(AppError::Forbidden);
    }

    if existing.state == crate::models::enums::PrState::Merged {
        return Err(AppError::BadRequest("Cannot close a merged pull request".into()));
    }

    let pr: PullRequest = sqlx::query_as(
        "UPDATE pull_requests SET
            state      = 'closed'::pr_state,
            closed_at  = now(),
            updated_at = now()
         WHERE repo_id = $1 AND number = $2
         RETURNING *"
    )
        .bind(repo.id)
        .bind(number)
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(pr))
}

// Reopen PR

pub async fn reopen_pr(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name, number)): Path<(String, String, i32)>,
) -> Result<impl IntoResponse, AppError> {
    let owner = users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    let repo = repos::find_by_owner_and_name(&state.db, owner.id, &repo_name)
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    let existing: PullRequest = sqlx::query_as(
        "SELECT * FROM pull_requests WHERE repo_id = $1 AND number = $2"
    )
        .bind(repo.id)
        .bind(number)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound("Pull request".into()))?;

    if existing.state == crate::models::enums::PrState::Merged {
        return Err(AppError::BadRequest("Cannot reopen a merged pull request".into()));
    }

    let pr: PullRequest = sqlx::query_as(
        "UPDATE pull_requests SET
            state      = 'open'::pr_state,
            closed_at  = NULL,
            updated_at = now()
         WHERE repo_id = $1 AND number = $2
         RETURNING *"
    )
        .bind(repo.id)
        .bind(number)
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(pr))
}

// Merge PR

#[derive(Debug, Deserialize)]
pub struct MergePrInput {
    pub commit_title:   Option<String>,
    pub commit_message: Option<String>,
    pub merge_method:   Option<String>,
}

pub async fn merge_pr(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name, number)): Path<(String, String, i32)>,
    Json(input):    Json<MergePrInput>,
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

    let existing: PullRequest = sqlx::query_as(
        "SELECT * FROM pull_requests WHERE repo_id = $1 AND number = $2"
    )
        .bind(repo.id)
        .bind(number)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound("Pull request".into()))?;

    if existing.state != crate::models::enums::PrState::Open {
        return Err(AppError::BadRequest("Pull request is not open".into()));
    }

    if existing.is_draft {
        return Err(AppError::BadRequest("Cannot merge a draft pull request".into()));
    }

    let merge_method = input.merge_method.as_deref().unwrap_or("merge");
    if !["merge", "squash", "rebase"].contains(&merge_method) {
        return Err(AppError::BadRequest("merge_method must be merge, squash, or rebase".into()));
    }

    let repo_path = crate::git::repository::repo_path(
        &state.config.git_base_dir,
        &repo.git_path,
    );

    // perform the git merge
    let merge_commit_sha = perform_git_merge(
        &repo_path,
        &existing.head_branch,
        &existing.base_branch,
        merge_method,
        input.commit_title.as_deref(),
        input.commit_message.as_deref(),
    ).await?;

    let pr: PullRequest = sqlx::query_as(
        "UPDATE pull_requests SET
            state            = 'merged'::pr_state,
            merged_at        = now(),
            merged_by_id     = $1,
            merge_commit_sha = $2,
            closed_at        = now(),
            updated_at       = now()
         WHERE repo_id = $3 AND number = $4
         RETURNING *"
    )
        .bind(auth_user.user_id)
        .bind(&merge_commit_sha)
        .bind(repo.id)
        .bind(number)
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    // delete head branch if configured
    if repo.delete_branch_on_merge {
        let _ = tokio::process::Command::new("git")
            .args(["branch", "-D", &existing.head_branch])
            .current_dir(&repo_path)
            .output()
            .await;
    }

    Ok(Json(json!({
        "merged":          true,
        "merge_method":    merge_method,
        "merge_commit_sha": merge_commit_sha,
        "pull_request":    pr,
    })))
}

async fn perform_git_merge(
    repo_path:      &std::path::PathBuf,
    head_branch:    &str,
    base_branch:    &str,
    method:         &str,
    commit_title:   Option<&str>,
    commit_message: Option<&str>,
) -> Result<String, AppError> {
    let default_title = format!("Merge branch '{}'", head_branch);
    let title = commit_title.unwrap_or(&default_title);

    match method {
        "merge" => {
            tokio::process::Command::new("git")
                .args(["checkout", base_branch])
                .current_dir(repo_path)
                .output().await
                .map_err(|e| AppError::Internal(anyhow::anyhow!("git checkout failed: {}", e)))?;

            tokio::process::Command::new("git")
                .args(["merge", "--no-ff", "-m", title, head_branch])
                .current_dir(repo_path)
                .output().await
                .map_err(|e| AppError::Internal(anyhow::anyhow!("git merge failed: {}", e)))?;
        }
        "squash" => {
            tokio::process::Command::new("git")
                .args(["checkout", base_branch])
                .current_dir(repo_path)
                .output().await
                .map_err(|e| AppError::Internal(anyhow::anyhow!("git checkout failed: {}", e)))?;

            tokio::process::Command::new("git")
                .args(["merge", "--squash", head_branch])
                .current_dir(repo_path)
                .output().await
                .map_err(|e| AppError::Internal(anyhow::anyhow!("git squash failed: {}", e)))?;

            tokio::process::Command::new("git")
                .args(["commit", "-m", title])
                .current_dir(repo_path)
                .output().await
                .map_err(|e| AppError::Internal(anyhow::anyhow!("git commit failed: {}", e)))?;
        }
        "rebase" => {
            tokio::process::Command::new("git")
                .args(["checkout", head_branch])
                .current_dir(repo_path)
                .output().await
                .map_err(|e| AppError::Internal(anyhow::anyhow!("git checkout failed: {}", e)))?;

            tokio::process::Command::new("git")
                .args(["rebase", base_branch])
                .current_dir(repo_path)
                .output().await
                .map_err(|e| AppError::Internal(anyhow::anyhow!("git rebase failed: {}", e)))?;

            tokio::process::Command::new("git")
                .args(["checkout", base_branch])
                .current_dir(repo_path)
                .output().await
                .map_err(|e| AppError::Internal(anyhow::anyhow!("git checkout failed: {}", e)))?;

            tokio::process::Command::new("git")
                .args(["merge", "--ff-only", head_branch])
                .current_dir(repo_path)
                .output().await
                .map_err(|e| AppError::Internal(anyhow::anyhow!("git ff-only failed: {}", e)))?;
        }
        _ => {}
    }

    let sha_output = tokio::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(repo_path)
        .output().await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("git rev-parse failed: {}", e)))?;

    let sha = String::from_utf8_lossy(&sha_output.stdout).trim().to_string();
    Ok(sha)
}

// Reviews

#[derive(Debug, Deserialize)]
pub struct CreateReviewInput {
    pub body:   Option<String>,
    pub event:  String,
    pub comments: Option<Vec<ReviewCommentInput>>,
}

#[derive(Debug, Deserialize)]
pub struct ReviewCommentInput {
    pub path:       String,
    pub line:       Option<i32>,
    pub body:       String,
    pub commit_sha: Option<String>,
}

pub async fn list_reviews(
    State(state):   State<AppState>,
    Path((owner, repo_name, number)): Path<(String, String, i32)>,
) -> Result<impl IntoResponse, AppError> {
    let owner = users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    let repo = repos::find_by_owner_and_name(&state.db, owner.id, &repo_name)
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    let pr: PullRequest = sqlx::query_as(
        "SELECT * FROM pull_requests WHERE repo_id = $1 AND number = $2"
    )
        .bind(repo.id)
        .bind(number)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound("Pull request".into()))?;

    let reviews: Vec<PrReview> = sqlx::query_as(
        "SELECT * FROM pr_reviews WHERE pr_id = $1 ORDER BY created_at ASC"
    )
        .bind(pr.id)
        .fetch_all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(reviews))
}

pub async fn create_review(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name, number)): Path<(String, String, i32)>,
    Json(input):    Json<CreateReviewInput>,
) -> Result<impl IntoResponse, AppError> {
    let valid_events = ["APPROVE", "REQUEST_CHANGES", "COMMENT", "PENDING"];
    if !valid_events.contains(&input.event.as_str()) {
        return Err(AppError::BadRequest(
            "event must be APPROVE, REQUEST_CHANGES, COMMENT, or PENDING".into()
        ));
    }

    let owner = users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    let repo = repos::find_by_owner_and_name(&state.db, owner.id, &repo_name)
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    let pr: PullRequest = sqlx::query_as(
        "SELECT * FROM pull_requests WHERE repo_id = $1 AND number = $2"
    )
        .bind(repo.id)
        .bind(number)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound("Pull request".into()))?;

    if pr.author_id == Some(auth_user.user_id) {
        return Err(AppError::BadRequest("You cannot review your own pull request".into()));
    }

    let review_state = match input.event.as_str() {
        "APPROVE"          => "approved",
        "REQUEST_CHANGES"  => "changes_requested",
        "COMMENT"          => "commented",
        _                  => "pending",
    };

    let review: PrReview = sqlx::query_as(
        "INSERT INTO pr_reviews (pr_id, reviewer_id, state, body, submitted_at)
         VALUES ($1, $2, $3::pr_review_state, $4, now())
         RETURNING *"
    )
        .bind(pr.id)
        .bind(auth_user.user_id)
        .bind(review_state)
        .bind(&input.body)
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    // add inline comments if provided
    if let Some(comments) = &input.comments {
        for c in comments {
            sqlx::query(
                "INSERT INTO pr_review_comments
                    (pr_id, review_id, author_id, body, path, line, commit_sha)
                 VALUES ($1, $2, $3, $4, $5, $6, $7)"
            )
                .bind(pr.id)
                .bind(review.id)
                .bind(auth_user.user_id)
                .bind(&c.body)
                .bind(&c.path)
                .bind(c.line)
                .bind(&c.commit_sha)
                .execute(&state.db)
                .await
                .map_err(AppError::Database)?;
        }
    }

    Ok((StatusCode::CREATED, Json(review)))
}

pub async fn dismiss_review(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name, number, review_id)): Path<(String, String, i32, Uuid)>,
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

    let pr: PullRequest = sqlx::query_as(
        "SELECT * FROM pull_requests WHERE repo_id = $1 AND number = $2"
    )
        .bind(repo.id)
        .bind(number)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound("Pull request".into()))?;

    let review: PrReview = sqlx::query_as(
        "UPDATE pr_reviews SET state = 'dismissed'::pr_review_state
         WHERE id = $1 AND pr_id = $2
         RETURNING *"
    )
        .bind(review_id)
        .bind(pr.id)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound("Review".into()))?;

    Ok(Json(review))
}

// Review comments

pub async fn list_review_comments(
    State(state):   State<AppState>,
    Path((owner, repo_name, number)): Path<(String, String, i32)>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<impl IntoResponse, AppError> {
    let owner = users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    let repo = repos::find_by_owner_and_name(&state.db, owner.id, &repo_name)
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    let pr: PullRequest = sqlx::query_as(
        "SELECT * FROM pull_requests WHERE repo_id = $1 AND number = $2"
    )
        .bind(repo.id)
        .bind(number)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound("Pull request".into()))?;

    let per_page = pagination.per_page.unwrap_or(30).min(100);
    let offset   = (pagination.page.unwrap_or(1) - 1) * per_page;

    let comments: Vec<PrReviewComment> = sqlx::query_as(
        "SELECT * FROM pr_review_comments
         WHERE pr_id = $1
         ORDER BY created_at ASC
         LIMIT $2 OFFSET $3"
    )
        .bind(pr.id)
        .bind(per_page)
        .bind(offset)
        .fetch_all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(comments))
}

#[derive(Debug, Deserialize)]
pub struct ReviewCommentBody {
    pub body:       String,
    pub path:       Option<String>,
    pub line:       Option<i32>,
    pub start_line: Option<i32>,
    pub side:       Option<String>,
    pub commit_sha: Option<String>,
    pub reply_to_id: Option<Uuid>,
}

pub async fn create_review_comment(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name, number)): Path<(String, String, i32)>,
    Json(input):    Json<ReviewCommentBody>,
) -> Result<impl IntoResponse, AppError> {
    if input.body.trim().is_empty() {
        return Err(AppError::BadRequest("Comment body is required".into()));
    }

    let owner = users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    let repo = repos::find_by_owner_and_name(&state.db, owner.id, &repo_name)
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    let pr: PullRequest = sqlx::query_as(
        "SELECT * FROM pull_requests WHERE repo_id = $1 AND number = $2"
    )
        .bind(repo.id)
        .bind(number)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound("Pull request".into()))?;

    let comment: PrReviewComment = sqlx::query_as(
        "INSERT INTO pr_review_comments
            (pr_id, author_id, body, path, line, start_line, side, commit_sha, reply_to_id)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
         RETURNING *"
    )
        .bind(pr.id)
        .bind(auth_user.user_id)
        .bind(&input.body)
        .bind(&input.path)
        .bind(input.line)
        .bind(input.start_line)
        .bind(input.side.as_deref().unwrap_or("RIGHT"))
        .bind(&input.commit_sha)
        .bind(input.reply_to_id)
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    sqlx::query(
        "UPDATE pull_requests SET comment_count = comment_count + 1, updated_at = now()
         WHERE id = $1"
    )
        .bind(pr.id)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok((StatusCode::CREATED, Json(comment)))
}

pub async fn update_review_comment(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name, comment_id)): Path<(String, String, Uuid)>,
    Json(input):    Json<ReviewCommentBody>,
) -> Result<impl IntoResponse, AppError> {
    let owner = users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    repos::find_by_owner_and_name(&state.db, owner.id, &repo_name)
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    let comment: PrReviewComment = sqlx::query_as(
        "UPDATE pr_review_comments
         SET body = $1, is_edited = true, updated_at = now()
         WHERE id = $2 AND author_id = $3
         RETURNING *"
    )
        .bind(&input.body)
        .bind(comment_id)
        .bind(auth_user.user_id)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound("Comment".into()))?;

    Ok(Json(comment))
}

pub async fn delete_review_comment(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name, comment_id)): Path<(String, String, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let owner = users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    repos::find_by_owner_and_name(&state.db, owner.id, &repo_name)
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    let result = sqlx::query(
        "DELETE FROM pr_review_comments WHERE id = $1 AND author_id = $2"
    )
        .bind(comment_id)
        .bind(auth_user.user_id)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Comment".into()));
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn resolve_review_comment(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name, comment_id)): Path<(String, String, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let owner = users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    repos::find_by_owner_and_name(&state.db, owner.id, &repo_name)
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    sqlx::query(
        "UPDATE pr_review_comments SET resolved = true WHERE id = $1"
    )
        .bind(comment_id)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// Requested reviewers

#[derive(Debug, Deserialize)]
pub struct ReviewersInput {
    pub reviewers: Vec<String>,
}

pub async fn request_reviewers(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name, number)): Path<(String, String, i32)>,
    Json(input):    Json<ReviewersInput>,
) -> Result<impl IntoResponse, AppError> {
    let owner = users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    let repo = repos::find_by_owner_and_name(&state.db, owner.id, &repo_name)
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    let pr: PullRequest = sqlx::query_as(
        "SELECT * FROM pull_requests WHERE repo_id = $1 AND number = $2"
    )
        .bind(repo.id)
        .bind(number)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound("Pull request".into()))?;

    for username in &input.reviewers {
        if let Ok(Some(u)) = users::find_by_username(&state.db, username).await {
            sqlx::query(
                "INSERT INTO pr_requested_reviewers (pr_id, user_id)
                 VALUES ($1, $2)"
            )
                .bind(pr.id)
                .bind(u.id)
                .execute(&state.db)
                .await
                .map_err(AppError::Database)?;
        }
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn remove_reviewers(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name, number)): Path<(String, String, i32)>,
    Json(input):    Json<ReviewersInput>,
) -> Result<impl IntoResponse, AppError> {
    let owner = users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    let repo = repos::find_by_owner_and_name(&state.db, owner.id, &repo_name)
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    let pr: PullRequest = sqlx::query_as(
        "SELECT * FROM pull_requests WHERE repo_id = $1 AND number = $2"
    )
        .bind(repo.id)
        .bind(number)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound("Pull request".into()))?;

    for username in &input.reviewers {
        if let Ok(Some(u)) = users::find_by_username(&state.db, username).await {
            sqlx::query(
                "DELETE FROM pr_requested_reviewers WHERE pr_id = $1 AND user_id = $2"
            )
                .bind(pr.id)
                .bind(u.id)
                .execute(&state.db)
                .await
                .map_err(AppError::Database)?;
        }
    }

    Ok(StatusCode::NO_CONTENT)
}

// Status checks

#[derive(Debug, Deserialize)]
pub struct CreateStatusInput {
    pub name:        String,
    pub context:     Option<String>,
    pub status:      String,
    pub conclusion:  Option<String>,
    pub target_url:  Option<String>,
    pub description: Option<String>,
}

pub async fn create_status(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name, sha)): Path<(String, String, String)>,
    Json(input):    Json<CreateStatusInput>,
) -> Result<impl IntoResponse, AppError> {
    let owner = users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    let repo = repos::find_by_owner_and_name(&state.db, owner.id, &repo_name)
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    let check: PrStatusCheck = sqlx::query_as(
        "INSERT INTO pr_status_checks
            (repo_id, sha, name, context, status, conclusion, target_url, description)
         VALUES ($1, $2, $3, $4, $5::check_status, $6::check_conclusion, $7, $8)
         RETURNING *"
    )
        .bind(repo.id)
        .bind(&sha)
        .bind(&input.name)
        .bind(&input.context)
        .bind(&input.status)
        .bind(&input.conclusion)
        .bind(&input.target_url)
        .bind(&input.description)
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok((StatusCode::CREATED, Json(check)))
}

pub async fn list_statuses(
    State(state):   State<AppState>,
    Path((owner, repo_name, sha)): Path<(String, String, String)>,
) -> Result<impl IntoResponse, AppError> {
    let owner = users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    let repo = repos::find_by_owner_and_name(&state.db, owner.id, &repo_name)
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    let checks: Vec<PrStatusCheck> = sqlx::query_as(
        "SELECT * FROM pr_status_checks
         WHERE repo_id = $1 AND sha = $2
         ORDER BY created_at DESC"
    )
        .bind(repo.id)
        .bind(&sha)
        .fetch_all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(checks))
}

// PR labels & assignees

#[derive(Debug, Deserialize)]
pub struct PrLabelsInput {
    pub labels: Vec<Uuid>,
}

pub async fn add_pr_labels(
    State(state):   State<AppState>,
    Path((owner, repo_name, number)): Path<(String, String, i32)>,
    Json(input):    Json<PrLabelsInput>,
) -> Result<impl IntoResponse, AppError> {
    let owner = users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    let repo = repos::find_by_owner_and_name(&state.db, owner.id, &repo_name)
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    let pr: PullRequest = sqlx::query_as(
        "SELECT * FROM pull_requests WHERE repo_id = $1 AND number = $2"
    )
        .bind(repo.id)
        .bind(number)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound("Pull request".into()))?;

    for label_id in &input.labels {
        sqlx::query(
            "INSERT INTO pr_labels (pr_id, label_id) VALUES ($1, $2) ON CONFLICT DO NOTHING"
        )
            .bind(pr.id)
            .bind(label_id)
            .execute(&state.db)
            .await
            .map_err(AppError::Database)?;
    }

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize)]
pub struct PrAssigneesInput {
    pub assignees: Vec<String>,
}

pub async fn add_pr_assignees(
    State(state):   State<AppState>,
    Path((owner, repo_name, number)): Path<(String, String, i32)>,
    Json(input):    Json<PrAssigneesInput>,
) -> Result<impl IntoResponse, AppError> {
    let owner = users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    let repo = repos::find_by_owner_and_name(&state.db, owner.id, &repo_name)
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    let pr: PullRequest = sqlx::query_as(
        "SELECT * FROM pull_requests WHERE repo_id = $1 AND number = $2"
    )
        .bind(repo.id)
        .bind(number)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound("Pull request".into()))?;

    for username in &input.assignees {
        if let Ok(Some(u)) = users::find_by_username(&state.db, username).await {
            sqlx::query(
                "INSERT INTO pr_assignees (pr_id, user_id) VALUES ($1, $2) ON CONFLICT DO NOTHING"
            )
                .bind(pr.id)
                .bind(u.id)
                .execute(&state.db)
                .await
                .map_err(AppError::Database)?;
        }
    }

    Ok(StatusCode::NO_CONTENT)
}

// Check if PR is merged

pub async fn is_merged(
    State(state):   State<AppState>,
    Path((owner, repo_name, number)): Path<(String, String, i32)>,
) -> Result<impl IntoResponse, AppError> {
    let owner = users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    let repo = repos::find_by_owner_and_name(&state.db, owner.id, &repo_name)
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    let pr: PullRequest = sqlx::query_as(
        "SELECT * FROM pull_requests WHERE repo_id = $1 AND number = $2"
    )
        .bind(repo.id)
        .bind(number)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound("Pull request".into()))?;

    if pr.state == crate::models::enums::PrState::Merged {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Ok(StatusCode::NOT_FOUND)
    }
}