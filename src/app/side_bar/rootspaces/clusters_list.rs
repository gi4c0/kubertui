use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{Frame, layout::Rect, style::Style, widgets::block::Title};
use serde::{Deserialize, Serialize};

use crate::{
    app::{
        cache::{ClusterCache, ClustersListCache},
        common::{
            FOCUS_COLOR, FilterableList, HelpMenuEnum, ListEvent, ListItemTrait,
            handle_general_keys,
        },
        events::{AppEvent, EventSender},
        notification::Notification,
    },
    error::AppResult,
    kubectl,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cluster {
    name: String,
    is_selected: bool,
}

impl From<ClusterCache> for Cluster {
    fn from(value: ClusterCache) -> Self {
        Self {
            is_selected: value.is_selected,
            name: value.name,
        }
    }
}

impl From<Cluster> for ClusterCache {
    fn from(value: Cluster) -> Self {
        Self {
            is_selected: value.is_selected,
            name: value.name,
        }
    }
}

impl ListItemTrait for Cluster {
    fn get_style(&self) -> Option<Style> {
        if self.is_selected {
            return Some(FOCUS_COLOR.into());
        }

        None
    }

    fn as_ref(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Clone)]
pub struct ClustersList {
    list: FilterableList<Cluster>,
    event_sender: EventSender,
}

impl From<ClustersList> for ClustersListCache {
    fn from(value: ClustersList) -> Self {
        Self {
            list: value.list.into(),
        }
    }
}

impl ClustersList {
    pub fn from_cache(value: ClustersListCache, event_sender: EventSender) -> Self {
        Self {
            list: value.list.into(),
            event_sender,
        }
    }

    pub fn new(event_sender: EventSender) -> Self {
        Self {
            list: FilterableList::new(String::from("Clusters"), true),
            event_sender,
        }
    }

    pub async fn load_clusters(&mut self) -> AppResult<()> {
        let clusters = kubectl::get_clusters().await?;
        self.list.set_items(
            clusters
                .into_iter()
                .map(|item| Cluster {
                    is_selected: false,
                    name: item,
                })
                .collect(),
        );
        Ok(())
    }

    pub fn draw<'a>(
        &'a mut self,
        area: Rect,
        frame: &mut Frame,
        is_focused: bool,
        title: impl Into<Title<'a>>,
    ) {
        self.list.draw_with_title(area, frame, is_focused, title);
    }

    pub async fn handle_key_event(&mut self, key: KeyEvent) -> bool {
        if let Some(list_event) = self.list.handle_key(key) {
            match list_event {
                ListEvent::Quit => {
                    self.event_sender.send(AppEvent::Quit);
                }
                ListEvent::SelectedItem(cluster) => {
                    self.event_sender
                        .send(AppEvent::LoadNamespaces(cluster.name.clone()));

                    self.list.inner_list.iter_mut().for_each(|item| {
                        item.is_selected = item.name == cluster.name;
                    });
                }
                ListEvent::StayInList => {}
            };
            return true;
        }

        match key.code {
            KeyCode::Char('r') => {
                if let Err(err) = self.load_clusters().await {
                    self.event_sender
                        .send(AppEvent::ShowNotification(Notification::error(err)));
                }

                return true;
            }
            KeyCode::Char('?') => {
                self.event_sender
                    .send(AppEvent::ShowHelp(HelpMenuEnum::Clusters));
            }
            _ => {}
        }

        handle_general_keys(key, &self.event_sender)
    }
}
