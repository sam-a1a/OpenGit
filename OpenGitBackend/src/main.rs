use anyhow::Result;
use redis::Client as RedisClient;
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

mod api;
mod cache;
mod config;
mod db;
mod error;
mod git;
mod grpc;
mod jobs;
mod models;
mod search;
mod services;
mod state;
mod storage;

use config::Config;
use state::AppState;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    info!("Starting OpenGit Backend");

    let config = Config::load()?;

    info!("Connecting to PostgreSQL...");
    let db = PgPoolOptions::new()
        .max_connections(20)
        .acquire_timeout(std::time::Duration::from_secs(5))
        .connect(&config.database_url)
        .await?;

    info!("Running migrations...");
    sqlx::migrate!("./migrations").run(&db).await?;
    info!("Migrations complete");

    info!("Connecting to Valkey...");
    let redis_client = RedisClient::open(config.valkey_url.as_str())?;
    let cache = redis::aio::ConnectionManager::new(redis_client).await?;
    info!("Valkey connected");

    let state     = AppState::new(db, cache, config.clone());
    let app       = api::router::build(state.clone());
    let ssh_port  = config.ssh_port;
    let ssh_state = state.clone();

    // spawn SSH server in background
    tokio::spawn(async move {
        if let Err(e) = git::ssh::server::start(ssh_state, ssh_port).await {
            tracing::error!("SSH server error: {}", e);
        }
    });

    let addr: SocketAddr = format!("{}:{}", config.server_host, config.server_port).parse()?;
    info!("HTTP listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}