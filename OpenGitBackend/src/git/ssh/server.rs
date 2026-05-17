use crate::{db::queries::users, state::AppState};
use async_trait::async_trait;
use russh::{server::*, Channel, ChannelId, CryptoVec, MethodSet};
use russh_keys::key::PublicKey;
use std::{net::SocketAddr, path::PathBuf, sync::Arc};

#[derive(Clone)]
pub struct SshServer {
    pub app_state: AppState,
}

pub struct SshSession {
    pub app_state: AppState,
    pub user_id:   Option<uuid::Uuid>,
    pub repo_path: Option<PathBuf>,
}

impl Server for SshServer {
    type Handler = SshSession;

    fn new_client(&mut self, _addr: Option<SocketAddr>) -> Self::Handler {
        SshSession {
            app_state: self.app_state.clone(),
            user_id:   None,
            repo_path: None,
        }
    }
}

#[async_trait]
impl Handler for SshSession {
    type Error = anyhow::Error;

    async fn auth_publickey(
        &mut self,
        _user: &str,
        key: &PublicKey,
    ) -> Result<Auth, Self::Error> {
        let fingerprint = key.fingerprint();
        let fp_str      = fingerprint.to_string();
        let prefix      = &fp_str[..16.min(fp_str.len())];

        let result = sqlx::query_as::<_, (uuid::Uuid,)>(
            "SELECT user_id FROM user_ssh_keys WHERE fingerprint LIKE $1"
        )
            .bind(format!("%{}%", prefix))
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

        match users::find_by_username(&self.app_state.db, username).await {
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
        _channel: Channel<Msg>,
        _session:  &mut Session,
    ) -> Result<bool, Self::Error> {
        Ok(true)
    }

    async fn exec_request(
        &mut self,
        channel: ChannelId,
        data:    &[u8],
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        let command = String::from_utf8_lossy(data).to_string();
        tracing::info!("SSH exec: {}", command);

        let (git_cmd, repo_arg) = match parse_git_command(&command) {
            Ok(v)  => v,
            Err(e) => {
                tracing::error!("SSH parse error: {}", e);
                session.exit_status_request(channel, 1);
                session.close(channel);
                return Ok(());
            }
        };

        let (owner_name, repo_name) = match parse_repo_arg(&repo_arg) {
            Ok(v)  => v,
            Err(e) => {
                tracing::error!("SSH repo arg error: {}", e);
                session.exit_status_request(channel, 1);
                session.close(channel);
                return Ok(());
            }
        };

        let owner = match users::find_by_username(&self.app_state.db, &owner_name).await {
            Ok(Some(u)) => u,
            _ => {
                session.exit_status_request(channel, 1);
                session.close(channel);
                return Ok(());
            }
        };

        let repo = match crate::db::queries::repos::find_by_owner_and_name(
            &self.app_state.db, owner.id, &repo_name,
        ).await {
            Ok(Some(r)) => r,
            _ => {
                session.exit_status_request(channel, 1);
                session.close(channel);
                return Ok(());
            }
        };

        let path = crate::git::repository::repo_path(
            &self.app_state.config.git_base_dir,
            &repo.git_path,
        );

        self.repo_path = Some(path.clone());

        let git_args: &[&str] = match git_cmd.as_str() {
            "git-upload-pack"  => &["upload-pack",  "."],
            "git-receive-pack" => &["receive-pack", "."],
            _ => {
                session.exit_status_request(channel, 1);
                session.close(channel);
                return Ok(());
            }
        };

        let output = tokio::process::Command::new("git")
            .args(git_args)
            .current_dir(&path)
            .output()
            .await?;

        session.data(channel, CryptoVec::from(output.stdout));
        session.exit_status_request(channel, 0);
        session.close(channel);

        Ok(())
    }
}

fn parse_git_command(command: &str) -> anyhow::Result<(String, String)> {
    let parts: Vec<&str> = command.splitn(2, ' ').collect();
    if parts.len() != 2 {
        return Err(anyhow::anyhow!("Invalid git command: {}", command));
    }
    let cmd      = parts[0].trim().to_string();
    let repo_arg = parts[1].trim().trim_matches('\'').to_string();
    Ok((cmd, repo_arg))
}

fn parse_repo_arg(arg: &str) -> anyhow::Result<(String, String)> {
    let arg   = arg.trim_start_matches('/');
    let parts: Vec<&str> = arg.splitn(2, '/').collect();
    if parts.len() != 2 {
        return Err(anyhow::anyhow!("Invalid repo path: {}", arg));
    }
    let owner = parts[0].to_string();
    let repo  = parts[1].trim_end_matches(".git").to_string();
    Ok((owner, repo))
}

pub async fn start(app_state: AppState, port: u16) -> anyhow::Result<()> {
    let key = russh_keys::key::KeyPair::generate_ed25519()
        .ok_or_else(|| anyhow::anyhow!("Failed to generate SSH host key"))?;

    let config = Arc::new(russh::server::Config {
        inactivity_timeout:  Some(std::time::Duration::from_secs(3600)),
        auth_rejection_time: std::time::Duration::from_secs(1),
        keys:                vec![key],
        ..Default::default()
    });

    let mut server = SshServer { app_state };

    tracing::info!("SSH server listening on 0.0.0.0:{}", port);

    server.run_on_address(config, ("0.0.0.0", port)).await
        .map_err(|e| anyhow::anyhow!("SSH server error: {}", e))?;

    Ok(())
}