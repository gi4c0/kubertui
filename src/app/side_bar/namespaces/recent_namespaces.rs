use crossterm::event::KeyCode;
use ratatui::{Frame, crossterm::event::KeyEvent, layout::Rect};

use crate::app::{
    cache::RecentNamespacesListCache,
    common::{FilterableList, HelpMenu, ListEvent, handle_general_keys},
    events::{AppEvent, EventSender},
    side_bar::namespaces::get_help_menu,
};

#[derive(Debug, Clone)]
pub struct RecentNamespacesList {
    recent_namespaces_list: FilterableList<String>,
    event_sender: EventSender,
    help_menu: HelpMenu,
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
            help_menu: get_help_menu(),
            event_sender,
            recent_namespaces_list: value.recent_namespaces_list.into(),
        }
    }

    pub fn new(event_sender: EventSender) -> Self {
        Self {
            event_sender,
            help_menu: get_help_menu(),
            recent_namespaces_list: FilterableList::new("Recent Namespaces".to_string(), false),
        }
    }

    pub fn add_to_list(&mut self, new_namespace: String) {
        let existing_index = self
            .recent_namespaces_list
            .inner_list
            .iter()
            .position(|i| i == new_namespace.as_str());

        match existing_index {
            Some(existing_index) => {
                self.recent_namespaces_list
                    .inner_list
                    .remove(existing_index);
                self.recent_namespaces_list
                    .inner_list
                    .insert(0, new_namespace);
            }
            None => self.recent_namespaces_list.append_to_list(new_namespace),
        };
    }

    pub fn draw(&mut self, area: Rect, frame: &mut Frame, is_focused: bool) {
        self.recent_namespaces_list.draw(area, frame, is_focused);
        self.help_menu.draw(frame);
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) -> bool {
        if self.help_menu.handle_key_event(key) {
            return false;
        }

        if let Some(list_event) = self.recent_namespaces_list.handle_key(key) {
            match list_event {
                ListEvent::Quit => {
                    self.event_sender.send(AppEvent::Quit);
                }
                ListEvent::SelectedItem(item) => {
                    self.event_sender.send(AppEvent::SelectNamespace(item));
                }
                ListEvent::StayInList => {}
            };
            return true;
        }

        if let KeyCode::Char('?') = key.code {
            self.help_menu.toggle_show_widget();
        }

        if handle_general_keys(key, &self.event_sender) {
            self.recent_namespaces_list.state.select(None);
            return true;
        }

        false
    }
}
