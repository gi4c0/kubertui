use crate::app::main::namespaces_list::NamespacesList;
use crate::app::main::pods::Pods;
use crate::app::main::port_forwards::PortForwardsList;
use crate::app::{MainWindowKind, main::pods::logs::PodLogs};
use ratatui::{Frame, crossterm::event::KeyEvent, layout::Rect};
use serde::{Deserialize, Serialize};

use crate::{
    app::{cache::MainWindowCache, events::EventSender, main::clusters_list::ClustersList},
    kubectl::pods::Pod,
};

mod clusters_list;
mod namespaces_list;
pub mod pods;
mod port_forwards;

#[derive(Debug, Clone)]
pub struct MainWindow {
    event_sender: EventSender,
    kind: MainWindowKind,
    clusters: ClustersList,
    namespaces: NamespacesList,
    pods: Pods,
    port_forwards: PortForwardsList,
}

// impl From<MainWindow> for MainWindowCache {
//     fn from(value: MainWindow) -> Self {
//         Self {
//             pods_list: value.pods_list.map(|pods_list| pods_list.into()),
//             kind: value.kind,
//         }
//     }
// }

impl MainWindow {
    pub fn show_logs(&mut self, logs: PodLogs) {
        self.pods.show_logs(logs);
    }

    pub fn new(event_sender: EventSender) -> Self {
        Self {
            event_sender: event_sender.clone(),
            kind: MainWindowKind::Clusters,
            clusters: ClustersList::new(event_sender.clone()),
            namespaces: NamespacesList::new(event_sender.clone()),
            pods: Pods::new(event_sender.clone()),
            port_forwards: PortForwardsList::new(event_sender.clone()),
        }
    }

    pub fn show_pods(&mut self, namespace: String, pods: Vec<Pod>) {
        self.pods.show_pods(namespace, pods);
    }

    pub fn from_cache(value: MainWindowCache, event_sender: EventSender) -> Self {
        todo!()
    }

    pub async fn handle_key_event(&mut self, key: KeyEvent) {
        match self.kind {
            MainWindowKind::Namespaces => {
                self.namespaces.handle_key_event(key).await;
            }

            MainWindowKind::Clusters => {
                self.clusters.handle_key_event(key).await;
            }

            MainWindowKind::PortForward => {
                self.port_forwards.handle_key_event(key).await;
            }

            MainWindowKind::Pods => {
                self.pods.handle_key_event(key).await;
            }
        }
    }

    pub fn draw(&mut self, area: Rect, frame: &mut Frame) {
        match self.kind {
            MainWindowKind::Clusters => self.clusters.draw(area, frame),
            MainWindowKind::Namespaces => self.namespaces.draw(area, frame),
            MainWindowKind::Pods => self.pods.draw(area, frame),
            MainWindowKind::PortForward => self.port_forwards.draw(area, frame),
        };
    }
}
