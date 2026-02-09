use crate::app::{
    cache::NamespacesListCache,
    common::{FilterableList, HelpMenu, ListEvent, handle_general_keys},
    events::{AppEvent, EventSender},
    side_bar::namespaces::get_help_menu,
};
use crossterm::event::KeyCode;
use ratatui::{Frame, crossterm::event::KeyEvent, layout::Rect};

#[derive(Debug, Clone)]
pub struct NamespacesList {
    namespace_list: FilterableList<String>,
    event_sender: EventSender,
    help_menu: HelpMenu,
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
            help_menu: get_help_menu(),
        }
    }

    pub fn draw(&mut self, area: Rect, frame: &mut Frame, is_focused: bool) {
        self.namespace_list.draw(area, frame, is_focused);
        self.help_menu.draw(frame);
    }

    pub fn from_cache(list: NamespacesListCache, event_sender: EventSender) -> Self {
        Self {
            event_sender,
            namespace_list: list.namespace_list.into(),
            help_menu: get_help_menu(),
        }
    }

    pub fn update_list(&mut self, new_list: Vec<String>) {
        self.namespace_list.set_items(new_list);
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) -> bool {
        if self.help_menu.handle_key_event(key) {
            return false;
        }

        if let Some(list_event) = self.namespace_list.handle_key(key) {
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

        handle_general_keys(key, &self.event_sender)
    }
}
