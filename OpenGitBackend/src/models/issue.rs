use crate::models::enums::{IssueState, IssueStateReason, ReactionType};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Milestone {
    pub id:             Uuid,
    pub repo_id:        Uuid,
    pub title:          String,
    pub description:    Option<String>,
    pub state:          IssueState,
    pub due_on:         Option<DateTime<Utc>>,
    pub closed_at:      Option<DateTime<Utc>>,
    pub created_at:     DateTime<Utc>,
    pub updated_at:     DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Label {
    pub id:             Uuid,
    pub repo_id:        Uuid,
    pub name:           String,
    pub color:          String,
    pub description:    Option<String>,
    pub created_at:     DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Issue {
    pub id:             Uuid,
    pub repo_id:        Uuid,
    pub author_id:      Option<Uuid>,
    pub milestone_id:   Option<Uuid>,
    pub number:         i32,
    pub title:          String,
    pub body:           Option<String>,
    pub state:          IssueState,
    pub state_reason:   Option<IssueStateReason>,
    pub locked:         bool,
    pub lock_reason:    Option<String>,
    pub is_pinned:      bool,
    pub comment_count:  i32,
    pub closed_at:      Option<DateTime<Utc>>,
    pub closed_by_id:   Option<Uuid>,
    pub created_at:     DateTime<Utc>,
    pub updated_at:     DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct IssueAssignee {
    pub issue_id:   Uuid,
    pub user_id:    Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct IssueLabel {
    pub issue_id:   Uuid,
    pub label_id:   Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct IssueComment {
    pub id:         Uuid,
    pub issue_id:   Uuid,
    pub author_id:  Option<Uuid>,
    pub body:       String,
    pub is_edited:  bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct IssueReaction {
    pub id:         Uuid,
    pub user_id:    Uuid,
    pub issue_id:   Option<Uuid>,
    pub comment_id: Option<Uuid>,
    pub reaction:   ReactionType,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct IssueSubscription {
    pub user_id:    Uuid,
    pub issue_id:   Uuid,
    pub subscribed: bool,
    pub created_at: DateTime<Utc>,
}