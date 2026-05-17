use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Release {
    pub id:             Uuid,
    pub repo_id:        Uuid,
    pub author_id:      Option<Uuid>,
    pub tag_name:       String,
    pub target_sha:     Option<String>,
    pub name:           Option<String>,
    pub body:           Option<String>,
    pub is_draft:       bool,
    pub is_prerelease:  bool,
    pub is_latest:      bool,
    pub published_at:   Option<DateTime<Utc>>,
    pub created_at:     DateTime<Utc>,
    pub updated_at:     DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ReleaseAsset {
    pub id:             Uuid,
    pub release_id:     Uuid,
    pub uploader_id:    Option<Uuid>,
    pub name:           String,
    pub label:          Option<String>,
    pub content_type:   String,
    pub size_bytes:     i64,
    pub download_count: i32,
    pub storage_key:    String,
    pub state:          String,
    pub created_at:     DateTime<Utc>,
    pub updated_at:     DateTime<Utc>,
}