use anyhow::{
  Context,
  Result
};

use super::super::{
  App,
  Screen
};
use crate::models::TokenResponse;

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
    self.needs_refresh = true;
    self.status = "Logged in. Loading \
                   feeds..."
      .to_string();

    Ok(())
  }
}
