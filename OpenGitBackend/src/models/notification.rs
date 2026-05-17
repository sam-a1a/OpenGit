use crate::models::enums::NotificationReason;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Notification {
    pub id:             Uuid,
    pub user_id:        Uuid,
    pub repo_id:        Option<Uuid>,
    pub subject_type:   String,
    pub subject_id:     Option<Uuid>,
    pub subject_title:  String,
    pub reason:         NotificationReason,
    pub is_read:        bool,
    pub is_saved:       bool,
    pub last_read_at:   Option<DateTime<Utc>>,
    pub created_at:     DateTime<Utc>,
    pub updated_at:     DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct NotificationSubscription {
    pub user_id:    Uuid,
    pub repo_id:    Option<Uuid>,
    pub org_id:     Option<Uuid>,
    pub subscribed: bool,
    pub ignored:    bool,
    pub created_at: DateTime<Utc>,
}