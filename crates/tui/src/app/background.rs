use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::thread;

use serde::de::DeserializeOwned;

use super::App;
use crate::models::{
  EntryListResponse,
  FeedEntryCounts,
  FeedSummary,
  FolderRow,
  SubscriptionRow
};

pub(crate) enum AppEvent {
  RefreshAll {
    feeds:         Vec<FeedSummary>,
    subscriptions: Vec<String>,
    feed_counts:   Vec<FeedEntryCounts>,
    favorites:     Vec<FeedSummary>,
    folders:       Vec<FolderRow>,
    entries: Option<EntryListResponse>
  },
  ToggleSubscribe {
    feed_id:            String,
    desired_subscribed: bool,
    ok:                 bool,
    message:            Option<String>
  },
  ToggleEntryRead {
    entry_id:     i64,
    desired_read: bool,
    ok:           bool,
    message:      Option<String>
  },
  Error {
    message: String
  }
}

impl App {
  pub(crate) fn set_event_sender(
    &mut self,
    sender: Sender<AppEvent>
  ) {
    self.event_tx = Some(sender);
  }

  pub(crate) fn apply_event(
    &mut self,
    event: AppEvent
  ) {
    match event {
      | AppEvent::RefreshAll {
        feeds,
        subscriptions,
        feed_counts,
        favorites,
        folders,
        entries
      } => {
        self.feeds = feeds;
        self.subscriptions =
          subscriptions
            .into_iter()
            .collect();
        self.feed_counts = feed_counts
          .into_iter()
          .map(|row| {
            (row.feed_id.clone(), row)
          })
          .collect::<HashMap<_, _>>();
        self.favorites = favorites;
        self.folders = folders;
        if let Some(data) = entries {
          self.entries = data.items;
          self.entries_next_offset =
            data.next_offset;
        }

        self.rebuild_views();
        self.sort_favorites();

        if self.selected_feed
          >= self.feeds_view.len()
        {
          self.selected_feed = 0;
          self.feeds_offset = 0;
        }

        if self.selected_favorite
          >= self.favorites.len()
        {
          self.selected_favorite = 0;
          self.favorites_offset = 0;
        }

        if self.selected_folder
          >= self.folders.len()
        {
          self.selected_folder = 0;
          self.folders_offset = 0;
        }

        if self.selected_subscription
          >= self
            .subscriptions_view
            .len()
        {
          self.selected_subscription =
            0;
          self.subscriptions_offset = 0;
        }

        self.loading = false;
        self.pending_requests = self
          .pending_requests
          .saturating_sub(1);
        self.error = None;

        self.status = format!(
          "Loaded {} feeds ({} shown)",
          self.feeds.len(),
          self.feeds_view.len()
        );
      }
      | AppEvent::ToggleSubscribe {
        feed_id,
        desired_subscribed,
        ok,
        message
      } => {
        self.loading = false;
        self.pending_requests = self
          .pending_requests
          .saturating_sub(1);

        if ok {
          self.status =
            if desired_subscribed {
              format!(
                "Subscribed to \
                 {feed_id}"
              )
            } else {
              format!(
                "Unsubscribed from \
                 {feed_id}"
              )
            };
          self.error = None;
        } else {
          if desired_subscribed {
            self
              .subscriptions
              .remove(&feed_id);
          } else {
            self
              .subscriptions
              .insert(feed_id.clone());
          }
          self.rebuild_views();
          self.error = Some(
            message.unwrap_or_else(
              || {
                "subscription update \
                 failed"
                  .to_string()
              }
            )
          );
        }
      }
      | AppEvent::ToggleEntryRead {
        entry_id,
        desired_read,
        ok,
        message
      } => {
        self.loading = false;
        self.pending_requests = self
          .pending_requests
          .saturating_sub(1);

        if ok {
          self.status = if desired_read
          {
            "Marked read".to_string()
          } else {
            "Marked unread".to_string()
          };
          self.error = None;
        } else {
          if let Some(entry) = self
            .entries
            .iter_mut()
            .find(|row| {
              row.id == entry_id
            })
          {
            entry.is_read =
              !desired_read;
          }
          self.error = Some(
            message.unwrap_or_else(
              || {
                "toggle read failed"
                  .to_string()
              }
            )
          );
        }
      }
      | AppEvent::Error {
        message
      } => {
        self.loading = false;
        self.pending_requests = self
          .pending_requests
          .saturating_sub(1);
        self.error = Some(message);
      }
    }
  }

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
    let entries_feed_id =
      self.entries_feed_id.clone();
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

      let entries =
        if let Some(feed_id) =
          entries_feed_id
        {
          let url = format!(
            "{base_url}/v1/feeds/\
             {feed_id}/entries?\
             limit={entries_page_size}&\
             offset={entries_offset}&\
             read=all"
          );
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
              return;
            }
          }
        } else {
          None
        };

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
}

fn get_json<T: DeserializeOwned>(
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
