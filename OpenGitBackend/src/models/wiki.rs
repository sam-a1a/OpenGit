use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct WikiPage {
    pub id:         Uuid,
    pub repo_id:    Uuid,
    pub author_id:  Option<Uuid>,
    pub title:      String,
    pub slug:       String,
    pub content:    String,
    pub is_sidebar: bool,
    pub is_footer:  bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct WikiRevision {
    pub id:         Uuid,
    pub page_id:    Uuid,
    pub author_id:  Option<Uuid>,
    pub content:    String,
    pub message:    Option<String>,
    pub created_at: DateTime<Utc>,
}