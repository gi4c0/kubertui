use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{Frame, layout::Rect};
use serde::{Deserialize, Serialize};

use crate::{
    app::{
        cache::NamespacesCache,
        common::{HelpItem, HelpMenu},
        events::EventSender,
        side_bar::namespaces::{
            namespaces_list::NamespacesList, recent_namespaces::RecentNamespacesList,
        },
    },
    error::AppResult,
    kubectl::namespace,
};

pub mod namespaces_list;
pub mod recent_namespaces;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum NamespacesWindowKind {
    Recent,
    All,
}

#[derive(Debug, Clone)]
pub struct Namespaces {
    recent: RecentNamespacesList,
    full_list: NamespacesList,
    event_sender: EventSender,
    kind: NamespacesWindowKind,
}

impl From<Namespaces> for NamespacesCache {
    fn from(value: Namespaces) -> Self {
        Self {
            recent: value.recent.into(),
            full_list: value.full_list.into(),
            kind: value.kind,
        }
    }
}

impl Namespaces {
    pub fn add_to_recent(&mut self, new_name_space: String) {
        self.recent.add_to_list(new_name_space);
    }

    pub fn initial_load(&mut self) -> AppResult<()> {
        let list = namespace::get_namespaces()?;
        self.full_list.update_list(list);
        Ok(())
    }

    pub fn from_cache(value: NamespacesCache, event_sender: EventSender) -> Self {
        Self {
            full_list: NamespacesList::from_cache(value.full_list, event_sender.clone()),
            kind: value.kind,
            recent: RecentNamespacesList::from_cache(value.recent, event_sender.clone()),
            event_sender,
        }
    }

    pub fn new(event_sender: EventSender) -> Self {
        Self {
            recent: RecentNamespacesList::new(event_sender.clone()),
            event_sender: event_sender.clone(),
            full_list: NamespacesList::new(event_sender.clone()),
            kind: NamespacesWindowKind::All,
        }
    }

    pub fn draw(&mut self, area: Rect, frame: &mut Frame, is_focused: bool) {
        match self.kind {
            NamespacesWindowKind::All => self.full_list.draw(area, frame, is_focused),
            NamespacesWindowKind::Recent => self.recent.draw(area, frame, is_focused),
        };
    }

    fn toggle_kind(&mut self) {
        self.kind = match self.kind {
            NamespacesWindowKind::All => NamespacesWindowKind::Recent,
            NamespacesWindowKind::Recent => NamespacesWindowKind::All,
        }
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) {
        match self.kind {
            NamespacesWindowKind::All => {
                if self.full_list.handle_key_event(key) {
                    return;
                }
            }
            NamespacesWindowKind::Recent => {
                if self.recent.handle_key_event(key) {
                    return;
                }
            }
        };

        match key.code {
            KeyCode::Char('[') | KeyCode::Char(']') => self.toggle_kind(),
            _ => {}
        }
    }
}

fn get_help_menu() -> HelpMenu {
    let help_items = vec![
        HelpItem {
            key: "j".to_string(),
            desc: String::from("Select below item"),
        },
        HelpItem {
            key: "<Down>".to_string(),
            desc: String::from("Select below item"),
        },
        HelpItem {
            key: "k".to_string(),
            desc: String::from("Select above item"),
        },
        HelpItem {
            key: "<Down>".to_string(),
            desc: String::from("Select above item"),
        },
        HelpItem {
            key: String::from("/"),
            desc: String::from("Search"),
        },
        HelpItem {
            key: String::from("<Esc> (in search)"),
            desc: String::from("Reset Search"),
        },
        HelpItem {
            key: String::from("["),
            desc: String::from("Select next submenu"),
        },
        HelpItem {
            key: String::from("]"),
            desc: String::from("Select prev submenu"),
        },
        HelpItem {
            key: String::from("<Enter>"),
            desc: String::from("Load pods for selected namespace"),
        },
    ];

    HelpMenu::new(String::from("SideBar"), help_items)
}
