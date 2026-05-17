use crate::models::enums::ReactionType;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DiscussionCategory {
    pub id:             Uuid,
    pub repo_id:        Uuid,
    pub name:           String,
    pub description:    Option<String>,
    pub emoji:          Option<String>,
    pub is_answerable:  bool,
    pub created_at:     DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Discussion {
    pub id:             Uuid,
    pub repo_id:        Uuid,
    pub author_id:      Option<Uuid>,
    pub category_id:    Uuid,
    pub number:         i32,
    pub title:          String,
    pub body:           Option<String>,
    pub is_locked:      bool,
    pub is_answered:    bool,
    pub answer_id:      Option<Uuid>,
    pub comment_count:  i32,
    pub created_at:     DateTime<Utc>,
    pub updated_at:     DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DiscussionComment {
    pub id:             Uuid,
    pub discussion_id:  Uuid,
    pub author_id:      Option<Uuid>,
    pub reply_to_id:    Option<Uuid>,
    pub body:           String,
    pub is_answer:      bool,
    pub created_at:     DateTime<Utc>,
    pub updated_at:     DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DiscussionReaction {
    pub id:             Uuid,
    pub user_id:        Uuid,
    pub discussion_id:  Option<Uuid>,
    pub comment_id:     Option<Uuid>,
    pub reaction:       ReactionType,
    pub created_at:     DateTime<Utc>,
}