use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyEvent},
    layout::Rect,
    widgets::{List, ListItem, ListState},
};

use crate::app::{
    common::{build_block, get_highlight_style},
    events::{AppEvent, EventSender},
};

pub struct RecentNamespacesList {
    state: ListState,
    list: Vec<String>,
    event_sender: EventSender,
}

impl RecentNamespacesList {
    pub fn new(event_sender: EventSender) -> Self {
        Self {
            event_sender,
            state: ListState::default(),
            list: Vec::new(),
        }
    }

    pub fn add_to_list(&mut self, new_namespace: String) {
        if let Some(index) = self.list.iter().position(|item| item == &new_namespace) {
            self.list.remove(index);
        }

        self.list.insert(0, new_namespace);
    }

    pub fn draw(&mut self, area: Rect, frame: &mut Frame) {
        let namespaces_list_items: Vec<ListItem> = self
            .list
            .iter()
            .map(|namespace| ListItem::new(namespace.as_str()))
            .collect();

        let list = List::new(namespaces_list_items)
            .block(build_block("Recent Namespaces"))
            .highlight_style(get_highlight_style());

        frame.render_stateful_widget(list, area, &mut self.state);
    }

    fn select_next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == self.list.len() - 1 {
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
                    self.list.len() - 1
                } else {
                    i - 1
                }
            }
            None => self.list.len() - 1,
        };

        self.state.select(Some(i));
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') => {
                let _ = self.event_sender.send(AppEvent::Quit);
            }
            KeyCode::Char('j') | KeyCode::Down => self.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.select_prev(),
            KeyCode::Enter => {
                let _ = self.event_sender.send(AppEvent::SelectNamespace(
                    self.list[self.state.selected().unwrap_or(0)].clone(),
                ));
            }
            _ => {}
        };
    }
}
