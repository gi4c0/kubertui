use crossterm::event::{KeyCode, KeyEvent};
use ratatui::text::Line;
use ratatui::{
    Frame,
    layout::Constraint,
    widgets::{Cell, Clear, Row, Table, TableState},
};

use crate::app::common::{build_block, centered_rect, get_highlight_style};

#[derive(Debug, Clone)]
pub struct HelpItem {
    pub key: String,
    pub desc: String,
}

#[derive(Debug, Clone)]
pub struct HelpMenu {
    help_items: Vec<HelpItem>,
    filtered_list: Vec<usize>,
    longest_key_len: usize,
    filter: String,
    menu_name: String,
    title: String,
    state: TableState,
    is_filter_mod: bool,
    show_widget: bool,
}

impl HelpMenu {
    pub fn new(menu_name: String, help_items: Vec<HelpItem>) -> Self {
        Self {
            show_widget: false,
            longest_key_len: help_items
                .iter()
                .map(|item| item.key.len())
                .max()
                .unwrap_or(25),
            filtered_list: help_items
                .iter()
                .enumerate()
                .map(|(index, _)| index)
                .collect(),
            help_items,
            title: format!("Help keys for {menu_name}"),
            filter: String::new(),
            menu_name,
            state: TableState::default(),
            is_filter_mod: false,
        }
    }

    pub fn toggle_show_widget(&mut self) {
        self.show_widget = !self.show_widget;
    }

    pub fn should_show_widget(&self) -> bool {
        self.show_widget
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        if !self.show_widget {
            return;
        }

        let header: Row = Row::new([
            Cell::from(Line::from("Key").centered()),
            Cell::from("   "),
            Cell::from(Line::from("Description").centered()),
        ]);

        let rows = self.filtered_list.iter().map(|index| {
            let item = &self.help_items[*index];
            Row::new([
                Cell::from(Line::from(item.key.as_str()).right_aligned()),
                Cell::from(" | "),
                Cell::from(item.desc.as_str()),
            ])
        });

        let block = build_block(self.title.as_str(), true);

        let table = Table::new(
            rows,
            [
                Constraint::Length(self.longest_key_len as u16),
                Constraint::Length(3),
                Constraint::Min(5),
            ],
        )
        .header(header)
        .block(block)
        .row_highlight_style(get_highlight_style());

        let area = centered_rect(frame.area(), 70, 32);

        frame.render_widget(Clear, area);
        frame.render_stateful_widget(table, area, &mut self.state);
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) -> bool {
        if !self.show_widget {
            return false;
        }

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
                    self.title = format!("Help keys for {}", self.menu_name.as_str());
                }
                KeyCode::Backspace => {
                    self.filter.pop();
                    self.update_filtered_list();
                }
                KeyCode::Char(ch) => {
                    self.filter.push(ch);
                    self.title = format!(
                        "Help keys for {} ({})",
                        self.menu_name.as_str(),
                        self.filter.as_str()
                    );
                    self.update_filtered_list();
                }
                _ => {}
            };

            return true;
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.show_widget = false;
            }
            KeyCode::Char('j') | KeyCode::Down => self.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.select_prev(),
            KeyCode::Char('/') => self.is_filter_mod = true,
            _ => {}
        };

        true
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

    fn update_filtered_list(&mut self) {
        self.filtered_list = self
            .help_items
            .iter()
            .enumerate()
            .filter(|(_, item)| {
                item.key
                    .to_lowercase()
                    .contains(self.filter.to_lowercase().as_str())
                    || item
                        .desc
                        .to_lowercase()
                        .contains(self.filter.to_lowercase().as_str())
            })
            .map(|(index, _)| index)
            .collect();

        if self.filter.is_empty() {
            self.title = format!("Help keys for {}", self.menu_name.as_str());
            return;
        }

        self.title = format!(
            "Help keys for {} ({})",
            self.menu_name.as_str(),
            self.filter.as_str()
        )
    }
}
