use std::collections::{
  HashMap,
  HashSet
};

use anyhow::Result;
use reqwest::blocking::Client;

use crate::config::{
  ResolvedKeybindings,
  TuiConfig
};
use crate::models::{
  EntrySummary,
  FeedEntryCounts,
  FeedSummary,
  FolderRow
};

#[derive(
  Debug, Clone, Copy, PartialEq, Eq,
)]
pub(crate) enum Screen {
  Login,
  Main
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum LoginField {
  Username,
  Password
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum ModalKind {
  Category,
  Tag,
  Sort
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum SortMode {
  Unread,
  Total,
  Ratio,
  Recent
}

#[derive(Debug, Clone)]
pub(crate) struct ModalState {
  pub(crate) kind:     ModalKind,
  pub(crate) options:  Vec<String>,
  pub(crate) selected: usize
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
  pub(crate) subscriptions:
    HashSet<String>,
  pub(crate) feed_counts:
    HashMap<String, FeedEntryCounts>,
  pub(crate) feeds_view: Vec<usize>,
  pub(crate) subscriptions_view:
    Vec<usize>,
  pub(crate) categories: Vec<String>,
  pub(crate) tags: Vec<String>,
  pub(crate) filter_category:
    Option<String>,
  pub(crate) filter_tag: Option<String>,
  pub(crate) sort_mode: SortMode,
  pub(crate) modal: Option<ModalState>,
  pub(crate) entries: Vec<EntrySummary>,
  pub(crate) tab: usize,
  pub(crate) selected_feed: usize,
  pub(crate) selected_entry: usize,
  pub(crate) selected_favorite: usize,
  pub(crate) selected_folder: usize,
  pub(crate) selected_subscription:
    usize,
  pub(crate) entries_page_size: u32,
  pub(crate) feeds_page_size: u32,
  pub(crate) favorites_page_size: u32,
  pub(crate) folders_page_size: u32,
  pub(crate) subscriptions_page_size:
    u32,
  pub(crate) feeds_offset: usize,
  pub(crate) favorites_offset: usize,
  pub(crate) folders_offset: usize,
  pub(crate) subscriptions_offset:
    usize,
  pub(crate) keys: ResolvedKeybindings,
  pub(crate) entries_feed_id:
    Option<String>,
  pub(crate) entries_offset: i64,
  pub(crate) entries_next_offset:
    Option<i64>,
  pub(crate) base_url: String,
  pub(crate) client: Client
}

impl App {
  pub(crate) fn new(
    config: &TuiConfig,
    keys: ResolvedKeybindings
  ) -> Result<Self> {
    let client = Client::builder()
      .timeout(std::time::Duration::from_millis(
        config.server.timeout_ms,
      ))
      .build()?;

    Ok(Self {
      screen: Screen::Login,
      focus: LoginField::Username,
      username: config
        .auth
        .username
        .clone(),
      password: config
        .auth
        .password
        .clone(),
      status: "Enter credentials. Tab \
               switches fields. Enter \
               to login."
        .to_string(),
      token: None,
      feeds: Vec::new(),
      favorites: Vec::new(),
      folders: Vec::new(),
      subscriptions: HashSet::new(),
      feed_counts: HashMap::new(),
      feeds_view: Vec::new(),
      subscriptions_view: Vec::new(),
      categories: Vec::new(),
      tags: Vec::new(),
      filter_category: None,
      filter_tag: None,
      sort_mode: SortMode::Unread,
      modal: None,
      entries: Vec::new(),
      tab: 0,
      selected_feed: 0,
      selected_entry: 0,
      selected_favorite: 0,
      selected_folder: 0,
      selected_subscription: 0,
      entries_page_size: config
        .ui
        .entries_page_size,
      feeds_page_size: config
        .ui
        .feeds_page_size,
      favorites_page_size: config
        .ui
        .favorites_page_size,
      folders_page_size: config
        .ui
        .folders_page_size,
      subscriptions_page_size: config
        .ui
        .subscriptions_page_size,
      feeds_offset: 0,
      favorites_offset: 0,
      folders_offset: 0,
      subscriptions_offset: 0,
      keys,
      entries_feed_id: None,
      entries_offset: 0,
      entries_next_offset: None,
      base_url: config
        .server
        .url
        .clone(),
      client
    })
  }
}
