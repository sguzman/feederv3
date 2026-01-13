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
}
