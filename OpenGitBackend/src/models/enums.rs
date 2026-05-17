use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "user_role", rename_all = "snake_case")]
pub enum UserRole {
    User,
    Admin,
    Superadmin,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "user_status_availability", rename_all = "snake_case")]
pub enum UserStatusAvailability {
    Available,
    Busy,
    Away,
    DoNotDisturb,
    Invisible,
    Sick,
    OnVacation,
    InAMeeting,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "repo_visibility", rename_all = "snake_case")]
pub enum RepoVisibility {
    Public,
    Private,
    Internal,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "org_visibility", rename_all = "snake_case")]
pub enum OrgVisibility {
    Public,
    Private,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "org_member_role", rename_all = "snake_case")]
pub enum OrgMemberRole {
    Owner,
    Member,
    BillingManager,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "team_permission", rename_all = "snake_case")]
pub enum TeamPermission {
    Pull,
    Triage,
    Push,
    Maintain,
    Admin,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "collaborator_permission", rename_all = "snake_case")]
pub enum CollaboratorPermission {
    Read,
    Triage,
    Write,
    Maintain,
    Admin,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "issue_state", rename_all = "snake_case")]
pub enum IssueState {
    Open,
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "issue_state_reason", rename_all = "snake_case")]
pub enum IssueStateReason {
    Completed,
    NotPlanned,
    Reopened,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "reaction_type", rename_all = "snake_case")]
pub enum ReactionType {
    ThumbsUp,
    ThumbsDown,
    Laugh,
    Hooray,
    Confused,
    Heart,
    Rocket,
    Eyes,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "pr_state", rename_all = "snake_case")]
pub enum PrState {
    Open,
    Closed,
    Merged,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "pr_review_state", rename_all = "snake_case")]
pub enum PrReviewState {
    Pending,
    Approved,
    ChangesRequested,
    Commented,
    Dismissed,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "check_status", rename_all = "snake_case")]
pub enum CheckStatus {
    Queued,
    InProgress,
    Completed,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "check_conclusion", rename_all = "snake_case")]
pub enum CheckConclusion {
    Success,
    Failure,
    Neutral,
    Cancelled,
    Skipped,
    TimedOut,
    ActionRequired,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "workflow_run_status", rename_all = "snake_case")]
pub enum WorkflowRunStatus {
    Queued,
    InProgress,
    Completed,
    Waiting,
    Requested,
    Pending,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "workflow_conclusion", rename_all = "snake_case")]
pub enum WorkflowConclusion {
    Success,
    Failure,
    Neutral,
    Cancelled,
    Skipped,
    TimedOut,
    ActionRequired,
    StartupFailure,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "runner_status", rename_all = "snake_case")]
pub enum RunnerStatus {
    Online,
    Offline,
    Busy,
    Disabled,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "notification_reason", rename_all = "snake_case")]
pub enum NotificationReason {
    Assign,
    Author,
    Comment,
    CiActivity,
    Invitation,
    Manual,
    Mention,
    ReviewRequested,
    SecurityAlert,
    StateChange,
    Subscribed,
    TeamMention,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "advisory_severity", rename_all = "snake_case")]
pub enum AdvisorySeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "alert_state", rename_all = "snake_case")]
pub enum AlertState {
    Open,
    Dismissed,
    Fixed,
    AutoDismissed,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "audit_action", rename_all = "snake_case")]
pub enum AuditAction {
    UserCreated,
    UserDeleted,
    UserBanned,
    UserPromoted,
    RepoCreated,
    RepoDeleted,
    RepoTransferred,
    RepoArchived,
    OrgCreated,
    OrgDeleted,
    MemberAdded,
    MemberRemoved,
    SettingsUpdated,
    WebhookCreated,
    WebhookDeleted,
    TokenCreated,
    TokenRevoked,
    LoginSuccess,
    LoginFailed,
    TwoFactorEnabled,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "project_visibility", rename_all = "snake_case")]
pub enum ProjectVisibility {
    Private,
    Public,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "gist_visibility", rename_all = "snake_case")]
pub enum GistVisibility {
    Public,
    Secret,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "pages_source", rename_all = "snake_case")]
pub enum PagesSource {
    Branch,
    Workflow,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "pages_status", rename_all = "snake_case")]
pub enum PagesStatus {
    Building,
    Built,
    Errored,
    Disabled,
}

impl std::fmt::Display for RepoVisibility {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RepoVisibility::Public   => write!(f, "public"),
            RepoVisibility::Private  => write!(f, "private"),
            RepoVisibility::Internal => write!(f, "internal"),
        }
    }
}