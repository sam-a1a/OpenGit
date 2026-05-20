use crate::{
    api::middleware::auth::AuthUser,
    db::queries::{repos, users},
    error::AppError,
    models::ci::{Artifact, Runner, RunnerGroup, Workflow, WorkflowJob, WorkflowRun, WorkflowStep},
    state::AppState,
};
use axum::{
    body::Bytes,
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
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

#[derive(Debug, Deserialize)]
pub struct RunQuery {
    pub status:   Option<String>,
    pub event:    Option<String>,
    pub branch:   Option<String>,
    pub page:     Option<i64>,
    pub per_page: Option<i64>,
}

// Private deduplication helper functions

fn get_limit_and_offset(page: Option<i64>, per_page: Option<i64>, default_per_page: i64) -> (i64, i64) {
    let limit = per_page.unwrap_or(default_per_page).min(100);
    let offset = (page.unwrap_or(1) - 1) * limit;
    (limit, offset)
}

async fn fetch_repo(
    db: &sqlx::PgPool,
    owner_name: &str,
    repo_name: &str,
) -> Result<(Uuid, crate::models::repo::Repository), AppError> {
    let owner = users::find_by_username(db, owner_name)
        .await?
        .ok_or_else(|| AppError::NotFound("User".into()))?;

    let repo = repos::find_by_owner_and_name(db, owner.id, repo_name)
        .await?
        .ok_or_else(|| AppError::NotFound("Repository".into()))?;

    Ok((owner.id, repo))
}

async fn fetch_repo_with_auth(
    db: &sqlx::PgPool,
    owner_name: &str,
    repo_name: &str,
    auth_user_id: Uuid,
) -> Result<crate::models::repo::Repository, AppError> {
    let (owner_id, repo) = fetch_repo(db, owner_name, repo_name).await?;
    if owner_id != auth_user_id {
        return Err(AppError::Forbidden);
    }
    Ok(repo)
}

fn get_s3_client(state: &AppState) -> aws_sdk_s3::Client {
    let creds = aws_sdk_s3::config::Credentials::new(
        &state.config.minio_access_key,
        &state.config.minio_secret_key,
        None, None, "opengit",
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

async fn fetch_workflow_by_repo(db: &sqlx::PgPool, id: Uuid, repo_id: Uuid) -> Result<Workflow, AppError> {
    sqlx::query_as("SELECT * FROM workflows WHERE id = $1 AND repo_id = $2")
        .bind(id)
        .bind(repo_id)
        .fetch_optional(db)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound("Workflow".into()))
}

async fn fetch_run_by_repo(db: &sqlx::PgPool, id: Uuid, repo_id: Uuid) -> Result<WorkflowRun, AppError> {
    sqlx::query_as("SELECT * FROM workflow_runs WHERE id = $1 AND repo_id = $2")
        .bind(id)
        .bind(repo_id)
        .fetch_optional(db)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound("Workflow run".into()))
}

async fn fetch_artifact_by_repo(db: &sqlx::PgPool, id: Uuid, repo_id: Uuid) -> Result<Artifact, AppError> {
    sqlx::query_as("SELECT * FROM artifacts WHERE id = $1 AND repo_id = $2")
        .bind(id)
        .bind(repo_id)
        .fetch_optional(db)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound("Artifact".into()))
}

async fn update_workflow_state(
    db: &sqlx::PgPool,
    workflow_id: Uuid,
    repo_id: Uuid,
    state: &str,
) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE workflows SET state = $1, updated_at = now()
         WHERE id = $2 AND repo_id = $3"
    )
        .bind(state)
        .bind(workflow_id)
        .bind(repo_id)
        .execute(db)
        .await
        .map_err(AppError::Database)?;
    Ok(())
}

async fn delete_entity_by_repo(
    db: &sqlx::PgPool,
    table: &str,
    id: Uuid,
    repo_id: Uuid,
) -> Result<(), AppError> {
    let query_str = match table {
        "workflows" => "DELETE FROM workflows WHERE id = $1 AND repo_id = $2",
        "workflow_runs" => "DELETE FROM workflow_runs WHERE id = $1 AND repo_id = $2",
        "artifacts" => "DELETE FROM artifacts WHERE id = $1 AND repo_id = $2",
        _ => return Err(AppError::Internal(anyhow::anyhow!("Unsupported delete table"))),
    };

    sqlx::query(query_str)
        .bind(id)
        .bind(repo_id)
        .execute(db)
        .await
        .map_err(AppError::Database)?;

    Ok(())
}

async fn query_workflow_runs(
    db: &sqlx::PgPool,
    repo_id: Uuid,
    workflow_id: Option<Uuid>,
    limit: i64,
    offset: i64,
) -> Result<Vec<WorkflowRun>, AppError> {
    if let Some(wf_id) = workflow_id {
        sqlx::query_as(
            "SELECT * FROM workflow_runs
             WHERE workflow_id = $1 AND repo_id = $2
             ORDER BY created_at DESC
             LIMIT $3 OFFSET $4"
        )
            .bind(wf_id)
            .bind(repo_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(db)
            .await
            .map_err(AppError::Database)
    } else {
        sqlx::query_as(
            "SELECT * FROM workflow_runs
             WHERE repo_id = $1
             ORDER BY created_at DESC
             LIMIT $2 OFFSET $3"
        )
            .bind(repo_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(db)
            .await
            .map_err(AppError::Database)
    }
}

// Workflows

#[derive(Debug, Deserialize)]
pub struct CreateWorkflowInput {
    pub name:  String,
    pub path:  String,
    pub state: Option<String>,
}

pub async fn list_workflows(
    State(state):   State<AppState>,
    Path((owner, repo_name)): Path<(String, String)>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<impl IntoResponse, AppError> {
    let (_, repo) = fetch_repo(&state.db, &owner, &repo_name).await?;
    let (per_page, offset) = get_limit_and_offset(pagination.page, pagination.per_page, 30);

    let workflows: Vec<Workflow> = sqlx::query_as(
        "SELECT * FROM workflows WHERE repo_id = $1
         ORDER BY created_at DESC LIMIT $2 OFFSET $3"
    )
        .bind(repo.id)
        .bind(per_page)
        .bind(offset)
        .fetch_all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(json!({
        "workflows": workflows,
        "total":     workflows.len(),
    })))
}

pub async fn get_workflow(
    State(state):   State<AppState>,
    Path((owner, repo_name, workflow_id)): Path<(String, String, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let (_, repo) = fetch_repo(&state.db, &owner, &repo_name).await?;
    let workflow = fetch_workflow_by_repo(&state.db, workflow_id, repo.id).await?;

    Ok(Json(workflow))
}

pub async fn create_workflow(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name)): Path<(String, String)>,
    Json(input):    Json<CreateWorkflowInput>,
) -> Result<impl IntoResponse, AppError> {
    if input.name.trim().is_empty() {
        return Err(AppError::BadRequest("Workflow name is required".into()));
    }

    let repo = fetch_repo_with_auth(&state.db, &owner, &repo_name, auth_user.user_id).await?;

    let workflow: Workflow = sqlx::query_as(
        "INSERT INTO workflows (repo_id, name, path, state)
         VALUES ($1, $2, $3, $4)
         RETURNING *"
    )
        .bind(repo.id)
        .bind(&input.name)
        .bind(&input.path)
        .bind(input.state.as_deref().unwrap_or("active"))
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok((StatusCode::CREATED, Json(workflow)))
}

pub async fn enable_workflow(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name, workflow_id)): Path<(String, String, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let repo = fetch_repo_with_auth(&state.db, &owner, &repo_name, auth_user.user_id).await?;
    update_workflow_state(&state.db, workflow_id, repo.id, "active").await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn disable_workflow(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name, workflow_id)): Path<(String, String, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let repo = fetch_repo_with_auth(&state.db, &owner, &repo_name, auth_user.user_id).await?;
    update_workflow_state(&state.db, workflow_id, repo.id, "disabled_manually").await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_workflow(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name, workflow_id)): Path<(String, String, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let repo = fetch_repo_with_auth(&state.db, &owner, &repo_name, auth_user.user_id).await?;
    delete_entity_by_repo(&state.db, "workflows", workflow_id, repo.id).await?;

    Ok(StatusCode::NO_CONTENT)
}

// Workflow runs

#[derive(Debug, Deserialize)]
pub struct TriggerRunInput {
    pub ref_name: String,
    pub inputs:   Option<serde_json::Value>,
}

pub async fn trigger_run(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name, workflow_id)): Path<(String, String, Uuid)>,
    Json(input):    Json<TriggerRunInput>,
) -> Result<impl IntoResponse, AppError> {
    let (_, repo) = fetch_repo(&state.db, &owner, &repo_name).await?;

    let workflow = fetch_workflow_by_repo(&state.db, workflow_id, repo.id).await?;
    if workflow.state != "active" {
        return Err(AppError::BadRequest("Workflow is not active".into()));
    }

    // get next run number
    let row: (i64,) = sqlx::query_as(
        "SELECT COALESCE(MAX(run_number), 0) + 1 FROM workflow_runs WHERE workflow_id = $1"
    )
        .bind(workflow_id)
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    let run_number = row.0 as i32;

    // resolve ref to sha
    let repo_path = crate::git::repository::repo_path(
        &state.config.git_base_dir,
        &repo.git_path,
    );

    let sha = tokio::process::Command::new("git")
        .args(["rev-parse", &input.ref_name])
        .current_dir(&repo_path)
        .output()
        .await
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();

    let run: WorkflowRun = sqlx::query_as(
        "INSERT INTO workflow_runs
            (workflow_id, repo_id, actor_id, run_number, event,
             status, head_sha, head_branch)
         VALUES ($1, $2, $3, $4, 'workflow_dispatch',
                 'queued', $5, $6)
         RETURNING *"
    )
        .bind(workflow_id)
        .bind(repo.id)
        .bind(auth_user.user_id)
        .bind(run_number)
        .bind(&sha)
        .bind(&input.ref_name)
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok((StatusCode::CREATED, Json(run)))
}

pub async fn list_runs(
    State(state):   State<AppState>,
    Path((owner, repo_name)): Path<(String, String)>,
    Query(params):  Query<RunQuery>,
) -> Result<impl IntoResponse, AppError> {
    let (_, repo) = fetch_repo(&state.db, &owner, &repo_name).await?;
    let (per_page, offset) = get_limit_and_offset(params.page, params.per_page, 30);

    let runs = query_workflow_runs(&state.db, repo.id, None, per_page, offset).await?;

    Ok(Json(json!({
        "workflow_runs": runs,
        "page":          params.page.unwrap_or(1),
        "per_page":      per_page,
    })))
}

pub async fn list_workflow_runs(
    State(state):   State<AppState>,
    Path((owner, repo_name, workflow_id)): Path<(String, String, Uuid)>,
    Query(params):  Query<RunQuery>,
) -> Result<impl IntoResponse, AppError> {
    let (_, repo) = fetch_repo(&state.db, &owner, &repo_name).await?;
    let (per_page, offset) = get_limit_and_offset(params.page, params.per_page, 30);

    let runs = query_workflow_runs(&state.db, repo.id, Some(workflow_id), per_page, offset).await?;

    Ok(Json(json!({
        "workflow_runs": runs,
        "page":          params.page.unwrap_or(1),
        "per_page":      per_page,
    })))
}

pub async fn get_run(
    State(state):   State<AppState>,
    Path((owner, repo_name, run_id)): Path<(String, String, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let (_, repo) = fetch_repo(&state.db, &owner, &repo_name).await?;
    let run = fetch_run_by_repo(&state.db, run_id, repo.id).await?;

    Ok(Json(run))
}

pub async fn cancel_run(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name, run_id)): Path<(String, String, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let repo = fetch_repo_with_auth(&state.db, &owner, &repo_name, auth_user.user_id).await?;

    sqlx::query(
        "UPDATE workflow_runs
         SET status = 'completed', conclusion = 'cancelled',
             completed_at = now()
         WHERE id = $1 AND repo_id = $2
           AND status IN ('queued', 'in_progress', 'waiting')"
    )
        .bind(run_id)
        .bind(repo.id)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    // cancel all in-progress jobs
    sqlx::query(
        "UPDATE workflow_jobs
         SET status = 'completed', conclusion = 'cancelled',
             completed_at = now()
         WHERE run_id = $1
           AND status IN ('queued', 'in_progress')"
    )
        .bind(run_id)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn rerun_workflow(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name, run_id)): Path<(String, String, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let (_, repo) = fetch_repo(&state.db, &owner, &repo_name).await?;
    let existing = fetch_run_by_repo(&state.db, run_id, repo.id).await?;

    let row: (i64,) = sqlx::query_as(
        "SELECT COALESCE(MAX(run_number), 0) + 1 FROM workflow_runs WHERE workflow_id = $1"
    )
        .bind(existing.workflow_id)
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    let new_run: WorkflowRun = sqlx::query_as(
        "INSERT INTO workflow_runs
            (workflow_id, repo_id, actor_id, run_number, event,
             status, head_sha, head_branch, run_attempt)
         VALUES ($1, $2, $3, $4, $5, 'queued', $6, $7, $8)
         RETURNING *"
    )
        .bind(existing.workflow_id)
        .bind(repo.id)
        .bind(auth_user.user_id)
        .bind(row.0 as i32)
        .bind(&existing.event)
        .bind(&existing.head_sha)
        .bind(&existing.head_branch)
        .bind(existing.run_attempt + 1)
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok((StatusCode::CREATED, Json(new_run)))
}

pub async fn delete_run(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name, run_id)): Path<(String, String, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let repo = fetch_repo_with_auth(&state.db, &owner, &repo_name, auth_user.user_id).await?;
    delete_entity_by_repo(&state.db, "workflow_runs", run_id, repo.id).await?;

    Ok(StatusCode::NO_CONTENT)
}

// Jobs

pub async fn list_jobs(
    State(state):   State<AppState>,
    Path((owner, repo_name, run_id)): Path<(String, String, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let (_, repo) = fetch_repo(&state.db, &owner, &repo_name).await?;
    let run = fetch_run_by_repo(&state.db, run_id, repo.id).await?;

    let jobs: Vec<WorkflowJob> = sqlx::query_as(
        "SELECT * FROM workflow_jobs WHERE run_id = $1 ORDER BY created_at ASC"
    )
        .bind(run.id)
        .fetch_all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(json!({ "jobs": jobs, "total": jobs.len() })))
}

pub async fn get_job(
    State(state):   State<AppState>,
    Path((owner, repo_name, job_id)): Path<(String, String, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let (_, _) = fetch_repo(&state.db, &owner, &repo_name).await?;

    let job: WorkflowJob = sqlx::query_as(
        "SELECT * FROM workflow_jobs WHERE id = $1"
    )
        .bind(job_id)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound("Job".into()))?;

    let steps: Vec<WorkflowStep> = sqlx::query_as(
        "SELECT * FROM workflow_steps WHERE job_id = $1 ORDER BY number ASC"
    )
        .bind(job_id)
        .fetch_all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(json!({ "job": job, "steps": steps })))
}

// Runner reporting (called by runners)

#[derive(Debug, Deserialize)]
pub struct UpdateJobInput {
    pub status:       String,
    pub conclusion:   Option<String>,
    pub started_at:   Option<chrono::DateTime<chrono::Utc>>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub async fn update_job(
    State(state):   State<AppState>,
    Path(job_id):   Path<Uuid>,
    Json(input):    Json<UpdateJobInput>,
) -> Result<impl IntoResponse, AppError> {
    let job: WorkflowJob = sqlx::query_as(
        "UPDATE workflow_jobs SET
            status       = $1::workflow_run_status,
            conclusion   = $2::workflow_conclusion,
            started_at   = COALESCE($3, started_at),
            completed_at = $4,
            updated_at   = now()
         WHERE id = $5
         RETURNING *"
    )
        .bind(&input.status)
        .bind(&input.conclusion)
        .bind(input.started_at)
        .bind(input.completed_at)
        .bind(job_id)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound("Job".into()))?;

    // update run status based on jobs
    update_run_status(&state, job.run_id).await;

    Ok(Json(job))
}

#[derive(Debug, Deserialize)]
pub struct UpdateStepInput {
    pub name:         String,
    pub number:       i32,
    pub status:       String,
    pub conclusion:   Option<String>,
    pub started_at:   Option<chrono::DateTime<chrono::Utc>>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub async fn upsert_step(
    State(state):   State<AppState>,
    Path(job_id):   Path<Uuid>,
    Json(input):    Json<UpdateStepInput>,
) -> Result<impl IntoResponse, AppError> {
    let step: WorkflowStep = sqlx::query_as(
        "INSERT INTO workflow_steps (job_id, name, number, status, conclusion, started_at, completed_at)
         VALUES ($1, $2, $3, $4::workflow_run_status, $5::workflow_conclusion, $6, $7)
         ON CONFLICT (job_id, number) DO UPDATE SET
             status       = $4::workflow_run_status,
             conclusion   = $5::workflow_conclusion,
             started_at   = COALESCE($6, workflow_steps.started_at),
             completed_at = $7
         RETURNING *"
    )
        .bind(job_id)
        .bind(&input.name)
        .bind(input.number)
        .bind(&input.status)
        .bind(&input.conclusion)
        .bind(input.started_at)
        .bind(input.completed_at)
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(step))
}

async fn update_run_status(state: &AppState, run_id: Uuid) {
    let counts: Option<(i64, i64, i64)> = sqlx::query_as(
        "SELECT
            COUNT(*) FILTER (WHERE status::text = 'in_progress'),
            COUNT(*) FILTER (WHERE status::text = 'queued'),
            COUNT(*) FILTER (WHERE status::text = 'completed')
         FROM workflow_jobs WHERE run_id = $1"
    )
        .bind(run_id)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten();

    if let Some((in_progress, queued, completed)) = counts {
        let (new_status, conclusion) = if in_progress > 0 {
            ("in_progress", None::<&str>)
        } else if queued > 0 {
            ("queued", None)
        } else {
            // check if any failed
            let failed: (i64,) = sqlx::query_as(
                "SELECT COUNT(*) FROM workflow_jobs
                 WHERE run_id = $1 AND conclusion::text = 'failure'"
            )
                .bind(run_id)
                .fetch_one(&state.db)
                .await
                .unwrap_or((0,));

            if failed.0 > 0 {
                ("completed", Some("failure"))
            } else {
                ("completed", Some("success"))
            }
        };

        let _ = sqlx::query(
            "UPDATE workflow_runs SET
                status      = $1::workflow_run_status,
                conclusion  = $2::workflow_conclusion,
                started_at  = CASE WHEN $1 = 'in_progress' AND started_at IS NULL THEN now() ELSE started_at END,
                completed_at = CASE WHEN $1 = 'completed' THEN now() ELSE completed_at END
             WHERE id = $3"
        )
            .bind(new_status)
            .bind(conclusion)
            .bind(run_id)
            .execute(&state.db)
            .await;
    }
}

// Artifacts

const CI_BUCKET: &str = "opengit-ci";

pub async fn list_artifacts(
    State(state):   State<AppState>,
    Path((owner, repo_name, run_id)): Path<(String, String, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let (_, repo) = fetch_repo(&state.db, &owner, &repo_name).await?;

    let artifacts: Vec<Artifact> = sqlx::query_as(
        "SELECT * FROM artifacts WHERE run_id = $1 AND repo_id = $2
         ORDER BY created_at DESC"
    )
        .bind(run_id)
        .bind(repo.id)
        .fetch_all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(json!({ "artifacts": artifacts, "total": artifacts.len() })))
}

pub async fn upload_artifact(
    State(state):   State<AppState>,
    Path((owner, repo_name, run_id)): Path<(String, String, Uuid)>,
    mut multipart:  Multipart,
) -> Result<impl IntoResponse, AppError> {
    let (_, repo) = fetch_repo(&state.db, &owner, &repo_name).await?;

    let field = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("multipart error: {}", e)))?
        .ok_or(AppError::BadRequest("No file in request".into()))?;

    let name  = field.file_name().unwrap_or("artifact").to_string();
    let data: Bytes = field.bytes().await
        .map_err(|e| AppError::BadRequest(format!("read error: {}", e)))?;

    let size_bytes   = data.len() as i64;
    let storage_key  = format!("artifacts/{}/{}/{}", repo.id, run_id, name);

    let s3 = get_s3_client(&state);

    let _ = s3.create_bucket().bucket(CI_BUCKET).send().await;

    s3.put_object()
        .bucket(CI_BUCKET)
        .key(&storage_key)
        .body(data.into())
        .send()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Upload failed: {}", e)))?;

    let artifact: Artifact = sqlx::query_as(
        "INSERT INTO artifacts (run_id, repo_id, name, size_bytes, storage_key,
                                expires_at)
         VALUES ($1, $2, $3, $4, $5, now() + interval '90 days')
         RETURNING *"
    )
        .bind(run_id)
        .bind(repo.id)
        .bind(&name)
        .bind(size_bytes)
        .bind(&storage_key)
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok((StatusCode::CREATED, Json(artifact)))
}

pub async fn download_artifact(
    State(state):   State<AppState>,
    Path((owner, repo_name, artifact_id)): Path<(String, String, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let (_, repo) = fetch_repo(&state.db, &owner, &repo_name).await?;
    let artifact = fetch_artifact_by_repo(&state.db, artifact_id, repo.id).await?;

    let s3 = get_s3_client(&state);

    let presigned = s3
        .get_object()
        .bucket(CI_BUCKET)
        .key(&artifact.storage_key)
        .presigned(
            aws_sdk_s3::presigning::PresigningConfig::expires_in(
                std::time::Duration::from_secs(3600)
            )
                .map_err(|e| AppError::Internal(anyhow::anyhow!("{}", e)))?,
        )
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Presign failed: {}", e)))?;

    Ok((
        StatusCode::FOUND,
        [(axum::http::header::LOCATION, presigned.uri().to_string())],
    ))
}

pub async fn delete_artifact(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name, artifact_id)): Path<(String, String, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let repo = fetch_repo_with_auth(&state.db, &owner, &repo_name, auth_user.user_id).await?;
    delete_entity_by_repo(&state.db, "artifacts", artifact_id, repo.id).await?;

    Ok(StatusCode::NO_CONTENT)
}

// Runners

#[derive(Debug, Deserialize)]
pub struct RegisterRunnerInput {
    pub name:         String,
    pub os:           Option<String>,
    pub architecture: Option<String>,
    pub labels:       Option<Vec<String>>,
    pub group_id:     Option<Uuid>,
}

pub async fn register_runner(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Json(input):    Json<RegisterRunnerInput>,
) -> Result<impl IntoResponse, AppError> {
    let token     = format!("{}", Uuid::new_v4().simple());
    let token_hash = blake3::hash(token.as_bytes()).to_hex().to_string();

    let runner: Runner = sqlx::query_as(
        "INSERT INTO runners
            (group_id, name, os, architecture, labels, token_hash)
         VALUES ($1, $2, $3, $4, $5, $6)
         RETURNING *"
    )
        .bind(input.group_id)
        .bind(&input.name)
        .bind(&input.os)
        .bind(&input.architecture)
        .bind(input.labels.as_deref().unwrap_or(&[]))
        .bind(&token_hash)
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok((StatusCode::CREATED, Json(json!({
        "runner": runner,
        "token":  token,
    }))))
}

pub async fn list_runners(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Query(pagination): Query<PaginationQuery>,
) -> Result<impl IntoResponse, AppError> {
    let (per_page, offset) = get_limit_and_offset(pagination.page, pagination.per_page, 30);

    let runners: Vec<Runner> = sqlx::query_as(
        "SELECT * FROM runners ORDER BY created_at DESC LIMIT $1 OFFSET $2"
    )
        .bind(per_page)
        .bind(offset)
        .fetch_all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(json!({ "runners": runners })))
}

pub async fn get_runner(
    State(state): State<AppState>,
    Path(runner_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let runner: Runner = sqlx::query_as("SELECT * FROM runners WHERE id = $1")
        .bind(runner_id)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound("Runner".into()))?;

    Ok(Json(runner))
}

pub async fn delete_runner(
    State(state):    State<AppState>,
    auth_user:       AuthUser,
    Path(runner_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    sqlx::query("DELETE FROM runners WHERE id = $1")
        .bind(runner_id)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// Runner heartbeat (called by runner agent)

#[derive(Debug, Deserialize)]
pub struct HeartbeatInput {
    pub token:  String,
    pub status: Option<String>,
}

pub async fn runner_heartbeat(
    State(state): State<AppState>,
    Json(input):  Json<HeartbeatInput>,
) -> Result<impl IntoResponse, AppError> {
    let token_hash = blake3::hash(input.token.as_bytes()).to_hex().to_string();
    let status     = input.status.as_deref().unwrap_or("online");

    let runner: Runner = sqlx::query_as(
        "UPDATE runners SET
            status       = $1::runner_status,
            last_seen_at = now()
         WHERE token_hash = $2
         RETURNING *"
    )
        .bind(status)
        .bind(&token_hash)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::Unauthorized)?;

    // check for queued job that matches this runner's labels
    let queued_job: Option<WorkflowJob> = sqlx::query_as(
        "SELECT * FROM workflow_jobs
         WHERE status = 'queued'::workflow_run_status
           AND (labels = '{}' OR labels && $1)
         ORDER BY created_at ASC
         LIMIT 1
         FOR UPDATE SKIP LOCKED"
    )
        .bind(&runner.labels)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?;

    if let Some(job) = queued_job {
        // assign to this runner
        sqlx::query(
            "UPDATE workflow_jobs SET runner_id = $1, status = 'in_progress',
             started_at = now() WHERE id = $2"
        )
            .bind(runner.id)
            .bind(job.id)
            .execute(&state.db)
            .await
            .map_err(AppError::Database)?;

        return Ok(Json(json!({
            "status":     "job_assigned",
            "runner_id":  runner.id,
            "job_id":     job.id,
            "run_id":     job.run_id,
        })));
    }

    Ok(Json(json!({
        "status":    "idle",
        "runner_id": runner.id,
    })))
}

// Runner groups

#[derive(Debug, Deserialize)]
pub struct CreateRunnerGroupInput {
    pub name:       String,
    pub visibility: Option<String>,
    pub org_id:     Option<Uuid>,
    pub repo_id:    Option<Uuid>,
}

pub async fn list_runner_groups(
    State(state): State<AppState>,
    auth_user:    AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let groups: Vec<RunnerGroup> = sqlx::query_as(
        "SELECT * FROM runner_groups ORDER BY created_at DESC"
    )
        .fetch_all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(json!({ "runner_groups": groups })))
}

pub async fn create_runner_group(
    State(state): State<AppState>,
    auth_user:    AuthUser,
    Json(input):  Json<CreateRunnerGroupInput>,
) -> Result<impl IntoResponse, AppError> {
    let group: RunnerGroup = sqlx::query_as(
        "INSERT INTO runner_groups (org_id, repo_id, name, visibility)
         VALUES ($1, $2, $3, $4)
         RETURNING *"
    )
        .bind(input.org_id)
        .bind(input.repo_id)
        .bind(&input.name)
        .bind(input.visibility.as_deref().unwrap_or("all"))
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok((StatusCode::CREATED, Json(group)))
}

pub async fn delete_runner_group(
    State(state):  State<AppState>,
    auth_user:     AuthUser,
    Path(group_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    sqlx::query("DELETE FROM runner_groups WHERE id = $1")
        .bind(group_id)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}