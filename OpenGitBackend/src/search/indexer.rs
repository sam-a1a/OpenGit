use crate::{error::AppError, state::AppState};
use meilisearch_sdk::client::Client;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

fn meili(state: &AppState) -> Result<Client, AppError> {
    Client::new(&state.config.meilisearch_url, Some(&state.config.meilisearch_key))
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Meilisearch client: {}", e)))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RepoDocument {
    pub id:          String,
    pub name:        String,
    pub description: Option<String>,
    pub owner:       String,
    pub topics:      Vec<String>,
    pub visibility:  String,
    pub stars:       i32,
    pub forks:       i32,
    pub updated_at:  String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IssueDocument {
    pub id:         String,
    pub repo_id:    String,
    pub repo_name:  String,
    pub owner:      String,
    pub number:     i32,
    pub title:      String,
    pub body:       Option<String>,
    pub state:      String,
    pub author:     Option<String>,
    pub labels:     Vec<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PrDocument {
    pub id:         String,
    pub repo_id:    String,
    pub repo_name:  String,
    pub owner:      String,
    pub number:     i32,
    pub title:      String,
    pub body:       Option<String>,
    pub state:      String,
    pub author:     Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserDocument {
    pub id:           String,
    pub username:     String,
    pub display_name: Option<String>,
    pub bio:          Option<String>,
    pub location:     Option<String>,
    pub company:      Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommentDocument {
    pub id:         String,
    pub issue_id:   String,
    pub repo_id:    String,
    pub repo_name:  String,
    pub owner:      String,
    pub body:       String,
    pub author:     Option<String>,
    pub created_at: String,
}

pub async fn setup_indexes(state: &AppState) -> Result<(), AppError> {
    let c = meili(state)?;

    let repos = c.index("repos");
    let _ = repos.set_searchable_attributes(
        &["name", "description", "owner", "topics"]
    ).await;
    let _ = repos.set_filterable_attributes(
        &["visibility", "owner", "stars", "forks"]
    ).await;
    let _ = repos.set_sortable_attributes(&["stars", "forks", "updated_at"]).await;

    let issues = c.index("issues");
    let _ = issues.set_searchable_attributes(
        &["title", "body", "author", "labels"]
    ).await;
    let _ = issues.set_filterable_attributes(
        &["state", "repo_id", "owner", "author"]
    ).await;
    let _ = issues.set_sortable_attributes(&["created_at"]).await;

    let prs = c.index("pull_requests");
    let _ = prs.set_searchable_attributes(&["title", "body", "author"]).await;
    let _ = prs.set_filterable_attributes(
        &["state", "repo_id", "owner", "author"]
    ).await;
    let _ = prs.set_sortable_attributes(&["created_at"]).await;

    let users = c.index("users");
    let _ = users.set_searchable_attributes(
        &["username", "display_name", "bio", "company", "location"]
    ).await;
    let _ = users.set_filterable_attributes(&["username"]).await;
    let _ = users.set_sortable_attributes(&["username"]).await;

    let comments = c.index("comments");
    let _ = comments.set_searchable_attributes(&["body", "author"]).await;
    let _ = comments.set_filterable_attributes(
        &["repo_id", "issue_id", "author"]
    ).await;
    let _ = comments.set_sortable_attributes(&["created_at"]).await;

    tracing::info!("Meilisearch indexes configured");
    Ok(())
}

pub async fn index_repo(state: &AppState, repo_id: Uuid) {
    let result: Result<(), AppError> = async {
        let row: Option<(Uuid, String, Option<String>, String, String, i32, i32, chrono::DateTime<chrono::Utc>)> =
            sqlx::query_as(
                "SELECT r.id, r.name, r.description,
                        r.visibility::text, u.username,
                        r.star_count, r.fork_count, r.updated_at
                 FROM repositories r
                 JOIN users u ON u.id = r.owner_id
                 WHERE r.id = $1"
            )
                .bind(repo_id)
                .fetch_optional(&state.db)
                .await
                .map_err(AppError::Database)?;

        let Some((id, name, description, visibility, owner, stars, forks, updated_at)) = row
        else { return Ok(()); };

        let topics: Vec<String> = sqlx::query_scalar(
            "SELECT topic FROM repo_topics WHERE repo_id = $1"
        )
            .bind(repo_id)
            .fetch_all(&state.db)
            .await
            .map_err(AppError::Database)?;

        let doc = RepoDocument {
            id: id.to_string(),
            name,
            description,
            owner,
            topics,
            visibility,
            stars,
            forks,
            updated_at: updated_at.to_rfc3339(),
        };

        meili(state)?
            .index("repos")
            .add_documents(&[doc], Some("id"))
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Index failed: {}", e)))?;

        Ok(())
    }.await;

    if let Err(e) = result {
        tracing::warn!("Failed to index repo {}: {:?}", repo_id, e);
    }
}

pub async fn index_issue(state: &AppState, issue_id: Uuid) {
    let result: Result<(), AppError> = async {
        let row: Option<(Uuid, i32, String, Option<String>, String, chrono::DateTime<chrono::Utc>, Uuid, String, String, Option<String>)> =
            sqlx::query_as(
                "SELECT i.id, i.number, i.title, i.body,
                        i.state::text, i.created_at,
                        r.id, r.name, u_owner.username,
                        u_author.username
                 FROM issues i
                 JOIN repositories r ON r.id = i.repo_id
                 JOIN users u_owner ON u_owner.id = r.owner_id
                 LEFT JOIN users u_author ON u_author.id = i.author_id
                 WHERE i.id = $1"
            )
                .bind(issue_id)
                .fetch_optional(&state.db)
                .await
                .map_err(AppError::Database)?;

        let Some((id, number, title, body, state_str, created_at, repo_id, repo_name, owner, author)) = row
        else { return Ok(()); };

        let labels: Vec<String> = sqlx::query_scalar(
            "SELECT l.name FROM labels l
             JOIN issue_labels il ON il.label_id = l.id
             WHERE il.issue_id = $1"
        )
            .bind(issue_id)
            .fetch_all(&state.db)
            .await
            .map_err(AppError::Database)?;

        let doc = IssueDocument {
            id: id.to_string(),
            repo_id: repo_id.to_string(),
            repo_name,
            owner,
            number,
            title,
            body,
            state: state_str,
            author,
            labels,
            created_at: created_at.to_rfc3339(),
        };

        meili(state)?
            .index("issues")
            .add_documents(&[doc], Some("id"))
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Index failed: {}", e)))?;

        Ok(())
    }.await;

    if let Err(e) = result {
        tracing::warn!("Failed to index issue {}: {:?}", issue_id, e);
    }
}

pub async fn index_pr(state: &AppState, pr_id: Uuid) {
    let result: Result<(), AppError> = async {
        let row: Option<(Uuid, i32, String, Option<String>, String, chrono::DateTime<chrono::Utc>, Uuid, String, String, Option<String>)> =
            sqlx::query_as(
                "SELECT p.id, p.number, p.title, p.body,
                        p.state::text, p.created_at,
                        r.id, r.name, u_owner.username,
                        u_author.username
                 FROM pull_requests p
                 JOIN repositories r ON r.id = p.repo_id
                 JOIN users u_owner ON u_owner.id = r.owner_id
                 LEFT JOIN users u_author ON u_author.id = p.author_id
                 WHERE p.id = $1"
            )
                .bind(pr_id)
                .fetch_optional(&state.db)
                .await
                .map_err(AppError::Database)?;

        let Some((id, number, title, body, state_str, created_at, repo_id, repo_name, owner, author)) = row
        else { return Ok(()); };

        let doc = PrDocument {
            id: id.to_string(),
            repo_id: repo_id.to_string(),
            repo_name,
            owner,
            number,
            title,
            body,
            state: state_str,
            author,
            created_at: created_at.to_rfc3339(),
        };

        meili(state)?
            .index("pull_requests")
            .add_documents(&[doc], Some("id"))
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Index failed: {}", e)))?;

        Ok(())
    }.await;

    if let Err(e) = result {
        tracing::warn!("Failed to index PR {}: {:?}", pr_id, e);
    }
}

pub async fn index_user(state: &AppState, user_id: Uuid) {
    let result: Result<(), AppError> = async {
        let row: Option<(Uuid, String, Option<String>, Option<String>, Option<String>, Option<String>)> =
            sqlx::query_as(
                "SELECT id, username, display_name, bio, location, company
                 FROM users
                 WHERE id = $1 AND is_active = true AND profile_private = false"
            )
                .bind(user_id)
                .fetch_optional(&state.db)
                .await
                .map_err(AppError::Database)?;

        let Some((id, username, display_name, bio, location, company)) = row
        else { return Ok(()); };

        let doc = UserDocument {
            id: id.to_string(),
            username,
            display_name,
            bio,
            location,
            company,
        };

        meili(state)?
            .index("users")
            .add_documents(&[doc], Some("id"))
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Index failed: {}", e)))?;

        Ok(())
    }.await;

    if let Err(e) = result {
        tracing::warn!("Failed to index user {}: {:?}", user_id, e);
    }
}

pub async fn index_comment(state: &AppState, comment_id: Uuid) {
    let result: Result<(), AppError> = async {
        let row: Option<(Uuid, String, chrono::DateTime<chrono::Utc>, Uuid, Uuid, String, String, Option<String>)> =
            sqlx::query_as(
                "SELECT c.id, c.body, c.created_at,
                        i.id, r.id, r.name,
                        u_owner.username, u_author.username
                 FROM issue_comments c
                 JOIN issues i ON i.id = c.issue_id
                 JOIN repositories r ON r.id = i.repo_id
                 JOIN users u_owner ON u_owner.id = r.owner_id
                 LEFT JOIN users u_author ON u_author.id = c.author_id
                 WHERE c.id = $1"
            )
                .bind(comment_id)
                .fetch_optional(&state.db)
                .await
                .map_err(AppError::Database)?;

        let Some((id, body, created_at, issue_id, repo_id, repo_name, owner, author)) = row
        else { return Ok(()); };

        let doc = CommentDocument {
            id: id.to_string(),
            issue_id: issue_id.to_string(),
            repo_id: repo_id.to_string(),
            repo_name,
            owner,
            body,
            author,
            created_at: created_at.to_rfc3339(),
        };

        meili(state)?
            .index("comments")
            .add_documents(&[doc], Some("id"))
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Index failed: {}", e)))?;

        Ok(())
    }.await;

    if let Err(e) = result {
        tracing::warn!("Failed to index comment {}: {:?}", comment_id, e);
    }
}

pub async fn delete_from_index(state: &AppState, index: &str, id: Uuid) {
    if let Ok(c) = meili(state) {
        let _ = c.index(index).delete_document(id.to_string()).await;
    }
}