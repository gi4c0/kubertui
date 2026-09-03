use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    widgets::ScrollbarState,
};

use crate::app::{
    common::{HelpMenuEnum, scroll},
    events::{AppEvent, KeyEventResult},
    main_window::logs::{log_item::LogItem, pod_logs::PodLogs},
    modal::Modal,
};

impl PodLogs {
    pub fn handle_key_event(&mut self, key: KeyEvent) -> KeyEventResult {
        if self.edit_filters_mod {
            return self.edit_filter_mod_key_handler(key);
        }

        if self.add_new_filter_mod {
            return self.add_new_filter_mod_key_handler(key);
        }

        self.general_key_handler(key)
    }

    fn edit_filter_mod_key_handler(&mut self, key: KeyEvent) -> KeyEventResult {
        match key.code {
            // Select next filter
            KeyCode::Tab | KeyCode::Char('l') | KeyCode::Right => {
                self.active_filter_index = if self.active_filter_index == self.filters.len() - 1 {
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
                if self.active_filter_index == self.filters.len() && self.active_filter_index > 0 {
                    self.active_filter_index -= 1;
                }

                if self.filters.is_empty() {
                    self.edit_filters_mod = false;
                }

                self.update_filtered_list();
            }

            KeyCode::Esc | KeyCode::Enter => self.edit_filters_mod = false,
            _ => return KeyEventResult::Ignored,
        };

        KeyEventResult::Consumed
    }

    fn add_new_filter_mod_key_handler(&mut self, key: KeyEvent) -> KeyEventResult {
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
            _ => return KeyEventResult::Ignored,
        };

        KeyEventResult::Consumed
    }

    fn general_key_handler(&mut self, key: KeyEvent) -> KeyEventResult {
        match key.code {
            KeyCode::Char('/') => {
                self.filters.push(String::new());
                self.add_new_filter_mod = true;
            }

            KeyCode::Char('f') => {
                if self.filters.is_empty() {
                    return KeyEventResult::Ignored;
                }

                self.edit_filters_mod = true;
            }

            KeyCode::Char('j') | KeyCode::Down => scroll::select_next(
                &self.filtered_list,
                &mut self.state,
                &mut self.scrollbar_state,
            ),

            KeyCode::Char('r') => self.event_sender.send(AppEvent::ReloadLogs),

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

            KeyCode::Char('?') => self
                .event_sender
                .send(AppEvent::OpenModal(Modal::help(HelpMenuEnum::Logs))),

            KeyCode::Enter => {
                if let Some(selected) = self.state.selected() {
                    let index = &self.filtered_list[selected];
                    let log = &self.logs[*index];

                    self.event_sender
                        .send(AppEvent::OpenModal(Modal::LogDetail(LogItem::new(
                            log.clone(),
                            self.pod_name.clone(),
                        ))));
                }
            }
            _ => return KeyEventResult::Ignored,
        };

        KeyEventResult::Consumed
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
        self.scrollbar_state = ScrollbarState::new(self.filtered_list.len());
        self.scrollbar_state.first();
    }
}
