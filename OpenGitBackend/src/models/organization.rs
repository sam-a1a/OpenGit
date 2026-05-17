use crate::models::enums::{OrgMemberRole, OrgVisibility, TeamPermission};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Organization {
    pub id:                 Uuid,
    pub name:               String,
    pub display_name:       Option<String>,
    pub description:        Option<String>,
    pub avatar_url:         Option<String>,
    pub website:            Option<String>,
    pub location:           Option<String>,
    pub email:              Option<String>,
    pub twitter_username:   Option<String>,
    pub visibility:         OrgVisibility,
    pub verified:           bool,
    pub created_at:         DateTime<Utc>,
    pub updated_at:         DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct OrgMember {
    pub org_id:     Uuid,
    pub user_id:    Uuid,
    pub role:       OrgMemberRole,
    pub joined_at:  DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct OrgInvitation {
    pub id:             Uuid,
    pub org_id:         Uuid,
    pub inviter_id:     Uuid,
    pub invitee_email:  String,
    pub role:           OrgMemberRole,
    pub token:          String,
    pub accepted:       Option<bool>,
    pub expires_at:     DateTime<Utc>,
    pub created_at:     DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct OrgTeam {
    pub id:             Uuid,
    pub org_id:         Uuid,
    pub parent_id:      Option<Uuid>,
    pub name:           String,
    pub slug:           String,
    pub description:    Option<String>,
    pub privacy:        String,
    pub created_at:     DateTime<Utc>,
    pub updated_at:     DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TeamMember {
    pub team_id:    Uuid,
    pub user_id:    Uuid,
    pub role:       String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TeamRepoPermission {
    pub team_id:    Uuid,
    pub repo_id:    Uuid,
    pub permission: TeamPermission,
    pub created_at: DateTime<Utc>,
}