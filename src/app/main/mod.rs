use crossterm::event::KeyEvent;
use ratatui::{Frame, layout::Rect};
use serde::{Deserialize, Serialize};

use crate::{
    app::{cache::MainWindowCache, events::EventSender, main::pods_list::PodsList},
    error::AppResult,
};

pub mod logs;
pub mod pods_list;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum MainWindowKind {
    Pods,
    Logs,
}

#[derive(Debug, Clone)]
pub struct MainWindow {
    pods_list: Option<PodsList>,
    event_sender: EventSender,
    kind: MainWindowKind,
}

impl From<MainWindow> for MainWindowCache {
    fn from(value: MainWindow) -> Self {
        Self {
            pods_list: value.pods_list.map(|pods_list| pods_list.into()),
            kind: value.kind,
        }
    }
}

impl MainWindow {
    pub fn is_empty(&self) -> bool {
        match self.kind {
            MainWindowKind::Pods => self.pods_list.is_none(),
        }
    }

    pub fn new(event_sender: EventSender) -> Self {
        Self {
            event_sender,
            pods_list: None,
            kind: MainWindowKind::Pods,
        }
    }

    pub async fn load_pods(&mut self, namespace: String) -> AppResult<()> {
        self.pods_list = Some(
            PodsList::new(self.event_sender.clone(), namespace)
                .load()
                .await?,
        );

        Ok(())
    }

    pub fn from_cache(value: MainWindowCache, event_sender: EventSender) -> Self {
        Self {
            pods_list: value
                .pods_list
                .map(|pods_list| PodsList::from_cache(pods_list, event_sender.clone())),
            event_sender,
            kind: value.kind,
        }
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) {
        match self.kind {
            MainWindowKind::Pods => {
                if let Some(pods_list) = &mut self.pods_list {
                    pods_list.handle_key_event(key);
                }
            }
        }
    }

    pub fn draw(&mut self, area: Rect, frame: &mut Frame, is_focused: bool) {
        match self.kind {
            MainWindowKind::Pods => self
                .pods_list
                .as_mut()
                .map(|pods_list| pods_list.draw(area, frame, is_focused)),
        };
    }
}
