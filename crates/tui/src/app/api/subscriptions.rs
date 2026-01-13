use anyhow::{
  Context,
  Result
};

use super::super::App;
use crate::models::SubscriptionRow;

impl App {
  pub(crate) fn refresh_subscriptions(
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

  pub(crate) fn toggle_subscribe(
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
