use crossterm::event::KeyEvent;
use ratatui::{Frame, layout::Rect};

use crate::{
    app::{
        events::EventSender,
        main::pods::{logs::PodLogs, pods_list::PodsList},
    },
    error::AppResult,
    kubectl::pods::Pod,
};

pub mod logs;
pub mod pods_list;

#[derive(Debug, Clone)]
pub enum PodsKind {
    List,
    Logs,
    Info,
}

#[derive(Debug, Clone)]
pub struct Pods {
    pods_list: Option<PodsList>,
    pod_logs: Option<PodLogs>,
    kind: PodsKind,
    event_sender: EventSender,
}

impl Pods {
    pub fn new(event_sender: EventSender) -> Self {
        Self {
            pods_list: None,
            event_sender,
            pod_logs: None,
            kind: PodsKind::List,
        }
    }

    pub fn show_pods(&mut self, namespace: String, pods: Vec<Pod>) {
        let pod_list = PodsList::new(self.event_sender.clone(), namespace, pods);

        self.pods_list = Some(pod_list);
        self.kind = PodsKind::List;
    }

    pub fn show_logs(&mut self, logs: PodLogs) {
        self.pod_logs = Some(logs);
        self.kind = PodsKind::Logs;
    }

    pub fn draw(&mut self, area: Rect, frame: &mut Frame) {
        match self.kind {
            PodsKind::List => {
                if let Some(pods_list) = self.pods_list.as_mut() {
                    pods_list.draw(area, frame);
                }
            }

            PodsKind::Logs => {
                if let Some(pod_logs) = self.pod_logs.as_mut() {
                    pod_logs.draw(area, frame);
                }
            }

            PodsKind::Info => todo!(),
        }
    }

    pub async fn handle_key_event(&mut self, key: KeyEvent) {
        match self.kind {
            PodsKind::List => {
                if let Some(pods_list) = self.pods_list.as_mut() {
                    pods_list.handle_key_event(key).await;
                }
            }

            PodsKind::Logs => {
                if let Some(pod_logs) = self.pod_logs.as_mut() {
                    pod_logs.handle_key_event(key);
                }
            }

            PodsKind::Info => todo!(),
        }
    }
}
