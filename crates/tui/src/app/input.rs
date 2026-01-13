use anyhow::Result;
use crossterm::event::{
  KeyCode,
  KeyEvent,
  KeyModifiers
};

use super::util::move_index;
use super::{
  App,
  LoginField,
  Screen
};
use crate::config::KeyBinding;

impl App {
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

  fn handle_main_key(
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

    if self.key_matches(
      &self.keys.tab_feeds,
      key
    ) {
      self.tab = 0;
      return Ok(false);
    }

    if self.key_matches(
      &self.keys.tab_entries,
      key
    ) {
      self.tab = 1;
      return Ok(false);
    }

    if self.key_matches(
      &self.keys.tab_favorites,
      key
    ) {
      self.tab = 2;
      return Ok(false);
    }

    if self.key_matches(
      &self.keys.tab_folders,
      key
    ) {
      self.tab = 3;
      return Ok(false);
    }

    if self.key_matches(
      &self.keys.tab_subscriptions,
      key
    ) {
      self.tab = 4;
      return Ok(false);
    }

    if self.key_matches(
      &self.keys.prev_tab,
      key
    ) {
      self.tab = if self.tab == 0 {
        4
      } else {
        self.tab - 1
      };
      return Ok(false);
    }

    if self.key_matches(
      &self.keys.next_tab,
      key
    ) {
      self.tab = (self.tab + 1) % 5;
      return Ok(false);
    }

    if self.key_matches(
      &self.keys.refresh,
      key
    ) {
      self.refresh_tab()?;
      return Ok(false);
    }

    if self.key_matches(
      &self.keys.move_down,
      key
    ) {
      self.move_selection(1);
      return Ok(false);
    }

    if self.key_matches(
      &self.keys.move_up,
      key
    ) {
      self.move_selection(-1);
      return Ok(false);
    }

    if self.key_matches(
      &self.keys.go_top,
      key
    ) {
      self.jump_top();
      return Ok(false);
    }

    if self.key_matches(
      &self.keys.go_middle,
      key
    ) {
      self.jump_middle();
      return Ok(false);
    }

    if self.key_matches(
      &self.keys.go_bottom,
      key
    ) {
      self.jump_bottom();
      return Ok(false);
    }

    if self.key_matches(
      &self.keys.open_category_menu,
      key
    ) {
      self.open_category_menu();
      return Ok(false);
    }

    if self.key_matches(
      &self.keys.open_tag_menu,
      key
    ) {
      self.open_tag_menu();
      return Ok(false);
    }

    if self.key_matches(
      &self.keys.open_sort_menu,
      key
    ) {
      self.open_sort_menu();
      return Ok(false);
    }

    if self.key_matches(
      &self.keys.clear_filters,
      key
    ) {
      self.clear_filters();
      return Ok(false);
    }

    if self.key_matches(
      &self.keys.open_entries,
      key
    ) {
      self.open_entries()?;
      return Ok(false);
    }

    if self.key_matches(
      &self.keys.toggle_read,
      key
    ) {
      self.toggle_entry_read()?;
      return Ok(false);
    }

    if self.key_matches(
      &self.keys.toggle_subscribe,
      key
    ) {
      self.toggle_subscribe()?;
      return Ok(false);
    }

    if self.key_matches(
      &self.keys.entries_next,
      key
    ) {
      if self.tab == 1 {
        self.next_entries_page()?;
      } else {
        self.next_list_page();
      }
      return Ok(false);
    }

    if self.key_matches(
      &self.keys.entries_prev,
      key
    ) {
      if self.tab == 1 {
        self.prev_entries_page()?;
      } else {
        self.prev_list_page();
      }
      return Ok(false);
    }

    if self.key_matches(
      &self.keys.feeds_next,
      key
    ) {
      if self.tab == 0 {
        self.next_list_page();
      }
      return Ok(false);
    }

    if self.key_matches(
      &self.keys.feeds_prev,
      key
    ) {
      if self.tab == 0 {
        self.prev_list_page();
      }
      return Ok(false);
    }

    Ok(false)
  }

  pub(crate) fn key_matches(
    &self,
    binding: &KeyBinding,
    key: KeyEvent
  ) -> bool {
    key.code == binding.code
      && key.modifiers
        == binding.modifiers
  }

  fn move_selection(
    &mut self,
    delta: i32
  ) {
    match self.tab {
      | 0 => {
        let len = self.feeds_view.len();
        self.selected_feed = move_index(
          self.selected_feed,
          len,
          delta
        );
        self.ensure_visible_for_tab();
      }
      | 1 => {
        let len = self.entries.len();
        self.selected_entry =
          move_index(
            self.selected_entry,
            len,
            delta
          );
      }
      | 2 => {
        let len = self.favorites.len();
        self.selected_favorite =
          move_index(
            self.selected_favorite,
            len,
            delta
          );
        self.ensure_visible_for_tab();
      }
      | 3 => {
        let len = self.folders.len();
        self.selected_folder =
          move_index(
            self.selected_folder,
            len,
            delta
          );
        self.ensure_visible_for_tab();
      }
      | _ => {
        let len =
          self.subscriptions_view.len();
        self.selected_subscription =
          move_index(
            self.selected_subscription,
            len,
            delta
          );
        self.ensure_visible_for_tab();
      }
    }
  }

  fn jump_top(&mut self) {
    match self.tab {
      | 0 => {
        self.selected_feed = 0;
        self.ensure_visible_for_tab();
      }
      | 1 => {
        self.selected_entry = 0;
      }
      | 2 => {
        self.selected_favorite = 0;
        self.ensure_visible_for_tab();
      }
      | 3 => {
        self.selected_folder = 0;
        self.ensure_visible_for_tab();
      }
      | _ => {
        self.selected_subscription = 0;
        self.ensure_visible_for_tab();
      }
    }
  }

  fn jump_middle(&mut self) {
    match self.tab {
      | 0 => {
        if !self.feeds_view.is_empty() {
          self.selected_feed =
            self.feeds_view.len() / 2;
          self.ensure_visible_for_tab();
        }
      }
      | 1 => {
        if !self.entries.is_empty() {
          self.selected_entry =
            self.entries.len() / 2;
        }
      }
      | 2 => {
        if !self.favorites.is_empty() {
          self.selected_favorite =
            self.favorites.len() / 2;
          self.ensure_visible_for_tab();
        }
      }
      | 3 => {
        if !self.folders.is_empty() {
          self.selected_folder =
            self.folders.len() / 2;
          self.ensure_visible_for_tab();
        }
      }
      | _ => {
        if !self
          .subscriptions_view
          .is_empty()
        {
          self.selected_subscription =
            self
              .subscriptions_view
              .len()
              / 2;
          self.ensure_visible_for_tab();
        }
      }
    }
  }

  fn jump_bottom(&mut self) {
    match self.tab {
      | 0 => {
        if !self.feeds_view.is_empty() {
          self.selected_feed =
            self.feeds_view.len() - 1;
          self.ensure_visible_for_tab();
        }
      }
      | 1 => {
        if !self.entries.is_empty() {
          self.selected_entry =
            self.entries.len() - 1;
        }
      }
      | 2 => {
        if !self.favorites.is_empty() {
          self.selected_favorite =
            self.favorites.len() - 1;
          self.ensure_visible_for_tab();
        }
      }
      | 3 => {
        if !self.folders.is_empty() {
          self.selected_folder =
            self.folders.len() - 1;
          self.ensure_visible_for_tab();
        }
      }
      | _ => {
        if !self
          .subscriptions_view
          .is_empty()
        {
          self.selected_subscription =
            self
              .subscriptions_view
              .len()
              - 1;
          self.ensure_visible_for_tab();
        }
      }
    }
  }
}
