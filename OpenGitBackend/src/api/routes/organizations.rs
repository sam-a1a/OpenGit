use crate::{
    api::middleware::auth::AuthUser,
    db::queries::users,
    error::AppError,
    models::organization::{OrgInvitation, OrgMember, OrgTeam, Organization, TeamMember},
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
pub struct PaginationQuery {
    pub page:     Option<i64>,
    pub per_page: Option<i64>,
}

// Create org

#[derive(Debug, Deserialize)]
pub struct CreateOrgInput {
    pub name:        String,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub visibility:  Option<String>,
    pub email:       Option<String>,
    pub website:     Option<String>,
    pub location:    Option<String>,
}

pub async fn create_org(
    State(state): State<AppState>,
    auth_user:    AuthUser,
    Json(input):  Json<CreateOrgInput>,
) -> Result<impl IntoResponse, AppError> {
    if input.name.trim().is_empty() {
        return Err(AppError::BadRequest("Organization name is required".into()));
    }
    if !input.name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return Err(AppError::BadRequest(
            "Name can only contain letters, numbers, hyphens and underscores".into()
        ));
    }

    // check name not taken by user or org
    let exists: (bool,) = sqlx::query_as(
        "SELECT EXISTS(SELECT 1 FROM organizations WHERE name = $1)
         OR EXISTS(SELECT 1 FROM users WHERE username = $1)"
    )
        .bind(&input.name)
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    if exists.0 {
        return Err(AppError::Conflict("Organization name".into()));
    }

    let visibility = input.visibility.as_deref().unwrap_or("public");

    let org: Organization = sqlx::query_as(
        "INSERT INTO organizations
            (name, display_name, description, visibility, email, website, location)
         VALUES ($1, $2, $3, $4::org_visibility, $5, $6, $7)
         RETURNING *"
    )
        .bind(&input.name)
        .bind(&input.display_name)
        .bind(&input.description)
        .bind(visibility)
        .bind(&input.email)
        .bind(&input.website)
        .bind(&input.location)
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    // add creator as owner
    sqlx::query(
        "INSERT INTO org_members (org_id, user_id, role)
         VALUES ($1, $2, 'owner'::org_member_role)"
    )
        .bind(org.id)
        .bind(auth_user.user_id)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok((StatusCode::CREATED, Json(org)))
}

// Get org

pub async fn get_org(
    State(state):   State<AppState>,
    Path(org_name): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let org = find_org_by_name(&state, &org_name).await?;
    Ok(Json(org))
}

// Update org

#[derive(Debug, Deserialize)]
pub struct UpdateOrgInput {
    pub display_name: Option<String>,
    pub description:  Option<String>,
    pub email:        Option<String>,
    pub website:      Option<String>,
    pub location:     Option<String>,
    pub twitter_username: Option<String>,
    pub visibility:   Option<String>,
}

pub async fn update_org(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path(org_name): Path<String>,
    Json(input):    Json<UpdateOrgInput>,
) -> Result<impl IntoResponse, AppError> {
    let org = find_org_by_name(&state, &org_name).await?;
    require_org_owner(&state, org.id, auth_user.user_id).await?;

    let updated: Organization = sqlx::query_as(
        "UPDATE organizations SET
            display_name     = COALESCE($1, display_name),
            description      = COALESCE($2, description),
            email            = COALESCE($3, email),
            website          = COALESCE($4, website),
            location         = COALESCE($5, location),
            twitter_username = COALESCE($6, twitter_username),
            updated_at       = now()
         WHERE id = $7
         RETURNING *"
    )
        .bind(&input.display_name)
        .bind(&input.description)
        .bind(&input.email)
        .bind(&input.website)
        .bind(&input.location)
        .bind(&input.twitter_username)
        .bind(org.id)
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(updated))
}

// Delete org

pub async fn delete_org(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path(org_name): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let org = find_org_by_name(&state, &org_name).await?;
    require_org_owner(&state, org.id, auth_user.user_id).await?;

    sqlx::query("DELETE FROM organizations WHERE id = $1")
        .bind(org.id)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// List orgs for authenticated user

pub async fn list_my_orgs(
    State(state): State<AppState>,
    auth_user:    AuthUser,
    Query(pagination): Query<PaginationQuery>,
) -> Result<impl IntoResponse, AppError> {
    let per_page = pagination.per_page.unwrap_or(30).min(100);
    let offset   = (pagination.page.unwrap_or(1) - 1) * per_page;

    let orgs: Vec<Organization> = sqlx::query_as(
        "SELECT o.* FROM organizations o
         JOIN org_members m ON m.org_id = o.id
         WHERE m.user_id = $1
         ORDER BY o.name
         LIMIT $2 OFFSET $3"
    )
        .bind(auth_user.user_id)
        .bind(per_page)
        .bind(offset)
        .fetch_all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(orgs))
}

// List orgs for a user

pub async fn list_user_orgs(
    State(state):   State<AppState>,
    Path(username): Path<String>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<impl IntoResponse, AppError> {
    let user = users::find_by_username(&state.db, &username)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    let per_page = pagination.per_page.unwrap_or(30).min(100);
    let offset   = (pagination.page.unwrap_or(1) - 1) * per_page;

    let orgs: Vec<Organization> = sqlx::query_as(
        "SELECT o.* FROM organizations o
         JOIN org_members m ON m.org_id = o.id
         WHERE m.user_id = $1 AND o.visibility = 'public'::org_visibility
         ORDER BY o.name
         LIMIT $2 OFFSET $3"
    )
        .bind(user.id)
        .bind(per_page)
        .bind(offset)
        .fetch_all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(orgs))
}

// Members

pub async fn list_members(
    State(state):   State<AppState>,
    Path(org_name): Path<String>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<impl IntoResponse, AppError> {
    let org      = find_org_by_name(&state, &org_name).await?;
    let per_page = pagination.per_page.unwrap_or(30).min(100);
    let offset   = (pagination.page.unwrap_or(1) - 1) * per_page;

    let members: Vec<crate::models::user::User> = sqlx::query_as(
        "SELECT u.* FROM users u
         JOIN org_members m ON m.user_id = u.id
         WHERE m.org_id = $1
         ORDER BY m.joined_at ASC
         LIMIT $2 OFFSET $3"
    )
        .bind(org.id)
        .bind(per_page)
        .bind(offset)
        .fetch_all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(members))
}

pub async fn get_member(
    State(state):   State<AppState>,
    Path((org_name, username)): Path<(String, String)>,
) -> Result<impl IntoResponse, AppError> {
    let org  = find_org_by_name(&state, &org_name).await?;
    let user = users::find_by_username(&state.db, &username)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    let member: OrgMember = sqlx::query_as(
        "SELECT * FROM org_members WHERE org_id = $1 AND user_id = $2"
    )
        .bind(org.id)
        .bind(user.id)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound("Member".into()))?;

    Ok(Json(json!({
        "user":   user,
        "role":   member.role,
        "joined": member.joined_at,
    })))
}

pub async fn remove_member(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((org_name, username)): Path<(String, String)>,
) -> Result<impl IntoResponse, AppError> {
    let org  = find_org_by_name(&state, &org_name).await?;
    require_org_owner(&state, org.id, auth_user.user_id).await?;

    let user = users::find_by_username(&state.db, &username)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    // cannot remove last owner
    let owner_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM org_members WHERE org_id = $1 AND role = 'owner'::org_member_role"
    )
        .bind(org.id)
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    let is_owner: (bool,) = sqlx::query_as(
        "SELECT EXISTS(SELECT 1 FROM org_members
         WHERE org_id = $1 AND user_id = $2 AND role = 'owner'::org_member_role)"
    )
        .bind(org.id)
        .bind(user.id)
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    if is_owner.0 && owner_count.0 <= 1 {
        return Err(AppError::BadRequest("Cannot remove the last owner of an organization".into()));
    }

    sqlx::query("DELETE FROM org_members WHERE org_id = $1 AND user_id = $2")
        .bind(org.id)
        .bind(user.id)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize)]
pub struct UpdateMemberRoleInput {
    pub role: String,
}

pub async fn update_member_role(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((org_name, username)): Path<(String, String)>,
    Json(input):    Json<UpdateMemberRoleInput>,
) -> Result<impl IntoResponse, AppError> {
    let org  = find_org_by_name(&state, &org_name).await?;
    require_org_owner(&state, org.id, auth_user.user_id).await?;

    let user = users::find_by_username(&state.db, &username)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    if !["owner", "member", "billing_manager"].contains(&input.role.as_str()) {
        return Err(AppError::BadRequest("role must be owner, member, or billing_manager".into()));
    }

    let member: OrgMember = sqlx::query_as(
        "UPDATE org_members SET role = $1::org_member_role
         WHERE org_id = $2 AND user_id = $3
         RETURNING *"
    )
        .bind(&input.role)
        .bind(org.id)
        .bind(user.id)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound("Member".into()))?;

    Ok(Json(member))
}

// Invitations

#[derive(Debug, Deserialize)]
pub struct InviteInput {
    pub email: String,
    pub role:  Option<String>,
}

pub async fn list_invitations(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path(org_name): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let org = find_org_by_name(&state, &org_name).await?;
    require_org_member(&state, org.id, auth_user.user_id).await?;

    let invitations: Vec<OrgInvitation> = sqlx::query_as(
        "SELECT * FROM org_invitations
         WHERE org_id = $1 AND accepted IS NULL
         ORDER BY created_at DESC"
    )
        .bind(org.id)
        .fetch_all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(invitations))
}

pub async fn create_invitation(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path(org_name): Path<String>,
    Json(input):    Json<InviteInput>,
) -> Result<impl IntoResponse, AppError> {
    let org = find_org_by_name(&state, &org_name).await?;
    require_org_owner(&state, org.id, auth_user.user_id).await?;

    if !input.email.contains('@') {
        return Err(AppError::BadRequest("Invalid email address".into()));
    }

    let role  = input.role.as_deref().unwrap_or("member");
    let token = format!("{}", Uuid::new_v4().simple());

    let invitation: OrgInvitation = sqlx::query_as(
        "INSERT INTO org_invitations
            (org_id, inviter_id, invitee_email, role, token, expires_at)
         VALUES ($1, $2, $3, $4::org_member_role, $5, now() + interval '7 days')
         RETURNING *"
    )
        .bind(org.id)
        .bind(auth_user.user_id)
        .bind(&input.email)
        .bind(role)
        .bind(&token)
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok((StatusCode::CREATED, Json(invitation)))
}

pub async fn accept_invitation(
    State(state): State<AppState>,
    auth_user:    AuthUser,
    Path(token):  Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let invitation: OrgInvitation = sqlx::query_as(
        "SELECT * FROM org_invitations
         WHERE token = $1 AND accepted IS NULL AND expires_at > now()"
    )
        .bind(&token)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound("Invitation".into()))?;

    // verify email matches
    let user = users::find_by_id(&state.db, auth_user.user_id)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    let email_match: (bool,) = sqlx::query_as(
        "SELECT EXISTS(SELECT 1 FROM user_emails WHERE user_id = $1 AND email = $2)"
    )
        .bind(user.id)
        .bind(&invitation.invitee_email)
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    if !email_match.0 {
        return Err(AppError::Forbidden);
    }

    // mark accepted
    sqlx::query(
        "UPDATE org_invitations SET accepted = true WHERE id = $1"
    )
        .bind(invitation.id)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    // add as member
    sqlx::query(
        "INSERT INTO org_members (org_id, user_id, role)
         VALUES ($1, $2, $3::org_member_role)
         ON CONFLICT (org_id, user_id) DO NOTHING"
    )
        .bind(invitation.org_id)
        .bind(user.id)
        .bind(invitation.role.clone())
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(json!({ "message": "Invitation accepted" })))
}

pub async fn decline_invitation(
    State(state): State<AppState>,
    Path(token):  Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let result = sqlx::query(
        "UPDATE org_invitations SET accepted = false
         WHERE token = $1 AND accepted IS NULL"
    )
        .bind(&token)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Invitation".into()));
    }

    Ok(Json(json!({ "message": "Invitation declined" })))
}

pub async fn cancel_invitation(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((org_name, invitation_id)): Path<(String, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let org = find_org_by_name(&state, &org_name).await?;
    require_org_owner(&state, org.id, auth_user.user_id).await?;

    sqlx::query(
        "DELETE FROM org_invitations WHERE id = $1 AND org_id = $2"
    )
        .bind(invitation_id)
        .bind(org.id)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// Teams

#[derive(Debug, Deserialize)]
pub struct CreateTeamInput {
    pub name:        String,
    pub description: Option<String>,
    pub privacy:     Option<String>,
    pub parent_id:   Option<Uuid>,
}

pub async fn list_teams(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path(org_name): Path<String>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<impl IntoResponse, AppError> {
    let org = find_org_by_name(&state, &org_name).await?;
    require_org_member(&state, org.id, auth_user.user_id).await?;

    let per_page = pagination.per_page.unwrap_or(30).min(100);
    let offset   = (pagination.page.unwrap_or(1) - 1) * per_page;

    let teams: Vec<OrgTeam> = sqlx::query_as(
        "SELECT * FROM org_teams WHERE org_id = $1
         ORDER BY name ASC
         LIMIT $2 OFFSET $3"
    )
        .bind(org.id)
        .bind(per_page)
        .bind(offset)
        .fetch_all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(teams))
}

pub async fn create_team(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path(org_name): Path<String>,
    Json(input):    Json<CreateTeamInput>,
) -> Result<impl IntoResponse, AppError> {
    let org = find_org_by_name(&state, &org_name).await?;
    require_org_owner(&state, org.id, auth_user.user_id).await?;

    if input.name.trim().is_empty() {
        return Err(AppError::BadRequest("Team name is required".into()));
    }

    let slug = input.name.to_lowercase().replace(' ', "-");

    let team: OrgTeam = sqlx::query_as(
        "INSERT INTO org_teams (org_id, name, slug, description, privacy, parent_id)
         VALUES ($1, $2, $3, $4, $5, $6)
         RETURNING *"
    )
        .bind(org.id)
        .bind(&input.name)
        .bind(&slug)
        .bind(&input.description)
        .bind(input.privacy.as_deref().unwrap_or("secret"))
        .bind(input.parent_id)
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok((StatusCode::CREATED, Json(team)))
}

pub async fn get_team(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((org_name, team_slug)): Path<(String, String)>,
) -> Result<impl IntoResponse, AppError> {
    let org = find_org_by_name(&state, &org_name).await?;
    require_org_member(&state, org.id, auth_user.user_id).await?;

    let team = find_team_by_slug(&state, org.id, &team_slug).await?;
    Ok(Json(team))
}

#[derive(Debug, Deserialize)]
pub struct UpdateTeamInput {
    pub name:        Option<String>,
    pub description: Option<String>,
    pub privacy:     Option<String>,
    pub parent_id:   Option<Uuid>,
}

pub async fn update_team(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((org_name, team_slug)): Path<(String, String)>,
    Json(input):    Json<UpdateTeamInput>,
) -> Result<impl IntoResponse, AppError> {
    let org = find_org_by_name(&state, &org_name).await?;
    require_org_owner(&state, org.id, auth_user.user_id).await?;

    let team = find_team_by_slug(&state, org.id, &team_slug).await?;

    let new_slug = input.name.as_ref()
        .map(|n| n.to_lowercase().replace(' ', "-"))
        .unwrap_or(team.slug.clone());

    let updated: OrgTeam = sqlx::query_as(
        "UPDATE org_teams SET
            name        = COALESCE($1, name),
            slug        = $2,
            description = COALESCE($3, description),
            privacy     = COALESCE($4, privacy),
            parent_id   = COALESCE($5, parent_id),
            updated_at  = now()
         WHERE id = $6
         RETURNING *"
    )
        .bind(&input.name)
        .bind(&new_slug)
        .bind(&input.description)
        .bind(&input.privacy)
        .bind(input.parent_id)
        .bind(team.id)
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(updated))
}

pub async fn delete_team(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((org_name, team_slug)): Path<(String, String)>,
) -> Result<impl IntoResponse, AppError> {
    let org = find_org_by_name(&state, &org_name).await?;
    require_org_owner(&state, org.id, auth_user.user_id).await?;

    let team = find_team_by_slug(&state, org.id, &team_slug).await?;

    sqlx::query("DELETE FROM org_teams WHERE id = $1")
        .bind(team.id)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// Team members

pub async fn list_team_members(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((org_name, team_slug)): Path<(String, String)>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<impl IntoResponse, AppError> {
    let org = find_org_by_name(&state, &org_name).await?;
    require_org_member(&state, org.id, auth_user.user_id).await?;

    let team     = find_team_by_slug(&state, org.id, &team_slug).await?;
    let per_page = pagination.per_page.unwrap_or(30).min(100);
    let offset   = (pagination.page.unwrap_or(1) - 1) * per_page;

    let members: Vec<crate::models::user::User> = sqlx::query_as(
        "SELECT u.* FROM users u
         JOIN team_members tm ON tm.user_id = u.id
         WHERE tm.team_id = $1
         ORDER BY tm.created_at ASC
         LIMIT $2 OFFSET $3"
    )
        .bind(team.id)
        .bind(per_page)
        .bind(offset)
        .fetch_all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(members))
}

pub async fn add_team_member(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((org_name, team_slug, username)): Path<(String, String, String)>,
) -> Result<impl IntoResponse, AppError> {
    let org = find_org_by_name(&state, &org_name).await?;
    require_org_owner(&state, org.id, auth_user.user_id).await?;

    let team = find_team_by_slug(&state, org.id, &team_slug).await?;
    let user = users::find_by_username(&state.db, &username)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    // user must be org member first
    require_org_member(&state, org.id, user.id).await
        .map_err(|_| AppError::BadRequest("User is not a member of this organization".into()))?;

    sqlx::query(
        "INSERT INTO team_members (team_id, user_id)
         VALUES ($1, $2)
         ON CONFLICT (team_id, user_id) DO NOTHING"
    )
        .bind(team.id)
        .bind(user.id)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn remove_team_member(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((org_name, team_slug, username)): Path<(String, String, String)>,
) -> Result<impl IntoResponse, AppError> {
    let org = find_org_by_name(&state, &org_name).await?;
    require_org_owner(&state, org.id, auth_user.user_id).await?;

    let team = find_team_by_slug(&state, org.id, &team_slug).await?;
    let user = users::find_by_username(&state.db, &username)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    sqlx::query(
        "DELETE FROM team_members WHERE team_id = $1 AND user_id = $2"
    )
        .bind(team.id)
        .bind(user.id)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// Team repos

pub async fn list_team_repos(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((org_name, team_slug)): Path<(String, String)>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<impl IntoResponse, AppError> {
    let org = find_org_by_name(&state, &org_name).await?;
    require_org_member(&state, org.id, auth_user.user_id).await?;

    let team     = find_team_by_slug(&state, org.id, &team_slug).await?;
    let per_page = pagination.per_page.unwrap_or(30).min(100);
    let offset   = (pagination.page.unwrap_or(1) - 1) * per_page;

    let repos: Vec<crate::models::repo::Repository> = sqlx::query_as(
        "SELECT r.* FROM repositories r
         JOIN team_repo_permissions tp ON tp.repo_id = r.id
         WHERE tp.team_id = $1
         ORDER BY r.name ASC
         LIMIT $2 OFFSET $3"
    )
        .bind(team.id)
        .bind(per_page)
        .bind(offset)
        .fetch_all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(repos))
}

#[derive(Debug, Deserialize)]
pub struct TeamRepoInput {
    pub permission: Option<String>,
}

pub async fn add_team_repo(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((org_name, team_slug, owner, repo_name)): Path<(String, String, String, String)>,
    Json(input):    Json<TeamRepoInput>,
) -> Result<impl IntoResponse, AppError> {
    let org = find_org_by_name(&state, &org_name).await?;
    require_org_owner(&state, org.id, auth_user.user_id).await?;

    let team = find_team_by_slug(&state, org.id, &team_slug).await?;

    let repo_owner = users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    let repo = crate::db::queries::repos::find_by_owner_and_name(
        &state.db, repo_owner.id, &repo_name
    )
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    let permission = input.permission.as_deref().unwrap_or("pull");

    sqlx::query(
        "INSERT INTO team_repo_permissions (team_id, repo_id, permission)
         VALUES ($1, $2, $3::team_permission)
         ON CONFLICT (team_id, repo_id)
         DO UPDATE SET permission = $3::team_permission"
    )
        .bind(team.id)
        .bind(repo.id)
        .bind(permission)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn remove_team_repo(
    State(state):   State<AppState>,
    auth_user:      AuthUser,
    Path((org_name, team_slug, owner, repo_name)): Path<(String, String, String, String)>,
) -> Result<impl IntoResponse, AppError> {
    let org = find_org_by_name(&state, &org_name).await?;
    require_org_owner(&state, org.id, auth_user.user_id).await?;

    let team = find_team_by_slug(&state, org.id, &team_slug).await?;

    let repo_owner = users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    let repo = crate::db::queries::repos::find_by_owner_and_name(
        &state.db, repo_owner.id, &repo_name
    )
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    sqlx::query(
        "DELETE FROM team_repo_permissions WHERE team_id = $1 AND repo_id = $2"
    )
        .bind(team.id)
        .bind(repo.id)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// Org repos

pub async fn list_org_repos(
    State(state):   State<AppState>,
    Path(org_name): Path<String>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<impl IntoResponse, AppError> {
    let org      = find_org_by_name(&state, &org_name).await?;
    let per_page = pagination.per_page.unwrap_or(30).min(100);
    let offset   = (pagination.page.unwrap_or(1) - 1) * per_page;

    let repos: Vec<crate::models::repo::Repository> = sqlx::query_as(
        "SELECT * FROM repositories
         WHERE org_id = $1 AND visibility = 'public'::repo_visibility
         ORDER BY updated_at DESC
         LIMIT $2 OFFSET $3"
    )
        .bind(org.id)
        .bind(per_page)
        .bind(offset)
        .fetch_all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(repos))
}

// Helpers

async fn find_org_by_name(state: &AppState, name: &str) -> Result<Organization, AppError> {
    sqlx::query_as("SELECT * FROM organizations WHERE name = $1")
        .bind(name)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound("Organization".into()))
}

async fn find_team_by_slug(
    state:  &AppState,
    org_id: Uuid,
    slug:   &str,
) -> Result<OrgTeam, AppError> {
    sqlx::query_as(
        "SELECT * FROM org_teams WHERE org_id = $1 AND slug = $2"
    )
        .bind(org_id)
        .bind(slug)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound("Team".into()))
}

async fn require_org_owner(
    state:   &AppState,
    org_id:  Uuid,
    user_id: Uuid,
) -> Result<(), AppError> {
    let is_owner: (bool,) = sqlx::query_as(
        "SELECT EXISTS(SELECT 1 FROM org_members
         WHERE org_id = $1 AND user_id = $2 AND role = 'owner'::org_member_role)"
    )
        .bind(org_id)
        .bind(user_id)
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    if !is_owner.0 {
        return Err(AppError::Forbidden);
    }
    Ok(())
}

async fn require_org_member(
    state:   &AppState,
    org_id:  Uuid,
    user_id: Uuid,
) -> Result<(), AppError> {
    let is_member: (bool,) = sqlx::query_as(
        "SELECT EXISTS(SELECT 1 FROM org_members WHERE org_id = $1 AND user_id = $2)"
    )
        .bind(org_id)
        .bind(user_id)
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    if !is_member.0 {
        return Err(AppError::Forbidden);
    }
    Ok(())
}