//! Small helpers shared across Postgres repo modules.
use std::time::{SystemTime, UNIX_EPOCH};

pub fn now_epoch_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

pub fn chunk_statements(schema: &str) -> impl Iterator<Item = &str> {
    schema.split(';').map(str::trim).filter(|s| !s.is_empty())
}
