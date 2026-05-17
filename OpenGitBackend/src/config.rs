use anyhow::Result;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub valkey_url: String,
    pub minio_endpoint: String,
    pub minio_access_key: String,
    pub minio_secret_key: String,
    pub meilisearch_url: String,
    pub meilisearch_key: String,
    pub jwt_secret: String,
    pub server_host: String,
    pub server_port: u16,
}

impl Config {
    pub fn load() -> Result<Self> {
        Ok(Self {
            database_url: require_env("DATABASE_URL")?,
            valkey_url: require_env("VALKEY_URL")?,
            minio_endpoint: require_env("MINIO_ENDPOINT")?,
            minio_access_key: require_env("MINIO_ROOT_USER")?,
            minio_secret_key: require_env("MINIO_ROOT_PASSWORD")?,
            meilisearch_url: require_env("MEILISEARCH_URL")?,
            meilisearch_key: require_env("MEILI_MASTER_KEY")?,
            jwt_secret: require_env("JWT_SECRET")?,
            server_host: std::env::var("SERVER_HOST")
                .unwrap_or_else(|_| "0.0.0.0".to_string()),
            server_port: std::env::var("SERVER_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()?,
        })
    }
}

fn require_env(key: &str) -> Result<String> {
    std::env::var(key).map_err(|_| anyhow::anyhow!("Missing required env var: {}", key))
}