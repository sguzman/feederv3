use ratatui::widgets::ListState;

pub(crate) fn list_state(
  selected: usize,
  len: usize
) -> ListState {
  let mut state = ListState::default();

  if len > 0 {
    state.select(Some(
      selected
        .min(len.saturating_sub(1))
    ));
  }

  state
}

pub(crate) fn page_bounds(
  len: usize,
  offset: usize,
  page_size: usize
) -> (usize, usize) {
  if len == 0 {
    return (0, 0);
  }

  let start = offset.min(len - 1);
  let end =
    (start + page_size.max(1)).min(len);

  (start, end)
}
