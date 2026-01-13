use std::thread;

use super::super::App;
use super::events::AppEvent;
use super::helpers::{
  build_entries_url,
  get_json
};
use crate::models::{
  EntryDetail,
  EntryListResponse,
  FeedDetail,
  FeedEntryCounts,
  FeedSummary,
  FolderFeedRow,
  FolderRow,
  SubscriptionRow
};

impl App {
  pub(crate) fn queue_refresh_all(
    &mut self
  ) {
    let Some(sender) =
      self.event_tx.clone()
    else {
      return;
    };

    let Some(token) =
      self.token.clone()
    else {
      return;
    };

    let base_url =
      self.base_url.clone();
    let entries_mode =
      self.entries_mode.clone();
    let entries_read_filter =
      self.entries_read_filter;
    let entries_page_size =
      self.entries_page_size;
    let entries_offset =
      self.entries_offset;

    self.pending_requests += 1;
    self.loading = true;

    thread::spawn(move || {
      let client =
        reqwest::blocking::Client::new(
        );

      let feeds: Vec<FeedSummary> =
        match get_json(
          &client,
          &format!(
            "{base_url}/v1/feeds"
          ),
          &token
        ) {
          | Ok(data) => data,
          | Err(err) => {
            let _ = sender.send(
              AppEvent::Error {
                message: err
              }
            );
            return;
          }
        };

      let subs: Vec<SubscriptionRow> =
        match get_json(
          &client,
          &format!(
            "{base_url}/v1/\
             subscriptions"
          ),
          &token
        ) {
          | Ok(data) => data,
          | Err(err) => {
            let _ = sender.send(
              AppEvent::Error {
                message: err
              }
            );
            return;
          }
        };

      let feed_counts: Vec<
        FeedEntryCounts
      > = match get_json(
        &client,
        &format!(
          "{base_url}/v1/feeds/counts"
        ),
        &token
      ) {
        | Ok(data) => data,
        | Err(err) => {
          let _ = sender.send(
            AppEvent::Error {
              message: err
            }
          );
          return;
        }
      };

      let favorites: Vec<FeedSummary> =
        match get_json(
          &client,
          &format!(
            "{base_url}/v1/favorites"
          ),
          &token
        ) {
          | Ok(data) => data,
          | Err(err) => {
            let _ = sender.send(
              AppEvent::Error {
                message: err
              }
            );
            return;
          }
        };

      let folders: Vec<FolderRow> =
        match get_json(
          &client,
          &format!(
            "{base_url}/v1/folders"
          ),
          &token
        ) {
          | Ok(data) => data,
          | Err(err) => {
            let _ = sender.send(
              AppEvent::Error {
                message: err
              }
            );
            return;
          }
        };

      let entries = build_entries_url(
        &base_url,
        &entries_mode,
        entries_read_filter,
        entries_page_size,
        entries_offset
      )
      .and_then(|url| {
        match get_json(
          &client, &url, &token
        ) {
          | Ok(data) => Some(data),
          | Err(err) => {
            let _ = sender.send(
              AppEvent::Error {
                message: err
              }
            );
            None
          }
        }
      });

      let subscription_ids = subs
        .into_iter()
        .map(|row| row.feed_id)
        .collect();

      let _ = sender.send(
        AppEvent::RefreshAll {
          feeds,
          subscriptions:
            subscription_ids,
          feed_counts,
          favorites,
          folders,
          entries
        }
      );
    });
  }

  pub(crate) fn queue_refresh_folders(
    &mut self,
    message: String
  ) {
    let Some(sender) =
      self.event_tx.clone()
    else {
      return;
    };

    let Some(token) =
      self.token.clone()
    else {
      return;
    };

    let base_url =
      self.base_url.clone();

    self.pending_requests += 1;
    self.loading = true;

    thread::spawn(move || {
      let client =
        reqwest::blocking::Client::new(
        );
      let folders: Vec<FolderRow> =
        match get_json(
          &client,
          &format!(
            "{base_url}/v1/folders"
          ),
          &token
        ) {
          | Ok(data) => data,
          | Err(err) => {
            let _ = sender.send(
              AppEvent::Error {
                message: err
              }
            );
            return;
          }
        };

      let _ = sender.send(
        AppEvent::RefreshFolders {
          folders,
          message
        }
      );
    });
  }

  pub(crate) fn queue_refresh_favorites(
    &mut self,
    message: String
  ) {
    let Some(sender) =
      self.event_tx.clone()
    else {
      return;
    };

    let Some(token) =
      self.token.clone()
    else {
      return;
    };

    let base_url =
      self.base_url.clone();

    self.pending_requests += 1;
    self.loading = true;

    thread::spawn(move || {
      let client =
        reqwest::blocking::Client::new(
        );
      let favorites: Vec<FeedSummary> =
        match get_json(
          &client,
          &format!(
            "{base_url}/v1/favorites"
          ),
          &token
        ) {
          | Ok(data) => data,
          | Err(err) => {
            let _ = sender.send(
              AppEvent::Error {
                message: err
              }
            );
            return;
          }
        };

      let _ = sender.send(
        AppEvent::RefreshFavorites {
          favorites,
          message
        }
      );
    });
  }

  pub(crate) fn queue_refresh_entries(
    &mut self,
    message: String
  ) {
    let Some(sender) =
      self.event_tx.clone()
    else {
      return;
    };

    let Some(token) =
      self.token.clone()
    else {
      return;
    };

    let base_url =
      self.base_url.clone();
    let entries_mode =
      self.entries_mode.clone();
    let entries_read_filter =
      self.entries_read_filter;
    let entries_page_size =
      self.entries_page_size;
    let entries_offset =
      self.entries_offset;

    self.pending_requests += 1;
    self.loading = true;

    thread::spawn(move || {
      let client =
        reqwest::blocking::Client::new(
        );
      let url = match build_entries_url(
        &base_url,
        &entries_mode,
        entries_read_filter,
        entries_page_size,
        entries_offset
      ) {
        | Some(url) => url,
        | None => {
          let _ = sender.send(
            AppEvent::Error {
              message:
                "no entries source \
                 selected"
                  .to_string()
            }
          );
          return;
        }
      };

      let entries: EntryListResponse =
        match get_json(
          &client, &url, &token
        ) {
          | Ok(data) => data,
          | Err(err) => {
            let _ = sender.send(
              AppEvent::Error {
                message: err
              }
            );
            return;
          }
        };

      let _ = sender.send(
        AppEvent::RefreshEntries {
          entries,
          message
        }
      );
    });
  }

  pub(crate) fn queue_toggle_subscribe(
    &mut self,
    feed_id: String,
    desired_subscribed: bool
  ) {
    let Some(sender) =
      self.event_tx.clone()
    else {
      return;
    };

    let Some(token) =
      self.token.clone()
    else {
      return;
    };

    let base_url =
      self.base_url.clone();

    self.pending_requests += 1;
    self.loading = true;

    thread::spawn(move || {
      let client =
        reqwest::blocking::Client::new(
        );
      let result = if desired_subscribed
      {
        let url = format!(
          "{base_url}/v1/subscriptions"
        );
        let body = serde_json::json!({
          "feed_id": feed_id
        });
        client
          .post(url)
          .bearer_auth(&token)
          .json(&body)
          .send()
      } else {
        let url = format!(
          "{base_url}/v1/\
           subscriptions/{feed_id}"
        );
        client
          .delete(url)
          .bearer_auth(&token)
          .send()
      };

      match result {
        | Ok(resp)
          if resp
            .status()
            .is_success() =>
        {
          let _ = sender.send(
            AppEvent::ToggleSubscribe {
              feed_id,
              desired_subscribed,
              ok: true,
              message: None
            }
          );
        }
        | Ok(resp) => {
          let message = resp
            .text()
            .unwrap_or_else(|_| {
              "subscription update \
               failed"
                .to_string()
            });
          let _ = sender.send(
            AppEvent::ToggleSubscribe {
              feed_id,
              desired_subscribed,
              ok: false,
              message: Some(message)
            }
          );
        }
        | Err(err) => {
          let _ = sender.send(
            AppEvent::ToggleSubscribe {
              feed_id,
              desired_subscribed,
              ok: false,
              message: Some(
                err.to_string()
              )
            }
          );
        }
      }
    });
  }

  pub(crate) fn queue_toggle_favorite(
    &mut self,
    feed_id: String,
    desired_favorite: bool
  ) {
    let Some(sender) =
      self.event_tx.clone()
    else {
      return;
    };

    let Some(token) =
      self.token.clone()
    else {
      return;
    };

    let base_url =
      self.base_url.clone();

    self.pending_requests += 1;
    self.loading = true;

    thread::spawn(move || {
      let client =
        reqwest::blocking::Client::new(
        );
      let result = if desired_favorite {
        let url = format!(
          "{base_url}/v1/favorites"
        );
        let body = serde_json::json!({
          "feed_id": feed_id
        });
        client
          .post(url)
          .bearer_auth(&token)
          .json(&body)
          .send()
      } else {
        let url = format!(
          "{base_url}/v1/favorites/\
           {feed_id}"
        );
        client
          .delete(url)
          .bearer_auth(&token)
          .send()
      };

      match result {
        | Ok(resp)
          if resp
            .status()
            .is_success() =>
        {
          let _ = sender.send(
            AppEvent::ToggleFavorite {
              feed_id,
              desired_favorite,
              ok: true,
              message: None
            }
          );
        }
        | Ok(resp) => {
          let message = resp
            .text()
            .unwrap_or_else(|_| {
              "favorite update failed"
                .to_string()
            });
          let _ = sender.send(
            AppEvent::ToggleFavorite {
              feed_id,
              desired_favorite,
              ok: false,
              message: Some(message)
            }
          );
        }
        | Err(err) => {
          let _ = sender.send(
            AppEvent::ToggleFavorite {
              feed_id,
              desired_favorite,
              ok: false,
              message: Some(
                err.to_string()
              )
            }
          );
        }
      }
    });
  }

  pub(crate) fn queue_toggle_entry_read(
    &mut self,
    entry_id: i64,
    desired_read: bool
  ) {
    let Some(sender) =
      self.event_tx.clone()
    else {
      return;
    };

    let Some(token) =
      self.token.clone()
    else {
      return;
    };

    let base_url =
      self.base_url.clone();

    self.pending_requests += 1;
    self.loading = true;

    thread::spawn(move || {
      let client =
        reqwest::blocking::Client::new(
        );
      let url = format!(
        "{base_url}/v1/entries/\
         {entry_id}/read"
      );

      let req = if desired_read {
        client.post(url)
      } else {
        client.delete(url)
      };

      match req
        .bearer_auth(&token)
        .send()
      {
        | Ok(resp)
          if resp
            .status()
            .is_success() =>
        {
          let _ = sender.send(
            AppEvent::ToggleEntryRead {
              entry_id,
              desired_read,
              ok: true,
              message: None
            }
          );
        }
        | Ok(resp) => {
          let message = resp
            .text()
            .unwrap_or_else(|_| {
              "toggle read failed"
                .to_string()
            });
          let _ = sender.send(
            AppEvent::ToggleEntryRead {
              entry_id,
              desired_read,
              ok: false,
              message: Some(message)
            }
          );
        }
        | Err(err) => {
          let _ = sender.send(
            AppEvent::ToggleEntryRead {
              entry_id,
              desired_read,
              ok: false,
              message: Some(
                err.to_string()
              )
            }
          );
        }
      }
    });
  }

  pub(crate) fn queue_create_folder(
    &mut self,
    name: String
  ) {
    let Some(sender) =
      self.event_tx.clone()
    else {
      return;
    };

    let Some(token) =
      self.token.clone()
    else {
      return;
    };

    let base_url =
      self.base_url.clone();

    self.pending_requests += 1;
    self.loading = true;

    thread::spawn(move || {
      let client =
        reqwest::blocking::Client::new(
        );
      let url = format!(
        "{base_url}/v1/folders"
      );
      let body = serde_json::json!({
        "name": name
      });

      let resp = client
        .post(url)
        .bearer_auth(&token)
        .json(&body)
        .send();

      match resp {
        | Ok(resp)
          if resp
            .status()
            .is_success() =>
        {
          let folders: Vec<FolderRow> =
            match get_json(
              &client,
              &format!(
                "{base_url}/v1/folders"
              ),
              &token
            ) {
              | Ok(data) => data,
              | Err(err) => {
                let _ = sender.send(
                  AppEvent::Error {
                    message: err
                  }
                );
                return;
              }
            };

          let _ = sender.send(
            AppEvent::RefreshFolders {
              folders,
              message: "Folder created"
                .to_string()
            }
          );
        }
        | Ok(resp) => {
          let message = resp
            .text()
            .unwrap_or_else(|_| {
              "folder create failed"
                .to_string()
            });
          let _ = sender.send(
            AppEvent::Error {
              message
            }
          );
        }
        | Err(err) => {
          let _ = sender.send(
            AppEvent::Error {
              message: err.to_string()
            }
          );
        }
      }
    });
  }

  pub(crate) fn queue_rename_folder(
    &mut self,
    folder_id: i64,
    name: String
  ) {
    let Some(sender) =
      self.event_tx.clone()
    else {
      return;
    };

    let Some(token) =
      self.token.clone()
    else {
      return;
    };

    let base_url =
      self.base_url.clone();

    self.pending_requests += 1;
    self.loading = true;

    thread::spawn(move || {
      let client =
        reqwest::blocking::Client::new(
        );
      let url = format!(
        "{base_url}/v1/folders/\
         {folder_id}"
      );
      let body = serde_json::json!({
        "name": name
      });

      let resp = client
        .patch(url)
        .bearer_auth(&token)
        .json(&body)
        .send();

      match resp {
        | Ok(resp)
          if resp
            .status()
            .is_success() =>
        {
          let folders: Vec<FolderRow> =
            match get_json(
              &client,
              &format!(
                "{base_url}/v1/folders"
              ),
              &token
            ) {
              | Ok(data) => data,
              | Err(err) => {
                let _ = sender.send(
                  AppEvent::Error {
                    message: err
                  }
                );
                return;
              }
            };

          let _ = sender.send(
            AppEvent::RefreshFolders {
              folders,
              message: "Folder renamed"
                .to_string()
            }
          );
        }
        | Ok(resp) => {
          let message = resp
            .text()
            .unwrap_or_else(|_| {
              "folder rename failed"
                .to_string()
            });
          let _ = sender.send(
            AppEvent::Error {
              message
            }
          );
        }
        | Err(err) => {
          let _ = sender.send(
            AppEvent::Error {
              message: err.to_string()
            }
          );
        }
      }
    });
  }

  pub(crate) fn queue_delete_folder(
    &mut self,
    folder_id: i64
  ) {
    let Some(sender) =
      self.event_tx.clone()
    else {
      return;
    };

    let Some(token) =
      self.token.clone()
    else {
      return;
    };

    let base_url =
      self.base_url.clone();

    self.pending_requests += 1;
    self.loading = true;

    thread::spawn(move || {
      let client =
        reqwest::blocking::Client::new(
        );
      let url = format!(
        "{base_url}/v1/folders/\
         {folder_id}"
      );

      let resp = client
        .delete(url)
        .bearer_auth(&token)
        .send();

      match resp {
        | Ok(resp)
          if resp
            .status()
            .is_success() =>
        {
          let folders: Vec<FolderRow> =
            match get_json(
              &client,
              &format!(
                "{base_url}/v1/folders"
              ),
              &token
            ) {
              | Ok(data) => data,
              | Err(err) => {
                let _ = sender.send(
                  AppEvent::Error {
                    message: err
                  }
                );
                return;
              }
            };

          let _ = sender.send(
            AppEvent::RefreshFolders {
              folders,
              message: "Folder deleted"
                .to_string()
            }
          );
        }
        | Ok(resp) => {
          let message = resp
            .text()
            .unwrap_or_else(|_| {
              "folder delete failed"
                .to_string()
            });
          let _ = sender.send(
            AppEvent::Error {
              message
            }
          );
        }
        | Err(err) => {
          let _ = sender.send(
            AppEvent::Error {
              message: err.to_string()
            }
          );
        }
      }
    });
  }

  pub(crate) fn queue_folder_feeds(
    &mut self,
    folder_id: i64
  ) {
    let Some(sender) =
      self.event_tx.clone()
    else {
      return;
    };

    let Some(token) =
      self.token.clone()
    else {
      return;
    };

    let base_url =
      self.base_url.clone();

    self.pending_requests += 1;
    self.loading = true;

    thread::spawn(move || {
      let client =
        reqwest::blocking::Client::new(
        );
      let url = format!(
        "{base_url}/v1/folders/\
         {folder_id}/feeds"
      );

      let feeds: Vec<FolderFeedRow> =
        match get_json(
          &client, &url, &token
        ) {
          | Ok(data) => data,
          | Err(err) => {
            let _ = sender.send(
              AppEvent::Error {
                message: err
              }
            );
            return;
          }
        };

      let feed_ids = feeds
        .into_iter()
        .map(|row| row.feed_id)
        .collect();

      let _ = sender.send(
        AppEvent::FolderFeeds {
          folder_id,
          feed_ids
        }
      );
    });
  }

  pub(crate) fn queue_assign_folder_feed(
    &mut self,
    folder_id: i64,
    feed_id: String
  ) {
    let Some(sender) =
      self.event_tx.clone()
    else {
      return;
    };

    let Some(token) =
      self.token.clone()
    else {
      return;
    };

    let base_url =
      self.base_url.clone();

    self.pending_requests += 1;
    self.loading = true;

    thread::spawn(move || {
      let client =
        reqwest::blocking::Client::new(
        );
      let url = format!(
        "{base_url}/v1/folders/\
         {folder_id}/feeds"
      );
      let body = serde_json::json!({
        "feed_id": feed_id
      });

      let resp = client
        .post(url)
        .bearer_auth(&token)
        .json(&body)
        .send();

      match resp {
        | Ok(resp)
          if resp
            .status()
            .is_success() =>
        {
          let _ = sender.send(
            AppEvent::FolderFeedUpdate {
              feed_id,
              folder_id,
              assigned: true,
              ok: true,
              message: None
            }
          );
        }
        | Ok(resp) => {
          let message = resp
            .text()
            .unwrap_or_else(|_| {
              "folder add failed"
                .to_string()
            });
          let _ = sender.send(
            AppEvent::FolderFeedUpdate {
              feed_id,
              folder_id,
              assigned: true,
              ok: false,
              message: Some(message)
            }
          );
        }
        | Err(err) => {
          let _ = sender.send(
            AppEvent::FolderFeedUpdate {
              feed_id,
              folder_id,
              assigned: true,
              ok: false,
              message: Some(
                err.to_string()
              )
            }
          );
        }
      }
    });
  }

  pub(crate) fn queue_unassign_folder_feed(
    &mut self,
    folder_id: i64,
    feed_id: String
  ) {
    let Some(sender) =
      self.event_tx.clone()
    else {
      return;
    };

    let Some(token) =
      self.token.clone()
    else {
      return;
    };

    let base_url =
      self.base_url.clone();

    self.pending_requests += 1;
    self.loading = true;

    thread::spawn(move || {
      let client =
        reqwest::blocking::Client::new(
        );
      let url = format!(
        "{base_url}/v1/folders/\
         {folder_id}/feeds/{feed_id}"
      );

      let resp = client
        .delete(url)
        .bearer_auth(&token)
        .send();

      match resp {
        | Ok(resp)
          if resp
            .status()
            .is_success() =>
        {
          let _ = sender.send(
            AppEvent::FolderFeedUpdate {
              feed_id,
              folder_id,
              assigned: false,
              ok: true,
              message: None
            }
          );
        }
        | Ok(resp) => {
          let message = resp
            .text()
            .unwrap_or_else(|_| {
              "folder remove failed"
                .to_string()
            });
          let _ = sender.send(
            AppEvent::FolderFeedUpdate {
              feed_id,
              folder_id,
              assigned: false,
              ok: false,
              message: Some(message)
            }
          );
        }
        | Err(err) => {
          let _ = sender.send(
            AppEvent::FolderFeedUpdate {
              feed_id,
              folder_id,
              assigned: false,
              ok: false,
              message: Some(
                err.to_string()
              )
            }
          );
        }
      }
    });
  }

  pub(crate) fn queue_feed_detail(
    &mut self,
    feed_id: String
  ) {
    let Some(sender) =
      self.event_tx.clone()
    else {
      return;
    };

    let Some(token) =
      self.token.clone()
    else {
      return;
    };

    let base_url =
      self.base_url.clone();

    thread::spawn(move || {
      let client =
        reqwest::blocking::Client::new(
        );
      let url = format!(
        "{base_url}/v1/feeds/{feed_id}"
      );
      match get_json::<FeedDetail>(
        &client, &url, &token
      ) {
        | Ok(detail) => {
          let _ = sender.send(
            AppEvent::FeedDetail {
              feed_id,
              detail
            }
          );
        }
        | Err(_) => {}
      }
    });
  }

  pub(crate) fn queue_entry_detail(
    &mut self,
    entry_id: i64
  ) {
    let Some(sender) =
      self.event_tx.clone()
    else {
      return;
    };

    let Some(token) =
      self.token.clone()
    else {
      return;
    };

    let base_url =
      self.base_url.clone();

    thread::spawn(move || {
      let client =
        reqwest::blocking::Client::new(
        );
      let url = format!(
        "{base_url}/v1/entries/\
         {entry_id}"
      );
      match get_json::<EntryDetail>(
        &client, &url, &token
      ) {
        | Ok(detail) => {
          let _ = sender.send(
            AppEvent::EntryDetail {
              entry_id,
              detail
            }
          );
        }
        | Err(_) => {}
      }
    });
  }
}
