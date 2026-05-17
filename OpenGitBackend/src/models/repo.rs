use crate::models::enums::{CollaboratorPermission, RepoVisibility};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Repository {
    pub id:                         Uuid,
    pub owner_id:                   Uuid,
    pub org_id:                     Option<Uuid>,
    pub name:                       String,
    pub description:                Option<String>,
    pub visibility:                 RepoVisibility,
    pub default_branch:             String,
    pub is_fork:                    bool,
    pub forked_from_id:             Option<Uuid>,
    pub is_template:                bool,
    pub template_from_id:           Option<Uuid>,
    pub is_archived:                bool,
    pub is_disabled:                bool,
    pub is_empty:                   bool,
    pub has_issues:                 bool,
    pub has_wiki:                   bool,
    pub has_projects:               bool,
    pub has_discussions:            bool,
    pub has_packages:               bool,
    pub has_pages:                  bool,
    pub allow_merge_commit:         bool,
    pub allow_squash_merge:         bool,
    pub allow_rebase_merge:         bool,
    pub delete_branch_on_merge:     bool,
    pub star_count:                 i32,
    pub fork_count:                 i32,
    pub watcher_count:              i32,
    pub open_issue_count:           i32,
    pub git_path:                   String,
    pub created_at:                 DateTime<Utc>,
    pub updated_at:                 DateTime<Utc>,
    pub pushed_at:                  Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RepoTopic {
    pub repo_id:    Uuid,
    pub topic:      String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RepoStar {
    pub user_id:    Uuid,
    pub repo_id:    Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RepoWatch {
    pub user_id:    Uuid,
    pub repo_id:    Uuid,
    pub level:      String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RepoCollaborator {
    pub repo_id:    Uuid,
    pub user_id:    Uuid,
    pub permission: CollaboratorPermission,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RepoDeployKey {
    pub id:             Uuid,
    pub repo_id:        Uuid,
    pub title:          String,
    pub key_data:       String,
    pub fingerprint:    String,
    pub read_only:      bool,
    pub created_at:     DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RepoBranchProtection {
    pub id:                             Uuid,
    pub repo_id:                        Uuid,
    pub pattern:                        String,
    pub require_pull_request:           bool,
    pub required_approving_review_count: i32,
    pub dismiss_stale_reviews:          bool,
    pub require_code_owner_reviews:     bool,
    pub require_status_checks:          bool,
    pub required_status_checks:         Vec<String>,
    pub require_up_to_date_branch:      bool,
    pub restrict_pushes:                bool,
    pub allow_force_pushes:             bool,
    pub allow_deletions:                bool,
    pub created_at:                     DateTime<Utc>,
    pub updated_at:                     DateTime<Utc>,
}