use crate::models::enums::{PagesSource, PagesStatus};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Page {
    pub id:             Uuid,
    pub repo_id:        Uuid,
    pub source:         PagesSource,
    pub branch:         Option<String>,
    pub path:           String,
    pub custom_domain:  Option<String>,
    pub https_enforced: bool,
    pub status:         PagesStatus,
    pub url:            Option<String>,
    pub created_at:     DateTime<Utc>,
    pub updated_at:     DateTime<Utc>,
}