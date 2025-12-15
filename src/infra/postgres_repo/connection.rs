//! Helpers to create/configure the Postgres pool.
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    PgPool,
};

use crate::domain::model::PostgresConfig;

pub async fn create_pool(cfg: &PostgresConfig) -> Result<PgPool, String> {
    let opts = connect_options(cfg, Some(&cfg.database));
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect_with(opts.clone())
        .await;

    match pool {
        Ok(p) => Ok(p),
        Err(_e) => {
            ensure_database_exists(cfg).await?;
            PgPoolOptions::new()
                .max_connections(10)
                .connect_with(opts)
                .await
                .map_err(|e| format!("postgres connect error after create: {e}"))
        }
    }
}

fn connect_options(cfg: &PostgresConfig, database: Option<&str>) -> PgConnectOptions {
    let mut opts = PgConnectOptions::new()
        .host(&cfg.host)
        .port(cfg.port)
        .username(&cfg.user)
        .password(&cfg.password);
    if let Some(db) = database {
        opts = opts.database(db);
    }
    opts
}

async fn ensure_database_exists(cfg: &PostgresConfig) -> Result<(), String> {
    validate_db_name(&cfg.database)?;
    let admin_opts = connect_options(cfg, Some("postgres"));
    let admin_pool = PgPoolOptions::new()
        .max_connections(2)
        .connect_with(admin_opts)
        .await
        .map_err(|e| format!("postgres connect error (admin db): {e}"))?;

    let create_sql = format!("CREATE DATABASE \"{}\";", &cfg.database);
    let res = sqlx::query(&create_sql).execute(&admin_pool).await;
    match res {
        Ok(_) => Ok(()),
        Err(e) if is_duplicate_db_error(&e) => Ok(()),
        Err(e) => Err(format!("postgres create database error: {e}")),
    }
}

fn validate_db_name(name: &str) -> Result<(), String> {
    if name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        Ok(())
    } else {
        Err(format!(
            "invalid postgres database name '{}': only alphanumeric, '_' and '-' allowed",
            name
        ))
    }
}

fn is_duplicate_db_error(e: &sqlx::Error) -> bool {
    matches!(e, sqlx::Error::Database(db_err) if db_err.code().as_deref() == Some("42P04"))
}
