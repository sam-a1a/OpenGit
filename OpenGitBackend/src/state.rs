use crate::config::Config;
use redis::aio::ConnectionManager;
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub cache: ConnectionManager,
    pub config: Arc<Config>,
}

impl AppState {
    pub fn new(db: PgPool, cache: ConnectionManager, config: Config) -> Self {
        Self {
            db,
            cache,
            config: Arc::new(config),
        }
    }
}