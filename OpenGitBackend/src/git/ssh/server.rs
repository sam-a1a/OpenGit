use crate::{db::queries::users, error::AppError, git::pack, state::AppState};
use async_trait::async_trait;
use russh::{server::*, MethodSet};
use russh_keys::key::PublicKey;
use std::{collections::HashMap, net::SocketAddr, path::PathBuf, sync::Arc};
use tokio::sync::Mutex;

// ── Session state ─────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct SshServer {
    pub app_state: AppState,
}

pub struct SshSession {
    pub app_state:   AppState,
    pub user_id:     Option<uuid::Uuid>,
    pub repo_path:   Option<PathBuf>,
    pub channel_id:  Option<ChannelId>,
}

// ── Server handler ────────────────────────────────────────────────────────────

impl Server for SshServer {
    type Handler = SshSession;

    fn new_client(&mut self, _addr: Option<SocketAddr>) -> Self::Handler {
        SshSession {
            app_state:  self.app_state.clone(),
            user_id:    None,
            repo_path:  None,
            channel_id: None,
        }
    }
}

// ── Session handler ───────────────────────────────────────────────────────────

#[async_trait]
impl Handler for SshSession {
    type Error = anyhow::Error;

    // allow public key and password auth methods
    async fn auth_publickey_offered(
        &mut self,
        _user: &str,
        _key: &PublicKey,
    ) -> Result<Auth, Self::Error> {
        Ok(Auth::Accept)
    }

    async fn auth_publickey(
        &mut self,
        _user: &str,
        key: &PublicKey,
    ) -> Result<Auth, Self::Error> {
        // get fingerprint from offered key
        let fingerprint = key.fingerprint();

        // look up user by SSH key fingerprint
        let result = sqlx::query_as::<_, (uuid::Uuid,)>(
            "SELECT user_id FROM user_ssh_keys WHERE fingerprint LIKE $1"
        )
            .bind(format!("%{}%", &fingerprint[..16.min(fingerprint.len())]))
            .fetch_optional(&self.app_state.db)
            .await;

        match result {
            Ok(Some((user_id,))) => {
                self.user_id = Some(user_id);
                Ok(Auth::Accept)
            }
            _ => Ok(Auth::Reject {
                proceed_with_methods: Some(MethodSet::PUBLICKEY),
            }),
        }
    }

    async fn auth_password(
        &mut self,
        username: &str,
        password: &str,
    ) -> Result<Auth, Self::Error> {
        use crate::services::auth::verify_password;

        let user = users::find_by_username(&self.app_state.db, username).await;
        match user {
            Ok(Some(u)) => {
                if let Some(hash) = &u.password_hash {
                    if verify_password(password, hash).unwrap_or(false) {
                        self.user_id = Some(u.id);
                        return Ok(Auth::Accept);
                    }
                }
                Ok(Auth::Reject {
                    proceed_with_methods: Some(MethodSet::PASSWORD | MethodSet::PUBLICKEY),
                })
            }
            _ => Ok(Auth::Reject {
                proceed_with_methods: Some(MethodSet::PASSWORD | MethodSet::PUBLICKEY),
            }),
        }
    }

    async fn channel_open_session(
        &mut self,
        channel: Channel<Msg>,
        _session: &mut Session,
    ) -> Result<bool, Self::Error> {
        self.channel_id = Some(channel.id());
        Ok(true)
    }

    async fn exec_request(
        &mut self,
        channel: ChannelId,
        data: &[u8],
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        let command = String::from_utf8_lossy(data).to_string();
        tracing::info!("SSH exec: {}", command);

        // parse git command — format: git-upload-pack 'owner/repo.git'
        let (git_cmd, repo_arg) = parse_git_command(&command)?;

        // resolve repo path
        let (owner_name, repo_name) = parse_repo_arg(&repo_arg)?;

        let owner = users::find_by_username(&self.app_state.db, &owner_name)
            .await?
            .ok_or_else(|| anyhow::anyhow!("User not found"))?;

        let repo = crate::db::queries::repos::find_by_owner_and_name(
            &self.app_state.db,
            owner.id,
            &repo_name,
        )
            .await?
            .ok_or_else(|| anyhow::anyhow!("Repository not found"))?;

        let path = crate::git::repository::repo_path(
            &self.app_state.config.git_base_dir,
            &repo.git_path,
        );

        self.repo_path = Some(path.clone());

        // run git command and stream output back
        match git_cmd.as_str() {
            "git-upload-pack" => {
                let output = tokio::process::Command::new("git")
                    .args(["upload-pack", "--stateless-rpc", "."])
                    .current_dir(&path)
                    .output()
                    .await?;

                session.data(channel, CryptoVec::from(output.stdout));
                session.exit_status_request(channel, 0);
                session.close(channel);
            }
            "git-receive-pack" => {
                let output = tokio::process::Command::new("git")
                    .args(["receive-pack", "."])
                    .current_dir(&path)
                    .output()
                    .await?;

                session.data(channel, CryptoVec::from(output.stdout));
                session.exit_status_request(channel, 0);
                session.close(channel);
            }
            _ => {
                session.exit_status_request(channel, 1);
                session.close(channel);
            }
        }

        Ok(())
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn parse_git_command(command: &str) -> anyhow::Result<(String, String)> {
    // e.g. "git-upload-pack '/sam/test-repo.git'"
    let parts: Vec<&str> = command.splitn(2, ' ').collect();
    if parts.len() != 2 {
        return Err(anyhow::anyhow!("Invalid git command: {}", command));
    }
    let cmd      = parts[0].trim().to_string();
    let repo_arg = parts[1].trim().trim_matches('\'').to_string();
    Ok((cmd, repo_arg))
}

fn parse_repo_arg(arg: &str) -> anyhow::Result<(String, String)> {
    // e.g. "/sam/test-repo.git" or "sam/test-repo.git"
    let arg = arg.trim_start_matches('/');
    let parts: Vec<&str> = arg.splitn(2, '/').collect();
    if parts.len() != 2 {
        return Err(anyhow::anyhow!("Invalid repo path: {}", arg));
    }
    let owner = parts[0].to_string();
    let repo  = parts[1].trim_end_matches(".git").to_string();
    Ok((owner, repo))
}

// ── Start SSH server ──────────────────────────────────────────────────────────

pub async fn start(app_state: AppState, port: u16) -> anyhow::Result<()> {
    let config = russh::server::Config {
        methods:         MethodSet::PASSWORD | MethodSet::PUBLICKEY,
        inactivity_timeout: Some(std::time::Duration::from_secs(3600)),
        auth_rejection_time: std::time::Duration::from_secs(1),
        keys: vec![
            russh_keys::key::KeyPair::generate_ed25519()
                .ok_or_else(|| anyhow::anyhow!("Failed to generate SSH host key"))?,
        ],
        ..Default::default()
    };

    let config = Arc::new(config);
    let server = SshServer { app_state };

    tracing::info!("SSH server listening on 0.0.0.0:{}", port);

    russh::server::run(config, ("0.0.0.0", port), server).await?;
    Ok(())
}