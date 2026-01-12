mod app;
mod models;
mod ui;

use std::io::{
  self,
  Stdout
};
use std::time::{
  Duration,
  Instant
};

use anyhow::Result;
use crossterm::event::{
  self,
  Event
};
use crossterm::execute;
use crossterm::terminal::{
  EnterAlternateScreen,
  LeaveAlternateScreen,
  disable_raw_mode,
  enable_raw_mode
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

use crate::app::{
  App,
  Screen
};
use crate::ui::{
  draw_login,
  draw_main
};

fn main() -> Result<()> {
  let base_url =
    std::env::var("FEEDRV3_SERVER_URL")
      .unwrap_or_else(|_| {
        "http://localhost:8091"
          .to_string()
      });

  enable_raw_mode()?;

  let mut stdout = io::stdout();

  execute!(
    stdout,
    EnterAlternateScreen
  )?;

  let backend =
    CrosstermBackend::new(stdout);

  let mut terminal =
    Terminal::new(backend)?;

  let mut app = App::new(base_url)?;

  app.status = "Attempting auto-login \
                as admin..."
    .to_string();

  if let Err(err) = app.login() {
    app.status = format!(
      "Auto-login failed: {err}"
    );
    app.screen = Screen::Login;
  }

  let tick_rate =
    Duration::from_millis(200);

  let mut last_tick = Instant::now();

  let res = run_app(
    &mut terminal,
    &mut app,
    tick_rate,
    &mut last_tick
  );

  disable_raw_mode()?;

  execute!(
    terminal.backend_mut(),
    LeaveAlternateScreen
  )?;

  terminal.show_cursor()?;

  res
}

fn run_app(
  terminal: &mut Terminal<
    CrosstermBackend<Stdout>
  >,
  app: &mut App,
  tick_rate: Duration,
  last_tick: &mut Instant
) -> Result<()> {
  loop {
    terminal.draw(|frame| {
      match app.screen {
        | Screen::Login => {
          draw_login(frame, app)
        }
        | Screen::Main => {
          draw_main(frame, app)
        }
      }
    })?;

    let timeout = tick_rate
      .saturating_sub(
        last_tick.elapsed()
      );

    if event::poll(timeout)? {
      if let Event::Key(key) =
        event::read()?
      {
        if app.handle_key(key)? {
          return Ok(());
        }
      }
    }

    if last_tick.elapsed() >= tick_rate
    {
      *last_tick = Instant::now();
    }
  }
}
