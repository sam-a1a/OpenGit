use crate::models::enums::{RunnerStatus, WorkflowConclusion, WorkflowRunStatus};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Workflow {
    pub id:         Uuid,
    pub repo_id:    Uuid,
    pub name:       String,
    pub path:       String,
    pub state:      String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct WorkflowRun {
    pub id:             Uuid,
    pub workflow_id:    Uuid,
    pub repo_id:        Uuid,
    pub actor_id:       Option<Uuid>,
    pub run_number:     i32,
    pub event:          String,
    pub status:         WorkflowRunStatus,
    pub conclusion:     Option<WorkflowConclusion>,
    pub head_sha:       String,
    pub head_branch:    Option<String>,
    pub run_attempt:    i32,
    pub started_at:     Option<DateTime<Utc>>,
    pub completed_at:   Option<DateTime<Utc>>,
    pub created_at:     DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct WorkflowJob {
    pub id:             Uuid,
    pub run_id:         Uuid,
    pub runner_id:      Option<Uuid>,
    pub name:           String,
    pub status:         WorkflowRunStatus,
    pub conclusion:     Option<WorkflowConclusion>,
    pub head_sha:       String,
    pub labels:         Vec<String>,
    pub started_at:     Option<DateTime<Utc>>,
    pub completed_at:   Option<DateTime<Utc>>,
    pub created_at:     DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct WorkflowStep {
    pub id:             Uuid,
    pub job_id:         Uuid,
    pub name:           String,
    pub status:         WorkflowRunStatus,
    pub conclusion:     Option<WorkflowConclusion>,
    pub number:         i32,
    pub started_at:     Option<DateTime<Utc>>,
    pub completed_at:   Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Artifact {
    pub id:             Uuid,
    pub run_id:         Uuid,
    pub repo_id:        Uuid,
    pub name:           String,
    pub size_bytes:     i64,
    pub storage_key:    String,
    pub expires_at:     Option<DateTime<Utc>>,
    pub created_at:     DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RunnerGroup {
    pub id:         Uuid,
    pub org_id:     Option<Uuid>,
    pub repo_id:    Option<Uuid>,
    pub name:       String,
    pub visibility: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Runner {
    pub id:             Uuid,
    pub group_id:       Option<Uuid>,
    pub name:           String,
    pub os:             Option<String>,
    pub architecture:   Option<String>,
    pub labels:         Vec<String>,
    pub status:         RunnerStatus,
    pub token_hash:     String,
    pub last_seen_at:   Option<DateTime<Utc>>,
    pub created_at:     DateTime<Utc>,
}