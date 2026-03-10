use crate::app::{
    cache::NamespacesListCache,
    common::{FilterableList, HelpMenuEnum, ListEvent, handle_general_keys},
    events::{AppEvent, EventSender},
};
use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyEvent},
    layout::Rect,
    text::Line,
};

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

    pub fn draw<'a>(
        &'a mut self,
        area: Rect,
        frame: &mut Frame,
        is_focused: bool,
        title: impl Into<Line<'a>>,
    ) {
        self.namespace_list
            .draw_with_title(area, frame, is_focused, title);
    }

    pub fn from_cache(list: NamespacesListCache, event_sender: EventSender) -> Self {
        Self {
            event_sender,
            namespace_list: list.namespace_list.into(),
        }
    }

    pub fn set_namespaces(&mut self, namespaces: Vec<String>) {
        self.namespace_list.set_items(namespaces);
    }

    pub async fn handle_key_event(&mut self, key: KeyEvent) -> bool {
        if let Some(list_event) = self.namespace_list.handle_key(key) {
            match list_event {
                ListEvent::Quit => {
                    self.event_sender.send(AppEvent::Quit);
                }
                ListEvent::SelectedItem(item) => {
                    self.event_sender.send(AppEvent::LoadPodsForNamespace(item));
                }
                ListEvent::StayInList => {}
            };
            return true;
        }

        if let KeyCode::Char('?') = key.code {
            self.event_sender
                .send(AppEvent::ShowHelp(HelpMenuEnum::Namespaces));
            return true;
        };

        handle_general_keys(key, &self.event_sender)
    }
}
