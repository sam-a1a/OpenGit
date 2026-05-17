use crate::error::AppError;
use axum::body::Bytes;
use std::path::PathBuf;
use tokio::process::Command;

// Upload pack (clone / fetch)

pub async fn upload_pack_info_refs(path: &PathBuf) -> Result<Vec<u8>, AppError> {
    let output = Command::new("git")
        .args(["upload-pack", "--stateless-rpc", "--advertise-refs", "."])
        .current_dir(path)
        .output()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("git upload-pack failed: {}", e)))?;

    if !output.status.success() {
        return Err(AppError::Internal(anyhow::anyhow!(
            "git upload-pack error: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    Ok(output.stdout)
}

pub async fn upload_pack(path: &PathBuf, body: Bytes) -> Result<Vec<u8>, AppError> {
    let mut child = Command::new("git")
        .args(["upload-pack", "--stateless-rpc", "."])
        .current_dir(path)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| AppError::Internal(anyhow::anyhow!("git upload-pack spawn failed: {}", e)))?;

    if let Some(stdin) = child.stdin.take() {
        use tokio::io::AsyncWriteExt;
        let mut stdin = stdin;
        stdin.write_all(&body).await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("stdin write failed: {}", e)))?;
    }

    let output = child.wait_with_output().await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("git upload-pack wait failed: {}", e)))?;

    Ok(output.stdout)
}

// Receive pack (push)

pub async fn receive_pack_info_refs(path: &PathBuf) -> Result<Vec<u8>, AppError> {
    let output = Command::new("git")
        .args(["receive-pack", "--stateless-rpc", "--advertise-refs", "."])
        .current_dir(path)
        .output()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("git receive-pack failed: {}", e)))?;

    if !output.status.success() {
        return Err(AppError::Internal(anyhow::anyhow!(
            "git receive-pack error: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    Ok(output.stdout)
}

pub async fn receive_pack(path: &PathBuf, body: Bytes) -> Result<Vec<u8>, AppError> {
    let mut child = Command::new("git")
        .args(["receive-pack", "--stateless-rpc", "."])
        .current_dir(path)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| AppError::Internal(anyhow::anyhow!("git receive-pack spawn failed: {}", e)))?;

    if let Some(stdin) = child.stdin.take() {
        use tokio::io::AsyncWriteExt;
        let mut stdin = stdin;
        stdin.write_all(&body).await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("stdin write failed: {}", e)))?;
    }

    let output = child.wait_with_output().await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("git receive-pack wait failed: {}", e)))?;

    Ok(output.stdout)
}

// PKT-LINE framing (required for info/refs response)

pub fn pkt_line(data: &str) -> Vec<u8> {
    let length = data.len() + 4;
    format!("{:04x}{}", length, data).into_bytes()
}

pub fn prefix_info_refs(service: &str, data: Vec<u8>) -> Vec<u8> {
    let mut result = Vec::new();
    result.extend(pkt_line(&format!("# service={}\n", service)));
    result.extend(b"0000"); // flush packet
    result.extend(data);
    result
}