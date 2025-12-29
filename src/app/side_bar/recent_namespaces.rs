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
    pub fn from_cache(value: RecentNamespacesListCache, event_sender: EventSender) -> Self {
        Self {
            event_sender,
            recent_namespaces_list: value.recent_namespaces_list.into(),
        }
    }

    pub fn new(event_sender: EventSender) -> Self {
        Self {
            event_sender,
            recent_namespaces_list: FilterableList::new("Recent Namespaces".to_string(), false),
        }
    }

    pub fn add_to_list(&mut self, new_namespace: String) {
        let existing_index = self
            .recent_namespaces_list
            .list
            .iter()
            .position(|i| i == new_namespace.as_str());

        match existing_index {
            Some(existing_index) => {
                self.recent_namespaces_list.list.remove(existing_index);
                self.recent_namespaces_list.list.insert(0, new_namespace);
            }
            None => self.recent_namespaces_list.append_to_list(new_namespace),
        };
    }

    pub fn draw(&mut self, area: Rect, frame: &mut Frame, is_focused: bool) {
        self.recent_namespaces_list.draw(area, frame, is_focused);
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) {
        if let Some(list_event) = self.recent_namespaces_list.handle_key(key) {
            match list_event {
                ListEvent::Quit => {
                    self.event_sender.send(AppEvent::Quit);
                }
                ListEvent::SelectedItem(item) => {
                    self.event_sender.send(AppEvent::SelectNamespace(item));
                }
            };
        }
    }
}
