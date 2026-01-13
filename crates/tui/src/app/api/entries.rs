use anyhow::{
  Context,
  Result
};

use super::super::App;
use crate::models::EntryListResponse;

impl App {
  pub(crate) fn refresh_entries(
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

    if let Some((current, total)) =
      self.entries_page_info()
    {
      self.status = format!(
        "Loaded {} entries (page \
         {current}/{total}, offset {})",
        self.entries.len(),
        self.entries_offset
      );
    } else {
      self.status = format!(
        "Loaded {} entries (offset {})",
        self.entries.len(),
        self.entries_offset
      );
    }

    Ok(())
  }

  pub(crate) fn open_entries(
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

  pub(crate) fn next_entries_page(
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

  pub(crate) fn prev_entries_page(
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
}
