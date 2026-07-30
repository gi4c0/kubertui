use crate::app::common::handle_general_keys;
use crate::app::main::namespaces_list::NamespacesList;
use crate::app::main::pods::Pods;
use crate::app::main::port_forwards::PortForwardsList;
use crate::app::{MainWindowKind, main::pods::logs::PodLogs};
use crate::error::AppResult;
use ratatui::{Frame, crossterm::event::KeyEvent, layout::Rect};

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

impl From<MainWindow> for MainWindowCache {
    fn from(value: MainWindow) -> Self {
        Self {
            pods: value.pods.into(),
            kind: value.kind,
            clusters: value.clusters.into(),
            namespaces: value.namespaces.into(),
            port_forwards: value.port_forwards.into(),
        }
    }
}

impl MainWindow {
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

    pub fn show_namespaces(&mut self, namespaces: Vec<String>) {
        self.namespaces.set_namespaces(namespaces);
    }

    pub fn add_to_list_and_port_forward(
        &mut self,
        namespace: String,
        pod_name: String,
        local_port: u16,
        app_port: u16,
    ) {
        self.port_forwards
            .add_to_list_and_port_forward(namespace, pod_name, local_port, app_port);
    }

    pub fn set_clusters(&mut self, clusters: Vec<String>) {
        self.clusters.set_clusters(clusters);
    }

    pub fn pods_updated(&mut self, namespace: &str, pods: Vec<Pod>) {
        self.pods.pods_updated(namespace, pods);
    }

    pub fn logs_reloaded(&mut self, pod_name: &str, logs: Vec<String>) {
        self.pods.logs_reloaded(pod_name, logs);
    }

    pub fn show_logs(&mut self, logs: PodLogs) {
        self.pods.show_logs(logs);
    }

    pub fn show_pods(&mut self, namespace: String, pods: Vec<Pod>) {
        self.kind = MainWindowKind::Pods;
        self.pods.show_pods(namespace, pods);
    }

    pub async fn initial_load(&mut self) -> AppResult<()> {
        self.clusters.load_clusters().await
    }

    pub fn from_cache(value: MainWindowCache, event_sender: EventSender) -> Self {
        Self {
            pods: Pods::from_cache(value.pods, event_sender.clone()),
            kind: value.kind,
            clusters: ClustersList::from_cache(value.clusters, event_sender.clone()),
            namespaces: NamespacesList::from_cache(value.namespaces, event_sender.clone()),
            port_forwards: PortForwardsList::from_cache(value.port_forwards, event_sender.clone()),
            event_sender: event_sender.clone(),
        }
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) {
        match self.kind {
            MainWindowKind::Namespaces => {
                if self.namespaces.handle_key_event(key) {
                    return;
                }
            }

            MainWindowKind::Clusters => {
                if self.clusters.handle_key_event(key) {
                    return;
                };
            }

            MainWindowKind::PortForward => {
                self.port_forwards.handle_key_event(key);
            }

            MainWindowKind::Pods => {
                if self.pods.handle_key_event(key) {
                    return;
                }
            }
        }

        handle_general_keys(key, &self.event_sender);
    }

    pub fn draw(&mut self, area: Rect, frame: &mut Frame) {
        match self.kind {
            MainWindowKind::Clusters => self.clusters.draw(area, frame),
            MainWindowKind::Namespaces => self.namespaces.draw(area, frame),
            MainWindowKind::Pods => self.pods.draw(area, frame),
            MainWindowKind::PortForward => self.port_forwards.draw(area, frame),
        };
    }

    pub fn update_active_window(&mut self, new_active_window: MainWindowKind) {
        self.kind = new_active_window;
    }
}
