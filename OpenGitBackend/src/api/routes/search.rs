use crate::{error::AppError, state::AppState};
use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};
use meilisearch_sdk::client::Client;
use serde::Deserialize;
use serde_json::json;

fn meili(state: &AppState) -> Result<Client, AppError> {
    Client::new(&state.config.meilisearch_url, Some(&state.config.meilisearch_key))
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Meilisearch client: {}", e)))
}

#[derive(Debug, Deserialize)]
pub struct SearchParams {
    pub q:        String,
    pub page:     Option<usize>,
    pub per_page: Option<usize>,
    pub filter:   Option<String>,
}

pub async fn unified_search(
    State(state):  State<AppState>,
    Query(params): Query<SearchParams>,
) -> Result<impl IntoResponse, AppError> {
    if params.q.trim().is_empty() {
        return Err(AppError::BadRequest("Search query is required".into()));
    }

    // Clone what we need so each async block owns its data
    let q = params.q.clone();

    let url = state.config.meilisearch_url.clone();
    let key = state.config.meilisearch_key.clone();

    // Each block creates its own client + index + search — no borrows cross await
    let repos_fut = {
        let url = url.clone();
        let key = key.clone();
        let q   = q.clone();
        async move {
            let c   = Client::new(&url, Some(&key)).ok()?;
            let idx = c.index("repos");
            let mut s = idx.search();
            s.with_query(&q);
            s.with_limit(5);
            s.execute::<serde_json::Value>().await.ok()
        }
    };

    let issues_fut = {
        let url = url.clone();
        let key = key.clone();
        let q   = q.clone();
        async move {
            let c   = Client::new(&url, Some(&key)).ok()?;
            let idx = c.index("issues");
            let mut s = idx.search();
            s.with_query(&q);
            s.with_limit(5);
            s.execute::<serde_json::Value>().await.ok()
        }
    };

    let prs_fut = {
        let url = url.clone();
        let key = key.clone();
        let q   = q.clone();
        async move {
            let c   = Client::new(&url, Some(&key)).ok()?;
            let idx = c.index("pull_requests");
            let mut s = idx.search();
            s.with_query(&q);
            s.with_limit(5);
            s.execute::<serde_json::Value>().await.ok()
        }
    };

    let users_fut = {
        let url = url.clone();
        let key = key.clone();
        let q   = q.clone();
        async move {
            let c   = Client::new(&url, Some(&key)).ok()?;
            let idx = c.index("users");
            let mut s = idx.search();
            s.with_query(&q);
            s.with_limit(5);
            s.execute::<serde_json::Value>().await.ok()
        }
    };

    let (repos, issues, prs, users) = tokio::join!(
        repos_fut,
        issues_fut,
        prs_fut,
        users_fut,
    );

    Ok(Json(json!({
        "query": params.q,
        "results": {
            "repositories":  repos.map(|r| r.hits).unwrap_or_default(),
            "issues":        issues.map(|r| r.hits).unwrap_or_default(),
            "pull_requests": prs.map(|r| r.hits).unwrap_or_default(),
            "users":         users.map(|r| r.hits).unwrap_or_default(),
        }
    })))
}

pub async fn search_repos_meili(
    State(state):  State<AppState>,
    Query(params): Query<SearchParams>,
) -> Result<impl IntoResponse, AppError> {
    if params.q.trim().is_empty() {
        return Err(AppError::BadRequest("Search query is required".into()));
    }

    let c      = meili(&state)?;
    let limit  = params.per_page.unwrap_or(30).min(100);
    let offset = (params.page.unwrap_or(1) - 1) * limit;

    let idx = c.index("repos");
    let mut s = idx.search();
    s.with_query(&params.q);
    s.with_limit(limit);
    s.with_offset(offset);
    if let Some(ref f) = params.filter { s.with_filter(f.as_str()); }

    let results = s.execute::<serde_json::Value>()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Search failed: {}", e)))?;

    Ok(Json(json!({
        "query":        params.q,
        "total":        results.estimated_total_hits,
        "page":         params.page.unwrap_or(1),
        "per_page":     limit,
        "repositories": results.hits,
    })))
}

pub async fn search_issues_meili(
    State(state):  State<AppState>,
    Query(params): Query<SearchParams>,
) -> Result<impl IntoResponse, AppError> {
    if params.q.trim().is_empty() {
        return Err(AppError::BadRequest("Search query is required".into()));
    }

    let c      = meili(&state)?;
    let limit  = params.per_page.unwrap_or(30).min(100);
    let offset = (params.page.unwrap_or(1) - 1) * limit;

    let idx = c.index("issues");
    let mut s = idx.search();
    s.with_query(&params.q);
    s.with_limit(limit);
    s.with_offset(offset);
    if let Some(ref f) = params.filter { s.with_filter(f.as_str()); }

    let results = s.execute::<serde_json::Value>()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Search failed: {}", e)))?;

    Ok(Json(json!({
        "query":    params.q,
        "total":    results.estimated_total_hits,
        "page":     params.page.unwrap_or(1),
        "per_page": limit,
        "issues":   results.hits,
    })))
}

pub async fn search_prs_meili(
    State(state):  State<AppState>,
    Query(params): Query<SearchParams>,
) -> Result<impl IntoResponse, AppError> {
    if params.q.trim().is_empty() {
        return Err(AppError::BadRequest("Search query is required".into()));
    }

    let c      = meili(&state)?;
    let limit  = params.per_page.unwrap_or(30).min(100);
    let offset = (params.page.unwrap_or(1) - 1) * limit;

    let idx = c.index("pull_requests");
    let mut s = idx.search();
    s.with_query(&params.q);
    s.with_limit(limit);
    s.with_offset(offset);
    if let Some(ref f) = params.filter { s.with_filter(f.as_str()); }

    let results = s.execute::<serde_json::Value>()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Search failed: {}", e)))?;

    Ok(Json(json!({
        "query":         params.q,
        "total":         results.estimated_total_hits,
        "page":          params.page.unwrap_or(1),
        "per_page":      limit,
        "pull_requests": results.hits,
    })))
}

pub async fn search_users_meili(
    State(state):  State<AppState>,
    Query(params): Query<SearchParams>,
) -> Result<impl IntoResponse, AppError> {
    if params.q.trim().is_empty() {
        return Err(AppError::BadRequest("Search query is required".into()));
    }

    let c      = meili(&state)?;
    let limit  = params.per_page.unwrap_or(30).min(100);
    let offset = (params.page.unwrap_or(1) - 1) * limit;

    let idx = c.index("users");
    let results = idx.search()
        .with_query(&params.q)
        .with_limit(limit)
        .with_offset(offset)
        .execute::<serde_json::Value>()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Search failed: {}", e)))?;

    Ok(Json(json!({
        "query":    params.q,
        "total":    results.estimated_total_hits,
        "page":     params.page.unwrap_or(1),
        "per_page": limit,
        "users":    results.hits,
    })))
}

pub async fn search_comments_meili(
    State(state):  State<AppState>,
    Query(params): Query<SearchParams>,
) -> Result<impl IntoResponse, AppError> {
    if params.q.trim().is_empty() {
        return Err(AppError::BadRequest("Search query is required".into()));
    }

    let c      = meili(&state)?;
    let limit  = params.per_page.unwrap_or(30).min(100);
    let offset = (params.page.unwrap_or(1) - 1) * limit;

    let idx = c.index("comments");
    let mut s = idx.search();
    s.with_query(&params.q);
    s.with_limit(limit);
    s.with_offset(offset);
    if let Some(ref f) = params.filter { s.with_filter(f.as_str()); }

    let results = s.execute::<serde_json::Value>()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Search failed: {}", e)))?;

    Ok(Json(json!({
        "query":    params.q,
        "total":    results.estimated_total_hits,
        "page":     params.page.unwrap_or(1),
        "per_page": limit,
        "comments": results.hits,
    })))
}

pub async fn reindex_all(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let s = state.clone();

    tokio::spawn(async move {
        tracing::info!("Starting full reindex...");

        let repo_ids: Vec<(uuid::Uuid,)> = sqlx::query_as(
            "SELECT id FROM repositories WHERE visibility = 'public'::repo_visibility"
        )
            .fetch_all(&s.db)
            .await
            .unwrap_or_default();

        for (id,) in repo_ids {
            crate::search::indexer::index_repo(&s, id).await;
        }

        let issue_ids: Vec<(uuid::Uuid,)> = sqlx::query_as("SELECT id FROM issues")
            .fetch_all(&s.db)
            .await
            .unwrap_or_default();

        for (id,) in issue_ids {
            crate::search::indexer::index_issue(&s, id).await;
        }

        let pr_ids: Vec<(uuid::Uuid,)> = sqlx::query_as("SELECT id FROM pull_requests")
            .fetch_all(&s.db)
            .await
            .unwrap_or_default();

        for (id,) in pr_ids {
            crate::search::indexer::index_pr(&s, id).await;
        }

        let user_ids: Vec<(uuid::Uuid,)> = sqlx::query_as(
            "SELECT id FROM users WHERE is_active = true AND profile_private = false"
        )
            .fetch_all(&s.db)
            .await
            .unwrap_or_default();

        for (id,) in user_ids {
            crate::search::indexer::index_user(&s, id).await;
        }

        tracing::info!("Full reindex complete");
    });

    Ok(Json(json!({ "message": "Reindex started in background" })))
}