use crossterm::event::KeyEvent;
use ratatui::{Frame, layout::Rect};
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, VariantArray};

use crate::{
    app::{
        cache::PodsCache,
        events::EventSender,
        main::pods::{
            logs::{LogsKeyEventResponse, PodLogs},
            pods_list::PodsList,
        },
    },
    kubectl::pods::Pod,
};

pub mod logs;
pub mod pods_list;

#[derive(Debug, Serialize, Deserialize, PartialEq, Copy, Clone, VariantArray, AsRefStr)]
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

    pub fn from_cache(value: PodsCache, event_sender: EventSender) -> Self {
        Self {
            kind: value.kind,
            pods_list: value
                .pods_list
                .map(|pods_list| PodsList::from_cache(pods_list, event_sender.clone())),
            event_sender,
            pod_logs: None,
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

    pub fn pods_updated(&mut self, namespace: &str, pods: Vec<Pod>) {
        if let Some(pods_list) = self.pods_list.as_mut() {
            pods_list.pods_updated(namespace, pods);
        }
    }

    pub fn logs_reloaded(&mut self, pod_name: &str, logs: Vec<String>) {
        if let Some(pod_logs) = self.pod_logs.as_mut() {
            pod_logs.logs_reloaded(pod_name, logs);
        }
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) -> bool {
        match self.kind {
            PodsKind::List => {
                if let Some(pods_list) = self.pods_list.as_mut() {
                    return pods_list.handle_key_event(key);
                }
            }

            PodsKind::Logs => {
                if let Some(pod_logs) = self.pod_logs.as_mut() {
                    match pod_logs.handle_key_event(key) {
                        LogsKeyEventResponse::CloseLogs => {
                            self.kind = PodsKind::List;
                            return true;
                        }

                        LogsKeyEventResponse::KeyHandled(result) => return result,
                    };
                }
            }

            PodsKind::Info => todo!(),
        };

        false
    }
}

impl From<Pods> for PodsCache {
    fn from(value: Pods) -> Self {
        Self {
            kind: value.kind,
            pods_list: value.pods_list.map(|item| item.into()),
        }
    }
}
