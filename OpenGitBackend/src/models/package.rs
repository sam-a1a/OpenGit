use crate::models::enums::RepoVisibility;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Package {
    pub id:             Uuid,
    pub repo_id:        Option<Uuid>,
    pub org_id:         Option<Uuid>,
    pub owner_id:       Uuid,
    pub name:           String,
    pub package_type:   String,
    pub visibility:     RepoVisibility,
    pub download_count: i64,
    pub created_at:     DateTime<Utc>,
    pub updated_at:     DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PackageVersion {
    pub id:             Uuid,
    pub package_id:     Uuid,
    pub version:        String,
    pub description:    Option<String>,
    pub metadata:       Value,
    pub download_count: i64,
    pub created_at:     DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PackageFile {
    pub id:             Uuid,
    pub version_id:     Uuid,
    pub name:           String,
    pub size_bytes:     i64,
    pub storage_key:    String,
    pub sha256:         Option<String>,
    pub created_at:     DateTime<Utc>,
}