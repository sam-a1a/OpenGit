use crate::models::enums::ProjectVisibility;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Project {
    pub id:             Uuid,
    pub owner_id:       Uuid,
    pub repo_id:        Option<Uuid>,
    pub org_id:         Option<Uuid>,
    pub name:           String,
    pub description:    Option<String>,
    pub visibility:     ProjectVisibility,
    pub closed:         bool,
    pub created_at:     DateTime<Utc>,
    pub updated_at:     DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ProjectColumn {
    pub id:         Uuid,
    pub project_id: Uuid,
    pub name:       String,
    pub position:   i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ProjectCard {
    pub id:         Uuid,
    pub column_id:  Uuid,
    pub creator_id: Option<Uuid>,
    pub note:       Option<String>,
    pub issue_id:   Option<Uuid>,
    pub pr_id:      Option<Uuid>,
    pub position:   i32,
    pub archived:   bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}