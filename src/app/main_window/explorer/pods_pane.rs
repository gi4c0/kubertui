use crossterm::event::KeyEvent;
use ratatui::{Frame, layout::Rect};
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, VariantArray};

use crate::{
    app::{
        cache::PodsCache,
        events::{EventSender, KeyEventResult},
        main_window::explorer::pods_pane::pods_list::PodsList,
    },
    kubectl::pods::Pod,
};

pub mod pods_list;

#[derive(Debug, Serialize, Deserialize, PartialEq, Copy, Clone, VariantArray, AsRefStr)]
pub enum PodsKind {
    List,
    Info,
}

#[derive(Debug, Clone)]
pub struct PodsPane {
    pods_list: Option<PodsList>,
    kind: PodsKind,
    event_sender: EventSender,
}

impl PodsPane {
    pub fn new(event_sender: EventSender) -> Self {
        Self {
            pods_list: None,
            event_sender,
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
        }
    }

    pub fn stop_spinner(&mut self, pod_name: &str) {
        if let Some(pods_list) = self.pods_list.as_mut() {
            pods_list.stop_spinner(pod_name);
        }
    }

    pub fn show_pods(&mut self, namespace: String, pods: Vec<Pod>) {
        let pod_list = PodsList::new(self.event_sender.clone(), namespace, pods);

        self.pods_list = Some(pod_list);
        self.kind = PodsKind::List;
    }

    pub fn draw(&mut self, area: Rect, frame: &mut Frame) {
        match self.kind {
            PodsKind::List => {
                if let Some(pods_list) = self.pods_list.as_mut() {
                    pods_list.draw(area, frame);
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

    pub fn handle_key_event(&mut self, key: KeyEvent) -> KeyEventResult {
        match self.kind {
            PodsKind::List => {
                if let Some(pods_list) = self.pods_list.as_mut() {
                    return pods_list.handle_key_event(key);
                }
            }

            PodsKind::Info => todo!(),
        };

        KeyEventResult::Ignored
    }
}

impl From<PodsPane> for PodsCache {
    fn from(value: PodsPane) -> Self {
        Self {
            kind: value.kind,
            pods_list: value.pods_list.map(|item| item.into()),
        }
    }
}
