mod config;

use std::net::SocketAddr;
use std::path::Path;

use axum::{extract::State, response::IntoResponse, routing::get, Json, Router};
use config::{validate_schema_name, AppMode, ConfigError, ServerConfig, SqlDialect};
use sqlx::{Pool, Postgres, Sqlite};
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::EnvFilter;


#[derive(Clone)]
struct AppState {
    sqlite: Option<Pool<Sqlite>>,
    postgres: Option<Pool<Postgres>>,
    fetcher_schema: Option<String>,
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
struct FeedSummary {
    id: String,
    url: String,
    domain: String,
    category: String,
    base_poll_seconds: i64,
}

#[derive(Debug)]
struct ServerError(String);

impl IntoResponse for ServerError {
    fn into_response(self) -> axum::response::Response {
        (axum::http::StatusCode::INTERNAL_SERVER_ERROR, self.0).into_response()
    }
}
#[tokio::main]
async fn main() -> Result<(), ConfigError> {
    let config_path = std::env::var("SERVER_CONFIG_PATH")
        .unwrap_or_else(|_| "crates/server/res/config.toml".to_string());

    let config = ServerConfig::load(Path::new(&config_path)).await?;
    init_tracing(&config)?;
    if let Some(tz) = config.app.timezone.as_deref() {
        tracing::info!(timezone = tz, "server timezone configured");
    }

    tracing::info!(mode = ?config.app.mode, "server mode configured");
    tracing::info!(host = %config.http.host, port = config.http.port, "server http bind");

    let state = connect_db(&config, Path::new(&config_path)).await?;

    if config.app.mode == AppMode::Dev && config.dev.reset_on_start {
        reset_server_data(&config, &state).await?;
    }

    let addr: SocketAddr = format!("{}:{}", config.http.host, config.http.port)
        .parse()
        .map_err(|e| ConfigError::Invalid(format!("invalid http bind: {e}")))?;

    let app = Router::new()
        .route("/health", get(health))
        .route("/v1/feeds", get(list_feeds))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await.map_err(|e| {
        ConfigError::Invalid(format!("http server error: {e}"))
    })?;

    Ok(())
}

async fn health() -> &'static str {
    "ok"
}

async fn list_feeds(State(state): State<AppState>) -> Result<Json<Vec<FeedSummary>>, ServerError> {
    if let Some(pool) = &state.postgres {
        let schema = state
            .fetcher_schema
            .as_deref()
            .unwrap_or("fetcher");
        let query = format!(
            "SELECT id, url, domain, category, base_poll_seconds FROM {}.feeds ORDER BY id",
            quote_ident(schema)
        );
        let rows = sqlx::query_as::<_, FeedSummary>(&query)
            .fetch_all(pool)
            .await
            .map_err(|e| ServerError(format!("feeds query failed: {e}")))?;
        return Ok(Json(rows));
    }

    let pool = state
        .sqlite
        .as_ref()
        .ok_or_else(|| ServerError("database pool missing".into()))?;
    let rows = sqlx::query_as::<_, FeedSummary>(
        "SELECT id, url, domain, category, base_poll_seconds FROM feeds ORDER BY id",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| ServerError(format!("feeds query failed: {e}")))?;
    Ok(Json(rows))
}


fn init_tracing(config: &ServerConfig) -> Result<(), ConfigError> {
    let level = config
        .logging
        .level
        .as_deref()
        .unwrap_or("info")
        .trim()
        .to_string();
    let filter = EnvFilter::try_new(level)
        .map_err(|e| ConfigError::Invalid(format!("invalid logging.level: {e}")))?;

    tracing_subscriber::fmt().with_env_filter(filter).init();
    Ok(())
}

async fn connect_db(
    config: &ServerConfig,
    config_path: &Path,
) -> Result<AppState, ConfigError> {
    match config.dialect()? {
        SqlDialect::Sqlite => {
            let base_dir = config_path
                .parent()
                .ok_or_else(|| ConfigError::Invalid("config path has no parent".into()))?;
            let path = config.sqlite_path(base_dir);
            let url = format!("sqlite://{}", path.display());
            let pool = sqlx::SqlitePool::connect(&url)
                .await
                .map_err(|e| ConfigError::Invalid(format!("sqlite connect failed: {e}")))?;
            Ok(AppState {
                sqlite: Some(pool),
                postgres: None,
                fetcher_schema: None,
            })
        }
        SqlDialect::Postgres => {
            let pg = config
                .postgres
                .as_ref()
                .ok_or_else(|| ConfigError::Invalid("postgres section missing".into()))?;
            let schema = validate_schema_name(&pg.schema)?;
            let fetcher_schema = validate_schema_name(&pg.fetcher_schema)?;
            let url = format!(
                "postgres://{}:{}@{}:{}/{}?sslmode={}",
                pg.user, pg.password, pg.host, pg.port, pg.database, pg.ssl_mode
            );
            let pool = PgPoolOptions::new()
                .max_connections(10)
                .after_connect(set_search_path(&schema))
                .connect(&url)
                .await
                .map_err(|e| ConfigError::Invalid(format!("postgres connect failed: {e}")))?;
            Ok(AppState {
                sqlite: None,
                postgres: Some(pool),
                fetcher_schema: Some(fetcher_schema),
            })
        }
    }
}

async fn reset_server_data(
    config: &ServerConfig,
    state: &AppState,
) -> Result<(), ConfigError> {
    let tables = [
        "user_tokens",
        "favorites",
        "entry_states",
        "folder_feeds",
        "folders",
        "subscriptions",
        "users",
    ];

    match config.dialect()? {
        SqlDialect::Sqlite => {
            let pool = state.sqlite
                .as_ref()
                .ok_or_else(|| ConfigError::Invalid("sqlite pool missing".into()))?;
            for table in tables {
                let query = format!("DELETE FROM {table}");
                if let Err(e) = sqlx::query(&query).execute(pool).await {
                    if !is_missing_table_error(&e) {
                        return Err(ConfigError::Invalid(format!("cleanup {table} failed: {e}")));
                    }
                }
            }
        }
        SqlDialect::Postgres => {
            let pool = state.postgres
                .as_ref()
                .ok_or_else(|| ConfigError::Invalid("postgres pool missing".into()))?;
            let schema = config
                .postgres
                .as_ref()
                .ok_or_else(|| ConfigError::Invalid("postgres section missing".into()))?
                .schema
                .as_str();
            let schema = validate_schema_name(schema)?;
            for table in tables {
                let stmt = format!(
                    "TRUNCATE TABLE {}.{} RESTART IDENTITY",
                    quote_ident(&schema),
                    quote_ident(table)
                );
                if let Err(e) = sqlx::query(&stmt).execute(pool).await {
                    if !is_missing_table_error(&e) {
                        return Err(ConfigError::Invalid(format!("cleanup failed: {e}")));
                    }
                }
            }
        }
    }

    Ok(())
}

fn set_search_path(
    schema: &str,
) -> impl Fn(
    &mut sqlx::PgConnection,
    sqlx::pool::PoolConnectionMetadata,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), sqlx::Error>> + Send + '_>> {
    let schema_name = schema.to_string();
    move |conn, _meta| {
        let schema_copy = schema_name.clone();
        Box::pin(async move {
            let schema_ident = quote_ident(&schema_copy);
            let create_stmt = format!("CREATE SCHEMA IF NOT EXISTS {schema_ident}");
            sqlx::query(&create_stmt).execute(&mut *conn).await?;
            let search_stmt = format!("SET search_path TO {schema_ident}");
            sqlx::query(&search_stmt).execute(&mut *conn).await?;
            Ok(())
        })
    }
}


fn is_missing_table_error(e: &sqlx::Error) -> bool {
    matches!(
        e,
        sqlx::Error::Database(db_err) if db_err.code().as_deref() == Some("42P01")
    )
}

fn quote_ident(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}
