use axum::Json;
use axum::extract::{
  Path as AxumPath,
  State
};
use axum::http::{
  HeaderMap,
  StatusCode
};

use crate::app_state::AppState;
use crate::auth::auth_user_id;
use crate::db::quote_ident;
use crate::errors::ServerError;
use crate::models::{
  FolderFeedUnreadCount,
  FolderUnreadCount
};

pub async fn folder_feed_unread_counts(
  State(state): State<AppState>,
  headers: HeaderMap,
  AxumPath(folder_id): AxumPath<i64>
) -> Result<
  Json<Vec<FolderFeedUnreadCount>>,
  ServerError
> {
  let user_id =
    auth_user_id(&state, &headers)
      .await?;

  if let Some(pool) = &state.postgres {
    let schema = state
      .fetcher_schema
      .as_deref()
      .unwrap_or("fetcher");

    let query = format!(
      "SELECT fi.feed_id, \
       COUNT(*)::BIGINT AS \
       unread_count FROM folder_feeds \
       ff JOIN folders f ON f.id = \
       ff.folder_id JOIN \
       {}.feed_items fi ON fi.feed_id \
       = ff.feed_id LEFT JOIN \
       entry_states es ON es.item_id \
       = fi.id AND es.user_id = $1 \
       WHERE ff.folder_id = $2 AND \
       f.user_id = $1 AND es.read_at \
       IS NULL GROUP BY fi.feed_id \
       ORDER BY fi.feed_id",
      quote_ident(schema)
    );

    let rows = sqlx::query_as::<_, FolderFeedUnreadCount>(&query)
      .bind(user_id)
      .bind(folder_id)
      .fetch_all(pool)
      .await
      .map_err(|e| {
        ServerError::new(
          StatusCode::INTERNAL_SERVER_ERROR,
          e.to_string(),
        )
      })?;

    return Ok(Json(rows));
  }

  let pool = state
    .sqlite
    .as_ref()
    .ok_or_else(|| {
      ServerError::new(
      StatusCode::INTERNAL_SERVER_ERROR,
      "database pool missing",
    )
    })?;

  let rows = sqlx::query_as::<
    _,
    FolderFeedUnreadCount
  >(
    "SELECT fi.feed_id, COUNT(*) AS \
     unread_count FROM folder_feeds \
     ff JOIN folders f ON f.id = \
     ff.folder_id JOIN feed_items fi \
     ON fi.feed_id = ff.feed_id LEFT \
     JOIN entry_states es ON \
     es.item_id = fi.id AND \
     es.user_id = ?1 WHERE \
     ff.folder_id = ?2 AND f.user_id \
     = ?1 AND es.read_at IS NULL \
     GROUP BY fi.feed_id ORDER BY \
     fi.feed_id"
  )
  .bind(user_id)
  .bind(folder_id)
  .fetch_all(pool)
  .await
  .map_err(|e| {
    ServerError::new(
      StatusCode::INTERNAL_SERVER_ERROR,
      e.to_string()
    )
  })?;

  Ok(Json(rows))
}

pub async fn folder_unread_counts(
  State(state): State<AppState>,
  headers: HeaderMap
) -> Result<
  Json<Vec<FolderUnreadCount>>,
  ServerError
> {
  let user_id =
    auth_user_id(&state, &headers)
      .await?;

  if let Some(pool) = &state.postgres {
    let schema = state
      .fetcher_schema
      .as_deref()
      .unwrap_or("fetcher");

    let query = format!(
      "SELECT f.id AS folder_id, \
       COUNT(*)::BIGINT AS \
       unread_count FROM folders f \
       JOIN folder_feeds ff ON \
       ff.folder_id = f.id JOIN \
       {}.feed_items fi ON fi.feed_id \
       = ff.feed_id LEFT JOIN \
       entry_states es ON es.item_id \
       = fi.id AND es.user_id = $1 \
       WHERE f.user_id = $1 AND \
       es.read_at IS NULL GROUP BY \
       f.id ORDER BY f.id",
      quote_ident(schema)
    );

    let rows = sqlx::query_as::<_, FolderUnreadCount>(&query)
      .bind(user_id)
      .fetch_all(pool)
      .await
      .map_err(|e| {
        ServerError::new(
          StatusCode::INTERNAL_SERVER_ERROR,
          e.to_string(),
        )
      })?;

    return Ok(Json(rows));
  }

  let pool = state
    .sqlite
    .as_ref()
    .ok_or_else(|| {
      ServerError::new(
      StatusCode::INTERNAL_SERVER_ERROR,
      "database pool missing",
    )
    })?;

  let rows = sqlx::query_as::<
    _,
    FolderUnreadCount
  >(
    "SELECT f.id AS folder_id, \
     COUNT(*) AS unread_count FROM \
     folders f JOIN folder_feeds ff \
     ON ff.folder_id = f.id JOIN \
     feed_items fi ON fi.feed_id = \
     ff.feed_id LEFT JOIN \
     entry_states es ON es.item_id = \
     fi.id AND es.user_id = ?1 WHERE \
     f.user_id = ?1 AND es.read_at IS \
     NULL GROUP BY f.id ORDER BY f.id"
  )
  .bind(user_id)
  .fetch_all(pool)
  .await
  .map_err(|e| {
    ServerError::new(
      StatusCode::INTERNAL_SERVER_ERROR,
      e.to_string()
    )
  })?;

  Ok(Json(rows))
}
