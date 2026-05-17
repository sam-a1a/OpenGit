use crate::models::enums::{AdvisorySeverity, AlertState};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SecurityAdvisory {
    pub id:             Uuid,
    pub repo_id:        Uuid,
    pub cve_id:         Option<String>,
    pub ghsa_id:        Option<String>,
    pub summary:        String,
    pub description:    Option<String>,
    pub severity:       AdvisorySeverity,
    pub cvss_score:     Option<f32>,
    pub published_at:   Option<DateTime<Utc>>,
    pub created_at:     DateTime<Utc>,
    pub updated_at:     DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SecretScanningAlert {
    pub id:             Uuid,
    pub repo_id:        Uuid,
    pub number:         i32,
    pub secret_type:    String,
    pub secret:         String,
    pub state:          AlertState,
    pub resolved_by_id: Option<Uuid>,
    pub resolved_at:    Option<DateTime<Utc>>,
    pub resolution:     Option<String>,
    pub created_at:     DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CodeScanningAlert {
    pub id:                 Uuid,
    pub repo_id:            Uuid,
    pub number:             i32,
    pub rule_id:            Option<String>,
    pub rule_severity:      Option<AdvisorySeverity>,
    pub rule_description:   Option<String>,
    pub state:              AlertState,
    pub path:               Option<String>,
    pub start_line:         Option<i32>,
    pub end_line:           Option<i32>,
    pub dismissed_by_id:    Option<Uuid>,
    pub dismissed_at:       Option<DateTime<Utc>>,
    pub created_at:         DateTime<Utc>,
}