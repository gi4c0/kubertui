use ratatui::{
    Frame,
    layout::Rect,
    widgets::{ListState, Scrollbar, ScrollbarOrientation, ScrollbarState},
};

pub fn select_next(
    filtered_list: &[usize],
    state: &mut ListState,
    scrollbar_state: &mut ScrollbarState,
) {
    if filtered_list.is_empty() {
        return state.select(None);
    }

    let prev_state = state.selected();
    let last = filtered_list.len() - 1;

    let new_state = match prev_state {
        Some(i) if i == last => 0,
        Some(i) => i + 1,
        None => 0,
    };

    state.select(Some(new_state));

    let scrollbar_first = match prev_state {
        None => true,
        Some(i) => i == last,
    };

    if scrollbar_first {
        scrollbar_state.first();
    } else {
        scrollbar_state.next();
    }
}

pub fn select_prev(
    filtered_list: &[usize],
    state: &mut ListState,
    scrollbar_state: &mut ScrollbarState,
) {
    if filtered_list.is_empty() {
        return state.select(None);
    }

    let perv_state = state.selected();
    let last = filtered_list.len() - 1;

    let new_state = match perv_state {
        Some(0) => last,
        Some(i) => i - 1,
        None => last,
    };

    state.select(Some(new_state));

    match perv_state {
        None | Some(0) => scrollbar_state.last(),
        Some(_) => scrollbar_state.prev(),
    }
}

pub fn render_scrollbar(area: Rect, frame: &mut Frame, scrollbar_state: &mut ScrollbarState) {
    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
    frame.render_stateful_widget(scrollbar, area, scrollbar_state);
}
