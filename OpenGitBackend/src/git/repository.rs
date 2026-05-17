use crate::error::AppError;
use std::path::{Path, PathBuf};

pub fn repo_path(base_dir: &str, git_path: &str) -> PathBuf {
    Path::new(base_dir).join(git_path)
}

pub fn init_bare(path: &PathBuf) -> Result<(), AppError> {
    if path.exists() {
        return Ok(());
    }

    std::fs::create_dir_all(path)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to create repo dir: {}", e)))?;

    gix::init_bare(path)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to init bare repo: {}", e)))?;

    Ok(())
}

pub fn open(path: &PathBuf) -> Result<gix::Repository, AppError> {
    gix::open(path)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to open repo: {}", e)))
}

pub fn exists(path: &PathBuf) -> bool {
    path.exists() && path.join("HEAD").exists()
}