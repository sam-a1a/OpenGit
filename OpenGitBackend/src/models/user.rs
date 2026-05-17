use crate::models::enums::{UserRole, UserStatusAvailability};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id:                     Uuid,
    pub username:               String,
    pub display_name:           Option<String>,
    pub bio:                    Option<String>,
    pub avatar_url:             Option<String>,
    pub website:                Option<String>,
    pub location:               Option<String>,
    pub pronouns:               Option<String>,
    pub company:                Option<String>,
    pub twitter_username:       Option<String>,
    pub role:                   UserRole,
    pub status_emoji:           Option<String>,
    pub status_message:         Option<String>,
    pub status_availability:    UserStatusAvailability,
    pub status_expires_at:      Option<DateTime<Utc>>,
    pub is_active:              bool,
    pub is_verified:            bool,
    pub two_factor_enabled:     bool,
    pub profile_private:        bool,
    pub created_at:             DateTime<Utc>,
    pub updated_at:             DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserEmail {
    pub id:             Uuid,
    pub user_id:        Uuid,
    pub email:          String,
    pub is_primary:     bool,
    pub is_verified:    bool,
    pub verified_at:    Option<DateTime<Utc>>,
    pub created_at:     DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserSshKey {
    pub id:             Uuid,
    pub user_id:        Uuid,
    pub title:          String,
    pub key_type:       String,
    pub key_data:       String,
    pub fingerprint:    String,
    pub last_used_at:   Option<DateTime<Utc>>,
    pub expires_at:     Option<DateTime<Utc>>,
    pub read_only:      bool,
    pub created_at:     DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserGpgKey {
    pub id:             Uuid,
    pub user_id:        Uuid,
    pub key_id:         String,
    pub public_key:     String,
    pub emails:         Vec<String>,
    pub expires_at:     Option<DateTime<Utc>>,
    pub created_at:     DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserOauthConnection {
    pub id:                 Uuid,
    pub user_id:            Uuid,
    pub provider:           String,
    pub provider_user_id:   String,
    pub access_token:       Option<String>,
    pub refresh_token:      Option<String>,
    pub expires_at:         Option<DateTime<Utc>>,
    pub created_at:         DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserFollow {
    pub follower_id:    Uuid,
    pub following_id:   Uuid,
    pub created_at:     DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserBlock {
    pub blocker_id:     Uuid,
    pub blocked_id:     Uuid,
    pub created_at:     DateTime<Utc>,
}