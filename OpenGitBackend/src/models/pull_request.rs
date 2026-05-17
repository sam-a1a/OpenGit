use crate::models::enums::{CheckConclusion, CheckStatus, PrReviewState, PrState};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PullRequest {
    pub id:                 Uuid,
    pub repo_id:            Uuid,
    pub author_id:          Option<Uuid>,
    pub milestone_id:       Option<Uuid>,
    pub number:             i32,
    pub title:              String,
    pub body:               Option<String>,
    pub state:              PrState,
    pub is_draft:           bool,
    pub locked:             bool,
    pub head_branch:        String,
    pub head_sha:           Option<String>,
    pub base_branch:        String,
    pub base_sha:           Option<String>,
    pub head_repo_id:       Option<Uuid>,
    pub merge_commit_sha:   Option<String>,
    pub merged_at:          Option<DateTime<Utc>>,
    pub merged_by_id:       Option<Uuid>,
    pub closed_at:          Option<DateTime<Utc>>,
    pub comment_count:      i32,
    pub commit_count:       i32,
    pub additions:          i32,
    pub deletions:          i32,
    pub changed_files:      i32,
    pub created_at:         DateTime<Utc>,
    pub updated_at:         DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PrReview {
    pub id:             Uuid,
    pub pr_id:          Uuid,
    pub reviewer_id:    Option<Uuid>,
    pub state:          PrReviewState,
    pub body:           Option<String>,
    pub commit_sha:     Option<String>,
    pub submitted_at:   Option<DateTime<Utc>>,
    pub created_at:     DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PrReviewComment {
    pub id:             Uuid,
    pub pr_id:          Uuid,
    pub review_id:      Option<Uuid>,
    pub author_id:      Option<Uuid>,
    pub reply_to_id:    Option<Uuid>,
    pub body:           String,
    pub path:           Option<String>,
    pub commit_sha:     Option<String>,
    pub line:           Option<i32>,
    pub start_line:     Option<i32>,
    pub side:           Option<String>,
    pub is_edited:      bool,
    pub resolved:       bool,
    pub created_at:     DateTime<Utc>,
    pub updated_at:     DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PrRequestedReviewer {
    pub pr_id:      Uuid,
    pub user_id:    Option<Uuid>,
    pub team_id:    Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PrStatusCheck {
    pub id:             Uuid,
    pub repo_id:        Uuid,
    pub sha:            String,
    pub name:           String,
    pub context:        Option<String>,
    pub status:         CheckStatus,
    pub conclusion:     Option<CheckConclusion>,
    pub target_url:     Option<String>,
    pub description:    Option<String>,
    pub started_at:     Option<DateTime<Utc>>,
    pub completed_at:   Option<DateTime<Utc>>,
    pub created_at:     DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PrAssignee {
    pub pr_id:      Uuid,
    pub user_id:    Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PrLabel {
    pub pr_id:      Uuid,
    pub label_id:   Uuid,
}