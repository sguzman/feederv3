use anyhow::{
  Context,
  Result
};

use super::util::ensure_offset;
use super::{
  App,
  Screen
};
use crate::models::{
  EntryListResponse,
  FeedEntryCounts,
  SubscriptionRow,
  TokenResponse
};

impl App {
  pub(crate) fn login(
    &mut self
  ) -> Result<()> {
    let url = format!(
      "{}/v1/auth/login",
      self.base_url
    );

    let body = serde_json::json!({
      "username": &self.username,
      "password": &self.password,
    });

    let resp = self
      .client
      .post(url)
      .json(&body)
      .send()
      .context(
        "login request failed"
      )?;

    if !resp.status().is_success() {
      let msg = resp
        .text()
        .unwrap_or_else(|_| {
          "login failed".to_string()
        });

      self.status =
        format!("Login failed: {msg}");

      return Ok(());
    }

    let token = resp
      .json::<TokenResponse>()?
      .token;

    self.token = Some(token);
    self.screen = Screen::Main;
    self.status = "Logged in. Press r \
                   to refresh."
      .to_string();

    self.refresh_all()?;

    Ok(())
  }

  pub(super) fn refresh_all(
    &mut self
  ) -> Result<()> {
    self.refresh_feeds()?;
    self.refresh_subscriptions()?;
    self.refresh_feed_counts()?;
    self.refresh_favorites()?;
    self.refresh_folders()?;

    if self.entries_feed_id.is_some() {
      self.refresh_entries()?;
    }

    Ok(())
  }

  pub(super) fn refresh_tab(
    &mut self
  ) -> Result<()> {
    match self.tab {
      | 0 => {
        self.refresh_feeds()?;
        self.refresh_subscriptions()?;
        self.refresh_feed_counts()
      }
      | 1 => self.refresh_entries(),
      | 2 => self.refresh_favorites(),
      | 4 => {
        self.refresh_subscriptions()?;
        self.refresh_feed_counts()
      }
      | _ => self.refresh_folders()
    }
  }

  pub(super) fn refresh_feeds(
    &mut self
  ) -> Result<()> {
    let token = self
      .token
      .as_deref()
      .unwrap_or_default();

    let url = format!(
      "{}/v1/feeds",
      self.base_url
    );

    let resp = self
      .client
      .get(url)
      .bearer_auth(token)
      .send()
      .context(
        "feeds request failed"
      )?;

    if !resp.status().is_success() {
      self.status = format!(
        "Failed to load feeds ({})",
        resp.status()
      );

      return Ok(());
    }

    self.feeds = resp.json().context(
      "failed to parse feeds"
    )?;

    self.rebuild_views();

    if self.selected_feed
      >= self.feeds_view.len()
    {
      self.selected_feed = 0;
      self.feeds_offset = 0;
    }

    self.status = format!(
      "Loaded {} feeds ({} shown)",
      self.feeds.len(),
      self.feeds_view.len()
    );

    Ok(())
  }

  pub(super) fn refresh_favorites(
    &mut self
  ) -> Result<()> {
    let token = self
      .token
      .as_deref()
      .unwrap_or_default();

    let url = format!(
      "{}/v1/favorites",
      self.base_url
    );

    let resp = self
      .client
      .get(url)
      .bearer_auth(token)
      .send()
      .context(
        "favorites request failed"
      )?;

    if !resp.status().is_success() {
      self.status = format!(
        "Failed to load favorites ({})",
        resp.status()
      );

      return Ok(());
    }

    self.favorites =
      resp.json().context(
        "failed to parse favorites"
      )?;

    self.sort_favorites();

    if self.selected_favorite
      >= self.favorites.len()
    {
      self.selected_favorite = 0;
      self.favorites_offset = 0;
    }

    self.favorites_offset =
      ensure_offset(
        self.selected_favorite,
        self.favorites_offset,
        self.favorites_page_size
          as usize,
        self.favorites.len()
      );

    self.status = format!(
      "Loaded {} favorites",
      self.favorites.len()
    );

    Ok(())
  }

  pub(super) fn refresh_folders(
    &mut self
  ) -> Result<()> {
    let token = self
      .token
      .as_deref()
      .unwrap_or_default();

    let url = format!(
      "{}/v1/folders",
      self.base_url
    );

    let resp = self
      .client
      .get(url)
      .bearer_auth(token)
      .send()
      .context(
        "folders request failed"
      )?;

    if !resp.status().is_success() {
      self.status = format!(
        "Failed to load folders ({})",
        resp.status()
      );

      return Ok(());
    }

    self.folders =
      resp.json().context(
        "failed to parse folders"
      )?;

    if self.selected_folder
      >= self.folders.len()
    {
      self.selected_folder = 0;
      self.folders_offset = 0;
    }

    self.folders_offset = ensure_offset(
      self.selected_folder,
      self.folders_offset,
      self.folders_page_size as usize,
      self.folders.len()
    );

    self.status = format!(
      "Loaded {} folders",
      self.folders.len()
    );

    Ok(())
  }

  pub(super) fn refresh_subscriptions(
    &mut self
  ) -> Result<()> {
    let token = self
      .token
      .as_deref()
      .unwrap_or_default();

    let url = format!(
      "{}/v1/subscriptions",
      self.base_url
    );

    let resp = self
      .client
      .get(url)
      .bearer_auth(token)
      .send()
      .context(
        "subscriptions request failed"
      )?;

    if !resp.status().is_success() {
      self.status = format!(
        "Failed to load subscriptions \
         ({})",
        resp.status()
      );

      return Ok(());
    }

    let rows = resp
      .json::<Vec<SubscriptionRow>>()
      .context(
        "failed to parse subscriptions"
      )?;

    self.subscriptions = rows
      .into_iter()
      .map(|row| row.feed_id)
      .collect();

    self.rebuild_views();

    Ok(())
  }

  pub(super) fn refresh_feed_counts(
    &mut self
  ) -> Result<()> {
    let token = self
      .token
      .as_deref()
      .unwrap_or_default();

    let url = format!(
      "{}/v1/feeds/counts",
      self.base_url
    );

    let resp = self
      .client
      .get(url)
      .bearer_auth(token)
      .send()
      .context(
        "feed counts request failed"
      )?;

    if !resp.status().is_success() {
      self.status = format!(
        "Failed to load feed counts \
         ({})",
        resp.status()
      );

      return Ok(());
    }

    let rows = resp
      .json::<Vec<FeedEntryCounts>>()
      .context(
        "failed to parse feed counts"
      )?;

    self.feed_counts = rows
      .into_iter()
      .map(|row| {
        (row.feed_id.clone(), row)
      })
      .collect();

    Ok(())
  }

  pub(super) fn refresh_entries(
    &mut self
  ) -> Result<()> {
    let Some(feed_id) =
      self.entries_feed_id.clone()
    else {
      self.status = "Select a feed \
                     and press the \
                     entries key."
        .to_string();
      return Ok(());
    };

    let token = self
      .token
      .as_deref()
      .unwrap_or_default();

    let url = format!(
      "{}/v1/feeds/{}/entries?\
       limit={}&offset={}&read=all",
      self.base_url,
      feed_id,
      self.entries_page_size,
      self.entries_offset
    );

    let resp = self
      .client
      .get(url)
      .bearer_auth(token)
      .send()
      .context(
        "entries request failed"
      )?;

    if !resp.status().is_success() {
      self.status = format!(
        "Failed to load entries ({})",
        resp.status()
      );
      return Ok(());
    }

    let data = resp
      .json::<EntryListResponse>()
      .context(
        "failed to parse entries"
      )?;

    self.entries = data.items;
    self.entries_next_offset =
      data.next_offset;

    if self.selected_entry
      >= self.entries.len()
    {
      self.selected_entry = 0;
    }

    self.status = format!(
      "Loaded {} entries (offset {})",
      self.entries.len(),
      self.entries_offset
    );

    Ok(())
  }

  pub(super) fn open_entries(
    &mut self
  ) -> Result<()> {
    if self.feeds_view.is_empty() {
      self.status =
        "No feeds loaded".to_string();
      return Ok(());
    }

    let feed = self
      .feeds_view
      .get(self.selected_feed)
      .and_then(|idx| {
        self.feeds.get(*idx)
      })
      .cloned();

    if let Some(feed) = feed {
      self.entries_feed_id =
        Some(feed.id);
      self.entries_offset = 0;
      self.selected_entry = 0;
      self.tab = 1;
      self.refresh_entries()?;
    }

    Ok(())
  }

  pub(super) fn next_entries_page(
    &mut self
  ) -> Result<()> {
    if self.tab != 1 {
      return Ok(());
    }

    if let Some(next) =
      self.entries_next_offset
    {
      self.entries_offset = next;
      self.refresh_entries()?;
    }

    Ok(())
  }

  pub(super) fn prev_entries_page(
    &mut self
  ) -> Result<()> {
    if self.tab != 1 {
      return Ok(());
    }

    if self.entries_offset == 0 {
      return Ok(());
    }

    let size =
      self.entries_page_size as i64;
    self.entries_offset =
      (self.entries_offset - size)
        .max(0);
    self.refresh_entries()?;

    Ok(())
  }

  pub(super) fn toggle_entry_read(
    &mut self
  ) -> Result<()> {
    if self.tab != 1 {
      return Ok(());
    }

    let entry = match self
      .entries
      .get(self.selected_entry)
    {
      | Some(entry) => entry.clone(),
      | None => return Ok(())
    };

    let token = self
      .token
      .as_deref()
      .unwrap_or_default();

    let url = format!(
      "{}/v1/entries/{}/read",
      self.base_url, entry.id
    );

    let req = if entry.is_read {
      self.client.delete(url)
    } else {
      self.client.post(url)
    };

    let resp = req
      .bearer_auth(token)
      .send()
      .context("toggle read failed")?;

    if !resp.status().is_success() {
      self.status = format!(
        "Failed to update read state \
         ({})",
        resp.status()
      );
      return Ok(());
    }

    if let Some(row) = self
      .entries
      .get_mut(self.selected_entry)
    {
      row.is_read = !entry.is_read;
    }

    Ok(())
  }

  pub(super) fn toggle_subscribe(
    &mut self
  ) -> Result<()> {
    if self.tab != 0 {
      return Ok(());
    }

    let feed = match self
      .feeds_view
      .get(self.selected_feed)
      .and_then(|idx| {
        self.feeds.get(*idx)
      }) {
      | Some(feed) => feed.clone(),
      | None => return Ok(())
    };

    let token = self
      .token
      .as_deref()
      .unwrap_or_default();

    let subscribed = self
      .subscriptions
      .contains(&feed.id);

    let resp = if subscribed {
      let url = format!(
        "{}/v1/subscriptions/{}",
        self.base_url, feed.id
      );
      self
        .client
        .delete(url)
        .bearer_auth(token)
        .send()
        .context(
          "unsubscribe request failed"
        )?
    } else {
      let url = format!(
        "{}/v1/subscriptions",
        self.base_url
      );
      let body = serde_json::json!({
        "feed_id": feed.id,
      });
      self
        .client
        .post(url)
        .bearer_auth(token)
        .json(&body)
        .send()
        .context(
          "subscribe request failed"
        )?
    };

    if !resp.status().is_success() {
      self.status = format!(
        "Failed to update \
         subscription ({})",
        resp.status()
      );
      return Ok(());
    }

    if subscribed {
      self
        .subscriptions
        .remove(&feed.id);
      self.status = format!(
        "Unsubscribed from {}",
        feed.id
      );
    } else {
      self
        .subscriptions
        .insert(feed.id);
      self.status =
        "Subscribed".to_string();
    }

    self.rebuild_views();

    Ok(())
  }
}
