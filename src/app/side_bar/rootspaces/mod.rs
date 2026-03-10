use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyEvent},
    layout::Rect,
};
use serde::{Deserialize, Serialize};
use strum::Display;

use crate::{
    app::{
        cache::RootSpaceCache,
        common::title::build_title,
        events::EventSender,
        side_bar::rootspaces::{
            clusters_list::ClustersList, namespaces_list::NamespacesList,
            recent_namespaces::RecentNamespacesList,
        },
    },
    error::AppResult,
};

mod clusters_list;
pub mod namespaces_list;
pub mod recent_namespaces;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Display)]
pub enum RootSpaceWindowKind {
    #[strum(to_string = "Recent Namespaces")]
    RecentNamespaces,

    #[strum(to_string = "All Namespaces")]
    AllNamespaces,

    #[strum(to_string = "Clusters")]
    Clusters,
}

#[derive(Debug, Clone)]
pub struct RootSpace {
    recent: RecentNamespacesList,
    full_list: NamespacesList,
    clusters_list: ClustersList,
    event_sender: EventSender,
    kind: RootSpaceWindowKind,
}

impl From<RootSpace> for RootSpaceCache {
    fn from(value: RootSpace) -> Self {
        Self {
            recent: value.recent.into(),
            full_list: value.full_list.into(),
            kind: value.kind,
            clusters: value.clusters_list.into(),
        }
    }
}

impl RootSpace {
    const VIEWS: [RootSpaceWindowKind; 3] = [
        RootSpaceWindowKind::Clusters,
        RootSpaceWindowKind::AllNamespaces,
        RootSpaceWindowKind::RecentNamespaces,
    ];

    pub fn add_to_recent(&mut self, new_name_space: String) {
        self.recent.add_to_list(new_name_space);
    }

    pub async fn load_namespaces(&mut self, namespaces: Vec<String>) -> AppResult<()> {
        self.full_list.set_namespaces(namespaces);
        self.kind = RootSpaceWindowKind::AllNamespaces;
        Ok(())
    }

    pub async fn initial_load(&mut self) -> AppResult<()> {
        self.clusters_list.load_clusters().await
    }

    pub fn from_cache(value: RootSpaceCache, event_sender: EventSender) -> Self {
        Self {
            full_list: NamespacesList::from_cache(value.full_list, event_sender.clone()),
            kind: value.kind,
            recent: RecentNamespacesList::from_cache(value.recent, event_sender.clone()),
            event_sender: event_sender.clone(),
            clusters_list: ClustersList::from_cache(value.clusters, event_sender.clone()),
        }
    }

    pub fn new(event_sender: EventSender) -> Self {
        Self {
            recent: RecentNamespacesList::new(event_sender.clone()),
            event_sender: event_sender.clone(),
            full_list: NamespacesList::new(event_sender.clone()),
            kind: RootSpaceWindowKind::Clusters,
            clusters_list: ClustersList::new(event_sender.clone()),
        }
    }

    pub fn draw(&mut self, area: Rect, frame: &mut Frame, is_focused: bool) {
        let title = build_title(&Self::VIEWS, self.kind);

        match self.kind {
            RootSpaceWindowKind::AllNamespaces => {
                self.full_list.draw(area, frame, is_focused, title);
            }
            RootSpaceWindowKind::RecentNamespaces => {
                self.recent.draw(area, frame, is_focused, title)
            }
            RootSpaceWindowKind::Clusters => {
                self.clusters_list.draw(area, frame, is_focused, title)
            }
        };
    }

    pub async fn handle_key_event(&mut self, key: KeyEvent) {
        match self.kind {
            RootSpaceWindowKind::AllNamespaces => {
                if self.full_list.handle_key_event(key).await {
                    return;
                }
            }
            RootSpaceWindowKind::RecentNamespaces => {
                if self.recent.handle_key_event(key) {
                    return;
                }
            }
            RootSpaceWindowKind::Clusters => {
                if self.clusters_list.handle_key_event(key).await {
                    return;
                }
            }
        };

        match key.code {
            KeyCode::Char('[') => self.prev_view(),
            KeyCode::Char(']') => self.next_view(),
            _ => {}
        }
    }

    fn next_view(&mut self) {
        let current_index = Self::VIEWS
            .iter()
            .position(|view| view == &self.kind)
            .unwrap();

        if current_index + 1 == Self::VIEWS.len() {
            self.kind = *Self::VIEWS.first().unwrap();
        } else {
            self.kind = Self::VIEWS[current_index + 1];
        }
    }

    fn prev_view(&mut self) {
        let current_index = Self::VIEWS
            .iter()
            .position(|view| view == &self.kind)
            .unwrap();

        if current_index == 0 {
            self.kind = *Self::VIEWS.last().unwrap();
        } else {
            self.kind = Self::VIEWS[current_index - 1];
        }
    }
}
