use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{Frame, layout::Rect, widgets::block::Title};

use crate::{
    app::{
        cache::ClustersListCache,
        common::{FilterableList, ListEvent, handle_general_keys},
        events::{AppEvent, EventSender},
        notification::Notification,
    },
    error::AppResult,
    kubectl,
};

#[derive(Debug, Clone)]
pub struct ClustersList {
    list: FilterableList<String>,
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
        self.list.set_items(clusters);
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
                    self.event_sender.send(AppEvent::LoadNamespaces(cluster));
                }
                ListEvent::StayInList => {}
            };
            return true;
        }

        // TODO: show help

        if let KeyCode::Char('r') = key.code {
            if let Err(err) = self.load_clusters().await {
                self.event_sender
                    .send(AppEvent::ShowNotification(Notification::error(err)));
            }

            return true;
        }

        handle_general_keys(key, &self.event_sender)
    }
}
