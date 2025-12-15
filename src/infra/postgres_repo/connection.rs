//! Helpers to create/configure the Postgres pool.
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    PgPool,
};

use crate::domain::model::PostgresConfig;

pub async fn create_pool(cfg: &PostgresConfig) -> Result<PgPool, String> {
    let opts = PgConnectOptions::new()
        .host(&cfg.host)
        .port(cfg.port)
        .username(&cfg.user)
        .password(&cfg.password)
        .database(&cfg.database);

    PgPoolOptions::new()
        .max_connections(10)
        .connect_with(opts)
        .await
        .map_err(|e| format!("postgres connect error: {e}"))
}
