pub(super) fn move_index(
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

pub(super) fn ensure_offset(
  selected: usize,
  offset: usize,
  page_size: usize,
  len: usize
) -> usize {
  if len == 0 {
    return 0;
  }

  let page = page_size.max(1);
  let mut next =
    offset.min(len.saturating_sub(1));

  if selected < next {
    next = selected;
  } else if selected >= next + page {
    next = selected + 1 - page;
  }

  next.min(len.saturating_sub(1))
}
