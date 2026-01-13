use anyhow::Result;

use super::super::App;

impl App {
  pub(crate) fn refresh_tab(
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
}
