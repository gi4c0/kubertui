use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Clear, List, ListItem, ListState, Paragraph},
};
use serde_json::Value;

use crate::{
    app::common::{FOCUS_COLOR, build_block},
    error::AppResult,
    kubectl,
};

#[derive(Debug, Clone)]
pub struct PodLogs {
    pod_name: String,
    logs: Vec<String>,
    state: ListState,
    filtered_list: Vec<usize>,
    filters: Vec<String>,
    add_new_filter_mod: bool,
    edit_filters_mod: bool,
    active_filter_index: usize,
}

impl PodLogs {
    pub async fn load(pod_name: String) -> AppResult<Self> {
        let mut logs = kubectl::load_logs(pod_name.as_str()).await?;
        logs.reverse();

        let prettified_logs: Vec<String> = logs
            .into_iter()
            .map(|log| {
                let parsed: Value = match serde_json::from_str(log.as_str()) {
                    Ok(value) => value,
                    _ => {
                        return log;
                    }
                };

                serde_json::to_string_pretty(&parsed).unwrap_or(log)
            })
            .collect();

        Ok(Self {
            active_filter_index: 0,
            filters: Vec::new(),
            add_new_filter_mod: false,
            edit_filters_mod: false,
            filtered_list: prettified_logs
                .iter()
                .enumerate()
                .map(|(index, _)| index)
                .collect(),
            pod_name,
            logs: prettified_logs,
            state: ListState::default(),
        })
    }

    pub fn draw(&mut self, area: Rect, frame: &mut Frame, is_focused: bool) {
        let list_items: Vec<ListItem> = self
            .filtered_list
            .iter()
            .map(|index| {
                let log = &self.logs[*index];
                ListItem::new(log.as_str())
            })
            .collect();

        let block = build_block(self.pod_name.as_str(), is_focused);
        let list = List::new(list_items).block(block).highlight_style(
            Style::default()
                .bg(Color::Gray)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        );

        if self.add_new_filter_mod || !self.filters.is_empty() {
            let layouts = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![Constraint::Length(3), Constraint::Min(1)])
                .split(area);

            let block = build_block("Filter", self.add_new_filter_mod);

            let mut filter_spans: Vec<Span> = Vec::with_capacity(self.filters.len() * 2);

            for (index, filter) in self.filters.iter().enumerate() {
                let mut span =
                    Span::from(filter).style(Style::default().bg(Color::Gray).fg(Color::Black));

                if self.edit_filters_mod && index == self.active_filter_index {
                    span = span.bg(FOCUS_COLOR);
                }

                filter_spans.push(span);
                filter_spans.push(Span::from(" "));
            }

            let filter_widget = Paragraph::new(Line::default().spans(filter_spans)).block(block);

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

    pub fn handle_key_event(&mut self, key: KeyEvent) -> bool {
        if self.edit_filters_mod {
            match key.code {
                // Select next filter
                KeyCode::Tab | KeyCode::Char('l') | KeyCode::Right => {
                    self.active_filter_index = if self.active_filter_index == self.filters.len() - 1
                    {
                        0
                    } else {
                        self.active_filter_index + 1
                    };
                }

                // Select prev filter
                KeyCode::BackTab | KeyCode::Char('h') | KeyCode::Left => {
                    self.active_filter_index = if self.active_filter_index == 0 {
                        self.filters.len() - 1
                    } else {
                        self.active_filter_index - 1
                    };
                }

                KeyCode::Char('d') => {
                    self.filters.remove(self.active_filter_index);
                    if self.active_filter_index == self.filters.len()
                        && self.active_filter_index > 0
                    {
                        self.active_filter_index -= 1;
                    }

                    if self.filters.is_empty() {
                        self.edit_filters_mod = false;
                    }
                }

                KeyCode::Esc | KeyCode::Enter => self.edit_filters_mod = false,
                _ => {}
            };

            return false;
        }

        if self.add_new_filter_mod {
            match key.code {
                KeyCode::Char(ch) => {
                    if let Some(filter) = self.filters.last_mut() {
                        filter.push(ch);
                    } else {
                        self.filters.push(String::from(ch));
                    }

                    self.update_filtered_list();
                }

                KeyCode::Enter => {
                    if let Some(filter) = self.filters.last()
                        && filter.is_empty()
                    {
                        self.filters.remove(self.filters.len() - 1);
                    }
                    self.add_new_filter_mod = false;
                }

                KeyCode::Backspace if self.add_new_filter_mod => {
                    if let Some(filter) = self.filters.last_mut() {
                        filter.pop();
                        self.update_filtered_list();
                    }
                }

                KeyCode::Esc => {
                    self.filters.remove(self.filters.len() - 1);
                    self.add_new_filter_mod = false;
                    self.update_filtered_list();
                }
                _ => {}
            };

            return false;
        }

        match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Char('/') => {
                self.filters.push(String::new());
                self.add_new_filter_mod = true;
            }
            KeyCode::Char('f') => {
                if self.filters.is_empty() {
                    return false;
                }

                self.edit_filters_mod = true;
            }
            KeyCode::Char('j') | KeyCode::Down => self.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.select_prev(),
            _ => return false,
        };

        false
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
            .logs
            .iter()
            .enumerate()
            .filter(|(_, item)| {
                if self.filters.is_empty() {
                    return true;
                }

                self.filters.iter().all(|filter| item.contains(filter))
            })
            .map(|(index, _)| index)
            .collect();
    }
}
