use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Branch {
    pub id:             Uuid,
    pub repo_id:        Uuid,
    pub name:           String,
    pub head_sha:       String,
    pub is_default:     bool,
    pub is_protected:   bool,
    pub updated_at:     DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Tag {
    pub id:             Uuid,
    pub repo_id:        Uuid,
    pub name:           String,
    pub sha:            String,
    pub is_annotated:   bool,
    pub tagger_name:    Option<String>,
    pub tagger_email:   Option<String>,
    pub message:        Option<String>,
    pub created_at:     DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Commit {
    pub id:                 Uuid,
    pub repo_id:            Uuid,
    pub sha:                String,
    pub message:            String,
    pub author_name:        String,
    pub author_email:       String,
    pub authored_at:        DateTime<Utc>,
    pub committer_name:     Option<String>,
    pub committer_email:    Option<String>,
    pub committed_at:       Option<DateTime<Utc>>,
    pub user_id:            Option<Uuid>,
    pub verified:           bool,
}