use super::{
  App,
  EntriesReadFilter
};

impl App {
  pub(crate) fn entries_filter_label(
    &self
  ) -> &'static str {
    match self.entries_read_filter {
      | EntriesReadFilter::All => "all",
      | EntriesReadFilter::Read => {
        "read"
      }
      | EntriesReadFilter::Unread => {
        "unread"
      }
    }
  }
}
