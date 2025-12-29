use crate::app::{
    cache::NamespacesListCache,
    common::{FilterableList, ListEvent},
    events::{AppEvent, EventSender},
};
use ratatui::{Frame, crossterm::event::KeyEvent, layout::Rect};

#[derive(Debug, Clone)]
pub struct NamespacesList {
    namespace_list: FilterableList<String>,
    event_sender: EventSender,
}

impl From<NamespacesList> for NamespacesListCache {
    fn from(value: NamespacesList) -> Self {
        Self {
            namespace_list: value.namespace_list.into(),
        }
    }
}

impl NamespacesList {
    pub fn new(event_sender: EventSender) -> Self {
        Self {
            event_sender,
            namespace_list: FilterableList::new("Namespaces".to_string(), true),
        }
    }

    pub fn draw(&mut self, area: Rect, frame: &mut Frame, is_focused: bool) {
        self.namespace_list.draw(area, frame, is_focused);
    }

    pub fn update_list(&mut self, new_list: Vec<String>) {
        self.namespace_list.set_items(new_list);
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) {
        if let Some(list_event) = self.namespace_list.handle_key(key) {
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
