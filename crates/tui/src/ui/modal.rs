use ratatui::Frame;
use ratatui::layout::{
  Constraint,
  Direction,
  Layout,
  Rect
};
use ratatui::style::{
  Color,
  Modifier,
  Style
};
use ratatui::widgets::{
  Block,
  Borders,
  List,
  ListItem
};

use super::common::list_state;

pub(crate) fn draw_modal_list(
  frame: &mut Frame,
  title: &str,
  options: &[String],
  selected: usize
) {
  let area =
    centered_rect(60, 60, frame.area());

  let items = options
    .iter()
    .map(|opt| {
      ListItem::new(opt.clone())
    })
    .collect::<Vec<_>>();

  let list = List::new(items)
    .block(
      Block::default()
        .borders(Borders::ALL)
        .title(title)
    )
    .highlight_style(
      Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD)
    )
    .highlight_symbol("> ");

  let mut state =
    list_state(selected, options.len());

  frame.render_stateful_widget(
    list, area, &mut state
  );
}

fn centered_rect(
  percent_x: u16,
  percent_y: u16,
  rect: Rect
) -> Rect {
  let popup_layout = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
      Constraint::Percentage(
        (100 - percent_y) / 2
      ),
      Constraint::Percentage(percent_y),
      Constraint::Percentage(
        (100 - percent_y) / 2
      )
    ])
    .split(rect);

  let vertical = popup_layout[1];

  let horizontal_layout =
    Layout::default()
      .direction(Direction::Horizontal)
      .constraints([
        Constraint::Percentage(
          (100 - percent_x) / 2
        ),
        Constraint::Percentage(
          percent_x
        ),
        Constraint::Percentage(
          (100 - percent_x) / 2
        )
      ])
      .split(vertical);

  horizontal_layout[1]
}
