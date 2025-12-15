//! Database wiring: creates repository implementations per SQL dialect.
use std::{path::Path, sync::Arc};

use crate::domain::model::SqlDialect;
use crate::infra::sqlite_repo::SqliteRepo;
use crate::ports::repo::Repo;

pub async fn create_repo(dialect: SqlDialect, db_path: &Path) -> Result<Arc<dyn Repo>, String> {
    match dialect {
        SqlDialect::Sqlite => Ok(Arc::new(SqliteRepo::new(db_path).await?)),
    }
}
