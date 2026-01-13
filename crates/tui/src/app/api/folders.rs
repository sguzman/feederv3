use anyhow::{
  Context,
  Result
};

use super::super::App;
use super::super::util::ensure_offset;

impl App {
  pub(crate) fn refresh_folders(
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
}
