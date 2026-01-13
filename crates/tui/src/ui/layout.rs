use ratatui::Frame;
use ratatui::layout::{
  Constraint,
  Direction,
  Layout
};
use ratatui::style::{
  Color,
  Modifier,
  Style
};
use ratatui::text::Line;
use ratatui::widgets::{
  Block,
  Borders,
  Paragraph,
  Tabs,
  Wrap
};

use super::detail::{
  draw_entry_detail,
  draw_feed_detail,
  draw_folder_detail
};
use super::lists::{
  draw_entries_list,
  draw_feed_list,
  draw_feed_view_list,
  draw_folder_list
};
use super::modal::draw_modal_list;
use crate::app::{
  App,
  LoginField,
  ModalKind
};

pub(crate) fn draw_login(
  frame: &mut Frame,
  app: &App
) {
  let chunks = Layout::default()
    .direction(Direction::Vertical)
    .margin(2)
    .constraints([
      Constraint::Length(3),
      Constraint::Length(3),
      Constraint::Min(3),
      Constraint::Length(3)
    ])
    .split(frame.area());

  let username_style = if matches!(
    app.focus,
    LoginField::Username
  ) {
    Style::default().fg(Color::Yellow)
  } else {
    Style::default()
  };

  let password_style = if matches!(
    app.focus,
    LoginField::Password
  ) {
    Style::default().fg(Color::Yellow)
  } else {
    Style::default()
  };

  let username = Paragraph::new(
    app.username.as_str()
  )
  .block(
    Block::default()
      .borders(Borders::ALL)
      .title("Username")
  )
  .style(username_style);

  let masked =
    "*".repeat(app.password.len());

  let password = Paragraph::new(masked)
    .block(
      Block::default()
        .borders(Borders::ALL)
        .title("Password")
    )
    .style(password_style);

  let help =
    Paragraph::new(app.status.as_str())
      .block(
        Block::default()
          .borders(Borders::ALL)
          .title("Status")
      )
      .wrap(Wrap {
        trim: true
      });

  frame
    .render_widget(username, chunks[0]);
  frame
    .render_widget(password, chunks[1]);
  frame.render_widget(help, chunks[2]);
  frame.render_widget(
    Paragraph::new(
      "Enter to login | Tab to switch \
       | q to quit"
    )
    .block(
      Block::default()
        .borders(Borders::ALL)
    ),
    chunks[3]
  );
}

pub(crate) fn draw_main(
  frame: &mut Frame,
  app: &App
) {
  let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
      Constraint::Length(3),
      Constraint::Min(3),
      Constraint::Length(3)
    ])
    .split(frame.area());

  let titles = [
    "Feeds (1)",
    "Entries (2)",
    "Favorites (3)",
    "Folders (4)",
    "Subscriptions (5)"
  ]
  .iter()
  .map(|t| {
    Line::styled(
      *t,
      Style::default().fg(Color::White)
    )
  })
  .collect::<Vec<_>>();

  let tabs = Tabs::new(titles)
    .select(app.tab)
    .block(
      Block::default()
        .borders(Borders::ALL)
        .title("feedrv3")
    )
    .highlight_style(
      Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD)
    );

  frame.render_widget(tabs, chunks[0]);

  let content = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([
      Constraint::Percentage(60),
      Constraint::Percentage(40)
    ])
    .split(chunks[1]);

  match app.tab {
    | 0 => {
      draw_feed_view_list(
        frame,
        content[0],
        &app.feeds,
        &app.feeds_view,
        app.feeds_offset,
        app.feeds_page_size as usize,
        app.selected_feed,
        Some(&app.subscriptions),
        Some(&app.feed_counts),
        "Feeds"
      );
      draw_feed_detail(
        frame,
        content[1],
        app
          .feeds_view
          .get(app.selected_feed)
          .and_then(|idx| {
            app.feeds.get(*idx)
          }),
        "Feed Details"
      );
    }
    | 1 => {
      draw_entries_list(
        frame,
        content[0],
        &app.entries,
        app.selected_entry
      );
      draw_entry_detail(
        frame,
        content[1],
        app
          .entries
          .get(app.selected_entry)
      );
    }
    | 2 => {
      draw_feed_list(
        frame,
        content[0],
        &app.favorites,
        app.favorites_offset,
        app.favorites_page_size
          as usize,
        app.selected_favorite,
        Some(&app.feed_counts),
        "Favorites"
      );
      draw_feed_detail(
        frame,
        content[1],
        app
          .favorites
          .get(app.selected_favorite),
        "Favorite Details"
      );
    }
    | 3 => {
      draw_folder_list(
        frame,
        content[0],
        &app.folders,
        app.selected_folder,
        app.folders_offset,
        app.folders_page_size as usize
      );
      draw_folder_detail(
        frame,
        content[1],
        app
          .folders
          .get(app.selected_folder)
      );
    }
    | _ => {
      draw_feed_view_list(
        frame,
        content[0],
        &app.feeds,
        &app.subscriptions_view,
        app.subscriptions_offset,
        app.subscriptions_page_size
          as usize,
        app.selected_subscription,
        Some(&app.subscriptions),
        Some(&app.feed_counts),
        "Subscriptions"
      );
      draw_feed_detail(
        frame,
        content[1],
        app
          .subscriptions_view
          .get(
            app.selected_subscription
          )
          .and_then(|idx| {
            app.feeds.get(*idx)
          }),
        "Feed Details"
      );
    }
  }

  let footer =
    Paragraph::new(app.status.as_str())
      .block(
        Block::default()
          .borders(Borders::ALL)
          .title("Status")
      )
      .wrap(Wrap {
        trim: true
      });

  frame
    .render_widget(footer, chunks[2]);

  if let Some(modal) = &app.modal {
    let title = match modal.kind {
      | ModalKind::Category => {
        "Select Category"
      }
      | ModalKind::Tag => "Select Tag",
      | ModalKind::Sort => "Sort Feeds"
    };
    draw_modal_list(
      frame,
      title,
      &modal.options,
      modal.selected
    );
  }
}
