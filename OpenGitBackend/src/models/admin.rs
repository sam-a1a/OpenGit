use crate::models::enums::AuditAction;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AuditLog {
    pub id:             Uuid,
    pub actor_id:       Option<Uuid>,
    pub actor_ip:       Option<String>,
    pub action:         AuditAction,
    pub target_type:    Option<String>,
    pub target_id:      Option<Uuid>,
    pub metadata:       Value,
    pub created_at:     DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SiteSetting {
    pub key:            String,
    pub value:          Value,
    pub updated_by_id:  Option<Uuid>,
    pub updated_at:     DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BannedUser {
    pub id:             Uuid,
    pub user_id:        Option<Uuid>,
    pub email:          Option<String>,
    pub ip_address:     Option<String>,
    pub reason:         Option<String>,
    pub banned_by_id:   Option<Uuid>,
    pub expires_at:     Option<DateTime<Utc>>,
    pub created_at:     DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AbuseReport {
    pub id:             Uuid,
    pub reporter_id:    Uuid,
    pub target_type:    String,
    pub target_id:      Uuid,
    pub reason:         String,
    pub description:    Option<String>,
    pub resolved:       bool,
    pub resolved_by_id: Option<Uuid>,
    pub created_at:     DateTime<Utc>,
}