use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Webhook {
    pub id:             Uuid,
    pub repo_id:        Option<Uuid>,
    pub org_id:         Option<Uuid>,
    pub creator_id:     Option<Uuid>,
    pub url:            String,
    pub content_type:   String,
    pub secret_hash:    Option<String>,
    pub events:         Vec<String>,
    pub is_active:      bool,
    pub insecure_ssl:   bool,
    pub created_at:     DateTime<Utc>,
    pub updated_at:     DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct WebhookDelivery {
    pub id:                 Uuid,
    pub webhook_id:         Uuid,
    pub event:              String,
    pub request_headers:    Value,
    pub request_body:       Value,
    pub response_status:    Option<i32>,
    pub response_headers:   Option<Value>,
    pub response_body:      Option<String>,
    pub duration_ms:        Option<i32>,
    pub redelivery:         bool,
    pub created_at:         DateTime<Utc>,
}