use ratatui::{Frame, crossterm::event::KeyEvent, layout::Rect};
use serde::{Deserialize, Serialize};

use crate::{
    app::{
        cache::MainWindowCache,
        events::EventSender,
        main::{logs::PodLogs, pods_list::PodsList},
    },
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
    pod_logs: Option<PodLogs>,
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
    pub fn show_logs(&mut self, logs: PodLogs) {
        self.pod_logs = Some(logs);
        self.kind = MainWindowKind::Logs;
    }

    pub fn new(event_sender: EventSender) -> Self {
        Self {
            event_sender,
            pods_list: None,
            kind: MainWindowKind::Pods,
            pod_logs: None,
        }
    }

    pub async fn load_pods(&mut self, namespace: String) -> AppResult<()> {
        let mut pod_list = PodsList::new(self.event_sender.clone(), namespace);
        pod_list.load().await?;

        self.pods_list = Some(pod_list);
        self.kind = MainWindowKind::Pods;
        Ok(())
    }

    pub fn from_cache(value: MainWindowCache, event_sender: EventSender) -> Self {
        Self {
            pods_list: value
                .pods_list
                .map(|pods_list| PodsList::from_cache(pods_list, event_sender.clone())),
            event_sender,
            kind: value.kind,
            pod_logs: None,
        }
    }

    pub async fn handle_key_event(&mut self, key: KeyEvent) {
        match self.kind {
            MainWindowKind::Pods => {
                if let Some(pods_list) = &mut self.pods_list {
                    pods_list.handle_key_event(key).await;
                }
            }

            MainWindowKind::Logs => {
                if let Some(pods_logs) = &mut self.pod_logs {
                    let should_close = pods_logs.handle_key_event(key);

                    if should_close {
                        self.pod_logs = None;
                        self.kind = MainWindowKind::Pods;
                    }
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

            MainWindowKind::Logs => self
                .pod_logs
                .as_mut()
                .map(|pod_logs| pod_logs.draw(area, frame, is_focused)),
        };
    }
}
