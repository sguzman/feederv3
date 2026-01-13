use anyhow::Result;
use crossterm::event::{
  KeyCode,
  KeyEvent,
  KeyModifiers
};

use super::super::{
  App,
  LoginField
};

impl App {
  pub(super) fn handle_login_key(
    &mut self,
    key: KeyEvent
  ) -> Result<bool> {
    if self
      .key_matches(&self.keys.quit, key)
      || (key.code
        == KeyCode::Char('c')
        && key.modifiers
          == KeyModifiers::CONTROL)
    {
      return Ok(true);
    }

    if self.modal.is_some() {
      return self.handle_modal_key(key);
    }

    match key {
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
}
