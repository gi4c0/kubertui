use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{List, ListItem, ListState, Paragraph},
};

use crate::app::{
    cache::{FilterableListCache, StateCache},
    common::{build_block, get_highlight_style},
};

#[derive(Default, Debug, Clone)]
pub struct FilterableList<T> {
    pub list: Vec<T>,
    pub state: ListState,
    list_name: String,
    is_filterable: bool,
    filtered_list: Vec<usize>,
    filter: String,
    is_filter_mod: bool,
}

impl<T> From<FilterableList<T>> for FilterableListCache<T> {
    fn from(value: FilterableList<T>) -> Self {
        Self {
            filter: value.filter,
            filtered_list: value.filtered_list,
            is_filter_mod: value.is_filter_mod,
            list: value.list,
            state: StateCache {
                selected: value.state.selected(),
            },
            is_filterable: value.is_filterable,
            list_name: value.list_name,
        }
    }
}

impl<T> From<FilterableListCache<T>> for FilterableList<T> {
    fn from(value: FilterableListCache<T>) -> Self {
        let mut state = ListState::default();
        state.select(value.state.selected);

        Self {
            filter: value.filter,
            filtered_list: value.filtered_list,
            is_filter_mod: value.is_filter_mod,
            list: value.list,
            state,
            is_filterable: value.is_filterable,
            list_name: value.list_name,
        }
    }
}

impl<Item> FilterableList<Item>
where
    Item: Clone + AsRef<str>,
{
    pub fn append_to_list(&mut self, new_item: Item) {
        self.list.insert(0, new_item);
        self.update_filtered_list();
    }

    pub fn new(list_name: String, is_filterable: bool) -> Self {
        let mut state = ListState::default();
        state.select(Some(0));

        Self {
            filter: String::new(),
            filtered_list: vec![],
            is_filter_mod: false,
            list: vec![],
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

        self.list = new_list;
        self.state.select(Some(0));
    }

    pub fn draw(&mut self, area: Rect, frame: &mut Frame, is_focused: bool) {
        let namespaces_list_items: Vec<ListItem> = self
            .filtered_list
            .iter()
            .map(|index| ListItem::new(self.list[*index].as_ref()))
            .collect();

        let block = build_block(self.list_name.as_str(), !self.is_filterable && is_focused);

        let list = List::new(namespaces_list_items)
            .block(block)
            .highlight_style(get_highlight_style());

        if self.is_filter_mod || !self.filter.is_empty() {
            let layouts = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![Constraint::Length(3), Constraint::Min(1)])
                .split(area);

            let block = build_block("Filter", self.is_filter_mod);

            let filter_widget = Paragraph::new(self.filter.as_str()).block(block);

            frame.render_widget(filter_widget, layouts[0]);
            frame.render_stateful_widget(list, layouts[1], &mut self.state);
            return;
        }
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

            return None;
        }

        match key.code {
            KeyCode::Char('/') if self.is_filterable => {
                self.is_filter_mod = true;
            }
            KeyCode::Char('j') | KeyCode::Down => self.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.select_prev(),
            KeyCode::Enter => {
                let index = self.filtered_list.get(self.state.selected().unwrap_or(0));
                return index.map(|&index| ListEvent::SelectedItem(self.list[index].clone()));
            }
            KeyCode::Char('q') => return Some(ListEvent::Quit),
            _ => {}
        };

        None
    }

    fn update_filtered_list(&mut self) {
        self.filtered_list = self
            .list
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
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.filtered_list.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };

        self.state.select(Some(i));
    }
}

pub enum ListEvent<T> {
    SelectedItem(T),
    Quit,
}
