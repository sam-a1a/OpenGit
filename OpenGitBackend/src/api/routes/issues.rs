use crate::{
    api::middleware::auth::AuthUser,
    db::queries::{repos, users},
    error::AppError,
    models::issue::{Issue, IssueComment, IssueReaction, IssueSubscription, Label, Milestone},
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
pub struct IssueQuery {
    pub state:    Option<String>,
    pub label:    Option<String>,
    pub assignee: Option<String>,
    pub milestone: Option<Uuid>,
    pub page:     Option<i64>,
    pub per_page: Option<i64>,
}

// Issues

#[derive(Debug, Deserialize)]
pub struct CreateIssueInput {
    pub title:        String,
    pub body:         Option<String>,
    pub assignees:    Option<Vec<String>>,
    pub labels:       Option<Vec<Uuid>>,
    pub milestone_id: Option<Uuid>,
}

pub async fn create_issue(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name)): Path<(String, String)>,
    Json(input):    Json<CreateIssueInput>,
) -> Result<impl IntoResponse, AppError> {
    if input.title.trim().is_empty() {
        return Err(AppError::BadRequest("Issue title is required".into()));
    }

    let owner = users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    let repo = repos::find_by_owner_and_name(&state.db, owner.id, &repo_name)
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    if !repo.has_issues {
        return Err(AppError::BadRequest("Issues are disabled for this repository".into()));
    }

    // get next issue number
    let row: (i64,) = sqlx::query_as(
        "SELECT COALESCE(MAX(number), 0) + 1 FROM issues WHERE repo_id = $1"
    )
        .bind(repo.id)
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    let number = row.0 as i32;

    let issue: Issue = sqlx::query_as(
        "INSERT INTO issues (repo_id, author_id, number, title, body, milestone_id)
         VALUES ($1, $2, $3, $4, $5, $6)
         RETURNING *"
    )
        .bind(repo.id)
        .bind(auth_user.user_id)
        .bind(number)
        .bind(&input.title)
        .bind(&input.body)
        .bind(input.milestone_id)
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    // add assignees
    if let Some(assignees) = &input.assignees {
        for username in assignees {
            if let Ok(Some(u)) = users::find_by_username(&state.db, username).await {
                sqlx::query(
                    "INSERT INTO issue_assignees (issue_id, user_id)
                     VALUES ($1, $2) ON CONFLICT DO NOTHING"
                )
                    .bind(issue.id)
                    .bind(u.id)
                    .execute(&state.db)
                    .await
                    .map_err(AppError::Database)?;
            }
        }
    }

    // add labels
    if let Some(labels) = &input.labels {
        for label_id in labels {
            sqlx::query(
                "INSERT INTO issue_labels (issue_id, label_id)
                 VALUES ($1, $2) ON CONFLICT DO NOTHING"
            )
                .bind(issue.id)
                .bind(label_id)
                .execute(&state.db)
                .await
                .map_err(AppError::Database)?;
        }
    }

    // update open issue count
    sqlx::query(
        "UPDATE repositories SET open_issue_count = open_issue_count + 1 WHERE id = $1"
    )
        .bind(repo.id)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok((StatusCode::CREATED, Json(issue)))
}

pub async fn list_issues(
    State(state):   State<AppState>,
    Path((owner, repo_name)): Path<(String, String)>,
    Query(params):  Query<IssueQuery>,
) -> Result<impl IntoResponse, AppError> {
    let owner = users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    let repo = repos::find_by_owner_and_name(&state.db, owner.id, &repo_name)
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    let per_page = params.per_page.unwrap_or(30).min(100);
    let offset   = (params.page.unwrap_or(1) - 1) * per_page;
    let state_filter = params.state.as_deref().unwrap_or("open");

    let issues: Vec<Issue> = sqlx::query_as(
        "SELECT i.* FROM issues i
         WHERE i.repo_id = $1
           AND i.state = $2::issue_state
         ORDER BY i.created_at DESC
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
        "issues":   issues,
        "page":     params.page.unwrap_or(1),
        "per_page": per_page,
    })))
}

pub async fn get_issue(
    State(state):   State<AppState>,
    Path((owner, repo_name, number)): Path<(String, String, i32)>,
) -> Result<impl IntoResponse, AppError> {
    let owner = users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    let repo = repos::find_by_owner_and_name(&state.db, owner.id, &repo_name)
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    let issue: Issue = sqlx::query_as(
        "SELECT * FROM issues WHERE repo_id = $1 AND number = $2"
    )
        .bind(repo.id)
        .bind(number)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound("Issue".into()))?;

    Ok(Json(issue))
}

#[derive(Debug, Deserialize)]
pub struct UpdateIssueInput {
    pub title:        Option<String>,
    pub body:         Option<String>,
    pub state:        Option<String>,
    pub state_reason: Option<String>,
    pub milestone_id: Option<Uuid>,
}

pub async fn update_issue(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name, number)): Path<(String, String, i32)>,
    Json(input):    Json<UpdateIssueInput>,
) -> Result<impl IntoResponse, AppError> {
    let owner = users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    let repo = repos::find_by_owner_and_name(&state.db, owner.id, &repo_name)
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    let existing: Issue = sqlx::query_as(
        "SELECT * FROM issues WHERE repo_id = $1 AND number = $2"
    )
        .bind(repo.id)
        .bind(number)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound("Issue".into()))?;

    if existing.author_id != Some(auth_user.user_id) && owner.id != auth_user.user_id {
        return Err(AppError::Forbidden);
    }

    let closing = input.state.as_deref() == Some("closed")
        && existing.state != crate::models::enums::IssueState::Closed;

    let opening = input.state.as_deref() == Some("open")
        && existing.state == crate::models::enums::IssueState::Closed;

    let issue: Issue = sqlx::query_as(
        "UPDATE issues SET
            title        = COALESCE($1, title),
            body         = COALESCE($2, body),
            milestone_id = COALESCE($3, milestone_id),
            state        = COALESCE($4::issue_state, state),
            state_reason = COALESCE($5::issue_state_reason, state_reason),
            closed_at    = CASE WHEN $4 = 'closed' THEN now() ELSE closed_at END,
            closed_by_id = CASE WHEN $4 = 'closed' THEN $6 ELSE closed_by_id END,
            updated_at   = now()
         WHERE repo_id = $7 AND number = $8
         RETURNING *"
    )
        .bind(&input.title)
        .bind(&input.body)
        .bind(input.milestone_id)
        .bind(input.state.as_deref())
        .bind(input.state_reason.as_deref())
        .bind(auth_user.user_id)
        .bind(repo.id)
        .bind(number)
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    // update open issue count
    if closing {
        sqlx::query(
            "UPDATE repositories SET open_issue_count = GREATEST(open_issue_count - 1, 0) WHERE id = $1"
        )
            .bind(repo.id)
            .execute(&state.db)
            .await
            .map_err(AppError::Database)?;
    } else if opening {
        sqlx::query(
            "UPDATE repositories SET open_issue_count = open_issue_count + 1 WHERE id = $1"
        )
            .bind(repo.id)
            .execute(&state.db)
            .await
            .map_err(AppError::Database)?;
    }

    Ok(Json(issue))
}

pub async fn lock_issue(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name, number)): Path<(String, String, i32)>,
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

    sqlx::query(
        "UPDATE issues SET locked = true, updated_at = now()
         WHERE repo_id = $1 AND number = $2"
    )
        .bind(repo.id)
        .bind(number)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn unlock_issue(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name, number)): Path<(String, String, i32)>,
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

    sqlx::query(
        "UPDATE issues SET locked = false, updated_at = now()
         WHERE repo_id = $1 AND number = $2"
    )
        .bind(repo.id)
        .bind(number)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn pin_issue(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name, number)): Path<(String, String, i32)>,
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

    sqlx::query(
        "UPDATE issues SET is_pinned = true, updated_at = now()
         WHERE repo_id = $1 AND number = $2"
    )
        .bind(repo.id)
        .bind(number)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn unpin_issue(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name, number)): Path<(String, String, i32)>,
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

    sqlx::query(
        "UPDATE issues SET is_pinned = false, updated_at = now()
         WHERE repo_id = $1 AND number = $2"
    )
        .bind(repo.id)
        .bind(number)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// Assignees

#[derive(Debug, Deserialize)]
pub struct AssigneesInput {
    pub assignees: Vec<String>,
}

pub async fn add_assignees(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name, number)): Path<(String, String, i32)>,
    Json(input):    Json<AssigneesInput>,
) -> Result<impl IntoResponse, AppError> {
    let owner = users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    let repo = repos::find_by_owner_and_name(&state.db, owner.id, &repo_name)
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    let issue: Issue = sqlx::query_as(
        "SELECT * FROM issues WHERE repo_id = $1 AND number = $2"
    )
        .bind(repo.id)
        .bind(number)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound("Issue".into()))?;

    for username in &input.assignees {
        if let Ok(Some(u)) = users::find_by_username(&state.db, username).await {
            sqlx::query(
                "INSERT INTO issue_assignees (issue_id, user_id)
                 VALUES ($1, $2) ON CONFLICT DO NOTHING"
            )
                .bind(issue.id)
                .bind(u.id)
                .execute(&state.db)
                .await
                .map_err(AppError::Database)?;
        }
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn remove_assignees(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name, number)): Path<(String, String, i32)>,
    Json(input):    Json<AssigneesInput>,
) -> Result<impl IntoResponse, AppError> {
    let owner = users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    let repo = repos::find_by_owner_and_name(&state.db, owner.id, &repo_name)
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    let issue: Issue = sqlx::query_as(
        "SELECT * FROM issues WHERE repo_id = $1 AND number = $2"
    )
        .bind(repo.id)
        .bind(number)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound("Issue".into()))?;

    for username in &input.assignees {
        if let Ok(Some(u)) = users::find_by_username(&state.db, username).await {
            sqlx::query(
                "DELETE FROM issue_assignees WHERE issue_id = $1 AND user_id = $2"
            )
                .bind(issue.id)
                .bind(u.id)
                .execute(&state.db)
                .await
                .map_err(AppError::Database)?;
        }
    }

    Ok(StatusCode::NO_CONTENT)
}

// Labels

#[derive(Debug, Deserialize)]
pub struct CreateLabelInput {
    pub name:        String,
    pub color:       String,
    pub description: Option<String>,
}

pub async fn list_labels(
    State(state):   State<AppState>,
    Path((owner, repo_name)): Path<(String, String)>,
) -> Result<impl IntoResponse, AppError> {
    let owner = users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    let repo = repos::find_by_owner_and_name(&state.db, owner.id, &repo_name)
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    let labels: Vec<Label> = sqlx::query_as(
        "SELECT * FROM labels WHERE repo_id = $1 ORDER BY name"
    )
        .bind(repo.id)
        .fetch_all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(labels))
}

pub async fn create_label(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name)): Path<(String, String)>,
    Json(input):    Json<CreateLabelInput>,
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

    let color = input.color.trim_start_matches('#').to_string();
    if color.len() != 6 {
        return Err(AppError::BadRequest("Color must be a 6-character hex code".into()));
    }

    let label: Label = sqlx::query_as(
        "INSERT INTO labels (repo_id, name, color, description)
         VALUES ($1, $2, $3, $4)
         RETURNING *"
    )
        .bind(repo.id)
        .bind(&input.name)
        .bind(&color)
        .bind(&input.description)
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok((StatusCode::CREATED, Json(label)))
}

pub async fn update_label(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name, label_id)): Path<(String, String, Uuid)>,
    Json(input):    Json<CreateLabelInput>,
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

    let label: Label = sqlx::query_as(
        "UPDATE labels SET name = $1, color = $2, description = $3
         WHERE id = $4 AND repo_id = $5
         RETURNING *"
    )
        .bind(&input.name)
        .bind(input.color.trim_start_matches('#'))
        .bind(&input.description)
        .bind(label_id)
        .bind(repo.id)
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(label))
}

pub async fn delete_label(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name, label_id)): Path<(String, String, Uuid)>,
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

    sqlx::query("DELETE FROM labels WHERE id = $1 AND repo_id = $2")
        .bind(label_id)
        .bind(repo.id)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize)]
pub struct IssueLabelsInput {
    pub labels: Vec<Uuid>,
}

pub async fn add_issue_labels(
    State(state):   State<AppState>,
    Path((owner, repo_name, number)): Path<(String, String, i32)>,
    Json(input):    Json<IssueLabelsInput>,
) -> Result<impl IntoResponse, AppError> {
    let owner = users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    let repo = repos::find_by_owner_and_name(&state.db, owner.id, &repo_name)
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    let issue: Issue = sqlx::query_as(
        "SELECT * FROM issues WHERE repo_id = $1 AND number = $2"
    )
        .bind(repo.id)
        .bind(number)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound("Issue".into()))?;

    for label_id in &input.labels {
        sqlx::query(
            "INSERT INTO issue_labels (issue_id, label_id)
             VALUES ($1, $2) ON CONFLICT DO NOTHING"
        )
            .bind(issue.id)
            .bind(label_id)
            .execute(&state.db)
            .await
            .map_err(AppError::Database)?;
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn remove_issue_label(
    State(state):   State<AppState>,
    Path((owner, repo_name, number, label_id)): Path<(String, String, i32, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let owner = users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    let repo = repos::find_by_owner_and_name(&state.db, owner.id, &repo_name)
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    let issue: Issue = sqlx::query_as(
        "SELECT * FROM issues WHERE repo_id = $1 AND number = $2"
    )
        .bind(repo.id)
        .bind(number)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound("Issue".into()))?;

    sqlx::query(
        "DELETE FROM issue_labels WHERE issue_id = $1 AND label_id = $2"
    )
        .bind(issue.id)
        .bind(label_id)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// Milestones

#[derive(Debug, Deserialize)]
pub struct CreateMilestoneInput {
    pub title:       String,
    pub description: Option<String>,
    pub due_on:      Option<chrono::DateTime<chrono::Utc>>,
}

pub async fn list_milestones(
    State(state):   State<AppState>,
    Path((owner, repo_name)): Path<(String, String)>,
) -> Result<impl IntoResponse, AppError> {
    let owner = users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    let repo = repos::find_by_owner_and_name(&state.db, owner.id, &repo_name)
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    let milestones: Vec<Milestone> = sqlx::query_as(
        "SELECT * FROM milestones WHERE repo_id = $1 ORDER BY created_at DESC"
    )
        .bind(repo.id)
        .fetch_all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(milestones))
}

pub async fn create_milestone(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name)): Path<(String, String)>,
    Json(input):    Json<CreateMilestoneInput>,
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

    let milestone: Milestone = sqlx::query_as(
        "INSERT INTO milestones (repo_id, title, description, due_on)
         VALUES ($1, $2, $3, $4)
         RETURNING *"
    )
        .bind(repo.id)
        .bind(&input.title)
        .bind(&input.description)
        .bind(input.due_on)
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok((StatusCode::CREATED, Json(milestone)))
}

pub async fn update_milestone(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name, milestone_id)): Path<(String, String, Uuid)>,
    Json(input):    Json<CreateMilestoneInput>,
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

    let milestone: Milestone = sqlx::query_as(
        "UPDATE milestones SET title = $1, description = $2, due_on = $3
         WHERE id = $4 AND repo_id = $5
         RETURNING *"
    )
        .bind(&input.title)
        .bind(&input.description)
        .bind(input.due_on)
        .bind(milestone_id)
        .bind(repo.id)
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(milestone))
}

pub async fn delete_milestone(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name, milestone_id)): Path<(String, String, Uuid)>,
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

    sqlx::query("DELETE FROM milestones WHERE id = $1 AND repo_id = $2")
        .bind(milestone_id)
        .bind(repo.id)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// Comments

#[derive(Debug, Deserialize)]
pub struct CommentInput {
    pub body: String,
}

pub async fn list_comments(
    State(state):   State<AppState>,
    Path((owner, repo_name, number)): Path<(String, String, i32)>,
    Query(pagination): Query<crate::api::routes::users::PaginationQuery>,
) -> Result<impl IntoResponse, AppError> {
    let owner = users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    let repo = repos::find_by_owner_and_name(&state.db, owner.id, &repo_name)
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    let issue: Issue = sqlx::query_as(
        "SELECT * FROM issues WHERE repo_id = $1 AND number = $2"
    )
        .bind(repo.id)
        .bind(number)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound("Issue".into()))?;

    let per_page = pagination.per_page.unwrap_or(30).min(100);
    let offset   = (pagination.page.unwrap_or(1) - 1) * per_page;

    let comments: Vec<IssueComment> = sqlx::query_as(
        "SELECT * FROM issue_comments
         WHERE issue_id = $1
         ORDER BY created_at ASC
         LIMIT $2 OFFSET $3"
    )
        .bind(issue.id)
        .bind(per_page)
        .bind(offset)
        .fetch_all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(comments))
}

pub async fn create_comment(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name, number)): Path<(String, String, i32)>,
    Json(input):    Json<CommentInput>,
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

    let issue: Issue = sqlx::query_as(
        "SELECT * FROM issues WHERE repo_id = $1 AND number = $2"
    )
        .bind(repo.id)
        .bind(number)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound("Issue".into()))?;

    if issue.locked && owner.id != auth_user.user_id {
        return Err(AppError::Forbidden);
    }

    let comment: IssueComment = sqlx::query_as(
        "INSERT INTO issue_comments (issue_id, author_id, body)
         VALUES ($1, $2, $3)
         RETURNING *"
    )
        .bind(issue.id)
        .bind(auth_user.user_id)
        .bind(&input.body)
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    sqlx::query(
        "UPDATE issues SET comment_count = comment_count + 1, updated_at = now()
         WHERE id = $1"
    )
        .bind(issue.id)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok((StatusCode::CREATED, Json(comment)))
}

pub async fn update_comment(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name, comment_id)): Path<(String, String, Uuid)>,
    Json(input):    Json<CommentInput>,
) -> Result<impl IntoResponse, AppError> {
    let owner = users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    let repo = repos::find_by_owner_and_name(&state.db, owner.id, &repo_name)
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    let comment: IssueComment = sqlx::query_as(
        "UPDATE issue_comments SET body = $1, is_edited = true, updated_at = now()
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

pub async fn delete_comment(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name, comment_id)): Path<(String, String, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let owner = users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    let repo = repos::find_by_owner_and_name(&state.db, owner.id, &repo_name)
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    let result = sqlx::query(
        "DELETE FROM issue_comments WHERE id = $1 AND author_id = $2"
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

// Reactions

#[derive(Debug, Deserialize)]
pub struct ReactionInput {
    pub reaction: String,
}

pub async fn add_issue_reaction(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name, number)): Path<(String, String, i32)>,
    Json(input):    Json<ReactionInput>,
) -> Result<impl IntoResponse, AppError> {
    let owner = users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    let repo = repos::find_by_owner_and_name(&state.db, owner.id, &repo_name)
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    let issue: Issue = sqlx::query_as(
        "SELECT * FROM issues WHERE repo_id = $1 AND number = $2"
    )
        .bind(repo.id)
        .bind(number)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound("Issue".into()))?;

    let reaction: IssueReaction = sqlx::query_as(
        "INSERT INTO issue_reactions (user_id, issue_id, reaction)
         VALUES ($1, $2, $3::reaction_type)
         ON CONFLICT (user_id, issue_id, reaction) DO NOTHING
         RETURNING *"
    )
        .bind(auth_user.user_id)
        .bind(issue.id)
        .bind(&input.reaction)
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok((StatusCode::CREATED, Json(reaction)))
}

pub async fn remove_issue_reaction(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name, number, reaction_id)): Path<(String, String, i32, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let owner = users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    repos::find_by_owner_and_name(&state.db, owner.id, &repo_name)
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    sqlx::query(
        "DELETE FROM issue_reactions WHERE id = $1 AND user_id = $2"
    )
        .bind(reaction_id)
        .bind(auth_user.user_id)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn add_comment_reaction(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name, comment_id)): Path<(String, String, Uuid)>,
    Json(input):    Json<ReactionInput>,
) -> Result<impl IntoResponse, AppError> {
    let owner = users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    repos::find_by_owner_and_name(&state.db, owner.id, &repo_name)
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    let reaction: IssueReaction = sqlx::query_as(
        "INSERT INTO issue_reactions (user_id, comment_id, reaction)
         VALUES ($1, $2, $3::reaction_type)
         ON CONFLICT (user_id, comment_id, reaction) DO NOTHING
         RETURNING *"
    )
        .bind(auth_user.user_id)
        .bind(comment_id)
        .bind(&input.reaction)
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok((StatusCode::CREATED, Json(reaction)))
}

// Subscriptions

pub async fn subscribe_issue(
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

    let issue: Issue = sqlx::query_as(
        "SELECT * FROM issues WHERE repo_id = $1 AND number = $2"
    )
        .bind(repo.id)
        .bind(number)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound("Issue".into()))?;

    sqlx::query(
        "INSERT INTO issue_subscriptions (user_id, issue_id, subscribed)
         VALUES ($1, $2, true)
         ON CONFLICT (user_id, issue_id) DO UPDATE SET subscribed = true"
    )
        .bind(auth_user.user_id)
        .bind(issue.id)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn unsubscribe_issue(
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

    let issue: Issue = sqlx::query_as(
        "SELECT * FROM issues WHERE repo_id = $1 AND number = $2"
    )
        .bind(repo.id)
        .bind(number)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound("Issue".into()))?;

    sqlx::query(
        "INSERT INTO issue_subscriptions (user_id, issue_id, subscribed)
         VALUES ($1, $2, false)
         ON CONFLICT (user_id, issue_id) DO UPDATE SET subscribed = false"
    )
        .bind(auth_user.user_id)
        .bind(issue.id)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}