mod log_item;

use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{
        Clear, List, ListItem, ListState, Paragraph, Scrollbar, ScrollbarOrientation,
        ScrollbarState,
    },
};
use serde_json::Value;

use crate::{
    app::{
        common::{FOCUS_COLOR, HelpMenuEnum, build_block},
        events::{AppEvent, EventSender},
        main::logs::log_item::LogItem,
    },
    error::AppResult,
    kubectl,
};

#[derive(Debug, Clone)]
pub struct PodLogs {
    pod_name: String,
    event_sender: EventSender,
    logs: Vec<String>,
    state: ListState,
    scrollbar_state: ScrollbarState,
    filtered_list: Vec<usize>,
    filters: Vec<String>,
    add_new_filter_mod: bool,
    edit_filters_mod: bool,
    active_filter_index: usize,
    selected_log: Option<LogItem>,
}

impl PodLogs {
    pub async fn load(
        namespace: String,
        pod_name: String,
        event_sender: EventSender,
    ) -> AppResult<Self> {
        let mut logs = kubectl::load_logs(namespace.as_str(), pod_name.as_str()).await?;
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
            selected_log: None,
            event_sender,
            active_filter_index: 0,
            filters: Vec::new(),
            scrollbar_state: ScrollbarState::new(prettified_logs.len()),
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
        if let Some(selected_log) = &mut self.selected_log {
            return selected_log.draw(area, frame, is_focused);
        }

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
            self.render_scrollbar(layouts[1], frame);
            return;
        }

        frame.render_widget(Clear, area);
        frame.render_stateful_widget(list, area, &mut self.state);
        self.render_scrollbar(area, frame);
    }

    fn render_scrollbar(&mut self, area: Rect, frame: &mut Frame) {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
        frame.render_stateful_widget(scrollbar, area, &mut self.scrollbar_state);
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) -> bool {
        if let Some(selected_log) = &mut self.selected_log {
            let should_close = selected_log.handle_key_event(key);

            if should_close {
                self.selected_log = None;
            }
            return false;
        }

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

                    self.update_filtered_list();
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
            KeyCode::Char('q') | KeyCode::Esc => return true,

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

            KeyCode::Char('G') => {
                if !self.filtered_list.is_empty() {
                    self.state.select(Some(self.filtered_list.len() - 1));
                    self.scrollbar_state.last();
                }
            }

            KeyCode::Tab | KeyCode::BackTab => {
                self.event_sender.send(AppEvent::FocusSwitch);
            }

            KeyCode::Char('g') => {
                if !self.filtered_list.is_empty() {
                    self.state.select(Some(0));
                    self.scrollbar_state.first();
                }
            }

            KeyCode::Char('?') => self
                .event_sender
                .send(AppEvent::ShowHelp(HelpMenuEnum::Logs)),

            KeyCode::Enter => {
                if let Some(selected) = self.state.selected() {
                    let index = &self.filtered_list[selected];
                    let log = &self.logs[*index];

                    self.selected_log = Some(LogItem::new(
                        log.clone(),
                        self.pod_name.clone(),
                        self.event_sender.clone(),
                    ));
                }
            }
            _ => return false,
        };

        false
    }

    fn select_next(&mut self) {
        if self.filtered_list.is_empty() {
            return self.state.select(None);
        }

        let prev_state = self.state.selected();
        let last = self.filtered_list.len() - 1;

        let new_state = match prev_state {
            Some(i) if i == last => 0,
            Some(i) => i + 1,
            None => 0,
        };

        self.state.select(Some(new_state));

        let scrollbar_first = match prev_state {
            None => true,
            Some(i) => i == last,
        };

        if scrollbar_first {
            self.scrollbar_state.first();
        } else {
            self.scrollbar_state.next();
        }
    }

    fn select_prev(&mut self) {
        if self.filtered_list.is_empty() {
            return self.state.select(None);
        }

        let perv_state = self.state.selected();
        let last = self.filtered_list.len() - 1;

        let new_state = match perv_state {
            Some(0) => last,
            Some(i) => i - 1,
            None => last,
        };

        self.state.select(Some(new_state));

        match perv_state {
            None | Some(0) => self.scrollbar_state.last(),
            Some(_) => self.scrollbar_state.prev(),
        }
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

        self.state.select(None);
        self.scrollbar_state.first();
    }
}
