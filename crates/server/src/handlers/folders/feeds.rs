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
use crate::errors::{
  ServerError,
  map_db_error
};
use crate::models::{
  FolderFeedRequest,
  FolderFeedRow
};

pub async fn list_folder_feeds(
  State(state): State<AppState>,
  headers: HeaderMap,
  AxumPath(folder_id): AxumPath<i64>
) -> Result<
  Json<Vec<FolderFeedRow>>,
  ServerError
> {
  let user_id =
    auth_user_id(&state, &headers)
      .await?;

  if let Some(pool) = &state.postgres {
    let rows = sqlx::query_as::<_, FolderFeedRow>(
      "SELECT ff.feed_id FROM folder_feeds ff JOIN folders f ON f.id = ff.folder_id WHERE ff.folder_id = $1 AND f.user_id = $2 ORDER BY ff.feed_id",
    )
    .bind(folder_id)
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

  let rows =
    sqlx::query_as::<_, FolderFeedRow>(
      "SELECT ff.feed_id FROM \
       folder_feeds ff JOIN folders f \
       ON f.id = ff.folder_id WHERE \
       ff.folder_id = ?1 AND \
       f.user_id = ?2 ORDER BY \
       ff.feed_id"
    )
    .bind(folder_id)
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map_err(|e| {
      ServerError::new(
      StatusCode::INTERNAL_SERVER_ERROR,
      e.to_string(),
    )
    })?;

  Ok(Json(rows))
}

pub async fn add_folder_feed(
  State(state): State<AppState>,
  headers: HeaderMap,
  AxumPath(folder_id): AxumPath<i64>,
  Json(payload): Json<
    FolderFeedRequest
  >
) -> Result<StatusCode, ServerError> {
  let user_id =
    auth_user_id(&state, &headers)
      .await?;

  let feed_id = payload.feed_id.trim();

  if feed_id.is_empty() {
    return Err(ServerError::new(
      StatusCode::BAD_REQUEST,
      "feed_id required"
    ));
  }

  let rows = if let Some(pool) =
    &state.postgres
  {
    sqlx::query(
      "INSERT INTO folder_feeds \
       (folder_id, feed_id, \
       created_at) SELECT $1, $2, \
       NOW() WHERE EXISTS (SELECT 1 \
       FROM folders WHERE id = $1 AND \
       user_id = $3)"
    )
    .bind(folder_id)
    .bind(feed_id)
    .bind(user_id)
    .execute(pool)
    .await
    .map_err(|e| {
      map_db_error(
        e,
        "folder feed create failed"
      )
    })?
    .rows_affected()
  } else {
    let pool = state.sqlite.as_ref().ok_or_else(|| {
      ServerError::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        "database pool missing",
      )
    })?;

    sqlx::query(
      "INSERT INTO folder_feeds \
       (folder_id, feed_id, \
       created_at) SELECT ?1, ?2, \
       datetime('now') WHERE EXISTS \
       (SELECT 1 FROM folders WHERE \
       id = ?1 AND user_id = ?3)"
    )
    .bind(folder_id)
    .bind(feed_id)
    .bind(user_id)
    .execute(pool)
    .await
    .map_err(|e| {
      map_db_error(
        e,
        "folder feed create failed"
      )
    })?
    .rows_affected()
  };

  if rows == 0 {
    return Err(ServerError::new(
      StatusCode::NOT_FOUND,
      "folder not found"
    ));
  }

  Ok(StatusCode::CREATED)
}

pub async fn delete_folder_feed(
  State(state): State<AppState>,
  headers: HeaderMap,
  AxumPath((folder_id, feed_id)): AxumPath<(i64, String)>
) -> Result<StatusCode, ServerError> {
  let user_id =
    auth_user_id(&state, &headers)
      .await?;

  let rows = if let Some(pool) =
    &state.postgres
  {
    sqlx::query(
      "DELETE FROM folder_feeds ff USING folders f WHERE ff.folder_id = f.id AND ff.folder_id = $1 AND ff.feed_id = $2 AND f.user_id = $3",
    )
    .bind(folder_id)
    .bind(&feed_id)
    .bind(user_id)
    .execute(pool)
    .await
    .map_err(|e| {
      ServerError::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        e.to_string(),
      )
    })?
    .rows_affected()
  } else {
    let pool = state.sqlite.as_ref().ok_or_else(|| {
      ServerError::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        "database pool missing",
      )
    })?;

    sqlx::query(
      "DELETE FROM folder_feeds WHERE folder_id = ?1 AND feed_id = ?2 AND EXISTS (SELECT 1 FROM folders WHERE id = ?1 AND user_id = ?3)",
    )
    .bind(folder_id)
    .bind(&feed_id)
    .bind(user_id)
    .execute(pool)
    .await
    .map_err(|e| {
      ServerError::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        e.to_string(),
      )
    })?
    .rows_affected()
  };

  if rows == 0 {
    return Err(ServerError::new(
      StatusCode::NOT_FOUND,
      "folder feed not found"
    ));
  }

  Ok(StatusCode::NO_CONTENT)
}
