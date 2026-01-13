use reqwest::Url;
use serde::de::DeserializeOwned;

use super::super::{
  EntriesMode,
  EntriesReadFilter
};

pub(super) fn get_json<
  T: DeserializeOwned
>(
  client: &reqwest::blocking::Client,
  url: &str,
  token: &str
) -> Result<T, String> {
  let resp = client
    .get(url)
    .bearer_auth(token)
    .send()
    .map_err(|e| e.to_string())?;

  if !resp.status().is_success() {
    let msg = resp
      .text()
      .unwrap_or_else(|_| {
        "request failed".to_string()
      });
    return Err(msg);
  }

  resp
    .json::<T>()
    .map_err(|e| e.to_string())
}

pub(super) fn build_entries_url(
  base_url: &str,
  mode: &EntriesMode,
  read_filter: EntriesReadFilter,
  limit: u32,
  offset: i64
) -> Option<String> {
  let (path, feed_id, query) =
    match mode {
      | EntriesMode::None => {
        return None
      }
      | EntriesMode::Feed(feed_id) => {
        (
          format!(
            "{base_url}/v1/feeds/\
             {feed_id}/entries"
          ),
          None,
          None
        )
      }
      | EntriesMode::Folder(
        folder_id
      ) => {
        (
          format!(
            "{base_url}/v1/folders/\
             {folder_id}/entries"
          ),
          None,
          None
        )
      }
      | EntriesMode::All => {
        (
          format!(
            "{base_url}/v1/entries"
          ),
          None,
          None
        )
      }
      | EntriesMode::Search {
        query,
        feed_id
      } => {
        (
          format!(
            "{base_url}/v1/entries/\
             search"
          ),
          feed_id.clone(),
          Some(query.clone())
        )
      }
    };

  let mut url =
    Url::parse(&path).ok()?;
  {
    let mut pairs =
      url.query_pairs_mut();
    pairs
      .append_pair(
        "limit",
        &limit.to_string()
      )
      .append_pair(
        "offset",
        &offset.to_string()
      );

    if let Some(feed_id) = feed_id {
      pairs.append_pair(
        "feed_id", &feed_id
      );
    }

    if let Some(query) = query {
      pairs.append_pair("q", &query);
    }

    match read_filter {
      | EntriesReadFilter::All => {}
      | EntriesReadFilter::Read => {
        pairs
          .append_pair("read", "read");
      }
      | EntriesReadFilter::Unread => {
        pairs.append_pair(
          "read", "unread"
        );
      }
    }
  }

  Some(url.to_string())
}
