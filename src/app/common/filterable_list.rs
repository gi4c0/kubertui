use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    text::Span,
    widgets::{Clear, List, ListItem, ListState, Paragraph},
};

use crate::app::{
    cache::{FilterableListCache, StateCache},
    common::build_block,
};

#[derive(Default, Debug, Clone)]
pub struct FilterableList<T> {
    pub inner_list: Vec<T>,
    pub state: ListState,
    list_name: String,
    is_filterable: bool,
    filtered_list: Vec<usize>,
    filter: String,
    is_filter_mod: bool,
}

pub trait ListItemTrait {
    fn as_ref(&self) -> &str;

    fn get_style(&self) -> Option<Style> {
        None
    }

    fn is_loading(&self) -> Option<String> {
        None
    }
}

impl ListItemTrait for String {
    fn as_ref(&self) -> &str {
        self.as_str()
    }

    fn get_style(&self) -> Option<Style> {
        None
    }
}

impl<Item, ItemCache> From<FilterableList<Item>> for FilterableListCache<ItemCache>
where
    Item: Into<ItemCache>,
{
    fn from(value: FilterableList<Item>) -> Self {
        Self {
            filter: value.filter,
            filtered_list: value.filtered_list,
            is_filter_mod: value.is_filter_mod,
            list: value
                .inner_list
                .into_iter()
                .map(|item| item.into())
                .collect(),

            state: StateCache {
                selected: value.state.selected(),
            },
            is_filterable: value.is_filterable,
            list_name: value.list_name,
        }
    }
}

impl<ItemCache, Item> From<FilterableListCache<ItemCache>> for FilterableList<Item>
where
    ItemCache: Into<Item>,
{
    fn from(value: FilterableListCache<ItemCache>) -> Self {
        let mut state = ListState::default();
        state.select(value.state.selected);

        Self {
            filter: value.filter,
            filtered_list: value.filtered_list,
            is_filter_mod: value.is_filter_mod,
            inner_list: value.list.into_iter().map(|item| item.into()).collect(),
            state,
            is_filterable: value.is_filterable,
            list_name: value.list_name,
        }
    }
}

impl<Item> FilterableList<Item>
where
    Item: Clone + ListItemTrait,
{
    pub fn append_to_list(&mut self, new_item: Item) {
        self.inner_list.insert(0, new_item);
        self.update_filtered_list();
    }

    pub fn new(list_name: String, is_filterable: bool) -> Self {
        let mut state = ListState::default();
        state.select(Some(0));

        Self {
            filter: String::new(),
            filtered_list: vec![],
            is_filter_mod: false,
            inner_list: vec![],
            is_filterable,
            list_name,
            state,
        }
    }

    pub fn set_items(&mut self, new_list: Vec<Item>) {
        self.filtered_list = new_list
            .iter()
            .enumerate()
            .map(|(index, _)| index)
            .collect();

        self.inner_list = new_list;
        self.state.select(Some(0));
    }

    pub fn draw(&mut self, area: Rect, frame: &mut Frame, is_focused: bool) {
        let list_items: Vec<ListItem> = self
            .filtered_list
            .iter()
            .map(|index| {
                let item = &self.inner_list[*index];
                // TODO: use Line
                let mut span = Span::from(item.as_ref());

                if let Some(spinner_text) = item.is_loading() {
                    let span_content = span.content;
                    span = Span::from(format!("{span_content} {spinner_text}"));
                }

                if let Some(style) = item.get_style() {
                    span = span.style(style);
                }

                ListItem::new(span)
            })
            .collect();

        let block = build_block(self.list_name.as_str(), !self.is_filter_mod && is_focused);

        let list = List::new(list_items)
            .block(block)
            .highlight_style(Style::default().underlined());

        if self.is_filter_mod || !self.filter.is_empty() {
            let layouts = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![Constraint::Length(3), Constraint::Min(1)])
                .split(area);

            let block = build_block("Filter", self.is_filter_mod);

            let filter_widget = Paragraph::new(self.filter.as_str()).block(block);

            for area in &*layouts {
                frame.render_widget(Clear, *area);
            }

            frame.render_widget(filter_widget, layouts[0]);
            frame.render_stateful_widget(list, layouts[1], &mut self.state);
            return;
        }

        frame.render_widget(Clear, area);
        frame.render_stateful_widget(list, area, &mut self.state);
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<ListEvent<Item>> {
        if self.is_filter_mod {
            match key.code {
                KeyCode::Enter => {
                    self.is_filter_mod = false;
                    self.state.select(Some(0));
                }
                KeyCode::Esc => {
                    self.filter.clear();
                    self.is_filter_mod = false;
                    self.update_filtered_list();
                    self.state.select(Some(0));
                }
                KeyCode::Backspace => {
                    self.filter.pop();
                    self.update_filtered_list();
                }
                KeyCode::Char(ch) => {
                    self.filter.push(ch);
                    self.update_filtered_list();
                }
                _ => {}
            };

            return Some(ListEvent::StayInList);
        }

        match key.code {
            KeyCode::Char('/') if self.is_filterable => {
                self.is_filter_mod = true;
            }
            KeyCode::Char('j') | KeyCode::Down => self.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.select_prev(),
            KeyCode::Char('G') => {
                if !self.filtered_list.is_empty() {
                    self.state.select(Some(self.filtered_list.len() - 1));
                }
            }
            KeyCode::Char('g') => {
                if !self.filtered_list.is_empty() {
                    self.state.select(Some(0));
                }
            }
            KeyCode::Enter => {
                let index = self.filtered_list.get(self.state.selected().unwrap_or(0));
                return index.map(|&index| ListEvent::SelectedItem(self.inner_list[index].clone()));
            }
            KeyCode::Char('q') => return Some(ListEvent::Quit),
            _ => {}
        };

        None
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
            .inner_list
            .iter()
            .enumerate()
            .filter(|(_, item)| {
                if self.filter.trim().is_empty() {
                    return true;
                }

                item.as_ref().contains(&self.filter)
            })
            .map(|(index, _)| index)
            .collect();
    }

    fn select_next(&mut self) {
        if self.filtered_list.is_empty() {
            return self.state.select(None);
        }

        let i = match self.state.selected() {
            Some(i) => {
                if i == self.filtered_list.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };

        self.state.select(Some(i));
    }

    fn select_prev(&mut self) {
        if self.filtered_list.is_empty() {
            return self.state.select(None);
        }

        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.filtered_list.len() - 1
                } else {
                    i - 1
                }
            }
            None => self.filtered_list.len() - 1,
        };

        self.state.select(Some(i));
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ListEvent<T> {
    SelectedItem(T),
    Quit,
    StayInList,
}
