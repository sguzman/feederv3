use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct FeedSummary {
  pub(crate) id:                String,
  pub(crate) url:               String,
  pub(crate) domain:            String,
  pub(crate) category:          String,
  pub(crate) base_poll_seconds: i64
}

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct FolderRow {
  pub(crate) id:   i64,
  pub(crate) name: String
}

#[derive(Debug, Deserialize)]
pub(crate) struct TokenResponse {
  pub(crate) token: String
}
