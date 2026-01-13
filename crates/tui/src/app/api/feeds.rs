use anyhow::{
  Context,
  Result
};

use super::super::App;
use crate::models::FeedEntryCounts;

impl App {
  pub(crate) fn refresh_feeds(
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

  pub(crate) fn refresh_feed_counts(
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
}
