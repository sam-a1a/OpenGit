use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Session {
    pub id:             Uuid,
    pub user_id:        Uuid,
    pub ip_address:     Option<String>,
    pub user_agent:     Option<String>,
    pub last_active_at: DateTime<Utc>,
    pub expires_at:     DateTime<Utc>,
    pub created_at:     DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RefreshToken {
    pub id:             Uuid,
    pub user_id:        Uuid,
    pub token_hash:     String,
    pub family_id:      Uuid,
    pub session_id:     Option<Uuid>,
    pub used:           bool,
    pub ip_address:     Option<String>,
    pub expires_at:     DateTime<Utc>,
    pub created_at:     DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PersonalAccessToken {
    pub id:             Uuid,
    pub user_id:        Uuid,
    pub name:           String,
    pub token_hash:     String,
    pub scopes:         Vec<String>,
    pub last_used_at:   Option<DateTime<Utc>>,
    pub expires_at:     Option<DateTime<Utc>>,
    pub created_at:     DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct OauthApp {
    pub id:             Uuid,
    pub owner_id:       Uuid,
    pub name:           String,
    pub description:    Option<String>,
    pub homepage_url:   String,
    pub callback_url:   String,
    pub client_id:      String,
    pub client_secret:  String,
    pub logo_url:       Option<String>,
    pub created_at:     DateTime<Utc>,
    pub updated_at:     DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct OauthAuthorization {
    pub id:             Uuid,
    pub user_id:        Uuid,
    pub app_id:         Uuid,
    pub scopes:         Vec<String>,
    pub access_token:   String,
    pub created_at:     DateTime<Utc>,
}