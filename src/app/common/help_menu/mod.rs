mod namespaces_help;
mod pod_list_help;
mod pod_logs_help;
mod recent_port_forwards_help;

use std::vec;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::text::Line;
use ratatui::{
    Frame,
    layout::Constraint,
    widgets::{Cell, Clear, Row, Table, TableState},
};

use crate::app::common::{build_block, centered_rect, get_highlight_style};

#[derive(Debug, Clone, Copy)]
pub enum HelpMenuEnum {
    Namespaces,
    RecentNamespaces,
    RecentPortForwards,
    Pods,
    Logs,
}

#[derive(Debug, Clone)]
struct HelpByDomain {
    namespaces: Vec<HelpItem>,
    recent_namespaces: Vec<HelpItem>,
    recent_port_forwards: Vec<HelpItem>,
    pods: Vec<HelpItem>,
    logs: Vec<HelpItem>,
}

#[derive(Debug, Clone)]
pub struct HelpItem {
    pub key: &'static str,
    pub desc: &'static str,
}

#[derive(Debug, Clone)]
pub struct HelpMenu {
    filtered_list: Vec<usize>,
    longest_key_len: usize,
    filter: String,
    title: String,
    state: TableState,
    is_filter_mod: bool,
    kind: Option<HelpMenuEnum>,
    help_by_domain: HelpByDomain,
}

impl Default for HelpMenu {
    fn default() -> Self {
        Self {
            kind: None,
            longest_key_len: 10,
            filtered_list: vec![],
            title: "Help Menu".to_string(),
            filter: String::new(),
            state: TableState::default(),
            is_filter_mod: false,
            help_by_domain: HelpByDomain {
                namespaces: namespaces_help::HELP_ITEMS.to_vec(),
                recent_namespaces: namespaces_help::HELP_ITEMS.to_vec(),
                recent_port_forwards: recent_port_forwards_help::HELP_ITEMS.to_vec(),
                pods: pod_list_help::HELP_ITEMS.to_vec(),
                logs: pod_logs_help::HELP_ITEMS.to_vec(),
            },
        }
    }
}

impl HelpMenu {
    const TITLE: &'static str = "Help Menu";

    pub fn show(&mut self, kind: HelpMenuEnum) {
        self.kind = Some(kind);
        self.update_filtered_list();
    }

    pub fn is_shown(&self) -> bool {
        self.kind.is_some()
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        if let Some(kind) = self.kind {
            let header: Row = Row::new([
                Cell::from(Line::from("Key").centered()),
                Cell::from("   "),
                Cell::from(Line::from("Description").centered()),
            ]);

            let rows = self.filtered_list.iter().map(|index| {
                let item = &self.get_items(kind)[*index];
                Row::new([
                    Cell::from(Line::from(item.key).right_aligned()),
                    Cell::from(" | "),
                    Cell::from(item.desc),
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
    }

    fn get_items(&self, kind: HelpMenuEnum) -> &[HelpItem] {
        match kind {
            HelpMenuEnum::RecentNamespaces => &self.help_by_domain.recent_namespaces,
            HelpMenuEnum::Namespaces => &self.help_by_domain.namespaces,
            HelpMenuEnum::RecentPortForwards => &self.help_by_domain.recent_port_forwards,
            HelpMenuEnum::Pods => &self.help_by_domain.pods,
            HelpMenuEnum::Logs => &self.help_by_domain.logs,
        }
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) -> bool {
        if self.kind.is_none() {
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
                    self.title = Self::TITLE.to_string();
                }
                KeyCode::Backspace => {
                    self.filter.pop();
                    self.update_filtered_list();
                }
                KeyCode::Char(ch) => {
                    self.filter.push(ch);
                    self.title = format!("{} (filter: {})", Self::TITLE, self.filter.as_str());
                    self.update_filtered_list();
                }
                _ => {}
            };

            return true;
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.kind = None;
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
        if let Some(kind) = self.kind {
            self.filtered_list = self
                .get_items(kind)
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
                self.title = Self::TITLE.to_string();
                return;
            }

            self.title = format!("{} ({})", Self::TITLE, self.filter.as_str())
        }
    }
}
