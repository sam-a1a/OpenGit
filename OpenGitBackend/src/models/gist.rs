use crate::models::enums::GistVisibility;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Gist {
    pub id:             Uuid,
    pub owner_id:       Uuid,
    pub description:    Option<String>,
    pub visibility:     GistVisibility,
    pub fork_of_id:     Option<Uuid>,
    pub comment_count:  i32,
    pub star_count:     i32,
    pub created_at:     DateTime<Utc>,
    pub updated_at:     DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct GistFile {
    pub id:         Uuid,
    pub gist_id:    Uuid,
    pub filename:   String,
    pub language:   Option<String>,
    pub content:    String,
    pub size_bytes: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct GistRevision {
    pub id:             Uuid,
    pub gist_id:        Uuid,
    pub version:        String,
    pub change_status:  Value,
    pub created_at:     DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct GistStar {
    pub user_id:    Uuid,
    pub gist_id:    Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct GistComment {
    pub id:         Uuid,
    pub gist_id:    Uuid,
    pub author_id:  Option<Uuid>,
    pub body:       String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}