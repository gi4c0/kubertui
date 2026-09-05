use std::fmt::Display;

use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyEvent},
    layout::Rect,
    style::{Style, Styled},
    text::{Line, Span},
    widgets::{Clear, List, ListItem, ListState, ScrollbarState},
};

use crate::app::common::{FOCUS_COLOR, Filter, FilterEvent, build_block, traits::ListItemTrait};

pub mod cache;
pub mod scroll;
pub mod traits;

#[derive(Default, Debug, Clone)]
pub struct FilterableList<T> {
    pub inner_list: Vec<T>,
    pub state: ListState,
    title: String,
    is_filterable: bool,
    scrollbar_state: ScrollbarState,
    filtered_list: Vec<usize>,
    filter: Filter,
    show_scrollable: bool,
}

impl<Item> FilterableList<Item>
where
    Item: Clone + ListItemTrait + Display,
{
    pub fn push(&mut self, new_item: Item) {
        self.inner_list.insert(0, new_item);
        self.update_filtered_list();
    }

    pub fn scrollable(self) -> Self {
        Self {
            show_scrollable: true,
            ..self
        }
    }

    pub fn filterable(self) -> Self {
        Self {
            is_filterable: true,
            ..self
        }
    }

    pub fn new(list_name: String) -> Self {
        let mut state = ListState::default();
        state.select(Some(0));

        Self {
            scrollbar_state: ScrollbarState::new(0),
            filter: Filter::default(),
            filtered_list: vec![],
            show_scrollable: false,
            inner_list: vec![],
            is_filterable: false,
            title: list_name,
            state,
        }
    }

    fn reset_filter(&mut self) {
        self.filter.clear();
        self.update_filtered_list();
    }

    pub fn set_items(&mut self, new_list: Vec<Item>) {
        self.reset_filter();

        self.filtered_list = new_list
            .iter()
            .enumerate()
            .map(|(index, _)| index)
            .collect();

        self.inner_list = new_list;
        self.state.select(Some(0));
        self.scrollbar_state = ScrollbarState::new(self.filtered_list.len());
    }

    pub fn draw(&mut self, area: Rect, frame: &mut Frame) {
        self.draw_with_title(area, frame, self.title.clone());
    }

    pub fn draw_with_title<'a>(
        &mut self,
        area: Rect,
        frame: &mut Frame,
        title: impl Into<Line<'a>>,
    ) {
        let list_items: Vec<ListItem> = self
            .filtered_list
            .iter()
            .map(|index| {
                let item = &self.inner_list[*index];
                let mut span = Span::from(item.to_string());

                if let Some(spinner_text) = item.spinner() {
                    let span_content = span.content;
                    span = Span::from(format!("{span_content} {spinner_text}"));
                }

                if let Some(style) = item.get_style() {
                    span = span.style(style);
                }

                ListItem::new(span)
            })
            .collect();

        let mut block = build_block(title, false);

        if self.filter.is_visible() {
            block = block
                .title_bottom(format!(" Filter: {} ", self.filter.text()).set_style(FOCUS_COLOR));
        }

        let list = List::new(list_items)
            .block(block)
            .highlight_style(Style::default().underlined());

        frame.render_widget(Clear, area);
        frame.render_stateful_widget(list, area, &mut self.state);

        if self.show_scrollable {
            scroll::render_scrollbar(area, frame, &mut self.scrollbar_state);
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> ListEvent<Item> {
        if self.filter.is_active() {
            return match self.filter.handle_key(key) {
                FilterEvent::Ignored => ListEvent::Ignored,
                FilterEvent::Changed | FilterEvent::Closed { changed: true } => {
                    self.update_filtered_list();
                    ListEvent::Consumed
                }
                FilterEvent::Closed { changed: false } => {
                    self.state.select(Some(0));
                    ListEvent::Consumed
                }
            };
        }

        match key.code {
            KeyCode::Char('/') if self.is_filterable => self.filter.activate(),

            KeyCode::Char('j') | KeyCode::Down => scroll::select_next(
                &self.filtered_list,
                &mut self.state,
                &mut self.scrollbar_state,
            ),

            KeyCode::Char('k') | KeyCode::Up => scroll::select_prev(
                &self.filtered_list,
                &mut self.state,
                &mut self.scrollbar_state,
            ),

            KeyCode::Char('G') => {
                if !self.filtered_list.is_empty() {
                    self.state.select(Some(self.filtered_list.len() - 1));
                    self.scrollbar_state.last();
                }
            }
            KeyCode::Char('g') => {
                if !self.filtered_list.is_empty() {
                    self.state.select(Some(0));
                    self.scrollbar_state.first();
                }
            }
            KeyCode::Enter => {
                if let Some(index) = self.filtered_list.get(self.state.selected().unwrap_or(0)) {
                    return ListEvent::SelectedItem(self.inner_list[*index].clone());
                }
            }
            KeyCode::Char('q') => return ListEvent::Quit,
            _ => return ListEvent::Ignored,
        };

        ListEvent::Consumed
    }

    pub fn remove(&mut self, index: usize) {
        let index_to_remove = self.filtered_list.remove(index);
        self.inner_list.remove(index_to_remove);
        self.update_filtered_list();

        if self.filtered_list.get(index).is_some() {
            return;
        }

        if index == 0 {
            return self.state.select(None);
        }

        if self.filtered_list.get(index - 1).is_some() {
            return self.state.select(Some(index - 1));
        }

        self.state.select(None);
    }

    fn update_filtered_list(&mut self) {
        self.filtered_list = self
            .filter
            .apply(self.inner_list.iter().map(|item| item.to_string()));

        self.state.select(Some(0));
        self.scrollbar_state = ScrollbarState::new(self.filtered_list.len());
        self.scrollbar_state.first();
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ListEvent<T> {
    SelectedItem(T),
    Quit,
    Consumed,
    Ignored,
}
