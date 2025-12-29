use ratatui::{Frame, crossterm::event::KeyEvent, layout::Rect};

use crate::app::{
    cache::RecentNamespacesListCache,
    common::{FilterableList, ListEvent},
    events::{AppEvent, EventSender},
};

#[derive(Debug, Clone)]
pub struct RecentNamespacesList {
    recent_namespaces_list: FilterableList<String>,
    event_sender: EventSender,
}

impl From<RecentNamespacesList> for RecentNamespacesListCache {
    fn from(value: RecentNamespacesList) -> Self {
        Self {
            recent_namespaces_list: value.recent_namespaces_list.into(),
        }
    }
}

impl RecentNamespacesList {
    pub fn new(event_sender: EventSender) -> Self {
        Self {
            event_sender,
            recent_namespaces_list: FilterableList::new("Recent Namespaces".to_string(), false),
        }
    }

    pub fn add_to_list(&mut self, new_namespace: String) {
        self.recent_namespaces_list.add_to_list(new_namespace);
    }

    pub fn draw(&mut self, area: Rect, frame: &mut Frame) {
        self.recent_namespaces_list.draw(area, frame);
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) {
        if let Some(list_event) = self.recent_namespaces_list.handle_key(key) {
            match list_event {
                ListEvent::Quit => {
                    let _ = self.event_sender.send(AppEvent::Quit);
                }
                ListEvent::SelectedItem(item) => {
                    let _ = self.event_sender.send(AppEvent::SelectNamespace(item));
                }
            };
        }
    }
}
