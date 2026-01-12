use anyhow::{
  Context,
  Result
};
use crossterm::event::{
  KeyCode,
  KeyEvent,
  KeyModifiers
};
use reqwest::blocking::Client;

use crate::models::{
  FeedSummary,
  FolderRow,
  TokenResponse
};

#[derive(Debug, Clone, Copy)]
pub(crate) enum Screen {
  Login,
  Main
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum LoginField {
  Username,
  Password
}

pub(crate) struct App {
  pub(crate) screen: Screen,
  pub(crate) focus: LoginField,
  pub(crate) username: String,
  pub(crate) password: String,
  pub(crate) status: String,
  pub(crate) token: Option<String>,
  pub(crate) feeds: Vec<FeedSummary>,
  pub(crate) favorites:
    Vec<FeedSummary>,
  pub(crate) folders: Vec<FolderRow>,
  pub(crate) tab: usize,
  pub(crate) selected_feed: usize,
  pub(crate) selected_favorite: usize,
  pub(crate) selected_folder: usize,
  base_url: String,
  client: Client
}

impl App {
  pub(crate) fn new(
    base_url: String
  ) -> Result<Self> {
    let client =
      Client::builder().build()?;

    Ok(Self {
      screen: Screen::Login,
      focus: LoginField::Username,
      username: "admin".to_string(),
      password: "admin".to_string(),
      status: "Enter credentials. Tab \
               switches fields. Enter \
               to login."
        .to_string(),
      token: None,
      feeds: Vec::new(),
      favorites: Vec::new(),
      folders: Vec::new(),
      tab: 0,
      selected_feed: 0,
      selected_favorite: 0,
      selected_folder: 0,
      base_url,
      client
    })
  }

  pub(crate) fn handle_key(
    &mut self,
    key: KeyEvent
  ) -> Result<bool> {
    match self.screen {
      | Screen::Login => {
        self.handle_login_key(key)
      }
      | Screen::Main => {
        self.handle_main_key(key)
      }
    }
  }

  fn handle_login_key(
    &mut self,
    key: KeyEvent
  ) -> Result<bool> {
    match key {
      | KeyEvent {
        code: KeyCode::Char('c'),
        modifiers: KeyModifiers::CONTROL,
        ..
      }
      | KeyEvent {
        code: KeyCode::Char('q'),
        modifiers: KeyModifiers::NONE,
        ..
      } => return Ok(true),
      | KeyEvent {
        code: KeyCode::Tab,
        ..
      } => {
        self.focus = match self.focus {
          | LoginField::Username => {
            LoginField::Password
          }
          | LoginField::Password => {
            LoginField::Username
          }
        };
      }
      | KeyEvent {
        code: KeyCode::Enter,
        ..
      } => {
        self.login()?;
      }
      | KeyEvent {
        code: KeyCode::Backspace,
        ..
      } => {
        match self.focus {
          | LoginField::Username => {
            self.username.pop();
          }
          | LoginField::Password => {
            self.password.pop();
          }
        }
      }
      | KeyEvent {
        code: KeyCode::Char(ch),
        modifiers: KeyModifiers::NONE,
        ..
      } => {
        match self.focus {
          | LoginField::Username => {
            self.username.push(ch)
          }
          | LoginField::Password => {
            self.password.push(ch)
          }
        }
      }
      | _ => {}
    }

    Ok(false)
  }

  fn handle_main_key(
    &mut self,
    key: KeyEvent
  ) -> Result<bool> {
    match key {
      | KeyEvent {
        code: KeyCode::Char('c'),
        modifiers: KeyModifiers::CONTROL,
        ..
      }
      | KeyEvent {
        code: KeyCode::Char('q'),
        modifiers: KeyModifiers::NONE,
        ..
      } => return Ok(true),
      | KeyEvent {
        code: KeyCode::Char('1'),
        ..
      } => self.tab = 0,
      | KeyEvent {
        code: KeyCode::Char('2'),
        ..
      } => self.tab = 1,
      | KeyEvent {
        code: KeyCode::Char('3'),
        ..
      } => self.tab = 2,
      | KeyEvent {
        code: KeyCode::Left,
        ..
      } => {
        self.tab = (self.tab + 2) % 3;
      }
      | KeyEvent {
        code: KeyCode::Right,
        ..
      } => {
        self.tab = (self.tab + 1) % 3;
      }
      | KeyEvent {
        code: KeyCode::Char('r'),
        ..
      } => {
        self.refresh_tab()?;
      }
      | KeyEvent {
        code: KeyCode::Down,
        ..
      }
      | KeyEvent {
        code: KeyCode::Char('j'),
        ..
      } => {
        self.move_selection(1);
      }
      | KeyEvent {
        code: KeyCode::Up,
        ..
      }
      | KeyEvent {
        code: KeyCode::Char('k'),
        ..
      } => {
        self.move_selection(-1);
      }
      | _ => {}
    }

    Ok(false)
  }

  fn move_selection(
    &mut self,
    delta: i32
  ) {
    match self.tab {
      | 0 => {
        let len = self.feeds.len();
        self.selected_feed = move_index(
          self.selected_feed,
          len,
          delta
        );
      }
      | 1 => {
        let len = self.favorites.len();
        self.selected_favorite =
          move_index(
            self.selected_favorite,
            len,
            delta
          );
      }
      | _ => {
        let len = self.folders.len();
        self.selected_folder =
          move_index(
            self.selected_folder,
            len,
            delta
          );
      }
    }
  }

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

  pub(crate) fn refresh_all(
    &mut self
  ) -> Result<()> {
    self.refresh_feeds()?;
    self.refresh_favorites()?;
    self.refresh_folders()?;

    Ok(())
  }

  fn refresh_tab(
    &mut self
  ) -> Result<()> {
    match self.tab {
      | 0 => self.refresh_feeds(),
      | 1 => self.refresh_favorites(),
      | _ => self.refresh_folders()
    }
  }

  fn refresh_feeds(
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

    if self.selected_feed
      >= self.feeds.len()
    {
      self.selected_feed = 0;
    }

    self.status = format!(
      "Loaded {} feeds",
      self.feeds.len()
    );

    Ok(())
  }

  fn refresh_favorites(
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

    if self.selected_favorite
      >= self.favorites.len()
    {
      self.selected_favorite = 0;
    }

    self.status = format!(
      "Loaded {} favorites",
      self.favorites.len()
    );

    Ok(())
  }

  fn refresh_folders(
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
    }

    self.status = format!(
      "Loaded {} folders",
      self.folders.len()
    );

    Ok(())
  }
}

fn move_index(
  current: usize,
  len: usize,
  delta: i32
) -> usize {
  if len == 0 {
    return 0;
  }

  let max =
    len.saturating_sub(1) as i32;

  let next = (current as i32 + delta)
    .clamp(0, max);

  next as usize
}
