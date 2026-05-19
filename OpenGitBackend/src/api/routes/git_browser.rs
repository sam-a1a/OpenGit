use crate::{
    db::queries::{repos, users},
    error::AppError,
    git::repository,
    state::AppState,
};
use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::process::Command;

// run git command, return stdout
async fn git(path: &PathBuf, args: &[&str]) -> Result<String, AppError> {
    let output = Command::new("git")
        .args(args)
        .current_dir(path)
        .output()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("git failed: {}", e)))?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(AppError::BadRequest(format!("git: {}", err.trim())));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

// resolve owner+repo to git path
async fn resolve_repo_path(
    state:     &AppState,
    owner:     &str,
    repo_name: &str,
) -> Result<PathBuf, AppError> {
    let owner = users::find_by_username(&state.db, owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    let repo = repos::find_by_owner_and_name(&state.db, owner.id, repo_name)
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    Ok(repository::repo_path(&state.config.git_base_dir, &repo.git_path))
}

// Response types

#[derive(Debug, Serialize)]
pub struct RefInfo {
    pub name:     String,
    pub sha:      String,
    pub ref_type: String,
}

#[derive(Debug, Serialize)]
pub struct CommitInfo {
    pub sha:             String,
    pub message:         String,
    pub author_name:     String,
    pub author_email:    String,
    pub authored_at:     String,
    pub committer_name:  String,
    pub committer_email: String,
    pub committed_at:    String,
    pub parents:         Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct TreeEntry {
    pub mode:       String,
    pub entry_type: String,
    pub sha:        String,
    pub name:       String,
    pub path:       String,
}

#[derive(Debug, Serialize)]
pub struct BlobContent {
    pub path:      String,
    pub sha:       String,
    pub content:   String,
    pub size:      usize,
    pub is_binary: bool,
    pub encoding:  String,
}

#[derive(Debug, Serialize)]
pub struct DiffFile {
    pub path:      String,
    pub additions: i64,
    pub deletions: i64,
    pub status:    String,
}

#[derive(Debug, Serialize)]
pub struct DiffResult {
    pub base:          String,
    pub head:          String,
    pub files_changed: usize,
    pub additions:     i64,
    pub deletions:     i64,
    pub files:         Vec<DiffFile>,
    pub patch:         String,
}

#[derive(Debug, Serialize)]
pub struct BlameLine {
    pub line_number:  usize,
    pub sha:          String,
    pub author_name:  String,
    pub author_email: String,
    pub authored_at:  String,
    pub content:      String,
}

#[derive(Debug, Serialize)]
pub struct RepoStats {
    pub default_branch:  String,
    pub branch_count:    usize,
    pub tag_count:       usize,
    pub commit_count:    usize,
    pub last_commit:     Option<CommitInfo>,
    pub languages:       Vec<LanguageStat>,
}

#[derive(Debug, Serialize)]
pub struct LanguageStat {
    pub language: String,
    pub bytes:    usize,
    pub percent:  f64,
}

// Query params

#[derive(Debug, Deserialize)]
pub struct PathQuery {
    pub path: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CommitQuery {
    pub page:     Option<i64>,
    pub per_page: Option<i64>,
    pub path:     Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DiffQuery {
    pub base: String,
    pub head: String,
}

// GET /git/refs - list all branches and tags

pub async fn list_refs(
    State(state): State<AppState>,
    Path((owner, repo_name)): Path<(String, String)>,
) -> Result<impl IntoResponse, AppError> {
    let path = resolve_repo_path(&state, &owner, &repo_name).await?;

    let output = git(
        &path,
        &["for-each-ref",
            "refs/heads", "refs/tags",
            "--format=%(refname:short)|%(objectname)|%(objecttype)"],
    ).await?;

    let mut branches = Vec::new();
    let mut tags     = Vec::new();

    for line in output.lines().filter(|l| !l.is_empty()) {
        let parts: Vec<&str> = line.splitn(3, '|').collect();
        if parts.len() != 3 { continue; }

        let info = RefInfo {
            name:     parts[0].to_string(),
            sha:      parts[1].to_string(),
            ref_type: if parts[2] == "commit" { "branch" } else { "tag" }.to_string(),
        };

        if line.contains("refs/tags") || parts[2] == "tag" {
            tags.push(info);
        } else {
            branches.push(info);
        }
    }

    Ok(Json(serde_json::json!({
        "branches": branches,
        "tags":     tags,
    })))
}

// GET /git/commits/{ref} - list commits

pub async fn list_commits(
    State(state):   State<AppState>,
    Path((owner, repo_name, git_ref)): Path<(String, String, String)>,
    Query(params):  Query<CommitQuery>,
) -> Result<impl IntoResponse, AppError> {
    let path     = resolve_repo_path(&state, &owner, &repo_name).await?;
    let per_page = params.per_page.unwrap_or(30).min(100);
    let skip     = (params.page.unwrap_or(1) - 1) * per_page;

    let sep = "|DELIM|";
    let fmt = format!(
        "%H{sep}%s{sep}%aN{sep}%aE{sep}%aI{sep}%cN{sep}%cE{sep}%cI{sep}%P",
        sep = sep
    );

    let mut git_args: Vec<String> = vec![
        "log".to_string(),
        git_ref.clone(),
        format!("--format={}", fmt),
        "-n".to_string(),
        format!("{}", per_page),
        format!("--skip={}", skip),
    ];

    if let Some(ref p) = params.path {
        git_args.push("--".to_string());
        git_args.push(p.clone());
    }

    let args_refs: Vec<&str> = git_args.iter().map(|s| s.as_str()).collect();
    let output = git(&path, &args_refs).await?;

    let commits: Vec<CommitInfo> = output
        .lines()
        .filter(|l| !l.is_empty())
        .filter_map(|line| parse_commit_line(line, sep))
        .collect();

    Ok(Json(serde_json::json!({
        "commits":  commits,
        "ref":      git_ref,
        "page":     params.page.unwrap_or(1),
        "per_page": per_page,
    })))
}

fn parse_commit_line(line: &str, sep: &str) -> Option<CommitInfo> {
    let parts: Vec<&str> = line.splitn(9, sep).collect();
    if parts.len() < 9 { return None; }

    Some(CommitInfo {
        sha:             parts[0].to_string(),
        message:         parts[1].to_string(),
        author_name:     parts[2].to_string(),
        author_email:    parts[3].to_string(),
        authored_at:     parts[4].to_string(),
        committer_name:  parts[5].to_string(),
        committer_email: parts[6].to_string(),
        committed_at:    parts[7].to_string(),
        parents:         parts[8].split_whitespace()
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect(),
    })
}

// GET /git/commits/{ref}/single - single commit

pub async fn get_commit(
    State(state):   State<AppState>,
    Path((owner, repo_name, sha)): Path<(String, String, String)>,
) -> Result<impl IntoResponse, AppError> {
    let path = resolve_repo_path(&state, &owner, &repo_name).await?;
    let sep  = "|DELIM|";
    let fmt  = format!(
        "%H{sep}%B{sep}%aN{sep}%aE{sep}%aI{sep}%cN{sep}%cE{sep}%cI{sep}%P",
        sep = sep
    );

    let output = git(&path, &["log", "-1", &format!("--format={}", fmt), &sha]).await?;

    let commit = output
        .lines()
        .find(|l| !l.is_empty())
        .and_then(|line| parse_commit_line(line, sep))
        .ok_or(AppError::NotFound("Commit".into()))?;

    // also get stats for this commit
    let stats = git(&path, &["diff-tree", "--stat", "-r", "--no-commit-id", &sha])
        .await
        .unwrap_or_default();

    Ok(Json(serde_json::json!({
        "commit": commit,
        "stats":  stats.trim(),
    })))
}

// GET /git/tree/{ref} - file tree

pub async fn get_tree(
    State(state):   State<AppState>,
    Path((owner, repo_name, git_ref)): Path<(String, String, String)>,
    Query(params):  Query<PathQuery>,
) -> Result<impl IntoResponse, AppError> {
    let path     = resolve_repo_path(&state, &owner, &repo_name).await?;
    let sub_path = params.path.clone().unwrap_or_default();

    let tree_ref = if sub_path.is_empty() {
        git_ref.clone()
    } else {
        format!("{}:{}", git_ref, sub_path)
    };

    let output = git(&path, &["ls-tree", &tree_ref]).await?;

    let entries: Vec<TreeEntry> = output
        .lines()
        .filter(|l| !l.is_empty())
        .filter_map(|line| parse_tree_entry(line, &sub_path))
        .collect();

    Ok(Json(serde_json::json!({
        "ref":     git_ref,
        "path":    sub_path,
        "entries": entries,
    })))
}

fn parse_tree_entry(line: &str, base_path: &str) -> Option<TreeEntry> {
    // format: <mode> <type> <sha>\t<name>
    let tab_pos = line.find('\t')?;
    let meta    = &line[..tab_pos];
    let name    = line[tab_pos + 1..].to_string();

    let meta_parts: Vec<&str> = meta.splitn(3, ' ').collect();
    if meta_parts.len() < 3 { return None; }

    let full_path = if base_path.is_empty() {
        name.clone()
    } else {
        format!("{}/{}", base_path, name)
    };

    Some(TreeEntry {
        mode:       meta_parts[0].to_string(),
        entry_type: meta_parts[1].to_string(),
        sha:        meta_parts[2].to_string(),
        name,
        path:       full_path,
    })
}

// GET /git/blob/{ref} - file content

pub async fn get_blob(
    State(state):   State<AppState>,
    Path((owner, repo_name, git_ref)): Path<(String, String, String)>,
    Query(params):  Query<PathQuery>,
) -> Result<impl IntoResponse, AppError> {
    let file_path = params.path
        .ok_or(AppError::BadRequest("path query param is required".into()))?;

    let path = resolve_repo_path(&state, &owner, &repo_name).await?;
    let spec = format!("{}:{}", git_ref, file_path);

    // get sha of the blob
    let sha = git(&path, &["rev-parse", &spec])
        .await
        .unwrap_or_default()
        .trim()
        .to_string();

    // get raw content
    let output = Command::new("git")
        .args(["show", &spec])
        .current_dir(&path)
        .output()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("git show failed: {}", e)))?;

    if !output.status.success() {
        return Err(AppError::NotFound("File".into()));
    }

    let raw     = &output.stdout;
    let size    = raw.len();
    let is_binary = raw.iter().take(8000).any(|&b| b == 0);

    let (content, encoding) = if is_binary {
        (base64_encode(raw), "base64".to_string())
    } else {
        (String::from_utf8_lossy(raw).to_string(), "utf-8".to_string())
    };

    Ok(Json(BlobContent {
        path: file_path,
        sha,
        content,
        size,
        is_binary,
        encoding,
    }))
}

fn base64_encode(data: &[u8]) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut i = 0;
    while i < data.len() {
        let b0 = data[i] as usize;
        let b1 = if i + 1 < data.len() { data[i + 1] as usize } else { 0 };
        let b2 = if i + 2 < data.len() { data[i + 2] as usize } else { 0 };
        out.push(CHARS[(b0 >> 2)] as char);
        out.push(CHARS[((b0 & 3) << 4) | (b1 >> 4)] as char);
        out.push(if i + 1 < data.len() { CHARS[((b1 & 15) << 2) | (b2 >> 6)] as char } else { '=' });
        out.push(if i + 2 < data.len() { CHARS[b2 & 63] as char } else { '=' });
        i += 3;
    }
    out
}

// GET /git/diff - diff between refs

pub async fn get_diff(
    State(state):   State<AppState>,
    Path((owner, repo_name)): Path<(String, String)>,
    Query(params):  Query<DiffQuery>,
) -> Result<impl IntoResponse, AppError> {
    let path      = resolve_repo_path(&state, &owner, &repo_name).await?;
    let range     = format!("{}..{}", params.base, params.head);

    // get patch
    let patch = git(&path, &["diff", &range]).await.unwrap_or_default();

    // get numstat
    let numstat = git(&path, &["diff", "--numstat", &range]).await.unwrap_or_default();

    let mut files         = Vec::new();
    let mut total_add     = 0i64;
    let mut total_del     = 0i64;

    for line in numstat.lines().filter(|l| !l.is_empty()) {
        let parts: Vec<&str> = line.splitn(3, '\t').collect();
        if parts.len() < 3 { continue; }

        let additions = parts[0].parse::<i64>().unwrap_or(0);
        let deletions = parts[1].parse::<i64>().unwrap_or(0);
        total_add += additions;
        total_del += deletions;

        files.push(DiffFile {
            path:      parts[2].to_string(),
            additions,
            deletions,
            status:    "modified".to_string(),
        });
    }

    Ok(Json(DiffResult {
        base:          params.base.clone(),
        head:          params.head.clone(),
        files_changed: files.len(),
        additions:     total_add,
        deletions:     total_del,
        files,
        patch,
    }))
}

// GET /git/blame/{ref} - blame a file

pub async fn get_blame(
    State(state):   State<AppState>,
    Path((owner, repo_name, git_ref)): Path<(String, String, String)>,
    Query(params):  Query<PathQuery>,
) -> Result<impl IntoResponse, AppError> {
    let file_path = params.path
        .ok_or(AppError::BadRequest("path query param is required".into()))?;

    let path   = resolve_repo_path(&state, &owner, &repo_name).await?;
    let output = git(&path, &["blame", "--line-porcelain", &git_ref, "--", &file_path]).await?;

    let lines = parse_blame_porcelain(&output);

    Ok(Json(serde_json::json!({
        "ref":   git_ref,
        "path":  file_path,
        "lines": lines,
    })))
}

fn parse_blame_porcelain(output: &str) -> Vec<BlameLine> {
    let mut result     = Vec::new();
    let mut line_num   = 0usize;
    let mut cur_sha    = String::new();
    let mut cur_author = String::new();
    let mut cur_email  = String::new();
    let mut cur_time   = String::new();

    for line in output.lines() {
        if line.starts_with('\t') {
            // content line
            line_num += 1;
            result.push(BlameLine {
                line_number:  line_num,
                sha:          cur_sha[..8.min(cur_sha.len())].to_string(),
                author_name:  cur_author.clone(),
                author_email: cur_email.clone(),
                authored_at:  cur_time.clone(),
                content:      line[1..].to_string(),
            });
        } else if line.len() >= 40 && line.chars().take(40).all(|c| c.is_ascii_hexdigit()) {
            cur_sha = line[..40].to_string();
        } else if let Some(val) = line.strip_prefix("author ") {
            cur_author = val.to_string();
        } else if let Some(val) = line.strip_prefix("author-mail ") {
            cur_email = val.trim_matches(|c| c == '<' || c == '>').to_string();
        } else if let Some(val) = line.strip_prefix("author-time ") {
            cur_time = val.to_string();
        }
    }

    result
}

// GET /git/stats - repo overview stats

pub async fn get_stats(
    State(state):   State<AppState>,
    Path((owner, repo_name)): Path<(String, String)>,
) -> Result<impl IntoResponse, AppError> {
    let owner_rec = users::find_by_username(&state.db, &owner)
        .await?
        .ok_or(AppError::NotFound("User".into()))?;

    let repo = repos::find_by_owner_and_name(&state.db, owner_rec.id, &repo_name)
        .await?
        .ok_or(AppError::NotFound("Repository".into()))?;

    let path           = repository::repo_path(&state.config.git_base_dir, &repo.git_path);
    let default_branch = &repo.default_branch;

    // branch count
    let branches = git(&path, &["branch", "--list"])
        .await.unwrap_or_default();
    let branch_count = branches.lines().filter(|l| !l.is_empty()).count();

    // tag count
    let tags = git(&path, &["tag", "--list"])
        .await.unwrap_or_default();
    let tag_count = tags.lines().filter(|l| !l.is_empty()).count();

    // commit count
    let commit_count_str = git(&path, &["rev-list", "--count", default_branch])
        .await.unwrap_or_default();
    let commit_count = commit_count_str.trim().parse::<usize>().unwrap_or(0);

    // last commit
    let sep = "|DELIM|";
    let fmt = format!(
        "%H{sep}%s{sep}%aN{sep}%aE{sep}%aI{sep}%cN{sep}%cE{sep}%cI{sep}%P",
        sep = sep
    );
    let last_commit_raw = git(&path, &["log", "-1", &format!("--format={}", fmt), default_branch])
        .await.unwrap_or_default();
    let last_commit = last_commit_raw
        .lines()
        .find(|l| !l.is_empty())
        .and_then(|l| parse_commit_line(l, sep));

    Ok(Json(RepoStats {
        default_branch: default_branch.clone(),
        branch_count,
        tag_count,
        commit_count,
        last_commit,
        languages: vec![],
    }))
}

// GET /git/raw/{ref} - raw file download

pub async fn get_raw(
    State(state):   State<AppState>,
    Path((owner, repo_name, git_ref)): Path<(String, String, String)>,
    Query(params):  Query<PathQuery>,
) -> Result<impl IntoResponse, AppError> {
    let file_path = params.path
        .ok_or(AppError::BadRequest("path query param is required".into()))?;

    let path = resolve_repo_path(&state, &owner, &repo_name).await?;
    let spec = format!("{}:{}", git_ref, file_path);

    let output = Command::new("git")
        .args(["show", &spec])
        .current_dir(&path)
        .output()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("git show failed: {}", e)))?;

    if !output.status.success() {
        return Err(AppError::NotFound("File".into()));
    }

    let ext         = file_path.rsplit('.').next().unwrap_or("");
    let content_type = guess_content_type(ext);

    Ok((
        [(axum::http::header::CONTENT_TYPE, content_type)],
        output.stdout,
    ))
}

fn guess_content_type(ext: &str) -> &'static str {
    match ext {
        "html" | "htm"  => "text/html",
        "css"           => "text/css",
        "js" | "mjs"    => "application/javascript",
        "json"          => "application/json",
        "svg"           => "image/svg+xml",
        "png"           => "image/png",
        "jpg" | "jpeg"  => "image/jpeg",
        "gif"           => "image/gif",
        "pdf"           => "application/pdf",
        "md"            => "text/markdown",
        _               => "text/plain",
    }
}