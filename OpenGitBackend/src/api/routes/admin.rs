use crate::{
    api::middleware::auth::AuthUser,
    error::AppError,
    models::{
        admin::{AbuseReport, AuditLog, BannedUser, SiteSetting},
        enums::UserRole,
        user::User,
    },
    state::AppState,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub page:     Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct AuditQuery {
    pub action:  Option<String>,
    pub actor:   Option<String>,
    pub page:    Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct UserSearchQuery {
    pub q:        Option<String>,
    pub role:     Option<String>,
    pub active:   Option<bool>,
    pub page:     Option<i64>,
    pub per_page: Option<i64>,
}

// Private deduplication helper functions

async fn get_user_role(state: &AppState, user_id: Uuid) -> Result<String, AppError> {
    let row: (String,) = sqlx::query_as(
        "SELECT role::text FROM users WHERE id = $1"
    )
        .bind(user_id)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::Unauthorized)?;

    Ok(row.0)
}

fn get_limit_and_offset(page: Option<i64>, per_page: Option<i64>, default_per_page: i64) -> (i64, i64) {
    let limit = per_page.unwrap_or(default_per_page).min(100);
    let offset = (page.unwrap_or(1) - 1) * limit;
    (limit, offset)
}

async fn fetch_user_by_username(db: &sqlx::PgPool, username: &str) -> Result<User, AppError> {
    sqlx::query_as("SELECT * FROM users WHERE username = $1")
        .bind(username)
        .fetch_optional(db)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound("User".into()))
}

async fn update_user_status(db: &sqlx::PgPool, username: &str, is_active: bool) -> Result<User, AppError> {
    sqlx::query_as(
        "UPDATE users SET is_active = $1, updated_at = now()
         WHERE username = $2 RETURNING *"
    )
        .bind(is_active)
        .bind(username)
        .fetch_optional(db)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound("User".into()))
}

// Admin guard

async fn require_admin(state: &AppState, user_id: Uuid) -> Result<(), AppError> {
    let role = get_user_role(state, user_id).await?;
    if role != "admin" && role != "superadmin" {
        return Err(AppError::Forbidden);
    }
    Ok(())
}

async fn require_superadmin(state: &AppState, user_id: Uuid) -> Result<(), AppError> {
    let role = get_user_role(state, user_id).await?;
    if role != "superadmin" {
        return Err(AppError::Forbidden);
    }
    Ok(())
}

// Site settings

pub async fn list_settings(
    State(state): State<AppState>,
    auth_user:    AuthUser,
) -> Result<impl IntoResponse, AppError> {
    require_admin(&state, auth_user.user_id).await?;

    let settings: Vec<SiteSetting> = sqlx::query_as(
        "SELECT * FROM site_settings ORDER BY key ASC"
    )
        .fetch_all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(settings))
}

pub async fn get_setting(
    State(state): State<AppState>,
    Path(key):    Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let setting: SiteSetting = sqlx::query_as(
        "SELECT * FROM site_settings WHERE key = $1"
    )
        .bind(&key)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound("Setting".into()))?;

    Ok(Json(setting))
}

#[derive(Debug, Deserialize)]
pub struct UpdateSettingInput {
    pub value: Value,
}

pub async fn update_setting(
    State(state): State<AppState>,
    auth_user:    AuthUser,
    Path(key):    Path<String>,
    Json(input):  Json<UpdateSettingInput>,
) -> Result<impl IntoResponse, AppError> {
    require_admin(&state, auth_user.user_id).await?;

    let setting: SiteSetting = sqlx::query_as(
        "INSERT INTO site_settings (key, value, updated_by_id)
         VALUES ($1, $2, $3)
         ON CONFLICT (key) DO UPDATE
         SET value = $2, updated_by_id = $3, updated_at = now()
         RETURNING *"
    )
        .bind(&key)
        .bind(&input.value)
        .bind(auth_user.user_id)
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(setting))
}

pub async fn delete_setting(
    State(state): State<AppState>,
    auth_user:    AuthUser,
    Path(key):    Path<String>,
) -> Result<impl IntoResponse, AppError> {
    require_superadmin(&state, auth_user.user_id).await?;

    sqlx::query("DELETE FROM site_settings WHERE key = $1")
        .bind(&key)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// Audit log

pub async fn list_audit_log(
    State(state):  State<AppState>,
    auth_user:     AuthUser,
    Query(params): Query<AuditQuery>,
) -> Result<impl IntoResponse, AppError> {
    require_admin(&state, auth_user.user_id).await?;

    let (per_page, offset) = get_limit_and_offset(params.page, params.per_page, 50);

    let logs: Vec<AuditLog> = if let Some(ref action) = params.action {
        sqlx::query_as(
            "SELECT * FROM audit_log
             WHERE action::text ILIKE $1
             ORDER BY created_at DESC
             LIMIT $2 OFFSET $3"
        )
            .bind(format!("%{}%", action))
            .bind(per_page)
            .bind(offset)
            .fetch_all(&state.db)
            .await
            .map_err(AppError::Database)?
    } else {
        sqlx::query_as(
            "SELECT * FROM audit_log
             ORDER BY created_at DESC
             LIMIT $1 OFFSET $2"
        )
            .bind(per_page)
            .bind(offset)
            .fetch_all(&state.db)
            .await
            .map_err(AppError::Database)?
    };

    let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM audit_log")
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(json!({
        "logs":     logs,
        "total":    total.0,
        "page":     params.page.unwrap_or(1),
        "per_page": per_page,
    })))
}

pub async fn get_audit_entry(
    State(state): State<AppState>,
    auth_user:    AuthUser,
    Path(log_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    require_admin(&state, auth_user.user_id).await?;

    let log: AuditLog = sqlx::query_as(
        "SELECT * FROM audit_log WHERE id = $1"
    )
        .bind(log_id)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound("Audit log entry".into()))?;

    Ok(Json(log))
}

// User management

pub async fn list_users(
    State(state):  State<AppState>,
    auth_user:     AuthUser,
    Query(params): Query<UserSearchQuery>,
) -> Result<impl IntoResponse, AppError> {
    require_admin(&state, auth_user.user_id).await?;

    let (per_page, offset) = get_limit_and_offset(params.page, params.per_page, 30);

    let users: Vec<User> = if let Some(ref q) = params.q {
        let pattern = format!("%{}%", q.to_lowercase());
        sqlx::query_as(
            "SELECT * FROM users
             WHERE LOWER(username) LIKE $1
                OR LOWER(display_name) LIKE $1
             ORDER BY created_at DESC
             LIMIT $2 OFFSET $3"
        )
            .bind(&pattern)
            .bind(per_page)
            .bind(offset)
            .fetch_all(&state.db)
            .await
            .map_err(AppError::Database)?
    } else {
        sqlx::query_as(
            "SELECT * FROM users
             ORDER BY created_at DESC
             LIMIT $1 OFFSET $2"
        )
            .bind(per_page)
            .bind(offset)
            .fetch_all(&state.db)
            .await
            .map_err(AppError::Database)?
    };

    let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(json!({
        "users":    users,
        "total":    total.0,
        "page":     params.page.unwrap_or(1),
        "per_page": per_page,
    })))
}

pub async fn get_user_admin(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path(username): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    require_admin(&state, auth_user.user_id).await?;

    let user = fetch_user_by_username(&state.db, &username).await?;

    let repo_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM repositories WHERE owner_id = $1"
    )
        .bind(user.id)
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    let email: Option<(String,)> = sqlx::query_as(
        "SELECT email FROM user_emails WHERE user_id = $1 AND is_primary = true"
    )
        .bind(user.id)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(json!({
        "user":        user,
        "repo_count":  repo_count.0,
        "email":       email.map(|(e,)| e),
    })))
}

#[derive(Debug, Deserialize)]
pub struct AdminUpdateUserInput {
    pub role:       Option<String>,
    pub is_active:  Option<bool>,
    pub is_verified: Option<bool>,
}

pub async fn update_user_admin(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path(username): Path<String>,
    Json(input):    Json<AdminUpdateUserInput>,
) -> Result<impl IntoResponse, AppError> {
    require_admin(&state, auth_user.user_id).await?;

    // only superadmin can promote to admin/superadmin
    if let Some(ref role) = input.role {
        if role == "admin" || role == "superadmin" {
            require_superadmin(&state, auth_user.user_id).await?;
        }
    }

    let user: User = sqlx::query_as(
        "UPDATE users SET
            role        = COALESCE($1::user_role, role),
            is_active   = COALESCE($2, is_active),
            is_verified = COALESCE($3, is_verified),
            updated_at  = now()
         WHERE username = $4
         RETURNING *"
    )
        .bind(input.role.as_deref())
        .bind(input.is_active)
        .bind(input.is_verified)
        .bind(&username)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound("User".into()))?;

    // write audit log
    write_audit_log(
        &state,
        auth_user.user_id,
        "user_promoted",
        "user",
        Some(user.id),
        json!({ "username": username }),
    ).await;

    Ok(Json(user))
}

pub async fn suspend_user(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path(username): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    require_admin(&state, auth_user.user_id).await?;

    let user = update_user_status(&state.db, &username, false).await?;

    write_audit_log(
        &state,
        auth_user.user_id,
        "user_banned",
        "user",
        Some(user.id),
        json!({ "username": username, "action": "suspended" }),
    ).await;

    Ok(Json(json!({ "message": "User suspended", "user": user })))
}

pub async fn unsuspend_user(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path(username): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    require_admin(&state, auth_user.user_id).await?;

    let user = update_user_status(&state.db, &username, true).await?;

    Ok(Json(json!({ "message": "User unsuspended", "user": user })))
}

pub async fn delete_user_admin(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path(username): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    require_superadmin(&state, auth_user.user_id).await?;

    let user = fetch_user_by_username(&state.db, &username).await?;

    // cannot delete yourself
    if user.id == auth_user.user_id {
        return Err(AppError::BadRequest("Cannot delete your own account".into()));
    }

    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user.id)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    write_audit_log(
        &state,
        auth_user.user_id,
        "user_deleted",
        "user",
        Some(user.id),
        json!({ "username": username }),
    ).await;

    Ok(StatusCode::NO_CONTENT)
}

// Bans

#[derive(Debug, Deserialize)]
pub struct BanInput {
    pub user_id:    Option<Uuid>,
    pub email:      Option<String>,
    pub ip_address: Option<String>,
    pub reason:     Option<String>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub async fn list_bans(
    State(state):  State<AppState>,
    auth_user:     AuthUser,
    Query(pagination): Query<PaginationQuery>,
) -> Result<impl IntoResponse, AppError> {
    require_admin(&state, auth_user.user_id).await?;

    let (per_page, offset) = get_limit_and_offset(pagination.page, pagination.per_page, 30);

    let bans: Vec<BannedUser> = sqlx::query_as(
        "SELECT * FROM banned_users
         ORDER BY created_at DESC
         LIMIT $1 OFFSET $2"
    )
        .bind(per_page)
        .bind(offset)
        .fetch_all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(json!({ "bans": bans })))
}

pub async fn create_ban(
    State(state): State<AppState>,
    auth_user:    AuthUser,
    Json(input):  Json<BanInput>,
) -> Result<impl IntoResponse, AppError> {
    require_admin(&state, auth_user.user_id).await?;

    if input.user_id.is_none() && input.email.is_none() && input.ip_address.is_none() {
        return Err(AppError::BadRequest(
            "At least one of user_id, email, or ip_address is required".into()
        ));
    }

    let ban: BannedUser = sqlx::query_as(
        "INSERT INTO banned_users
            (user_id, email, ip_address, reason, banned_by_id, expires_at)
         VALUES ($1, $2, $3, $4, $5, $6)
         RETURNING *"
    )
        .bind(input.user_id)
        .bind(&input.email)
        .bind(&input.ip_address)
        .bind(&input.reason)
        .bind(auth_user.user_id)
        .bind(input.expires_at)
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    // deactivate user if ban includes user_id
    if let Some(uid) = input.user_id {
        sqlx::query("UPDATE users SET is_active = false WHERE id = $1")
            .bind(uid)
            .execute(&state.db)
            .await
            .map_err(AppError::Database)?;
    }

    write_audit_log(
        &state,
        auth_user.user_id,
        "user_banned",
        "banned_user",
        Some(ban.id),
        json!({ "user_id": input.user_id, "email": input.email }),
    ).await;

    Ok((StatusCode::CREATED, Json(ban)))
}

pub async fn delete_ban(
    State(state): State<AppState>,
    auth_user:    AuthUser,
    Path(ban_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    require_admin(&state, auth_user.user_id).await?;

    let ban: BannedUser = sqlx::query_as(
        "SELECT * FROM banned_users WHERE id = $1"
    )
        .bind(ban_id)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound("Ban".into()))?;

    sqlx::query("DELETE FROM banned_users WHERE id = $1")
        .bind(ban_id)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    // reactivate user if applicable
    if let Some(uid) = ban.user_id {
        sqlx::query("UPDATE users SET is_active = true WHERE id = $1")
            .bind(uid)
            .execute(&state.db)
            .await
            .map_err(AppError::Database)?;
    }

    Ok(StatusCode::NO_CONTENT)
}

// Abuse reports

pub async fn list_abuse_reports(
    State(state):  State<AppState>,
    auth_user:     AuthUser,
    Query(pagination): Query<PaginationQuery>,
) -> Result<impl IntoResponse, AppError> {
    require_admin(&state, auth_user.user_id).await?;

    let (per_page, offset) = get_limit_and_offset(pagination.page, pagination.per_page, 30);

    let reports: Vec<AbuseReport> = sqlx::query_as(
        "SELECT * FROM abuse_reports
         WHERE resolved = false
         ORDER BY created_at DESC
         LIMIT $1 OFFSET $2"
    )
        .bind(per_page)
        .bind(offset)
        .fetch_all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(json!({ "reports": reports })))
}

pub async fn get_abuse_report(
    State(state):  State<AppState>,
    auth_user:     AuthUser,
    Path(report_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    require_admin(&state, auth_user.user_id).await?;

    let report: AbuseReport = sqlx::query_as(
        "SELECT * FROM abuse_reports WHERE id = $1"
    )
        .bind(report_id)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound("Report".into()))?;

    Ok(Json(report))
}

pub async fn create_abuse_report(
    State(state): State<AppState>,
    auth_user:    AuthUser,
    Json(input):  Json<CreateAbuseReportInput>,
) -> Result<impl IntoResponse, AppError> {
    let report: AbuseReport = sqlx::query_as(
        "INSERT INTO abuse_reports
            (reporter_id, target_type, target_id, reason, description)
         VALUES ($1, $2, $3, $4, $5)
         RETURNING *"
    )
        .bind(auth_user.user_id)
        .bind(&input.target_type)
        .bind(input.target_id)
        .bind(&input.reason)
        .bind(&input.description)
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok((StatusCode::CREATED, Json(report)))
}

#[derive(Debug, Deserialize)]
pub struct CreateAbuseReportInput {
    pub target_type:  String,
    pub target_id:    Uuid,
    pub reason:       String,
    pub description:  Option<String>,
}

pub async fn resolve_abuse_report(
    State(state):    State<AppState>,
    auth_user:       AuthUser,
    Path(report_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    require_admin(&state, auth_user.user_id).await?;

    let report: AbuseReport = sqlx::query_as(
        "UPDATE abuse_reports
         SET resolved = true, resolved_by_id = $1
         WHERE id = $2
         RETURNING *"
    )
        .bind(auth_user.user_id)
        .bind(report_id)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound("Report".into()))?;

    Ok(Json(report))
}

// Instance stats

pub async fn instance_stats(
    State(state): State<AppState>,
    auth_user:    AuthUser,
) -> Result<impl IntoResponse, AppError> {
    require_admin(&state, auth_user.user_id).await?;

    let users: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM users WHERE is_active = true"
    )
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    let repos: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM repositories")
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    let issues: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM issues")
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    let prs: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM pull_requests")
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    let orgs: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM organizations")
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    let ci_runs: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM workflow_runs")
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    let new_users_today: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM users WHERE created_at > now() - interval '24 hours'"
    )
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    let open_issues: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM issues WHERE state = 'open'::issue_state"
    )
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    let open_prs: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM pull_requests WHERE state = 'open'::pr_state"
    )
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    let active_runners: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM runners WHERE status IN ('online', 'busy')"
    )
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    let pending_reports: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM abuse_reports WHERE resolved = false"
    )
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(json!({
        "users": {
            "total":     users.0,
            "new_today": new_users_today.0,
        },
        "repositories":    repos.0,
        "organizations":   orgs.0,
        "issues": {
            "total": issues.0,
            "open":  open_issues.0,
        },
        "pull_requests": {
            "total": prs.0,
            "open":  open_prs.0,
        },
        "ci": {
            "total_runs":      ci_runs.0,
            "active_runners":  active_runners.0,
        },
        "pending_abuse_reports": pending_reports.0,
    })))
}

// Repo management

pub async fn list_repos_admin(
    State(state):  State<AppState>,
    auth_user:     AuthUser,
    Query(params): Query<PaginationQuery>,
) -> Result<impl IntoResponse, AppError> {
    require_admin(&state, auth_user.user_id).await?;

    let (per_page, offset) = get_limit_and_offset(params.page, params.per_page, 30);

    let repos: Vec<crate::models::repo::Repository> = sqlx::query_as(
        "SELECT * FROM repositories
         ORDER BY created_at DESC
         LIMIT $1 OFFSET $2"
    )
        .bind(per_page)
        .bind(offset)
        .fetch_all(&state.db)
        .await
        .map_err(AppError::Database)?;

    let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM repositories")
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(json!({
        "repositories": repos,
        "total":        total.0,
    })))
}

pub async fn delete_repo_admin(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((owner, repo_name)): Path<(String, String)>,
) -> Result<impl IntoResponse, AppError> {
    require_admin(&state, auth_user.user_id).await?;

    let owner = crate::db::queries::users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    let repo = crate::db::queries::repos::find_by_owner_and_name(
        &state.db, owner.id, &repo_name
    )
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    sqlx::query("DELETE FROM repositories WHERE id = $1")
        .bind(repo.id)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    write_audit_log(
        &state,
        auth_user.user_id,
        "repo_deleted",
        "repository",
        Some(repo.id),
        json!({ "owner": owner.username, "repo": repo_name }),
    ).await;

    Ok(StatusCode::NO_CONTENT)
}

// Audit log helper

pub async fn write_audit_log(
    state:       &AppState,
    actor_id:    Uuid,
    action:      &str,
    target_type: &str,
    target_id:   Option<Uuid>,
    metadata:    Value,
) {
    let _ = sqlx::query(
        "INSERT INTO audit_log (actor_id, action, target_type, target_id, metadata)
         VALUES ($1, $2::audit_action, $3, $4, $5)"
    )
        .bind(actor_id)
        .bind(action)
        .bind(target_type)
        .bind(target_id)
        .bind(metadata)
        .execute(&state.db)
        .await;
}