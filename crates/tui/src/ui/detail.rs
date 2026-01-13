use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{
  Block,
  Borders,
  Paragraph,
  Wrap
};

use crate::models::{
  EntrySummary,
  FeedSummary,
  FolderRow
};

pub(crate) fn draw_feed_detail(
  frame: &mut Frame,
  area: Rect,
  feed: Option<&FeedSummary>,
  title: &str
) {
  let lines = if let Some(feed) = feed {
    vec![
      Line::from(format!(
        "id: {}",
        feed.id
      )),
      Line::from(format!(
        "url: {}",
        feed.url
      )),
      Line::from(format!(
        "domain: {}",
        feed.domain
      )),
      Line::from(format!(
        "category: {}",
        feed.category
      )),
      Line::from(format!(
        "base_poll_seconds: {}",
        feed.base_poll_seconds
      )),
      Line::from(format!(
        "tags: {}",
        feed
          .tags
          .as_ref()
          .map(|tags| tags.join(", "))
          .unwrap_or_else(|| {
            "-".to_string()
          })
      )),
    ]
  } else {
    vec![Line::from("No selection")]
  };

  let widget = Paragraph::new(lines)
    .block(
      Block::default()
        .borders(Borders::ALL)
        .title(title)
    )
    .wrap(Wrap {
      trim: true
    });

  frame.render_widget(widget, area);
}

pub(crate) fn draw_entry_detail(
  frame: &mut Frame,
  area: Rect,
  entry: Option<&EntrySummary>
) {
  let lines = if let Some(entry) = entry
  {
    vec![
      Line::from(format!(
        "id: {}",
        entry.id
      )),
      Line::from(format!(
        "feed: {}",
        entry.feed_id
      )),
      Line::from(format!(
        "read: {}",
        if entry.is_read {
          "yes"
        } else {
          "no"
        }
      )),
      Line::from(format!(
        "published: {:?}",
        entry.published_at_ms
      )),
      Line::from(format!(
        "title: {}",
        entry
          .title
          .as_deref()
          .unwrap_or("(untitled)")
      )),
      Line::from(format!(
        "link: {}",
        entry
          .link
          .as_deref()
          .unwrap_or("-")
      )),
    ]
  } else {
    vec![Line::from("No selection")]
  };

  let widget = Paragraph::new(lines)
    .block(
      Block::default()
        .borders(Borders::ALL)
        .title("Entry Details")
    )
    .wrap(Wrap {
      trim: true
    });

  frame.render_widget(widget, area);
}

pub(crate) fn draw_folder_detail(
  frame: &mut Frame,
  area: Rect,
  folder: Option<&FolderRow>
) {
  let lines =
    if let Some(folder) = folder {
      vec![
        Line::from(format!(
          "id: {}",
          folder.id
        )),
        Line::from(format!(
          "name: {}",
          folder.name
        )),
      ]
    } else {
      vec![Line::from("No selection")]
    };

  let widget = Paragraph::new(lines)
    .block(
      Block::default()
        .borders(Borders::ALL)
        .title("Folder Details")
    )
    .wrap(Wrap {
      trim: true
    });

  frame.render_widget(widget, area);
}
